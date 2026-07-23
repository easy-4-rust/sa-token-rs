//! FreeMarker→Tera 方言契约测试。

use std::sync::Arc;

use sa_token_context_mock::SaTokenContextMockUtil;
use sa_token_core::config::sa_token_config::SaTokenConfig;
use sa_token_core::sa_manager::SaManager;
use sa_token_core::stp::stp_interface::StpInterface;
use sa_token_core::stp::stp_logic::StpLogic;
use sa_token_dao_memory::SaTokenDaoMemory;
use sa_token_tera::{DEFAULT_ATTR_NAME, SaTokenTemplateDirectiveModel, SaTokenTemplateModel};
use tera::{Context, Tera};

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

/// 验证指令断言（对应 Java `SaTokenTemplateDirectiveModel.execute`）。
#[test]
fn directive_evaluate() {
    let d = SaTokenTemplateDirectiveModel::new(DEFAULT_ATTR_NAME, Arc::new(|v| v == Some("ok")));
    assert!(d.evaluate(Some("ok")));
    assert!(!d.evaluate(Some("no")));
}

/// 验证 `to_array`（对应 Java `SaTokenTemplateModel.toArray`）。
#[test]
fn template_model_to_array() {
    assert_eq!(
        SaTokenTemplateModel::to_array("admin, ceo, cto"),
        vec!["admin", "ceo", "cto"]
    );
}

/// 验证注册到 Tera 后登录前后标签渲染（对应 Java Freemarker 方言）。
#[test]
fn tera_sa_login_directives() {
    setup();
    let logic = SaManager::get_stp_logic("login").expect("stp");
    let model = SaTokenTemplateModel::new(Arc::clone(&logic));
    let mut tera = Tera::default();
    model.register_into(&mut tera);
    tera.add_raw_template(
        "t",
        "{% if sa_login() %}IN{% endif %}{% if sa_not_login() %}OUT{% endif %}",
    )
    .expect("template");

    let out = tera
        .render("t", &Context::new())
        .expect("render before login");
    assert_eq!(out, "OUT");

    logic.login("10001").expect("login");
    let out2 = tera
        .render("t", &Context::new())
        .expect("render after login");
    assert_eq!(out2, "IN");
}
