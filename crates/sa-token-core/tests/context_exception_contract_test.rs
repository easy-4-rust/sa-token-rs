//! Context / Exception 契约测试（Wave 2）。
//!
//! 对应 Java：
//! - `cn.dev33.satoken.context.*`
//! - `cn.dev33.satoken.exception.*`

use std::sync::Arc;

use sa_token_core::context::mock::sa_request_for_mock::SaRequestForMock;
use sa_token_core::context::mock::sa_response_for_mock::SaResponseForMock;
use sa_token_core::context::mock::sa_storage_for_mock::SaStorageForMock;
use sa_token_core::context::mock::sa_token_context_mock_util::SaTokenContextMockUtil;
use sa_token_core::context::model::sa_cookie::SaCookie;
use sa_token_core::context::model::sa_request::SaRequest;
use sa_token_core::context::model::sa_storage::SaStorage;
use sa_token_core::context::sa_holder::SaHolder;
use sa_token_core::context::sa_token_context::SaTokenContext;
use sa_token_core::context::sa_token_context_default_impl::{
    SaTokenContextDefaultImpl, DEFAULT_CONTEXT, ERROR_MESSAGE,
};
use sa_token_core::context::sa_token_context_for_read_only::SaTokenContextForReadOnly;
use sa_token_core::context::sa_token_context_for_thread_local::SaTokenContextForThreadLocal;
use sa_token_core::context::sa_token_context_for_thread_local_staff::SaTokenContextForThreadLocalStaff;
use sa_token_core::error::SaErrorCode;
use sa_token_core::exception::{
    ApiDisabledException, BackResultException, DisableServiceException, FirewallCheckException,
    InvalidContextException, NotHttpBasicAuthException, NotHttpDigestAuthException,
    NotImplException, NotLoginException, NotPermissionException, NotRoleException,
    NotSafeException, NotWebContextException, RequestPathInvalidException, SaJsonConvertException,
    SaTokenContextException, SaTokenException, SaTokenPluginException, SameTokenInvalidException,
    StopMatchException, TotpAuthException, ABNORMAL_LIST, BE_REPLACED, INVALID_TOKEN, NOT_TOKEN,
    NOT_TOKEN_MESSAGE,
};
use sa_token_core::sa_manager::SaManager;

/// ThreadLocal 上下文生命周期（对应 Java `SaTokenContextForThreadLocal` + Staff）。
#[test]
fn thread_local_context_roundtrip() {
    let ctx = SaTokenContextForThreadLocal;
    SaTokenContextForThreadLocalStaff::clear_model_box();
    assert!(!ctx.is_valid());

    let (req, res, stg) = SaTokenContextMockUtil::create_mock_context("/user/list", "GET");
    ctx.set_context(req, res, stg);
    assert!(ctx.is_valid());

    let box_req = ctx.model_box().get_request();
    assert_eq!(box_req.get_url(), "/user/list");
    assert_eq!(
        SaTokenContextForThreadLocalStaff::get_request().get_url(),
        "/user/list"
    );

    ctx.clear_context();
    assert!(!ctx.is_valid());
}

/// Mock 工具应写入并在回调结束后清除上下文（对应 Java `SaTokenContextMockUtil`）。
#[test]
fn mock_util_scoped_context() {
    SaManager::set_sa_token_context(Arc::new(SaTokenContextForThreadLocal));
    SaTokenContextForThreadLocalStaff::clear_model_box();

    SaTokenContextMockUtil::set_mock_context_fn(|| {
        assert!(SaHolder::get_context().is_valid());
    });
    assert!(!SaHolder::get_context().is_valid());

    let value = SaTokenContextMockUtil::set_mock_context_with(|| 42);
    assert_eq!(value, 42);
    assert!(!SaHolder::get_context().is_valid());
}

/// 只读上下文忽略写入但可读 inner（对应 Java `SaTokenContextForReadOnly`）。
#[test]
fn read_only_context_delegates_reads() {
    let inner: Box<dyn SaTokenContext> = Box::new(SaTokenContextForThreadLocal);
    let (req, res, stg) = SaTokenContextMockUtil::create_mock_context("/read", "GET");
    inner.set_context(req, res, stg);

    let read_only = SaTokenContextForReadOnly::new(inner);
    read_only.set_context(
        Arc::new(SaRequestForMock::new().with_url("/ignored")),
        Arc::new(SaResponseForMock::new()),
        Arc::new(SaStorageForMock::new()),
    );
    assert!(read_only.is_valid());
    assert_eq!(read_only.request().get_url(), "/read");
}

/// 默认上下文实现应抛出处理器无效错误（对应 Java `SaTokenContextDefaultImpl` + `CODE_10001`）。
#[test]
#[should_panic(expected = "SaTokenContextException")]
fn default_context_rejects_access() {
    let _ = DEFAULT_CONTEXT.is_valid();
}

/// Cookie 头格式与校验（对应 Java `SaCookie.toHeaderValue`）。
#[test]
fn sa_cookie_header_contract() {
    let mut cookie = SaCookie::new("satoken", "abc");
    cookie.set_path("/").set_same_site("Lax");
    let header = cookie.to_header_value().expect("valid cookie");
    assert!(header.contains("satoken=abc"));
    assert!(header.contains("Path=/"));
    assert!(header.contains("SameSite=Lax"));
}

/// NotLogin 场景常量与实例构造（对应 Java `NotLoginException`）。
#[test]
fn not_login_exception_contract() {
    assert_eq!(NOT_TOKEN, "-1");
    assert_eq!(INVALID_TOKEN, "-2");
    let err = NotLoginException::new_instance("login", NOT_TOKEN, NOT_TOKEN_MESSAGE, None)
        .with_code(SaErrorCode::CODE_11011);
    let mapped: SaTokenException = err.into();
    assert_eq!(mapped.code(), SaErrorCode::CODE_11011);
    assert!(matches!(
        mapped,
        SaTokenException::NotLogin { scene, .. } if scene == NOT_TOKEN
    ));
}

/// 各业务异常到统一枚举及默认错误码映射。
#[test]
fn exception_code_mapping_contract() {
    assert_eq!(
        SaTokenException::from(NotPermissionException::new("user:add", "login")),
        SaTokenException::NotPermission {
            permission: "user:add".into(),
            login_type: "login".into(),
        }
    );
    assert_eq!(
        SaTokenException::from(NotPermissionException::new("user:add", "login")).code(),
        SaErrorCode::CODE_11051
    );
    assert_eq!(
        SaTokenException::from(NotRoleException::new("admin", "login")).code(),
        SaErrorCode::CODE_11041
    );
    assert_eq!(
        SaTokenException::from(NotSafeException::new("login", "tok", "pay")).code(),
        SaErrorCode::CODE_11071
    );
    assert_eq!(
        SaTokenException::from(DisableServiceException::new("10001", "login", 60)).code(),
        SaErrorCode::CODE_11061
    );
    assert_eq!(
        SaTokenException::from(SameTokenInvalidException::new("无效Same-Token")).code(),
        SaErrorCode::CODE_10301
    );
    assert_eq!(
        SaTokenContextException::with_code(ERROR_MESSAGE, SaErrorCode::CODE_10001).get_code(),
        SaErrorCode::CODE_10001
    );
    assert_eq!(
        SaTokenException::from(NotHttpBasicAuthException::new()).code(),
        SaErrorCode::CODE_10311
    );
    assert_eq!(
        SaTokenException::from(NotHttpDigestAuthException::new()).code(),
        SaErrorCode::CODE_10312
    );
    assert_eq!(
        SaTokenException::from(InvalidContextException::new("ctx")).code(),
        SaErrorCode::CODE_10002
    );
}

/// 其余异常类型可往返转换为 `SaTokenException`。
#[test]
fn exception_conversion_contract() {
    let cases: Vec<SaTokenException> = vec![
        ApiDisabledException::new().into(),
        BackResultException::new("ok").into(),
        FirewallCheckException::new("blocked").into(),
        NotWebContextException::new().into(),
        RequestPathInvalidException::new("/bad").into(),
        SaJsonConvertException::new("bad json").into(),
        SaTokenPluginException::new("plugin").into(),
        StopMatchException::new().into(),
        TotpAuthException::new("bad totp").into(),
        NotImplException::new("todo").into(),
    ];
    assert_eq!(cases.len(), 10);
    for err in cases {
        let text = err.to_string();
        assert!(!text.is_empty());
    }
}

/// Mock Request 基础行为（对应 Java `SaRequestForMock`）。
#[test]
fn mock_request_contract() {
    let req = SaRequestForMock::new()
        .with_url("http://demo.test/user?id=1")
        .with_method("POST")
        .with_query("id", "1")
        .with_header("X-Requested-With", "XMLHttpRequest");
    assert_eq!(req.get_param("id"), Some("1".into()));
    assert_eq!(req.get_param_or_default("missing", "x"), "x");
    assert!(req.is_param("id", "1"));
    assert!(req.has_param("id"));
    assert_eq!(req.get_request_path(), "http://demo.test/user");
    assert!(req.is_ajax());
}

/// Mock Storage trait 接口（对应 Java `SaStorageForMock`）。
#[test]
fn mock_storage_trait_contract() {
    let stg = SaStorageForMock::new();
    stg.set("k", "v");
    assert_eq!(stg.get("k"), Some("v".into()));
    stg.delete("k");
    assert_eq!(stg.get("k"), None);
}

/// 未初始化 ThreadLocal 访问应触发上下文异常（`CODE_10002`）。
#[test]
#[should_panic(expected = "SaTokenContextException")]
fn thread_local_uninitialized_panics() {
    SaTokenContextForThreadLocalStaff::clear_model_box();
    let ctx = SaTokenContextForThreadLocal;
    let _ = ctx.model_box();
}

/// 默认实现常量与 Java 对齐。
#[test]
fn default_context_constants() {
    assert_eq!(ERROR_MESSAGE, "未能获取有效的上下文处理器");
    let _: &SaTokenContextDefaultImpl = &DEFAULT_CONTEXT;
}

/// NotLogin 异常场景列表与 Java `ABNORMAL_LIST` 对齐。
#[test]
fn not_login_abnormal_list_contains_be_replaced() {
    assert!(ABNORMAL_LIST.contains(&BE_REPLACED));
}
