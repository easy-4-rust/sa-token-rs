//! 注解模块契约测试（对应 Java `cn.dev33.satoken.annotation` / `.handler`）。
//!
//! 覆盖元数据默认值与可注入校验路径，作为 file-map `test_evidence`。

use sa_token_core::annotation::handler::sa_annotation_handler_interface::SaAnnotationHandlerInterface;
use sa_token_core::annotation::handler::sa_check_disable_handler::SaCheckDisableHandler;
use sa_token_core::annotation::handler::sa_check_http_basic_handler::SaCheckHttpBasicHandler;
use sa_token_core::annotation::handler::sa_check_http_digest_handler::SaCheckHttpDigestHandler;
use sa_token_core::annotation::handler::sa_check_login_handler::SaCheckLoginHandler;
use sa_token_core::annotation::handler::sa_check_or_handler::SaCheckOrHandler;
use sa_token_core::annotation::handler::sa_check_permission_handler::SaCheckPermissionHandler;
use sa_token_core::annotation::handler::sa_check_role_handler::SaCheckRoleHandler;
use sa_token_core::annotation::handler::sa_check_safe_handler::SaCheckSafeHandler;
use sa_token_core::annotation::handler::sa_ignore_handler::SaIgnoreHandler;
use sa_token_core::annotation::sa_check_disable::SaCheckDisableMeta;
use sa_token_core::annotation::sa_check_http_basic::SaCheckHttpBasicMeta;
use sa_token_core::annotation::sa_check_http_digest::SaCheckHttpDigestMeta;
use sa_token_core::annotation::sa_check_login::SaCheckLoginMeta;
use sa_token_core::annotation::sa_check_or::SaCheckOrMeta;
use sa_token_core::annotation::sa_check_permission::SaCheckPermissionMeta;
use sa_token_core::annotation::sa_check_role::SaCheckRoleMeta;
use sa_token_core::annotation::sa_check_safe::SaCheckSafeMeta;
use sa_token_core::annotation::sa_ignore::SaIgnoreMeta;
use sa_token_core::annotation::sa_mode::SaMode;
use sa_token_core::util::sa_token_consts::{
    DEFAULT_DISABLE_LEVEL, DEFAULT_DISABLE_SERVICE, DEFAULT_HTTP_AUTH_REALM,
    DEFAULT_SAFE_AUTH_SERVICE,
};

/// 验证 `SaMode` 默认值为 And（对应 Java `SaMode.AND`）。
#[test]
fn sa_mode_default_is_and() {
    assert_eq!(SaMode::default(), SaMode::And);
}

/// 验证登录注解默认 `type=""`（对应 Java `SaCheckLogin.type()`）。
#[test]
fn sa_check_login_meta_defaults() {
    let meta = SaCheckLoginMeta::new();
    assert_eq!(meta.r#type, "");
    assert_eq!(SaCheckLoginMeta::with_type("user").r#type, "user");
}

/// 验证登录处理器：无 loginId 失败，有 loginId 成功。
#[test]
fn sa_check_login_handler_with_login_id() {
    let handler = SaCheckLoginHandler::new(SaCheckLoginMeta::new());
    assert!(handler.check_with_login_id(None).is_err());
    assert!(handler.check_with_login_id(Some("10001")).is_ok());
    assert_eq!(SaCheckLoginHandler::ANNOTATION, "SaCheckLogin");
}

/// 验证权限 And/Or 判定（对应 Java `SaCheckPermissionHandler`）。
#[test]
fn sa_check_permission_and_or() {
    let and_meta = SaCheckPermissionMeta::new(&["user.add", "user.delete"]);
    let and_h = SaCheckPermissionHandler::new(and_meta);
    assert!(
        and_h
            .check_with_permissions(&["user.add", "user.delete"])
            .is_ok()
    );
    assert!(and_h.check_with_permissions(&["user.add"]).is_err());

    let or_meta = SaCheckPermissionMeta::new(&["a", "b"]).with_mode(SaMode::Or);
    let or_h = SaCheckPermissionHandler::new(or_meta);
    assert!(or_h.check_with_permissions(&["b"]).is_ok());
    assert!(or_h.check_with_permissions(&["c"]).is_err());
}

/// 验证角色 And/Or 判定（对应 Java `SaCheckRoleHandler`）。
#[test]
fn sa_check_role_and_or() {
    let and_meta = SaCheckRoleMeta::new(&["admin", "ops"]);
    let and_h = SaCheckRoleHandler::new(and_meta);
    assert!(and_h.check_with_roles(&["admin", "ops"]).is_ok());
    assert!(and_h.check_with_roles(&["admin"]).is_err());

    let or_meta = SaCheckRoleMeta::new(&["admin", "guest"]).with_mode(SaMode::Or);
    let or_h = SaCheckRoleHandler::new(or_meta);
    assert!(or_h.check_with_roles(&["guest"]).is_ok());
}

/// 验证忽略鉴权处理器恒成功（对应 Java `SaIgnoreHandler.checkMethod` 空实现）。
#[test]
fn sa_ignore_handler_always_ok() {
    let handler = SaIgnoreHandler::new(SaIgnoreMeta::new());
    assert!(handler.check().is_ok());
}

/// 验证 Disable / Safe / HttpBasic / Digest / Or 元数据默认值对齐 Java。
#[test]
fn annotation_meta_java_defaults() {
    let disable = SaCheckDisableMeta::new();
    assert_eq!(disable.r#type, "");
    assert_eq!(disable.value, &[DEFAULT_DISABLE_SERVICE]);
    assert_eq!(disable.level, DEFAULT_DISABLE_LEVEL);

    let safe = SaCheckSafeMeta::new();
    assert_eq!(safe.r#type, "");
    assert_eq!(safe.value, DEFAULT_SAFE_AUTH_SERVICE);

    let basic = SaCheckHttpBasicMeta::new();
    assert_eq!(basic.realm, DEFAULT_HTTP_AUTH_REALM);
    assert_eq!(basic.account, "");
    assert_eq!(basic.password, "");

    let digest = SaCheckHttpDigestMeta::new();
    assert_eq!(digest.realm, DEFAULT_HTTP_AUTH_REALM);

    let or_meta = SaCheckOrMeta::new();
    assert_eq!(or_meta.r#type, "");
}

/// 验证 Disable 等级阈值与各 handler 构造可用。
#[test]
fn remaining_handlers_construct_and_disable_level() {
    let disable_h = SaCheckDisableHandler::new(SaCheckDisableMeta::new().with_level(2));
    assert!(disable_h.check_with_level("10001", Some(1)).is_ok());
    assert!(disable_h.check_with_level("10001", Some(2)).is_err());

    assert!(
        SaCheckSafeHandler::new(SaCheckSafeMeta::new())
            .check()
            .is_ok()
    );
    assert!(
        SaCheckHttpBasicHandler::new(SaCheckHttpBasicMeta::new())
            .check()
            .is_ok()
    );
    assert!(
        SaCheckHttpDigestHandler::new(SaCheckHttpDigestMeta::new())
            .check()
            .is_ok()
    );
    assert!(SaCheckOrHandler::new(SaCheckOrMeta::new()).check().is_ok());
}
