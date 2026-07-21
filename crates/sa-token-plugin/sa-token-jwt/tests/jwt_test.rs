//! JWT 插件测试

use sa_token_jwt::{JwtConfig, SaJwtTemplate};

/// 测试 JWT Token 生成与解析
#[test]
fn test_jwt_create_and_parse() {
    let config = JwtConfig::new("test-secret-key");
    let jwt = SaJwtTemplate::new(config);

    // 生成 Token
    let token = jwt.create_token("10001", "login", 3600).unwrap();
    assert!(!token.is_empty());

    // 解析 Token
    let claims = jwt.parse_token(&token).unwrap();
    assert_eq!(claims.sub, "10001");
    assert_eq!(claims.login_type, "login");
}

/// 测试 JWT Token 验证
#[test]
fn test_jwt_verify() {
    let config = JwtConfig::new("test-secret-key");
    let jwt = SaJwtTemplate::new(config);

    // 生成 Token
    let token = jwt.create_token("10001", "login", 3600).unwrap();

    // 验证 Token
    assert!(jwt.verify_token(&token));

    // 获取 login_id
    assert_eq!(jwt.get_login_id(&token).unwrap(), "10001");
}

/// 测试 JWT Token 过期
#[test]
fn test_jwt_expired() {
    let config = JwtConfig::new("test-secret-key");
    let jwt = SaJwtTemplate::new(config);

    // 生成已过期的 Token（timeout = 1，1秒后过期）
    let token = jwt.create_token("10001", "login", 1).unwrap();

    // 等待 Token 过期
    std::thread::sleep(std::time::Duration::from_secs(2));

    // 检查是否过期
    assert!(jwt.is_expired(&token));
}

/// 测试 JWT Token 永不过期
#[test]
fn test_jwt_never_expire() {
    let config = JwtConfig::new("test-secret-key");
    let jwt = SaJwtTemplate::new(config);

    // 生成永不过期的 Token（timeout = 0）
    let token = jwt.create_token("10001", "login", 0).unwrap();

    // 检查是否过期（应该不过期）
    assert!(!jwt.is_expired(&token));
}

/// 测试 JWT Token 带扩展数据
#[test]
fn test_jwt_with_extra() {
    let config = JwtConfig::new("test-secret-key");
    let jwt = SaJwtTemplate::new(config);

    // 生成带扩展数据的 Token
    let mut extra = std::collections::HashMap::new();
    extra.insert("role".to_string(), serde_json::json!("admin"));
    extra.insert("tenant_id".to_string(), serde_json::json!("t001"));

    let token = jwt
        .create_token_with_extra("10001", "login", 3600, Some(extra))
        .unwrap();

    // 解析 Token
    let claims = jwt.parse_token(&token).unwrap();
    assert_eq!(claims.sub, "10001");

    // 检查扩展数据
    let extra = claims.extra.unwrap();
    assert_eq!(extra.get("role").unwrap(), &serde_json::json!("admin"));
    assert_eq!(extra.get("tenant_id").unwrap(), &serde_json::json!("t001"));
}

/// 测试 JWT Token 签发者和受众
#[test]
fn test_jwt_issuer_audience() {
    let config = JwtConfig::new("test-secret-key")
        .with_issuer("sa-token")
        .with_audience("my-app");
    let jwt = SaJwtTemplate::new(config);

    // 生成 Token
    let token = jwt.create_token("10001", "login", 3600).unwrap();

    // 解析 Token
    let claims = jwt.parse_token(&token).unwrap();
    assert_eq!(claims.iss.unwrap(), "sa-token");
    assert_eq!(claims.aud.unwrap(), "my-app");
}

/// 测试 JWT Token 无效签名
#[test]
fn test_jwt_invalid_signature() {
    let config = JwtConfig::new("test-secret-key");
    let jwt = SaJwtTemplate::new(config);

    // 生成 Token
    let token = jwt.create_token("10001", "login", 3600).unwrap();

    // 用不同的密钥解析
    let config2 = JwtConfig::new("wrong-secret-key");
    let jwt2 = SaJwtTemplate::new(config2);

    // 应该失败
    assert!(jwt2.parse_token(&token).is_err());
}
