#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;
extern crate num_bigint;
extern crate sgx_tseal;

use bytes::{BufMut, BytesMut};
use mesatee_core::{Error, ErrorKind, Result};
use ring::aead::{
    BoundKey, Nonce, NonceSequence, OpeningKey, SealingKey, UnboundKey, AES_256_GCM, NONCE_LEN,
};
use sgx_tseal::SgxSealedData;
use sgx_types::marker::ContiguousMemory;
use sgx_types::{
    sgx_hmac_256bit_key_t, sgx_sealed_data_t
};

use std::collections::HashMap;

pub const SGX_AES_GCM_256_SIZE: usize = 32;
pub static HMAC_1ST_DERIVATION_LABEL: &'static str =
    "XUPERDATA-SGX-LEDGER-SEALING-KEY-DERIVATION-KEY";
pub static HMAC_2ND_DERIVATION_LABEL: &'static str = "XUPERDATA-SGX-LEDGER-RECORD-SEALING-KEY";

#[allow(non_camel_case_types)]
pub type sgx_aes_256bit_gcm_key_t = [u8; SGX_AES_GCM_256_SIZE];

pub struct RingAeadNonceSequence {
    nonce: [u8; NONCE_LEN],
}

impl RingAeadNonceSequence {
    fn new() -> RingAeadNonceSequence {
        RingAeadNonceSequence {
            nonce: [0u8; NONCE_LEN],
        }
    }
}

impl NonceSequence for RingAeadNonceSequence {
    fn advance(&mut self) -> std::result::Result<Nonce, ring::error::Unspecified> {
        let nonce = Nonce::assume_unique_for_key(self.nonce);
        increase_nonce(&mut self.nonce);
        Ok(nonce)
    }
}

pub fn increase_nonce(nonce: &mut [u8]) {
    for i in nonce {
        if std::u8::MAX == *i {
            *i = 0;
        } else {
            *i += 1;
            return;
        }
    }
}

pub fn encryptex(
    plaintext: &[u8],
    ciphertext: &mut [u8],
    key: sgx_aes_256bit_gcm_key_t,
) -> Result<()> {
    let in_out_len = plaintext.len() + AES_256_GCM.tag_len();
    if ciphertext.len() != in_out_len {
        return Err(Error::from(ErrorKind::InvalidInputError));
    }

    let ubk = match UnboundKey::new(&AES_256_GCM, &key) {
        Ok(x) => x,
        Err(e) => {
            error!("UnboundKey::new {:?}", e);
            return Err(Error::from(ErrorKind::InvalidInputError));
        }
    };

    let nonce = RingAeadNonceSequence::new();
    // Opening key used to decrypt data
    let mut sealing_key = SealingKey::new(ubk, nonce);

    let mut in_out = BytesMut::with_capacity(in_out_len);
    in_out.put_slice(plaintext);
    let aad = ring::aead::Aad::empty();
    // Encrypt data into in_out variable
    if let Err(e) = sealing_key.seal_in_place_append_tag(aad, &mut in_out) {
        error!("sealing_key.seal_in_place_append_tag {:?}", e);
        return Err(Error::from(ErrorKind::InvalidInputError));
    };
    ciphertext.copy_from_slice(&in_out[..in_out_len]);
    Ok(())
}

pub fn decryptex(
    ciphertext: &[u8],
    plaintext: &mut [u8],
    sk_key: sgx_aes_256bit_gcm_key_t,
) -> Result<()> {
    if plaintext.len() + AES_256_GCM.tag_len() != ciphertext.len() {
        return Err(Error::from(ErrorKind::InvalidInputError));
    }
    let ubk = match UnboundKey::new(&AES_256_GCM, &sk_key) {
        Ok(x) => x,
        Err(e) => {
            error!("UnboundKey::new {:?}", e);
            return Err(Error::from(ErrorKind::InvalidInputError));
        }
    };

    let nonce = RingAeadNonceSequence::new();
    // Opening key used to decrypt data
    let mut opening_key = OpeningKey::new(ubk, nonce);
    let mut in_out = BytesMut::with_capacity(ciphertext.len());
    in_out.put_slice(ciphertext);
    let aad = ring::aead::Aad::empty();
    let decrypted_data = match opening_key.open_in_place(aad, &mut in_out) {
        Ok(x) => x,
        Err(e) => {
            error!("opening_key.open_in_place {:?}", e);
            return Err(Error::from(ErrorKind::InvalidInputError));
        }
    };
    plaintext.copy_from_slice(decrypted_data);
    Ok(())
}

pub fn generate_previous_svn_kds(kds: &[u8], svn: u32) -> Result<sgx_hmac_256bit_key_t> {
    derive_32bytes_key_from_double_hmac_sha_256(
        kds,
        HMAC_1ST_DERIVATION_LABEL.as_bytes(),
        &svn.to_be_bytes(),
        HMAC_2ND_DERIVATION_LABEL.as_bytes(),
        kds,
    )
}

pub fn generate_aes_siv_key(
    kds: &[u8],
    args_hash: &[u8],
    address: &[u8],
) -> Result<sgx_aes_256bit_gcm_key_t> {
    let kdf32_key_result = derive_32bytes_key_from_double_hmac_sha_256(
        kds,
        HMAC_1ST_DERIVATION_LABEL.as_bytes(),
        args_hash,
        HMAC_2ND_DERIVATION_LABEL.as_bytes(),
        address,
    );
    kdf32_key_result
}

#[derive(Copy, Clone, Default, Debug)]
struct KeySeal {
    svn: u32,	
    kds: sgx_hmac_256bit_key_t,
}
unsafe impl ContiguousMemory for KeySeal {}

#[derive(Default, Debug)]
pub struct KeyManagerment {
    pub kds_map: HashMap<u32, sgx_hmac_256bit_key_t>,
    pub current_svn: u32,
    pub is_ready: bool,
}

impl KeyManagerment {
    pub fn new() -> KeyManagerment {
        let rep = sgx_tse::rsgx_self_report();
        KeyManagerment {
            kds_map: HashMap::new(),
            current_svn: rep.body.isv_svn as u32,
            is_ready: false,
        }
    }
    pub fn get_kds(&mut self, svn: u32) -> Result<sgx_hmac_256bit_key_t> {
        if svn > self.current_svn {
            error!("current svn is less than svn:{:?}", svn);
            return Err(Error::from(ErrorKind::InvalidInputError));
        } else if svn == self.current_svn {
            return Ok(*self.kds_map.get(&svn).unwrap());
        }
        match self.kds_map.get(&svn) {
            Some(x) => Ok(*x),
            _ => {
                // get kds in [svn, current_svn]
                let mut idx = self.current_svn;
                loop {
                    let prev = self.kds_map.get(&idx).unwrap();
                    let prev2 = generate_previous_svn_kds(prev, idx).unwrap();
                    self.kds_map.insert(idx - 1, prev2);
                    if svn == idx {
                        return Ok(*self.kds_map.get(&svn).unwrap());
                    }
                    idx = idx - 1;
                }
            }
        }
    }
    #[allow(dead_code)]
    pub fn seal_keys_for_serializable(
        &self,
        sealed_log: *mut u8,
        seal_log_size: usize,
    ) -> Result<u32> {
        //let mut key_policy = SGX_KEYPOLICY_MRENCLAVE;
	let current_kds = self.kds_map.get(&self.current_svn).ok_or(Error::from(ErrorKind::InvalidInputError))?;
	let km_for_sealing = KeySeal {
	    svn: self.current_svn,
	    kds: *current_kds, 
	};
        let additional_text: [u8; 0] = [0u8; 0];
        let sealed_data = SgxSealedData::<KeySeal>::seal_data(
            &additional_text,
            &km_for_sealing,
        )?;
         	
        let err = Error::from(ErrorKind::InvalidInputError);
        to_sealed_log_for_slice(&sealed_data, sealed_log, seal_log_size as u32).ok_or(err)?;
        Ok(sealed_data.get_payload_size())
    }
    #[allow(dead_code)]
    pub fn unseal_keys(&mut self, cipher: *mut u8, cipher_size: usize) -> Result<()> {
	debug!("begin to from_sealed");
        let sealed_data = from_sealed_log_for_slice::<KeySeal>(cipher, cipher_size as u32)
            .ok_or(Error::from(ErrorKind::InvalidInputError))?;
	debug!("begin to unseal data");
        let unseal_data = match sealed_data.unseal_data() {
            Ok(x) => x,
            Err(e) => {
                error!("sealed_data.unseal_data error: {:?}", e);
                return Err(Error::from(ErrorKind::InvalidInputError));
            }
        };
        let km = unseal_data.get_decrypt_txt();
	debug!("begin to parse json");
        self.kds_map.insert(km.svn, km.kds);
        self.current_svn = km.svn;
        self.is_ready = true;
	debug!("done");
        Ok(())
    }
}

fn to_sealed_log_for_slice<T: Copy + ContiguousMemory>(
    sealed_data: &SgxSealedData<T>,
    sealed_log: *mut u8,
    sealed_log_size: u32,
) -> Option<*mut sgx_sealed_data_t> {
    unsafe {
        sealed_data.to_raw_sealed_data_t(sealed_log as *mut sgx_sealed_data_t, sealed_log_size)
    }
}

fn from_sealed_log_for_slice<'a, T: Copy + ContiguousMemory>(
    sealed_log: *mut u8,
    sealed_log_size: u32,
) -> Option<SgxSealedData<'a, T>> {
    unsafe {
        SgxSealedData::<T>::from_raw_sealed_data_t(
            sealed_log as *mut sgx_sealed_data_t,
            sealed_log_size,
        )
    }
}

pub fn derive_32bytes_key_from_double_hmac_sha_256(
    nonce1: &[u8],
    label1: &[u8],
    nonce2: &[u8],
    label2: &[u8],
    data: &[u8],
) -> Result<sgx_hmac_256bit_key_t> {
    let key: sgx_hmac_256bit_key_t = [0; 32];
    let mut databuf: Vec<u8> = Vec::new();
    databuf.extend_from_slice(nonce1);
    databuf.extend_from_slice(label1);
    let result = sgx_tcrypto::rsgx_hmac_sha256_slice(&key, databuf.as_slice());
    let derived_key = match result {
        Ok(x) => x,
        Err(e) => {
            error!("sgx_tcrypto::rsgx_hmac_sha256_slice: {:?}", e);
            return Err(Error::from(ErrorKind::InvalidInputError));
        }
    };
    let mut databuf: Vec<u8> = Vec::new();
    databuf.extend_from_slice(nonce2);
    databuf.extend_from_slice(label2);
    databuf.extend_from_slice(data);
    match sgx_tcrypto::rsgx_hmac_sha256_slice(&derived_key, databuf.as_slice()) {
        Ok(x) => Ok(x),
        Err(e) => {
            error!("sgx_tcrypto::rsgx_hmac_sha256_slice: {:?}", e);
            return Err(Error::from(ErrorKind::InvalidInputError));
        }
    }
}

// 验证请求签名是否正确, 目前是ecdsa算法
pub fn check_sign(msg: &String, pks: &String, sig: &String) -> Result<()> {
    let pk = match hex::decode(pks) {
        Ok(x) => x,
        Err(e) => {
            error!("hex::decode: {:?}", e);
            return Err(Error::from(ErrorKind::InvalidInputError));
        }
    };
    let public_key =
        ring::signature::UnparsedPublicKey::new(&ring::signature::ECDSA_P256_SHA256_ASN1, pk);
    let sig = match hex::decode(sig) {
        Ok(x) => x,
        Err(e) => {
            error!("hex::decode: {:?}", e);
            return Err(Error::from(ErrorKind::InvalidInputError));
        }
    };
    match public_key.verify(msg.as_bytes(), &sig) {
        Ok(()) => Ok(()),
        Err(e) => {
            error!("verify error: {:?}", e);
	    Err(Error::from(ErrorKind::InvalidInputError))
        }
    }
}
