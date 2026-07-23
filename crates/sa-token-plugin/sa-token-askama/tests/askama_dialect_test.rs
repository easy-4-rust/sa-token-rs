//! Thymeleaf→Askama 方言契约测试。

use std::sync::Arc;

use askama::Template;
use sa_token_askama::{SaTokenDialect, SaTokenTagProcessor};
use sa_token_context_mock::SaTokenContextMockUtil;
use sa_token_core::config::sa_token_config::SaTokenConfig;
use sa_token_core::sa_manager::SaManager;
use sa_token_core::stp::stp_interface::StpInterface;
use sa_token_core::stp::stp_logic::StpLogic;
use sa_token_dao_memory::SaTokenDaoMemory;

/// 测试用权限数据源。
struct DemoStp;

impl StpInterface for DemoStp {
    fn get_permission_list(&self, _login_id: &str, _login_type: &str) -> Vec<String> {
        vec!["user-add".into(), "user-delete".into()]
    }

    fn get_role_list(&self, _login_id: &str, _login_type: &str) -> Vec<String> {
        vec!["admin".into()]
    }
}

fn setup() {
    SaManager::reset();
    SaManager::set_config(Arc::new(SaTokenConfig::default()));
    SaManager::set_sa_token_dao(Arc::new(SaTokenDaoMemory::new()));
    SaManager::set_stp_interface(Arc::new(DemoStp));
    SaManager::put_stp_logic(Arc::new(StpLogic::new("login")));
    SaTokenContextMockUtil::set_mock_context();
}

/// 验证标签断言（对应 Java `SaTokenTagProcessor.doProcess`）。
#[test]
fn tag_processor_evaluate() {
    let p = SaTokenTagProcessor::new("sa", "hasRole", Arc::new(|v| v == Some("admin")));
    assert!(p.should_render(Some("admin")));
    assert!(!p.should_render(Some("user")));
}

/// 验证 `to_array`（对应 Java `SaTokenDialect.toArray`）。
#[test]
fn dialect_to_array() {
    assert_eq!(
        SaTokenDialect::to_array("admin, ceo, cto"),
        vec!["admin", "ceo", "cto"]
    );
}

/// Askama 内联模板：登录前后 `sa:login` / `sa:notLogin` 语义。
#[derive(Template)]
#[template(
    ext = "html",
    source = r#"{% if login %}IN{% endif %}{% if not_login %}OUT{% endif %}"#
)]
struct LoginSnippet {
    login: bool,
    not_login: bool,
}

/// 验证方言处理器 + Askama 渲染（对应 Java Thymeleaf 方言）。
#[test]
fn askama_sa_login_directives() {
    setup();
    let logic = SaManager::get_stp_logic("login").expect("stp");
    let dialect = SaTokenDialect::new(Arc::clone(&logic));

    assert_eq!(dialect.processors().len(), 12);

    let before = LoginSnippet {
        login: dialect.evaluate("login", None),
        not_login: dialect.evaluate("notLogin", None),
    };
    assert_eq!(before.render().expect("render"), "OUT");

    logic.login("10001").expect("login");
    let after = LoginSnippet {
        login: dialect.evaluate("login", None),
        not_login: dialect.evaluate("notLogin", None),
    };
    assert_eq!(after.render().expect("render"), "IN");

    assert!(dialect.evaluate("hasRole", Some("admin")));
    assert!(!dialect.evaluate("hasRole", Some("guest")));
    assert!(dialect.evaluate("hasPermission", Some("user-add")));
}
