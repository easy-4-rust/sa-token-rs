//! Phase 2 集成测试
//!
//! 测试权限、角色、禁用、安全认证、切换账号等功能。

use sa_token::prelude::*;
use sa_token_core::stp::stp_interface::StpInterface;

/// 自定义权限数据源
struct MyStpInterface;

impl StpInterface for MyStpInterface {
    fn get_permission_list(&self, login_id: &str, _login_type: &str) -> Vec<String> {
        match login_id {
            "10001" => vec![
                "user:add".to_string(),
                "user:list".to_string(),
                "user:delete".to_string(),
            ],
            "10002" => vec!["user:list".to_string()],
            _ => vec![],
        }
    }

    fn get_role_list(&self, login_id: &str, _login_type: &str) -> Vec<String> {
        match login_id {
            "10001" => vec!["admin".to_string(), "user".to_string()],
            "10002" => vec!["user".to_string()],
            _ => vec![],
        }
    }
}

/// 初始化测试环境
fn setup() {
    SaManager::reset();
    SaManager::set_config(Arc::new(SaTokenConfig::default()));
    SaManager::set_sa_token_dao(Arc::new(SaTokenDaoMemory::new()));
    SaTokenContextMockUtil::set_mock_context();
    SaManager::set_stp_interface(Arc::new(MyStpInterface));
    SaManager::put_stp_logic(Arc::new(StpLogic::new("login")));
}

// ==================== 角色测试 ====================

/// 测试获取角色列表
#[test]
fn test_get_role_list() {
    setup();
    StpUtil::login("10001").unwrap();

    let roles = StpUtil::get_role_list().unwrap();
    assert!(roles.contains(&"admin".to_string()));
    assert!(roles.contains(&"user".to_string()));
}

/// 测试 has_role
#[test]
fn test_has_role() {
    setup();
    StpUtil::login("10001").unwrap();

    assert!(StpUtil::has_role("admin").unwrap());
    assert!(StpUtil::has_role("user").unwrap());
    assert!(!StpUtil::has_role("superadmin").unwrap());
}

/// 测试 has_role_and
#[test]
fn test_has_role_and() {
    setup();
    StpUtil::login("10001").unwrap();

    assert!(StpUtil::has_role_and(&["admin", "user"]).unwrap());
    assert!(!StpUtil::has_role_and(&["admin", "superadmin"]).unwrap());
}

/// 测试 has_role_or
#[test]
fn test_has_role_or() {
    setup();
    StpUtil::login("10001").unwrap();

    assert!(StpUtil::has_role_or(&["admin", "superadmin"]).unwrap());
    assert!(!StpUtil::has_role_or(&["superadmin", "guest"]).unwrap());
}

/// 测试 check_role 通过
#[test]
fn test_check_role_pass() {
    setup();
    StpUtil::login("10001").unwrap();

    assert!(StpUtil::check_role("admin").is_ok());
}

/// 测试 check_role 失败
#[test]
fn test_check_role_fail() {
    setup();
    StpUtil::login("10002").unwrap();

    let result = StpUtil::check_role("admin");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), SaTokenException::NotRole { .. }));
}

// ==================== 权限测试 ====================

/// 测试获取权限列表
#[test]
fn test_get_permission_list() {
    setup();
    StpUtil::login("10001").unwrap();

    let permissions = StpUtil::get_permission_list().unwrap();
    assert!(permissions.contains(&"user:add".to_string()));
    assert!(permissions.contains(&"user:list".to_string()));
    assert!(permissions.contains(&"user:delete".to_string()));
}

/// 测试 has_permission
#[test]
fn test_has_permission() {
    setup();
    StpUtil::login("10001").unwrap();

    assert!(StpUtil::has_permission("user:add").unwrap());
    assert!(StpUtil::has_permission("user:list").unwrap());
    assert!(!StpUtil::has_permission("user:update").unwrap());
}

/// 测试 has_permission_and
#[test]
fn test_has_permission_and() {
    setup();
    StpUtil::login("10001").unwrap();

    assert!(StpUtil::has_permission_and(&["user:add", "user:list"]).unwrap());
    assert!(!StpUtil::has_permission_and(&["user:add", "user:update"]).unwrap());
}

/// 测试 has_permission_or
#[test]
fn test_has_permission_or() {
    setup();
    StpUtil::login("10001").unwrap();

    assert!(StpUtil::has_permission_or(&["user:add", "user:update"]).unwrap());
    assert!(!StpUtil::has_permission_or(&["user:update", "user:export"]).unwrap());
}

/// 测试 check_permission 通过
#[test]
fn test_check_permission_pass() {
    setup();
    StpUtil::login("10001").unwrap();

    assert!(StpUtil::check_permission("user:add").is_ok());
}

/// 测试 check_permission 失败
#[test]
fn test_check_permission_fail() {
    setup();
    StpUtil::login("10002").unwrap();

    let result = StpUtil::check_permission("user:add");
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        SaTokenException::NotPermission { .. }
    ));
}

// ==================== 禁用测试 ====================

/// 测试封禁账号
#[test]
fn test_disable() {
    setup();
    StpUtil::login("10001").unwrap();

    // 封禁
    StpUtil::disable("10001", 60).unwrap();
    assert!(StpUtil::is_disable("10001"));

    // 解封
    StpUtil::untie_disable("10001").unwrap();
    assert!(!StpUtil::is_disable("10001"));
}

/// 测试 check_disable
#[test]
fn test_check_disable() {
    setup();
    StpUtil::login("10001").unwrap();

    // 未封禁时通过
    assert!(StpUtil::check_disable("10001").is_ok());

    // 封禁后失败
    StpUtil::disable("10001", 60).unwrap();
    let result = StpUtil::check_disable("10001");
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        SaTokenException::DisableService { .. }
    ));
}

/// 测试获取封禁剩余时间
#[test]
fn test_get_disable_time() {
    setup();
    StpUtil::login("10001").unwrap();

    StpUtil::disable("10001", 60).unwrap();
    let time = StpUtil::get_disable_time("10001");
    assert!(time > 0 && time <= 60);
}

// ==================== 安全认证测试 ====================

/// 测试开启二级认证
#[test]
fn test_open_safe() {
    setup();
    StpUtil::login("10001").unwrap();

    // 开启
    StpUtil::open_safe(60).unwrap();
    assert!(StpUtil::is_safe());

    // 关闭
    StpUtil::close_safe().unwrap();
    assert!(!StpUtil::is_safe());
}

/// 测试 check_safe
#[test]
fn test_check_safe() {
    setup();
    StpUtil::login("10001").unwrap();

    // 未开启时失败
    let result = StpUtil::check_safe();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), SaTokenException::NotSafe { .. }));

    // 开启后通过
    StpUtil::open_safe(60).unwrap();
    assert!(StpUtil::check_safe().is_ok());
}

// ==================== 切换账号测试 ====================

/// 测试切换账号
#[test]
fn test_switch_to() {
    setup();
    StpUtil::login("10001").unwrap();

    // 切换
    StpUtil::switch_to("10002").unwrap();
    assert!(StpUtil::is_switch());
    assert_eq!(StpUtil::get_switch_login_id().unwrap(), "10002");

    // 结束切换
    StpUtil::end_switch().unwrap();
    assert!(!StpUtil::is_switch());
}

// ==================== proc-macro 注解测试 ====================

/// 测试 sa_check_login 宏
#[test]
fn test_sa_check_login_macro() {
    setup();

    // 未登录时调用
    let result = mock_check_login();
    assert!(result.is_err());

    // 登录后调用
    StpUtil::login("10001").unwrap();
    let result = mock_check_login();
    assert!(result.is_ok());
}

/// 测试 sa_check_permission 宏
#[test]
fn test_sa_check_permission_macro() {
    setup();

    // 有权限
    StpUtil::login("10001").unwrap();
    let result = mock_check_permission_user_add();
    assert!(result.is_ok());

    // 无权限
    StpUtil::logout().unwrap();
    StpUtil::login("10002").unwrap();
    let result = mock_check_permission_user_add();
    assert!(result.is_err());
}

/// 测试 sa_check_role 宏
#[test]
fn test_sa_check_role_macro() {
    setup();

    // 有角色
    StpUtil::login("10001").unwrap();
    let result = mock_check_role_admin();
    assert!(result.is_ok());

    // 无角色
    StpUtil::logout().unwrap();
    StpUtil::login("10002").unwrap();
    let result = mock_check_role_admin();
    assert!(result.is_err());
}

// ==================== 辅助函数 ====================

fn mock_check_login() -> SaResult<()> {
    // 模拟 #[sa_check_login] 的行为
    StpUtil::check_login()
}

fn mock_check_permission_user_add() -> SaResult<()> {
    // 模拟 #[sa_check_permission("user:add")] 的行为
    StpUtil::check_permission("user:add")
}

fn mock_check_role_admin() -> SaResult<()> {
    // 模拟 #[sa_check_role("admin")] 的行为
    StpUtil::check_role("admin")
}
