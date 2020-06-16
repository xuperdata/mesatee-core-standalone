#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use ring::aead::AES_256_GCM;
use ring::digest;

use std::time::*;
use std::untrusted::time::{SystemTimeEx};

use super::auth;
use super::ciphertext_helper::*;
use super::crypto_helper::*;

use prost::Message;
use serde::{Deserialize, Serialize};

use sgx_types::sgx_hmac_256bit_key_t;

use std::boxed::Box;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sgxfs::SgxFile;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::vec;
use std::vec::Vec;

use crate::worker::{Worker, WorkerContext};
use mesatee_core::{Error, ErrorKind, Result};
static DEFAULT_KEY_PATH: &str = "/var/xuperdata/kds.key";

pub struct XChainKMSWorker {
    worker_id: u32,
    func_name: String,
    input: Option<XChainKMSWorkerInput>,
}
impl XChainKMSWorker {
    pub fn new() -> Self {
        XChainKMSWorker {
            worker_id: 1,
            func_name: "xchainkms".to_string(),
            input: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct XChainKMSWorkerInput {
    method: String,
    kds: String,
    svn: u32,
    signature: String,
    timestamp: i64,
}

impl Worker for XChainKMSWorker {
    fn function_name(&self) -> &str {
        self.func_name.as_str()
    }
    fn set_id(&mut self, worker_id: u32) {
        self.worker_id = worker_id;
    }
    fn id(&self) -> u32 {
        self.worker_id
    }
    fn prepare_input(&mut self, dynamic_input: Option<String>) -> Result<()> {
        if dynamic_input.is_none() {
            return Err(Error::from(ErrorKind::InvalidInputError));
        }
	debug!("from debug?");
        self.input = serde_json::from_str(&dynamic_input.unwrap())?;
	debug!("from debug?");
        Ok(())
    }
    fn execute(&mut self, _context: WorkerContext) -> Result<String> {
	debug!("from debug?");
        let input = self
            .input
            .take()
            .ok_or_else(|| Error::from(ErrorKind::InvalidInputError))?;
        // 验证签名, 只有管理员才能操作这些接口,
	debug!("from debug?");
        let mut msg = input.method.to_owned();
        msg.push_str(input.kds.as_str());
        msg.push_str(input.svn.to_string().as_str());
        msg.push_str(input.timestamp.to_string().as_str());
        // 加上签名的过期时间
	debug!("from debug?");
	let now = SystemTime::now().get_tup().0;
        debug!("{:?} - {:?} = {:?}", now, input.timestamp, now - input.timestamp);
        if now - input.timestamp > 24 * 60 * 60 {
 	    error!("the sig expired");
            return Err(Error::from(ErrorKind::InvalidInputError));
        }

        auth::admin_require(&msg, &input.signature)?;
        match &input.method[..] {
            "init" => init_key_manager(&input.kds, input.svn),
            "mint" => mint_kds(&input.kds, input.svn),
            "inc" => inc_svn(&input.kds, input.svn),
            "dump" => dump_kds(input.svn),
            _ => Err(Error::from(ErrorKind::InvalidInputError)),
        }
    }
}

pub struct XChainTFWorker {
    worker_id: u32,
    func_name: String,
    input: Option<XChainTFWorkerInput>,
}
impl XChainTFWorker {
    pub fn new() -> Self {
        XChainTFWorker {
            worker_id: 0,
            func_name: "xchaintf".to_string(),
            input: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct XChainTFWorkerInput {
    method: String,
    args: String,
    svn: u32,
    address: String, //ecdsa address
    public_key: String,
    signature: String,
}

fn need_sig(m: &String) -> bool {
    match &m[..] {
        "encrypt" | "decrypt" | "authorize" => true,
        _ => false,
    }
}

impl Worker for XChainTFWorker {
    fn function_name(&self) -> &str {
        self.func_name.as_str()
    }
    fn set_id(&mut self, worker_id: u32) {
        self.worker_id = worker_id;
    }
    fn id(&self) -> u32 {
        self.worker_id
    }
    fn prepare_input(&mut self, dynamic_input: Option<String>) -> Result<()> {
        if dynamic_input.is_none() {
            return Err(Error::from(ErrorKind::InvalidInputError));
        }
        self.input = serde_json::from_str(&dynamic_input.unwrap())?;
        Ok(())
    }
    fn execute(&mut self, _context: WorkerContext) -> Result<String> {
        let input_req = self
            .input
            .take()
            .ok_or_else(|| Error::from(ErrorKind::InvalidInputError))?;

        //判断签名
        if need_sig(&input_req.method) {
            let mut msg = input_req.method.to_owned();
            msg.push_str(&input_req.args);
            //TODO 判断地址是否一样, 等待https://github.com/duanbing/xchain-rust-crypto上线
            check_sign(&msg, &input_req.public_key, &input_req.signature)?;
        } else {
            let input: HashMap<String, String> = serde_json::from_str(&input_req.args)?;
            check_commitment(&input, &input_req.address, "commitment", "l")?;
            check_commitment(&input, &input_req.address, "commitment2", "r")?;
        }
        input_req.run()
    }
}

enum BinaryOpType {
    ADD,
    SUB,
    MUL,
}

impl XChainTFWorkerInput {
    pub fn run(&self) -> Result<String> {
        let input_map: HashMap<String, String> = serde_json::from_str(&self.args)?;
        //---- business logic begin ---//
        let result_map = match &self.method[..] {
            "encrypt" => self.tf_encrypt(input_map),
            "authorize" => self.authorize(input_map),
            "add" => self.tf_binary(input_map, BinaryOpType::ADD),
            "sub" => self.tf_binary(input_map, BinaryOpType::SUB),
            "mul" => self.tf_binary(input_map, BinaryOpType::MUL),
            "decrypt" => self.tf_decrypt(input_map),
            _ => {
                error!("unsupported method");
                return Err(Error::from(ErrorKind::InvalidInputError));
            }
        };
        match result_map {
            Ok(x) => {
                let output_str = serde_json::to_string(&x)?;
                Ok(output_str)
            }
            Err(e) => {
                return Err(Error::from(e));
            }
        }
    }

    fn tf_encrypt(&self, input: HashMap<String, String>) -> Result<HashMap<String, String>> {
        let mut ret_map = HashMap::new();
        for (k, v) in input {
            let res = encrypt_from_kds(self.svn, &v.as_bytes(), &self.address.as_bytes())?;
            ret_map.insert(k, res);
        }
        Ok(ret_map)
    }

    fn tf_decrypt(&self, input: HashMap<String, String>) -> Result<HashMap<String, String>> {
        let mut ret_map = HashMap::new();
        for (k, v) in input {
            let decoded = match base64::decode(&v[..]) {
                Ok(x) => x,
                Err(e) => {
                    error!("base64::decode: {:?}", e);
                    return Err(Error::from(ErrorKind::InvalidInputError));
                }
            };
            let res = decrypt_from_kds(self.svn, &decoded)?;
            let base64_encoded = base64::encode(&res);
            ret_map.insert(k, base64_encoded);
        }
        Ok(ret_map)
    }

    fn cipher_to_u128(&self, op0: &String) -> Result<u128> {
        let decoded_op0 = match base64::decode(&op0[..]) {
            Ok(x) => x,
            Err(e) => {
                error!("base64::decode: {:?}", e);
                return Err(Error::from(ErrorKind::InvalidInputError));
            }
        };
        let op0_byte = decrypt_from_kds(self.svn, &decoded_op0)?;
        let op0_str = match String::from_utf8(op0_byte) {
            Ok(x) => x,
            Err(e) => {
                error!("String::from_utf8: {:?}", e);
                return Err(Error::from(ErrorKind::InvalidInputError));
            }
        };
        match op0_str.parse::<u128>() {
            Ok(x) => Ok(x),
            Err(e) => {
                error!("op0_str.parse::<u128>: {:?}", e);
                return Err(Error::from(ErrorKind::InvalidInputError));
            }
        }
    }

    fn tf_binary(
        &self,
        input: HashMap<String, String>,
        op: BinaryOpType,
    ) -> Result<HashMap<String, String>> {
        let mut ret_map = HashMap::new();
        let op0_str = match input.get(&String::from("l")) {
            Some(x) => x,
            None => {
                return Err(Error::from(ErrorKind::KeyNotFoundError));
            }
        };
        let op0 = self.cipher_to_u128(op0_str)?;
        let op1_str = match input.get(&String::from("r")) {
            Some(x) => x,
            None => {
                return Err(Error::from(ErrorKind::KeyNotFoundError));
            }
        };
        let op1 = self.cipher_to_u128(op1_str)?;
        let op3 = match op {
            BinaryOpType::ADD => op0 + op1,
            BinaryOpType::SUB => op0 - op1,
            BinaryOpType::MUL => op0 * op1,
        };
        // get output key from input and wrap the ret_map
        let op0_str = match input.get(&String::from("o")) {
            Some(x) => x,
            None => {
                return Err(Error::from(ErrorKind::KeyNotFoundError));
            }
        };
        ret_map.insert(
            String::from(op0_str),
            encrypt_from_kds(
                self.svn,
                &(op3.to_string().as_bytes()),
                &self.address.as_bytes(),
            )?,
        );
        Ok(ret_map)
    }
    /// 授权， access表示使用权，ownership是所有权
    /// 参数:  ciphertext, to_pk, access|ownership
    /// 返回： commitment: 承诺， 如果kind == ownership, 返回新的密文cipher
    pub fn authorize(&self, input: HashMap<String, String>) -> Result<HashMap<String, String>> {
        let ciphertext = match input.get(&String::from("ciphertext")) {
            Some(x) => x,
            _ => {
                return Err(Error::from(ErrorKind::KeyNotFoundError));
            }
        };
        let to_pk = match input.get(&String::from("to")) {
            Some(x) => x,
            _ => {
                return Err(Error::from(ErrorKind::KeyNotFoundError));
            }
        };
        let kind = match input.get(&String::from("kind")) {
            Some(x) => x,
            _ => {
                return Err(Error::from(ErrorKind::KeyNotFoundError));
            }
        };

        let ciphertext_slice = match base64::decode(&ciphertext[..]) {
            Ok(x) => x,
            Err(e) => {
                error!("base64::decode: {:?}", e);
                return Err(Error::from(ErrorKind::InvalidInputError));
            }
        };
        let plaintext = decrypt_from_kds(self.svn, &ciphertext_slice)?;
        //计算明文hash
        let args_hash = digest::digest(&digest::SHA256, &plaintext);

        let mut ret = HashMap::new();
        // 如果是所属权，返回回新的密文. 用被授期权的人的地址加密数据
        if kind == &String::from("ownership") {
            let newcipher = encrypt_from_kds(self.svn, &plaintext, &to_pk.as_bytes())?;
            ret.insert(String::from("cipher"), newcipher);
        } else {
            // 如果是使用权，返回授权的commitment，利用self.address生成承诺
            let commitment = calc_commitment(
                self.svn,
                args_hash.as_ref(),
                &to_pk.as_bytes(),
                &self.address.as_bytes(),
            )?;
            ret.insert(String::from("commitment"), base64::encode(&commitment));
        }
        Ok(ret)
    }
}

// 解码密文结构体，判断承诺是否正确
// TODO: 在优化指令设计之后(http://wiki.baidu.com/pages/viewpage.action?pageId=1114362384), 把承诺检测放到后面去。
fn decrypt_from_kds(svn: u32, cipher_args_slice_raw: &[u8]) -> Result<Vec<u8>> {
    let ptr: *mut KeyManagerment = LEDGER_KEY.load(Ordering::SeqCst) as *mut KeyManagerment;
    let km = unsafe { &mut (*ptr) };
    if svn > km.current_svn || !km.is_ready {
        error!("svn doesn't match,{} != {}", svn, km.current_svn);
        return Err(Error::from(ErrorKind::InvalidInputError));
    }
    let cipher_text: CipherText = match CipherText::decode(cipher_args_slice_raw) {
        Ok(x) => x,
        Err(e) => {
            error!("protobuf::parse_from_bytes::<CipherText>: {:?}", e);
            return Err(Error::from(ErrorKind::InvalidInputError));
        }
    };

    let current_kds = km.get_kds(cipher_text.svn)?;
    let plain_slice_size: usize = cipher_text.content.len() - AES_256_GCM.tag_len();
    let mut plain_slice = vec![0; plain_slice_size];
    let key = generate_aes_siv_key(&current_kds, &cipher_text.args_hash, &cipher_text.address)?;
    decryptex(&cipher_text.content, &mut plain_slice, key)?;
    Ok(plain_slice)
}

fn encrypt_from_kds(svn: u32, plain_slice: &[u8], address: &[u8]) -> Result<String> {
    let ptr: *mut KeyManagerment = LEDGER_KEY.load(Ordering::SeqCst) as *mut KeyManagerment;
    let km = unsafe { &mut (*ptr) };
    if svn > km.current_svn || !km.is_ready {
        return Err(Error::from(ErrorKind::InvalidInputError));
    }

    let args_hash = digest::digest(&digest::SHA256, plain_slice);
    let current_kds = km.get_kds(svn)?;
    let real_key = generate_aes_siv_key(&current_kds, args_hash.as_ref(), address)?;
    let output_slice_len = plain_slice.len() + AES_256_GCM.tag_len();
    let mut cipher1 = vec![0; output_slice_len];
    if let Err(e) = encryptex(plain_slice, cipher1.as_mut_slice(), real_key) {
        error!(
            "encryptex failed, data : {:?}, key : {:?}, error: {:?}",
            plain_slice, real_key, e
        );
        return Err(Error::from(ErrorKind::InvalidInputError));
    }
    // 默认数据都是给自己
    let ct = CipherText {
        svn: svn,
        address: address.to_vec(),
        args_hash: args_hash.as_ref().to_vec(),
        content: cipher1,
    };

    let mut cipher = Vec::new();
    if let Err(e) = ct.encode(&mut cipher) {
        error!("base64::decode: {:?}", e);
        return Err(Error::from(ErrorKind::InvalidInputError));
    };
    Ok(base64::encode(&cipher))
}

lazy_static! {
    static ref LEDGER_KEY: AtomicPtr<()> = {
        let ptr: *mut KeyManagerment = Box::into_raw(Box::new(KeyManagerment::new()));
    let km = unsafe {&mut (*ptr)};
    // 从文件里面读出kds和svn
    let mut data = [0_u8; 2048];
        let mut file = match SgxFile::open(DEFAULT_KEY_PATH) {
        Ok(x) => x,
        Err(e) => {
             trace!("failed to read kms.key, when lazy intializing, usually happens when admin calls the init_key_manager: {:?}", e);
             return AtomicPtr::new(0 as *mut ());
        }
        };
        let read_size = file.read(&mut data).expect("read file error");
        trace!("read_size = {:?}", read_size);
        km.unseal_keys(data.as_mut_ptr(), 2048).expect("unseal keys error");
        trace!("lazy_static init: svn = {:?}, kds_size = {:?}", km.current_svn, km.kds_map.len());
        //LEDGER_KEY.store(ptr as *mut (), Ordering::SeqCst);
        AtomicPtr::new(ptr as *mut ())
    };
}

pub fn dump_kds(svn: u32) -> Result<String> {
    let ptr: *mut KeyManagerment = LEDGER_KEY.load(Ordering::SeqCst) as *mut KeyManagerment;
    if ptr.is_null() {
        return Err(Error::from(ErrorKind::InvalidInputError));
    }
    let km = unsafe { &mut (*ptr) };
    if km.current_svn != svn {
        error!("svn doesn't match when dumping");
        return Err(Error::from(ErrorKind::InvalidInputError));
    }
    let mut seal_log = [0u8; 2048];
    let _rsize = km.seal_keys_for_serializable(seal_log.as_mut_ptr(), 2048)?;
    let mut file = SgxFile::create(DEFAULT_KEY_PATH)?;
    let data = seal_log; 
    let write_size = file.write(&data)?;
    debug!("dump done: write_size = {:?}", write_size);
    Ok(String::from("done"))
}

// inc_svn is a test function
pub fn inc_svn(kds_str: &String, svn: u32) -> Result<String> {
    let ptr: *mut KeyManagerment = LEDGER_KEY.load(Ordering::SeqCst) as *mut KeyManagerment;
    if ptr.is_null() {
        return Err(Error::from(ErrorKind::InvalidInputError));
    }
    let km = unsafe { &mut (*ptr) };
    if svn != km.current_svn + 1 || !km.is_ready {
        error!(
            "inc_svn svn doesn't match,{} != {}, km={:?}",
            svn, km.current_svn, km
        );
        return Err(Error::from(ErrorKind::InvalidInputError));
    }

    let mut t = sgx_hmac_256bit_key_t::default();
    let kds_slice = match hex::decode(kds_str) {
        Ok(x) => x,
        Err(e) => {
            error!("base64::decode: {:?}", e);
            return Err(Error::from(ErrorKind::InvalidInputError));
        }
    };
    t.copy_from_slice(&kds_slice);

    let deriviated_kds = generate_previous_svn_kds(&t, svn)?;
    let oldkds = km.get_kds(km.current_svn)?;

    if oldkds != deriviated_kds {
        error!(
            "inc_svn: {:?} doesn't equal {:?}",
            hex::encode(deriviated_kds),
            hex::encode(oldkds)
        );
        return Err(Error::from(ErrorKind::InvalidInputError));
    }
    km.kds_map.insert(svn, t);
    km.current_svn = svn;
    Ok(String::from("inc done"))
}

///  generates all the kds by base key derivation secret and svn
/// ```
/// kds: bds(0)  ->  kds(1)  -> kds(2) -> ... -> kds(2^16-1)
/// svn: 2^16-1  ->  2^16-2  ->  ...   -> ... ->  0
/// ```
pub fn mint_kds(bds: &String, dest_svn: u32) -> Result<String> {
    let max_svn = 2_u32.pow(16);
    if max_svn <= dest_svn {
        error!("mint_kds: dest svn is greater than max_svn");
        return Err(Error::from(ErrorKind::InvalidInputError));
    }
    let mut kds = match hex::decode(bds) {
        Ok(x) => x,
        Err(e) => {
            error!("base64::decode: {:?}", e);
            return Err(Error::from(ErrorKind::InvalidInputError));
        }
    };
    for i in ((dest_svn + 1)..max_svn).rev() {
        let prev2 = generate_previous_svn_kds(&kds, i)?;
        kds.copy_from_slice(&prev2);
    }
    let hex_str = hex::encode(kds);
    Ok(hex_str)
}

pub fn init_key_manager(kds_str: &String, svn: u32) -> Result<String> {
    let ptr = LEDGER_KEY.load(Ordering::SeqCst);
    if ptr.is_null() {
        let ptr: *mut KeyManagerment = Box::into_raw(Box::new(KeyManagerment::new()));
        LEDGER_KEY.store(ptr as *mut (), Ordering::SeqCst);
    }
    let ptr: *mut KeyManagerment = LEDGER_KEY.load(Ordering::SeqCst) as *mut KeyManagerment;
    let km = unsafe { &mut (*ptr) };
    if km.is_ready {
        return Ok(km.current_svn.to_string());
    }
    let mut t = sgx_hmac_256bit_key_t::default();
    let kds_slice = match hex::decode(kds_str) {
        Ok(x) => x,
        Err(e) => {
            error!("base64::decode: {:?}", e);
            return Err(Error::from(ErrorKind::InvalidInputError));
        }
    };
    t.copy_from_slice(&kds_slice);

    km.is_ready = true;
    km.kds_map.insert(svn, t);
    Ok(String::from(km.current_svn.to_string()))
}

// addr生成自己to的承诺
fn calc_commitment(svn: u32, msg: &[u8], to: &[u8], addr: &[u8]) -> Result<Vec<u8>> {
    let mut msg = msg.clone().to_vec();
    msg.append(&mut to.clone().to_vec());
    let commitment = encrypt_from_kds(svn, &msg, addr)?;
    //TODO hmac修改为ring::hmac
    let hmac_value = derive_32bytes_key_from_double_hmac_sha_256(
        &[0],
        HMAC_1ST_DERIVATION_LABEL.as_bytes(),
        &svn.to_be_bytes(),
        HMAC_2ND_DERIVATION_LABEL.as_bytes(),
        &commitment.as_bytes(),
    )?;
    Ok(hmac_value.to_vec())
}

fn check_commitment(
    input: &HashMap<String, String>,
    user_str: &String,
    c: &str,
    l: &str,
) -> Result<()> {
    let commitment_str = input
        .get(&String::from(c))
        .ok_or(Error::from(ErrorKind::KeyNotFoundError))?;
    let ciphertext = match input.get(&String::from(l)) {
        Some(x) => x,
        _ => {
            return Err(Error::from(ErrorKind::KeyNotFoundError));
        }
    };
    let ciphertext_slice = match base64::decode(&ciphertext[..]) {
        Ok(x) => x,
        Err(e) => {
            error!("base64::decode: {:?}", e);
            return Err(Error::from(ErrorKind::KeyNotFoundError));
        }
    };
    // get from
    let cipher_text: CipherText = match CipherText::decode(ciphertext_slice) {
        Ok(x) => x,
        Err(e) => {
            error!("protobuf::parse_from_bytes::<CipherText>: {:?}", e);
            return Err(Error::from(ErrorKind::InvalidInputError));
        }
    };
    // 如果数据是自己的，直接验证通过
    if  cipher_text.address == user_str.as_bytes() {
        return Ok(());
    }
    //生成承诺，并且对比
    let commitment = calc_commitment(
        cipher_text.svn,
        &cipher_text.args_hash,
        &user_str.as_bytes(),
        &cipher_text.address,
    )?;
    if base64::encode(&commitment).as_bytes() != commitment_str.as_bytes() {
        error!("permission denied");
        return Err(Error::from(ErrorKind::InvalidInputError));
    }
    Ok(())
}
