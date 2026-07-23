use sa_token_core::dao::sa_token_dao::SaTokenDao;
use sa_token_core::dao::sa_token_dao_default_impl::SaTokenDaoDefaultImpl;
use sa_token_sign::sign::SaSignConfig;
use sa_token_sso::sso::SaSsoManager;
use sa_token_sso::sso::config::{
    SaSsoClientConfig, SaSsoClientModel as RegisteredClient, SaSsoServerConfig,
};
use sa_token_sso::sso::error::SaSsoErrorCode;
use sa_token_sso::sso::exception::SaSsoException;
use sa_token_sso::sso::message::{SaSsoMessage, SaSsoMessageSimpleHandle};
use sa_token_sso::sso::model::{
    SaCheckTicketResult, SaSsoClientInfo, SaSsoClientModel, TicketModel,
};
use sa_token_sso::sso::name::{ApiName, ParamName};
use sa_token_sso::sso::processor::{
    SaSsoClientProcessor, SaSsoClientSession, SaSsoProcessorHelper, SaSsoProcessorResult,
    SaSsoRequest, SaSsoServerProcessor, SaSsoServerSession,
};
use sa_token_sso::sso::strategy::{SaSsoClientStrategy, SaSsoServerStrategy};
use sa_token_sso::sso::template::{
    SaSsoClientTemplate, SaSsoClientUtil, SaSsoServerAuth, SaSsoServerTemplate, SaSsoServerUtil,
    SaSsoTemplate,
};
use sa_token_sso::sso::util::SaSsoConsts;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct TestServerAuth {
    logout_calls: Mutex<Vec<(serde_json::Value, Option<String>)>>,
}

impl SaSsoServerAuth for TestServerAuth {
    fn login_device_id_by_token(&self, _: &str) -> Result<Option<String>, SaSsoException> {
        Ok(Some("browser".into()))
    }

    fn token_timeout(&self, _: &str) -> Result<i64, SaSsoException> {
        Ok(120)
    }

    fn session_timeout(&self, _: &serde_json::Value) -> Result<i64, SaSsoException> {
        Ok(240)
    }

    fn logout(
        &self,
        login_id: &serde_json::Value,
        device_id: Option<String>,
    ) -> Result<(), SaSsoException> {
        self.logout_calls
            .lock()
            .map_err(|_| SaSsoException::new(0, "test lock poisoned"))?
            .push((login_id.clone(), device_id));
        Ok(())
    }
}

#[derive(Default)]
struct TestClientSession {
    logins: Mutex<Vec<SaCheckTicketResult>>,
}

impl SaSsoClientSession for TestClientSession {
    fn is_login(&self) -> Result<bool, SaSsoException> {
        Ok(false)
    }

    fn login(&self, result: &SaCheckTicketResult) -> Result<(), SaSsoException> {
        self.logins
            .lock()
            .map_err(|_| SaSsoException::new(0, "test lock poisoned"))?
            .push(result.clone());
        Ok(())
    }

    fn login_id(&self) -> Result<Option<serde_json::Value>, SaSsoException> {
        Ok(None)
    }

    fn device_id(&self) -> Result<Option<String>, SaSsoException> {
        Ok(None)
    }

    fn logout(&self) -> Result<(), SaSsoException> {
        Ok(())
    }
}

struct TestServerSession;

impl SaSsoServerSession for TestServerSession {
    fn current_login(
        &self,
    ) -> Result<Option<(serde_json::Value, String, Option<String>)>, SaSsoException> {
        Ok(Some((json!(10001), "TOKEN".into(), Some("browser".into()))))
    }

    fn renew_timeout(&self) -> Result<(), SaSsoException> {
        Ok(())
    }
}

#[test]
fn configuration_defaults_and_url_splicing_match_java() {
    let client = SaSsoClientConfig::default();
    assert_eq!(client.auth_url, "/sso/auth");
    assert_eq!(client.signout_url, "/sso/signout");
    assert_eq!(client.push_url, "/sso/pushS");
    assert_eq!(client.get_data_url, "/sso/getData");
    assert!(!client.is_http);
    assert!(client.is_slo && client.is_check_sign);

    let client = SaSsoClientConfig {
        server_url: Some("https://sso.example/".into()),
        ..client
    };
    assert_eq!(client.splicing_auth_url(), "https://sso.example/sso/auth");
    let server = SaSsoServerConfig::default();
    assert_eq!(server.ticket_timeout, 300);
    assert_eq!(server.max_reg_client, 32);
    assert!(server.is_slo && server.is_check_sign);
    assert!(!server.auto_renew_timeout && !server.allow_anon_client);
}

#[test]
fn registered_client_validates_push_url_without_exposing_secret() {
    let valid = RegisteredClient {
        client: "app".into(),
        server_url: Some("https://app.example".into()),
        ..Default::default()
    };
    assert_eq!(
        valid.splicing_push_url().expect("valid URL"),
        "https://app.example/sso/pushC"
    );
    let invalid = RegisteredClient {
        client: "app".into(),
        ..Default::default()
    };
    assert_eq!(
        invalid.splicing_push_url().expect_err("relative URL").code,
        SaSsoErrorCode::CODE_30023
    );

    let secret = RegisteredClient {
        secret_key: Some("must-not-leak".into()),
        ..Default::default()
    };
    let debug = format!("{secret:?}");
    assert!(debug.contains("[REDACTED]"));
    assert!(!debug.contains("must-not-leak"));
}

#[test]
fn models_round_trip_all_java_fields() {
    let ticket = TicketModel::new("T", "app", json!(10001), "TOKEN");
    let encoded = serde_json::to_string(&ticket).expect("ticket serialization");
    assert_eq!(
        serde_json::from_str::<TicketModel>(&encoded).expect("ticket decode"),
        ticket
    );
    let info = SaSsoClientInfo::mode_three("app", "https://app.example/logout", 2);
    assert_eq!(info.mode, SaSsoConsts::SSO_MODE_3);
    let deprecated = SaSsoClientModel { info: info.clone() };
    assert_eq!(deprecated.info, info);
    let result = SaCheckTicketResult {
        login_id: Some(json!(10001)),
        token_value: Some("TOKEN".into()),
        remain_token_timeout: Some(60),
        ..Default::default()
    };
    assert_eq!(
        serde_json::from_value::<SaCheckTicketResult>(
            serde_json::to_value(&result).expect("result encode")
        )
        .expect("result decode"),
        result
    );
}

#[test]
fn names_constants_and_error_codes_are_java_compatible() {
    let api = ApiName::default();
    assert_eq!(api.sso_check_ticket, "/sso/checkTicket");
    assert_eq!(
        api.clone().add_prefix("/tenant").sso_auth,
        "/tenant/sso/auth"
    );
    assert_eq!(
        api.replace_prefix("/auth").sso_logout_call,
        "/auth/logoutCall"
    );
    let params = ParamName::default();
    assert_eq!(params.secret_key, "secretkey");
    assert_eq!(params.single_device_id_logout, "singleDeviceIdLogout");
    assert_eq!(SaSsoConsts::CLIENT_WILDCARD, "*");
    assert_eq!(SaSsoConsts::MESSAGE_LOGOUT_CALL, "logoutCall");
    assert_eq!(SaSsoErrorCode::CODE_30024, 30024);
    assert_eq!(
        SaSsoException::new(SaSsoErrorCode::CODE_30004, "bad ticket").code,
        30004
    );
}

#[test]
fn message_validation_uses_java_error_codes() {
    let missing_type = SaSsoMessage::new()
        .check_type()
        .expect_err("missing message type");
    assert_eq!(missing_type.code, SaSsoErrorCode::CODE_30022);

    let message = SaSsoMessage::with_type("custom");
    let missing_value = message
        .get_value_not_null("loginId")
        .expect_err("missing required value");
    assert_eq!(missing_value.code, SaSsoErrorCode::CODE_30024);
}

#[test]
fn message_holder_dispatches_registered_simple_handler() {
    let template = SaSsoTemplate::new();
    let missing = template
        .handle_message(&SaSsoMessage::with_type("custom"))
        .expect_err("unregistered handler");
    assert_eq!(missing.code, SaSsoErrorCode::CODE_30021);

    let handler = SaSsoMessageSimpleHandle::new(
        "custom",
        Arc::new(|_, message| {
            Ok(json!({
                "handled": true,
                "loginId": message.get_value_not_null("loginId")?
            }))
        }),
    );
    template
        .message_holder
        .add_handle(Arc::new(handler))
        .expect("register handler");

    let response = template
        .handle_message(&SaSsoMessage::with_type("custom").set("loginId", 10001))
        .expect("dispatch message");
    assert_eq!(response, json!({"handled": true, "loginId": 10001}));
}

#[test]
fn strategies_preserve_java_defaults_and_explicit_transport_errors() {
    let client = SaSsoClientStrategy::default();
    assert_eq!(
        (client.convert_center_id_to_login_id)(json!(10001)),
        json!(10001)
    );
    assert_eq!(
        client
            .request_as_result("https://sso.example/check")
            .expect_err("transport must be configured")
            .code,
        SaSsoErrorCode::CODE_30001
    );

    let server = SaSsoServerStrategy::default();
    assert_eq!(
        (server.not_login_view)(),
        json!("当前会话在 SSO-Server 认证中心尚未登录（当前未配置登录视图）")
    );
    assert_eq!(
        (server.do_login_handle)("name", "pwd"),
        json!({"code": 500, "msg": "error"})
    );
}

#[test]
fn strategies_decode_configured_transport_responses() {
    let client = SaSsoClientStrategy {
        send_request: Arc::new(|_| Ok(r#"{"code":200,"data":"ok"}"#.into())),
        ..Default::default()
    };
    assert_eq!(
        client
            .request_as_result("https://sso.example/check")
            .expect("decode response"),
        json!({"code": 200, "data": "ok"})
    );

    let server = SaSsoServerStrategy {
        send_request: Arc::new(|_| Ok("not-json".into())),
        ..Default::default()
    };
    assert_eq!(
        server
            .request_as_result("https://client.example/push")
            .expect_err("reject invalid response")
            .code,
        SaSsoErrorCode::CODE_30001
    );
}

#[test]
fn manager_keeps_runtime_configuration_isolated() {
    let first = SaSsoManager::default();
    let second = SaSsoManager::default();
    let changed = SaSsoClientConfig {
        client: Some("app-a".into()),
        ..Default::default()
    };
    first
        .set_client_config(changed)
        .expect("replace first runtime config");

    assert_eq!(
        first
            .client_config()
            .expect("first runtime config")
            .client
            .as_deref(),
        Some("app-a")
    );
    assert_eq!(
        second
            .client_config()
            .expect("second runtime config")
            .client,
        None
    );
}

#[test]
fn client_template_builds_signed_urls_and_handles_logout_messages() {
    let config = Arc::new(SaSsoClientConfig {
        client: Some("app-a".into()),
        server_url: Some("https://sso.example".into()),
        secret_key: Some("sso-secret".into()),
        ..Default::default()
    });
    let strategy = Arc::new(SaSsoClientStrategy {
        send_request: Arc::new(|url| Ok(url.to_owned())),
        ..Default::default()
    });
    let dao: Arc<dyn SaTokenDao> = Arc::new(SaTokenDaoDefaultImpl::new());
    let calls = Arc::new(Mutex::new(Vec::new()));
    let calls_for_logout = Arc::clone(&calls);
    let template = SaSsoClientTemplate::new(
        config,
        strategy,
        Arc::new(SaSignConfig::default()),
        dao,
        "satoken",
        Arc::new(move |login_id, device_id| {
            calls_for_logout
                .lock()
                .map_err(|_| SaSsoException::new(0, "test lock poisoned"))?
                .push((login_id, device_id));
            Ok(())
        }),
    )
    .expect("client template");

    let params = HashMap::from([("scope".into(), json!("profile"))]);
    let signed = template
        .build_custom_path_url("/sso/getData", &params)
        .expect("signed URL");
    let url = url::Url::parse(&signed).expect("parse signed URL");
    let query: HashMap<_, _> = url.query_pairs().into_owned().collect();
    assert_eq!(query.get("client").map(String::as_str), Some("app-a"));
    assert_eq!(query.get("scope").map(String::as_str), Some("profile"));
    assert!(query.contains_key("timestamp"));
    assert!(query.contains_key("nonce"));
    assert!(query.contains_key("sign"));
    assert_eq!(
        template
            .build_server_auth_url("https://client.example/sso/login", Some("/home?a=1"),)
            .expect("server auth URL"),
        "https://sso.example/sso/auth?client=app-a&redirect=https://client.example/sso/login?back=%2Fhome%3Fa%3D1"
    );

    let response = template
        .handle_message(
            &SaSsoMessage::with_type(SaSsoConsts::MESSAGE_LOGOUT_CALL)
                .set("loginId", 10001)
                .set("deviceId", "browser"),
        )
        .expect("logout callback");
    assert_eq!(response["code"], 200);
    assert_eq!(
        calls.lock().expect("logout calls").as_slice(),
        &[(json!(10001), Some("browser".into()))]
    );
}

#[test]
fn server_template_consumes_tickets_validates_redirects_and_signs_out() {
    let mut config = SaSsoServerConfig::default();
    config.clients.insert(
        "app-a".into(),
        RegisteredClient {
            client: "app-a".into(),
            allow_url: "https://client.example/*".into(),
            secret_key: Some("client-secret".into()),
            ..Default::default()
        },
    );
    let dao_impl = Arc::new(SaTokenDaoDefaultImpl::new());
    let dao: Arc<dyn SaTokenDao> = dao_impl.clone();
    let auth = Arc::new(TestServerAuth::default());
    let auth_port: Arc<dyn SaSsoServerAuth> = auth.clone();
    let template = SaSsoServerTemplate::new(
        Arc::new(config),
        Arc::new(SaSsoServerStrategy::default()),
        Arc::new(SaSignConfig::default()),
        dao,
        auth_port,
        "satoken",
    )
    .expect("server template");

    let ticket = template
        .create_ticket_and_save("app-a", json!(10001), "TOKEN")
        .expect("save ticket");
    let result = template
        .handle_message(
            &SaSsoMessage::with_type(SaSsoConsts::MESSAGE_CHECK_TICKET)
                .set("client", "app-a")
                .set("ticket", ticket.clone())
                .set("ssoLogoutCall", "https://client.example/logout"),
        )
        .expect("check ticket");
    assert_eq!(result["loginId"], 10001);
    assert_eq!(result["deviceId"], "browser");
    assert_eq!(result["remainTokenTimeout"], 120);
    assert_eq!(result["remainSessionTimeout"], 240);
    assert_eq!(
        template
            .check_ticket_and_delete(&ticket, "app-a")
            .expect_err("ticket is one-time")
            .code,
        SaSsoErrorCode::CODE_30004
    );

    let wrong_client_ticket = template
        .create_ticket_and_save("app-a", json!(10002), "TOKEN-2")
        .expect("save second ticket");
    assert_eq!(
        template
            .check_ticket_and_delete(&wrong_client_ticket, "app-b")
            .expect_err("client ownership")
            .code,
        SaSsoErrorCode::CODE_30011
    );

    let redirect = template
        .build_redirect_url(
            "app-a",
            "https://client.example/sso/login?back=/home",
            json!(10003),
            "TOKEN-3",
        )
        .expect("allowed redirect");
    assert!(redirect.contains("ticket="));
    assert_eq!(
        template
            .check_redirect_url("app-a", "https://client.example@evil.example/callback")
            .expect_err("reject userinfo bypass")
            .code,
        SaSsoErrorCode::CODE_30001
    );
    assert_eq!(
        template.encode_back_param("https://client.example/sso/login?back=/home?a=1"),
        "https://client.example/sso/login?back=%2Fhome%3Fa%3D1"
    );

    let signout = template
        .handle_message(
            &SaSsoMessage::with_type(SaSsoConsts::MESSAGE_SIGNOUT)
                .set("loginId", 10001)
                .set("deviceId", "browser"),
        )
        .expect("sign out");
    assert_eq!(signout["code"], 200);
    assert_eq!(
        auth.logout_calls.lock().expect("logout calls").as_slice(),
        &[(json!(10001), Some("browser".into()))]
    );

    let registrations = dao_impl
        .get_object("satoken:sso-client:10001")
        .expect("read SLO registrations")
        .expect("SLO registration");
    assert_eq!(registrations[0]["client"], "app-a");
}

#[test]
fn framework_neutral_processors_cover_client_and_server_routes() {
    let mut server_config = SaSsoServerConfig::default();
    server_config.clients.insert(
        "app-a".into(),
        RegisteredClient {
            client: "app-a".into(),
            allow_url: "https://client.example/*".into(),
            ..Default::default()
        },
    );
    let dao: Arc<dyn SaTokenDao> = Arc::new(SaTokenDaoDefaultImpl::new());
    let server_template = Arc::new(
        SaSsoServerTemplate::new(
            Arc::new(server_config),
            Arc::new(SaSsoServerStrategy::default()),
            Arc::new(SaSignConfig::default()),
            Arc::clone(&dao),
            Arc::new(TestServerAuth::default()),
            "satoken",
        )
        .expect("server template"),
    );

    let client_template = Arc::new(
        SaSsoClientTemplate::new(
            Arc::new(SaSsoClientConfig {
                client: Some("app-a".into()),
                server_url: Some("https://sso.example".into()),
                ..Default::default()
            }),
            Arc::new(SaSsoClientStrategy::default()),
            Arc::new(SaSignConfig::default()),
            Arc::clone(&dao),
            "satoken",
            Arc::new(|_, _| Ok(())),
        )
        .expect("client template"),
    );
    let client_session = Arc::new(TestClientSession::default());
    let client_session_port: Arc<dyn SaSsoClientSession> = client_session.clone();
    let client_processor = SaSsoClientProcessor::new(client_template, client_session_port)
        .with_direct_server(Arc::clone(&server_template));
    let client_util = SaSsoClientUtil::new(&client_processor.template);
    assert_eq!(
        client_util.build_check_ticket_message("T", None).get_type(),
        Some(SaSsoConsts::MESSAGE_CHECK_TICKET)
    );
    let server_util = SaSsoServerUtil::new(&server_template);
    assert_eq!(
        server_util
            .client("app-a")
            .expect("server util client")
            .client,
        "app-a"
    );
    #[allow(deprecated)]
    let combined =
        sa_token_sso::sso::template::SaSsoUtil::new(&client_processor.template, &server_template);
    assert_eq!(combined.client().template().client(), Some("app-a"));
    let ticket = server_template
        .create_ticket_and_save("app-a", json!(10001), "TOKEN")
        .expect("ticket");
    let result = client_processor
        .dispatch(
            &SaSsoRequest::new("/sso/login")
                .with_param("ticket", ticket)
                .with_param("back", "/home"),
        )
        .expect("client login dispatch");
    assert_eq!(result, SaSsoProcessorResult::Redirect("/home".into()));
    assert_eq!(
        client_session.logins.lock().expect("client logins")[0].login_id,
        Some(json!(10001))
    );

    let server_processor = SaSsoServerProcessor::new(server_template, Arc::new(TestServerSession));
    let result = server_processor
        .dispatch(
            &SaSsoRequest::new("/sso/auth")
                .with_param("client", "app-a")
                .with_param("redirect", "https://client.example/sso/login"),
        )
        .expect("server auth dispatch");
    let SaSsoProcessorResult::Redirect(redirect) = result else {
        panic!("expected redirect");
    };
    assert!(redirect.starts_with("https://client.example/sso/login?ticket="));
    assert_eq!(
        server_processor
            .dispatch(&SaSsoRequest::new("/not-sso"))
            .expect("not handled"),
        SaSsoProcessorResult::NotHandled
    );

    assert!(matches!(
        SaSsoProcessorHelper::sso_logout_back(
            &SaSsoRequest::new("/sso/logout").with_param("back", "self"),
            &ParamName::default(),
        ),
        SaSsoProcessorResult::Html(_)
    ));
}
