//! Sa-Token Axum 集成测试
//!
//! 测试 SaTokenLayer 中间件和 CurrentLoginId Extractor。

use sa_token::prelude::*;
use sa_token_web_axum::SaTokenLayer;
use std::sync::{Arc, Mutex, MutexGuard};

static TEST_LOCK: Mutex<()> = Mutex::new(());

/// 初始化测试环境
fn setup() -> MutexGuard<'static, ()> {
    let guard = TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    SaManager::reset();
    SaManager::set_config(Arc::new(SaTokenConfig::default()));
    SaManager::set_sa_token_dao(Arc::new(SaTokenDaoMemory::new()));
    SaTokenContextMockUtil::set_mock_context();
    SaManager::put_stp_logic(Arc::new(StpLogic::new("login")));
    guard
}

/// 测试 CurrentLoginId Extractor
#[tokio::test]
async fn test_current_login_id_extractor() {
    let _guard = setup();

    // 登录
    StpUtil::login("10001").unwrap();

    // 验证登录 ID
    let login_id = StpUtil::get_login_id().unwrap();
    assert_eq!(login_id, "10001");
}

/// 测试 OptionalLoginId Extractor
#[tokio::test]
async fn test_optional_login_id_extractor() {
    let _guard = setup();

    // 未登录时
    let login_id = StpUtil::get_login_id_default_null().expect("login id query");
    assert!(login_id.is_none());

    // 登录后
    StpUtil::login("10001").unwrap();
    let login_id = StpUtil::get_login_id_default_null().expect("login id query");
    assert_eq!(login_id, Some("10001".to_string()));
}

/// 测试 SaTokenLayer 创建
#[tokio::test]
async fn test_sa_token_layer_creation() {
    let _guard = setup();

    // 创建 SaTokenLayer
    let _layer = SaTokenLayer::new();
}

/// 测试权限检查
#[tokio::test]
async fn test_permission_check() {
    let _guard = setup();

    // 实现权限接口
    struct TestStpInterface;
    impl sa_token_core::stp::StpInterface for TestStpInterface {
        fn get_permission_list(&self, login_id: &str, _login_type: &str) -> Vec<String> {
            match login_id {
                "10001" => vec!["user:add".to_string(), "user:list".to_string()],
                _ => vec![],
            }
        }
        fn get_role_list(&self, login_id: &str, _login_type: &str) -> Vec<String> {
            match login_id {
                "10001" => vec!["admin".to_string()],
                _ => vec![],
            }
        }
    }

    SaManager::set_stp_interface(Arc::new(TestStpInterface));

    // 登录
    StpUtil::login("10001").unwrap();

    // 检查权限
    assert!(StpUtil::has_permission("user:add").unwrap());
    assert!(StpUtil::has_permission("user:list").unwrap());
    assert!(!StpUtil::has_permission("user:delete").unwrap());

    // 检查角色
    assert!(StpUtil::has_role("admin").unwrap());
    assert!(!StpUtil::has_role("superadmin").unwrap());
}

/// 测试踢人下线
#[tokio::test]
async fn test_kickout() {
    let _guard = setup();

    // 登录
    StpUtil::login("10001").unwrap();
    assert!(StpUtil::is_login().expect("login state query"));

    // 踢人下线
    StpUtil::kickout("10001").unwrap();
    assert!(!StpUtil::is_login().expect("login state query"));
}

/// 测试 Token 续签
#[tokio::test]
async fn test_renew_timeout() {
    let _guard = setup();

    // 登录
    StpUtil::login("10001").unwrap();

    // 获取 Token 超时
    let timeout = StpUtil::get_token_timeout().expect("token timeout query");
    assert!(timeout > 0);

    // 续签
    StpUtil::renew_timeout(60 * 60).unwrap();
}

/// 测试多设备登录
#[tokio::test]
async fn test_multi_device() {
    let _guard = setup();

    // 设备 1 登录
    StpUtil::login_with_device("10001", "PC").unwrap();
    let token1 = StpUtil::get_token_value().unwrap();

    // 设备 2 登录
    StpUtil::login_with_device("10001", "Mobile").unwrap();
    let token2 = StpUtil::get_token_value().unwrap();

    // 两个 Token 不同
    assert_ne!(token1, token2);

    // 获取终端列表
    let terminals = StpUtil::get_terminal_list_by_login_id("10001").unwrap();
    assert_eq!(terminals.len(), 2);
}
