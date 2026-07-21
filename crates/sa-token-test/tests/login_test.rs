//! 登录集成测试
//!
//! 验证 Sa-Token-Rs 的核心登录流程：
//! StpUtil::login("10001") → is_login → get_login_id → logout

use sa_token::prelude::*;

/// 初始化测试环境
fn setup() {
    SaManager::reset();
    SaManager::set_config(Arc::new(SaTokenConfig::default()));
    SaManager::set_sa_token_dao(Arc::new(SaTokenDaoMemory::new()));
    SaTokenContextMockUtil::set_mock_context();
    SaManager::put_stp_logic(Arc::new(StpLogic::new("login")));
}

/// 测试登录和登录状态
#[test]
fn test_login_and_is_login() {
    setup();

    // 登录前未登录
    assert!(!StpUtil::is_login());

    // 登录
    StpUtil::login("10001").unwrap();

    // 登录后已登录
    assert!(StpUtil::is_login());
    assert_eq!(StpUtil::get_login_id().unwrap(), "10001");

    // 登出
    StpUtil::logout().unwrap();

    // 登出后未登录
    assert!(!StpUtil::is_login());
}

/// 测试获取登录 ID
#[test]
fn test_get_login_id() {
    setup();

    // 未登录时获取登录 ID 应返回错误
    assert!(StpUtil::get_login_id().is_err());

    // 登录后获取登录 ID
    StpUtil::login("10001").unwrap();
    assert_eq!(StpUtil::get_login_id_as_string().unwrap(), "10001");
    assert_eq!(StpUtil::get_login_id_as_i64().unwrap(), 10001);

    // 未登录时返回 None
    StpUtil::logout().unwrap();
    assert!(StpUtil::get_login_id_default_null().is_none());
}

/// 测试 Token 获取
#[test]
fn test_token_value() {
    setup();

    // 登录
    StpUtil::login("10001").unwrap();

    // 获取 Token
    let token = StpUtil::get_token_value();
    assert!(token.is_some());
    assert!(!token.unwrap().is_empty());

    // 获取 Token 详情
    let info = StpUtil::get_token_info().unwrap();
    assert_eq!(info.login_id, "10001");
    assert!(!info.token_value.is_empty());
}

/// 测试设备类型登录
#[test]
fn test_login_with_device() {
    setup();

    // 指定设备类型登录
    StpUtil::login_with_device("10001", "PC").unwrap();

    // 获取设备类型
    assert_eq!(StpUtil::get_login_device_type().unwrap(), "PC");
}

/// 测试踢人下线
#[test]
fn test_kickout() {
    setup();

    // 登录
    StpUtil::login("10001").unwrap();
    assert!(StpUtil::is_login());

    // 踢人下线
    StpUtil::kickout("10001").unwrap();
    assert!(!StpUtil::is_login());
}

/// 测试顶替下线
#[test]
fn test_replaced() {
    setup();

    // 登录
    StpUtil::login("10001").unwrap();
    assert!(StpUtil::is_login());

    // 顶替下线
    StpUtil::replaced("10001").unwrap();
    assert!(!StpUtil::is_login());
}

/// 测试会话
#[test]
fn test_session() {
    setup();

    // 登录
    StpUtil::login("10001").unwrap();

    // 获取会话
    let session = StpUtil::get_session().unwrap();
    assert_eq!(session.login_id(), "10001");

    // 设置数据
    let mut session = session;
    session.set("key1", serde_json::json!("value1"));
    assert_eq!(session.get("key1"), Some(&serde_json::json!("value1")));
}

/// 测试 Token 超时
#[test]
fn test_token_timeout() {
    setup();

    // 登录
    StpUtil::login("10001").unwrap();

    // 获取 Token 超时
    let timeout = StpUtil::get_token_timeout();
    assert!(timeout > 0);
}

/// 测试续签
#[test]
fn test_renew_timeout() {
    setup();

    // 登录
    StpUtil::login("10001").unwrap();

    // 续签
    StpUtil::renew_timeout(60 * 60).unwrap();
}

/// 测试多设备登录
#[test]
fn test_multi_device() {
    setup();

    // 使用唯一 ID 避免与其他测试冲突
    let user_id = "multi_device_user";

    // 设备 1 登录
    StpUtil::login_with_device(user_id, "PC").unwrap();
    let token1 = StpUtil::get_token_value().unwrap();

    // 设备 2 登录
    StpUtil::login_with_device(user_id, "Mobile").unwrap();
    let token2 = StpUtil::get_token_value().unwrap();

    // 两个 Token 不同
    assert_ne!(token1, token2);

    // 获取终端列表
    let terminals = StpUtil::get_terminal_list_by_login_id(user_id).unwrap();
    assert_eq!(terminals.len(), 2);
}

/// 测试 check_login
#[test]
fn test_check_login() {
    setup();

    // 未登录时 check_login 应返回错误
    let result = StpUtil::check_login();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), SaTokenException::NotLogin { .. }));

    // 登录后 check_login 应通过
    StpUtil::login("10001").unwrap();
    assert!(StpUtil::check_login().is_ok());
}

/// 测试按 Token 注销
#[test]
fn test_logout_by_token() {
    setup();

    // 登录
    StpUtil::login("10001").unwrap();
    let token = StpUtil::get_token_value().unwrap();

    // 按 Token 注销
    StpUtil::logout_by_token_value(&token).unwrap();
    assert!(!StpUtil::is_login());
}

/// 测试 SaManager 初始化
#[test]
fn test_manager_init() {
    SaManager::reset();
    SaManager::init_defaults();

    // 验证默认组件已初始化
    assert!(SaManager::get_stp_logic("login").is_some());
    // DAO 可能已被其他测试设置，这里只验证 StpLogic 已初始化
}
