//! 加密工具（对应 Java `cn.dev33.satoken.secure.SaSecureUtil`）。
//!
//! 提供 MD5/SHA 系列摘要、AES 对称加密、RSA 非对称加密。

use cbc::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};
use md5::{Digest as Md5Digest, Md5};
use openssl::encrypt::{Decrypter, Encrypter};
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::rsa::Padding;
use sha1::Sha1;
use sha2::{Sha256, Sha384, Sha512};

use crate::exception::{SaResult, SaTokenException};

use super::sa_base64_util::SaBase64Util;

type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

/// 加密工具
pub struct SaSecureUtil;

impl SaSecureUtil {
    // ----------- 摘要算法 -----------

    /// MD5 摘要
    pub fn md5(s: &str) -> String {
        let mut hasher = Md5::new();
        hasher.update(s.as_bytes());
        hex_lower(&hasher.finalize())
    }

    /// MD5 加盐摘要
    pub fn md5_by_salt(salt: &str, s: &str) -> String {
        Self::md5(&format!("{salt}{s}"))
    }

    /// SHA1
    pub fn sha1(s: &str) -> String {
        let mut hasher = Sha1::new();
        hasher.update(s.as_bytes());
        hex_lower(&hasher.finalize())
    }

    /// SHA256
    pub fn sha256(s: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(s.as_bytes());
        hex_lower(&hasher.finalize())
    }

    /// SHA256 加盐
    pub fn sha256_by_salt(salt: &str, s: &str) -> String {
        Self::sha256(&format!("{salt}{s}"))
    }

    /// SHA384
    pub fn sha384(s: &str) -> String {
        let mut hasher = Sha384::new();
        hasher.update(s.as_bytes());
        hex_lower(&hasher.finalize())
    }

    /// SHA512
    pub fn sha512(s: &str) -> String {
        let mut hasher = Sha512::new();
        hasher.update(s.as_bytes());
        hex_lower(&hasher.finalize())
    }

    // ----------- AES -----------

    /// AES 加密（Base64 输出），key 必须是 16 字节
    pub fn aes_encrypt(key: &str, text: &str) -> SaResult<String> {
        let key_bytes: [u8; 16] =
            key.as_bytes()
                .try_into()
                .map_err(|_| SaTokenException::Other {
                    message: "AES key 必须是 16 字节".into(),
                })?;
        let iv: [u8; 16] = key_bytes; // IV = key（CBC 模式约定）
        let cipher = Aes128CbcEnc::new(&key_bytes.into(), &iv.into());
        // 预留 block_size (16) 字节给 PKCS7 padding
        let mut buf = text.as_bytes().to_vec();
        buf.resize(buf.len() + 16, 0);
        let result = cipher
            .encrypt_padded_mut::<Pkcs7>(&mut buf, text.len())
            .map_err(|e| SaTokenException::Other {
                message: format!("AES 加密失败: {e}"),
            })?;
        Ok(SaBase64Util::encode(result))
    }

    /// AES 解密
    pub fn aes_decrypt(key: &str, ciphertext_b64: &str) -> SaResult<String> {
        let key_bytes: [u8; 16] =
            key.as_bytes()
                .try_into()
                .map_err(|_| SaTokenException::Other {
                    message: "AES key 必须是 16 字节".into(),
                })?;
        let iv: [u8; 16] = key_bytes;
        let cipher = Aes128CbcDec::new(&key_bytes.into(), &iv.into());
        let mut ciphertext =
            SaBase64Util::decode(ciphertext_b64).map_err(|e| SaTokenException::Other {
                message: format!("Base64 解码失败: {e}"),
            })?;
        let plaintext = cipher
            .decrypt_padded_mut::<Pkcs7>(&mut ciphertext)
            .map_err(|e| SaTokenException::Other {
                message: format!("AES 解密失败: {e}"),
            })?;
        String::from_utf8(plaintext.to_vec()).map_err(|e| SaTokenException::Other {
            message: format!("UTF-8 解码失败: {e}"),
        })
    }

    // ----------- RSA -----------

    /// RSA 公钥加密（OAEP 填充）
    pub fn rsa_encrypt_by_public(public_key_pem: &str, content: &str) -> SaResult<String> {
        let key = PKey::public_key_from_pem(public_key_pem.as_bytes())
            .map_err(|error| crypto_error("公钥解析失败", error))?;
        let mut encrypter =
            Encrypter::new(&key).map_err(|error| crypto_error("RSA 加密器初始化失败", error))?;
        configure_oaep(&mut encrypter)?;
        let mut encrypted = vec![
            0;
            encrypter.encrypt_len(content.as_bytes()).map_err(|error| {
                crypto_error("RSA 加密缓冲区计算失败", error)
            })?
        ];
        let length = encrypter
            .encrypt(content.as_bytes(), &mut encrypted)
            .map_err(|error| crypto_error("RSA 加密失败", error))?;
        encrypted.truncate(length);
        Ok(SaBase64Util::encode(&encrypted))
    }

    /// RSA 私钥加密（不支持 —— 请改用公钥加密 + 私钥解密）
    pub fn rsa_encrypt_by_private(_private_key_pem: &str, _content: &str) -> SaResult<String> {
        Err(SaTokenException::Other {
            message: "私钥加密不是受支持的机密性协议，请改用公钥加密或标准数字签名".into(),
        })
    }

    /// RSA 公钥解密（不支持）
    pub fn rsa_decrypt_by_public(_public_key_pem: &str, _ciphertext_b64: &str) -> SaResult<String> {
        Err(SaTokenException::Other {
            message: "公钥解密不是受支持的机密性协议，请改用私钥解密或标准数字签名验证".into(),
        })
    }

    /// RSA 私钥解密（OAEP 填充）
    pub fn rsa_decrypt_by_private(private_key_pem: &str, ciphertext_b64: &str) -> SaResult<String> {
        let key = PKey::private_key_from_pem(private_key_pem.as_bytes())
            .map_err(|error| crypto_error("私钥解析失败", error))?;
        let ciphertext =
            SaBase64Util::decode(ciphertext_b64).map_err(|e| SaTokenException::Other {
                message: format!("Base64 解码失败: {e}"),
            })?;
        let mut decrypter =
            Decrypter::new(&key).map_err(|error| crypto_error("RSA 解密器初始化失败", error))?;
        configure_oaep(&mut decrypter)?;
        let mut decrypted = vec![
            0;
            decrypter.decrypt_len(&ciphertext).map_err(|error| {
                crypto_error("RSA 解密缓冲区计算失败", error)
            })?
        ];
        let length = decrypter
            .decrypt(&ciphertext, &mut decrypted)
            .map_err(|error| crypto_error("RSA 解密失败", error))?;
        decrypted.truncate(length);
        String::from_utf8(decrypted).map_err(|e| SaTokenException::Other {
            message: format!("UTF-8 解码失败: {e}"),
        })
    }

    /// 从 PEM 字符串中提取 RSA 公钥 n,e
    pub fn rsa_public_key_components(public_key_pem: &str) -> SaResult<(Vec<u8>, Vec<u8>)> {
        let key = PKey::public_key_from_pem(public_key_pem.as_bytes())
            .map_err(|error| crypto_error("公钥解析失败", error))?;
        let rsa = key
            .rsa()
            .map_err(|error| crypto_error("公钥不是 RSA 密钥", error))?;
        Ok((rsa.n().to_vec(), rsa.e().to_vec()))
    }
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

fn configure_oaep<T>(crypt: &mut T) -> SaResult<()>
where
    T: RsaOaepConfig,
{
    crypt
        .set_rsa_padding(Padding::PKCS1_OAEP)
        .and_then(|()| crypt.set_rsa_oaep_md(MessageDigest::sha256()))
        .and_then(|()| crypt.set_rsa_mgf1_md(MessageDigest::sha256()))
        .map_err(|error| crypto_error("RSA OAEP-SHA256 配置失败", error))
}

trait RsaOaepConfig {
    fn set_rsa_padding(&mut self, padding: Padding) -> Result<(), openssl::error::ErrorStack>;
    fn set_rsa_oaep_md(&mut self, digest: MessageDigest) -> Result<(), openssl::error::ErrorStack>;
    fn set_rsa_mgf1_md(&mut self, digest: MessageDigest) -> Result<(), openssl::error::ErrorStack>;
}

impl RsaOaepConfig for Encrypter<'_> {
    fn set_rsa_padding(&mut self, padding: Padding) -> Result<(), openssl::error::ErrorStack> {
        Encrypter::set_rsa_padding(self, padding)
    }

    fn set_rsa_oaep_md(&mut self, digest: MessageDigest) -> Result<(), openssl::error::ErrorStack> {
        Encrypter::set_rsa_oaep_md(self, digest)
    }

    fn set_rsa_mgf1_md(&mut self, digest: MessageDigest) -> Result<(), openssl::error::ErrorStack> {
        Encrypter::set_rsa_mgf1_md(self, digest)
    }
}

impl RsaOaepConfig for Decrypter<'_> {
    fn set_rsa_padding(&mut self, padding: Padding) -> Result<(), openssl::error::ErrorStack> {
        Decrypter::set_rsa_padding(self, padding)
    }

    fn set_rsa_oaep_md(&mut self, digest: MessageDigest) -> Result<(), openssl::error::ErrorStack> {
        Decrypter::set_rsa_oaep_md(self, digest)
    }

    fn set_rsa_mgf1_md(&mut self, digest: MessageDigest) -> Result<(), openssl::error::ErrorStack> {
        Decrypter::set_rsa_mgf1_md(self, digest)
    }
}

fn crypto_error(context: &str, error: openssl::error::ErrorStack) -> SaTokenException {
    SaTokenException::Other {
        message: format!("{context}: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use openssl::rsa::Rsa;

    #[test]
    fn md5_known() {
        assert_eq!(
            SaSecureUtil::md5("123456"),
            "e10adc3949ba59abbe56e057f20f883e"
        );
    }

    #[test]
    fn sha256_known() {
        assert_eq!(
            SaSecureUtil::sha256("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn aes_roundtrip() {
        let key = "1234567890123456";
        let plain = "hello, sa-token";
        let enc = SaSecureUtil::aes_encrypt(key, plain).unwrap();
        let dec = SaSecureUtil::aes_decrypt(key, &enc).unwrap();
        assert_eq!(dec, plain);
    }

    #[test]
    fn rsa_roundtrip() {
        let rsa = Rsa::generate(2048).expect("generate RSA key");
        let key = PKey::from_rsa(rsa).expect("wrap RSA key");
        let pk_pem = String::from_utf8(key.public_key_to_pem().expect("encode public key"))
            .expect("public key PEM is UTF-8");
        let sk_pem = String::from_utf8(key.private_key_to_pem_pkcs8().expect("encode private key"))
            .expect("private key PEM is UTF-8");

        let plain = "hello rsa";
        let enc = SaSecureUtil::rsa_encrypt_by_public(&pk_pem, plain).unwrap();
        let dec = SaSecureUtil::rsa_decrypt_by_private(&sk_pem, &enc).unwrap();
        assert_eq!(dec, plain);
    }

    #[test]
    fn rsa_private_encrypt_unsupported() {
        let res = SaSecureUtil::rsa_encrypt_by_private(
            "-----BEGIN PRIVATE KEY-----\n-----END PRIVATE KEY-----",
            "x",
        );
        assert!(res.is_err());
    }
}
