use kms_client::KMSClient;
use kms_proto;
use mesatee_core::config::{OutboundDesc, TargetDesc};
pub use mesatee_core::{Error, ErrorKind, Result};
use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use teaclave_utils;

pub struct Mesatee {
    user_id: String,
    user_token: String,
    kms_desc: TargetDesc,
}

pub struct MesateeEnclaveInfo {
    enclave_signers: Vec<(Vec<u8>, PathBuf)>,
    enclave_info_file_path: PathBuf,
}

impl MesateeEnclaveInfo {
    pub fn load(auditors: Vec<(&str, &str)>, enclave_info_file_path: &str) -> Result<Self> {
        let mut enclave_signers: Vec<(Vec<u8>, PathBuf)> = vec![];

        for (der, sha) in auditors.iter() {
            let der_content = fs::read(der)?;
            enclave_signers.push((der_content, PathBuf::from_str(sha).expect("infallible")));
        }
        let enclave_info_file_path = PathBuf::from_str(enclave_info_file_path).expect("infallible");
        let enclave_info = MesateeEnclaveInfo {
            enclave_signers,
            enclave_info_file_path,
        };
        Ok(enclave_info)
    }
}

impl Mesatee {
    pub fn new(
        enclave_info: &MesateeEnclaveInfo,
        user_id: &str,
        user_token: &str,
        kms_addr: SocketAddr,
    ) -> Result<Self> {
        let mut enclave_signers: Vec<(&[u8], &Path)> = vec![];
        for (der, hash) in enclave_info.enclave_signers.iter() {
            enclave_signers.push((&der, hash.as_path()));
        }
        let enclave_info_content = fs::read_to_string(&enclave_info.enclave_info_file_path)
            .unwrap_or_else(|_| {
                panic!(
                    "Cannot find enclave info at {:?}.",
                    enclave_info.enclave_info_file_path
                )
            });
        let enclave_identities = teaclave_utils::load_enclave_info(&enclave_info_content);

        let tms_outbound_desc = OutboundDesc::new(
            *enclave_identities
                .get("kms")
                .ok_or_else(|| Error::from(ErrorKind::MissingValue))?,
        );
        let kms_desc = TargetDesc::new(kms_addr, tms_outbound_desc);

        let mesatee = Self {
            user_id: user_id.to_owned(),
            user_token: user_token.to_owned(),
            kms_desc,
        };
        Ok(mesatee)
    }

    pub fn create_key(&self) -> Result<kms_proto::proto::CreateKeyResponse> {
        let mut kms_client = KMSClient::new(&self.kms_desc, &self.user_id, &self.user_token)?;
        let resp = kms_client.create_key();
        println!("mesatee kms create_key: {:?}", resp);
	resp
    }
}
