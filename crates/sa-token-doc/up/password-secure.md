# 密码加密

> Sa-Token → Sa-Token-Rs。模块：`SaSecureUtil` / `SaBase64Util` / `BCrypt` 等（`sa_token_core::secure`）。

| Java | Rust |
|---|---|
| `SaSecureUtil.md5` | `SaSecureUtil::md5` |
| `SaSecureUtil.aesEncrypt` | `SaSecureUtil::aes_encrypt` |
| `SaBase64Util.encode` | `SaBase64Util::encode` |
| `BCrypt.hashpw` | `BCrypt::hashpw`（以实现为准） |

严格来讲，密码加密不属于 [权限认证] 的范畴，但是对于大多数系统来讲，密码加密又是安全认证不可或缺的部分，
所以，应大家要求，`Sa-Token-Rs` 同样提供密码加密模块，该模块非常简单，仅仅封装了一些常见的加密算法。



### 摘要加密
md5、sha1、sha256
``` rust
use sa_token::prelude::*; // 或 sa_token_core::secure::SaSecureUtil

// md5加密
SaSecureUtil::md5("123456");

// sha1加密
SaSecureUtil::sha1("123456");

// sha256加密
SaSecureUtil::sha256("123456");
```


### 对称加密
AES加密
``` rust
use sa_token_core::secure::SaSecureUtil;

// 定义秘钥和明文（Rust 实现要求 key 为 16 字节）
let key = "1234567890123456";
let text = "Sa-Token-Rs 一个轻量级 Rust 权限认证框架";

// 加密
let ciphertext = SaSecureUtil::aes_encrypt(key, text)?;
println!("AES加密后：{ciphertext}");

// 解密
let text2 = SaSecureUtil::aes_decrypt(key, &ciphertext)?;
println!("AES解密后：{text2}");
```

附：Java 原版内部密钥生成策略（SHA1PRNG）便于其它语言对接；Rust 侧当前以 **16 字节原始 key** 为准，跨语言对接时请确认双方算法与 padding 一致。

```java
    private static SecretKeySpec getSecretKey(final String password) throws NoSuchAlgorithmException {
        KeyGenerator kg = KeyGenerator.getInstance("AES");
        SecureRandom random = SecureRandom.getInstance("SHA1PRNG");
        random.setSeed(password.getBytes());
        kg.init(128, random);
        SecretKey secretKey = kg.generateKey();
        return new SecretKeySpec(secretKey.getEncoded(), "AES");
    }
```


### 非对称加密
~~RSA加密(已过时)~~
``` rust
use sa_token_core::secure::SaSecureUtil;

// 定义私钥和公钥（PEM）
let private_key = "-----BEGIN PRIVATE KEY-----\n...\n-----END PRIVATE KEY-----";
let public_key = "-----BEGIN PUBLIC KEY-----\n...\n-----END PUBLIC KEY-----";
let text = "Sa-Token-Rs 一个轻量级 Rust 权限认证框架";

// 使用公钥加密
let ciphertext = SaSecureUtil::rsa_encrypt_by_public(public_key, text)?;
println!("公钥加密后：{ciphertext}");

// 使用私钥解密
let text2 = SaSecureUtil::rsa_decrypt_by_private(private_key, &ciphertext)?;
println!("私钥解密后：{text2}");
```

你可能会有疑问，私钥和公钥这么长的一大串，我怎么弄出来，手写吗？当然不是，可用 openssl / 其它工具生成；Java 原版：
``` java
System.out.println(SaSecureUtil.rsaGenerateKeyPair());
```

### Base64编码与解码
``` rust
use sa_token_core::secure::SaBase64Util;

let text = "Sa-Token-Rs 一个轻量级 Rust 权限认证框架";

// 使用Base64编码
let base64_text = SaBase64Util::encode(text.as_bytes());
println!("Base64编码后：{base64_text}");

// 使用Base64解码
let bytes = SaBase64Util::decode(&base64_text)?;
let text2 = String::from_utf8(bytes)?;
println!("Base64解码后：{text2}");
```


### Base32编码与解码
``` rust
// 若已移植 SaBase32Util：
// let base32_text = SaBase32Util::encode(text);
// let text2 = SaBase32Util::decode(&base32_text)?;
```

Java 对照：
``` java
String base32Text = SaBase32Util.encode(text);
String text2 = SaBase32Util.decode(base32Text);
```


### TOTP 验证器

``` rust
use sa_token_core::secure::totp::SaTotpUtil;

// 1、生成密钥
let secret_key = SaTotpUtil::generate_secret_key();
println!("TOTP 秘钥: {secret_key}");

// 2、生成扫码字符串（account；若需绑定已有密钥请查阅 SaTotpTemplate）
let qr_string = SaTotpUtil::generate_google_secret_key("zhangsan");
println!("扫码字符串: {qr_string}");

// 3、计算当前 TOTP 码
let code = SaTotpUtil::generate_totp(&secret_key);
println!("当前时间戳对应的 TOTP 码: {code}");

// 4、验证用户输入
let is_valid = SaTotpUtil::validate_totp(&secret_key, &code, 1);
println!("验证结果: {is_valid}");
```

在线 TOTP 管理器推荐： [TOTP 密码生成管理 - 工具哇](https://toolwa.com/totp/)


### BCrypt加密
由它加密的文件可在所有支持的操作系统和处理器上进行转移

它的口令必须是8至56个字符，并将在内部被转化为448位的密钥

> 此类来自于https://github.com/jeremyh/jBCrypt/；Rust 侧见 `sa_token_core::secure::bcrypt::BCrypt`
``` rust
use sa_token_core::secure::bcrypt::BCrypt;

// 使用方法
let pw_hash = BCrypt::hashpw(plain_password, &BCrypt::gensalt());

// 使用 checkpw 方法检查被加密的字符串是否与原始字符串匹配：
let ok = BCrypt::checkpw(candidate_password, &pw_hash);

// gensalt 方法提供了可选参数 (cost) 来定义加盐多少，也决定了加密的复杂度:
let strong_salt = BCrypt::gensalt_with_cost(10);
let stronger_salt = BCrypt::gensalt_with_cost(12);
```


<br>

如需更多加密算法，可参考 Rust 生态：`ring`、`openssl`、`argon2` 等。


---

<a class="case-btn" href="https://github.com/sa-token-rust/sa-token-rs/tree/main/crates/sa-token-demo"
	target="_blank">
	本章代码示例：Sa-Token-Rs 密码加密 —— 参考 core tests / Secure 相关 demo
</a>
