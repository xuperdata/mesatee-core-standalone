#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use super::crypto_helper;
use mesatee_core::{Result};

/// 管理员的公钥写死在程序内部,随着软件发布
/// 未压缩状态的公钥格式: 4:x:y
/// D: ea07ded1156e152ef8615661581cf73495c33b431f3fbe372f57370dc80b375b,
/// X: 0bf4ab3b2918fd62ac0f7a718c24f68e7f31c44d4f874580eab031619aeb0fe2,
/// Y: 9471bf2a52ecf14cbcadc1d5d65188d25bb9a274f5dcf44e460e4e364c6b1c94,
/// public key: 040bf4ab3b2918fd62ac0f7a718c24f68e7f31c44d4f874580eab031619aeb0fe29471bf2a52ecf14cbcadc1d5d65188d25bb9a274f5dcf44e460e4e364c6b1c94,
static ADMIN_PUBLIC_KEY: &str = "040bf4ab3b2918fd62ac0f7a718c24f68e7f31c44d4f874580eab031619aeb0fe29471bf2a52ecf14cbcadc1d5d65188d25bb9a274f5dcf44e460e4e364c6b1c94";

/// 验证管理员的签名
pub fn admin_require(msg: &String, sig: &String) -> Result<()> {
    crypto_helper::check_sign(msg, &ADMIN_PUBLIC_KEY.to_string(), sig)
}
