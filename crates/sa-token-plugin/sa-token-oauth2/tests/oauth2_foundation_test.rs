use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use sa_token_core::dao::sa_token_dao::SaTokenDao;
use sa_token_core::plugin::sa_token_plugin::SaTokenPlugin;
use sa_token_core::secure::sa_base64_util::SaBase64Util;
use sa_token_dao_memory::SaTokenDaoMemory;
use sa_token_oauth2::oauth2::annotation::{
    SaCheckAccessToken, SaCheckAccessTokenHandler, SaCheckClientIdSecret,
    SaCheckClientIdSecretHandler, SaCheckClientToken, SaCheckClientTokenHandler,
    SaOAuth2AnnotationValidator,
};
use sa_token_oauth2::oauth2::config::{SaOAuth2OidcConfig, SaOAuth2ServerConfig};
use sa_token_oauth2::oauth2::consts::{
    GrantType, SaOAuth2Api, SaOAuth2Consts, SaOAuth2ExtraField, SaOAuth2Param,
    SaOAuth2ResponseType, SaOAuth2TokenType,
};
use sa_token_oauth2::oauth2::dao::SaOAuth2Dao;
use sa_token_oauth2::oauth2::data::convert::{
    SaOAuth2DataConverter, SaOAuth2DataConverterDefaultImpl, SaOAuth2TokenGenerator,
};
use sa_token_oauth2::oauth2::data::generate::{
    SaOAuth2DataGenerate, SaOAuth2DataGenerateDefaultImpl, SaOAuth2GenerateHooks,
};
use sa_token_oauth2::oauth2::data::loader::{SaOAuth2DataLoader, SaOAuth2DataLoaderDefaultImpl};
use sa_token_oauth2::oauth2::data::model::loader::SaClientModel;
use sa_token_oauth2::oauth2::data::model::oidc::IdTokenModel;
use sa_token_oauth2::oauth2::data::model::request::{ClientIdAndSecretModel, RequestAuthModel};
use sa_token_oauth2::oauth2::data::model::{
    AccessTokenModel, ClientTokenModel, CodeModel, RefreshTokenModel,
};
use sa_token_oauth2::oauth2::data::resolver::{
    SaOAuth2DataResolver, SaOAuth2DataResolverDefaultImpl, SaOAuth2Request,
};
use sa_token_oauth2::oauth2::error::SaOAuth2ErrorCode;
use sa_token_oauth2::oauth2::exception::{
    SaOAuth2AccessTokenException, SaOAuth2AccessTokenScopeException,
    SaOAuth2AuthorizationCodeException, SaOAuth2ClientModelException,
    SaOAuth2ClientModelScopeException, SaOAuth2ClientTokenException,
    SaOAuth2ClientTokenScopeException, SaOAuth2Exception, SaOAuth2RefreshTokenException,
};
use sa_token_oauth2::oauth2::function::strategy::{
    SaOAuth2CreateAccessTokenValueFunction, SaOAuth2CreateClientTokenValueFunction,
    SaOAuth2CreateCodeValueFunction, SaOAuth2CreateRefreshTokenValueFunction,
    SaOAuth2GrantTypeAuthFunction, SaOAuth2ScopeWorkAccessTokenFunction,
    SaOAuth2ScopeWorkClientTokenFunction,
};
use sa_token_oauth2::oauth2::function::{
    SaOAuth2ConfirmViewFunction, SaOAuth2DoLoginHandleFunction, SaOAuth2NotLoginViewFunction,
};
use sa_token_oauth2::oauth2::grant_type::handler::{
    AuthorizationCodeGrantTypeHandler, AuthorizationCodeParameterChecker, PasswordAuthResult,
    PasswordGrantTypeHandler, RefreshTokenGrantTypeHandler, SaOAuth2GrantTypeHandlerInterface,
};
use sa_token_oauth2::oauth2::processor::{SaOAuth2ProcessorResponse, SaOAuth2ServerProcessor};
use sa_token_oauth2::oauth2::scope::CommonScope;
use sa_token_oauth2::oauth2::scope::handler::{
    OidcScopeContext, OidcScopeHandler, OpenIdScopeHandler, SaOAuth2IdTokenGenerator,
    SaOAuth2ScopeHandlerInterface, UnionIdScopeHandler, UserIdScopeHandler,
};
use sa_token_oauth2::oauth2::strategy::SaOAuth2Strategy;
use sa_token_oauth2::oauth2::template::{SaOAuth2Template, SaOAuth2Util};
use sa_token_oauth2::oauth2::{SaOAuth2Manager, SaOAuth2Runtime};
use sa_token_oauth2::plugin::{SaOAuth2AnnotationRegistry, SaTokenPluginForOAuth2};

#[derive(Default)]
struct RecordingValidator {
    calls: Mutex<Vec<String>>,
    failure: Mutex<Option<&'static str>>,
}

#[derive(Default)]
struct RecordingAnnotationRegistry {
    events: Mutex<Vec<&'static str>>,
}

impl SaOAuth2AnnotationRegistry<RecordingValidator> for RecordingAnnotationRegistry {
    fn register_access_token_handler(&self, _: Arc<SaCheckAccessTokenHandler<RecordingValidator>>) {
        self.events
            .lock()
            .expect("annotation registry lock")
            .push("access");
    }

    fn register_client_token_handler(&self, _: Arc<SaCheckClientTokenHandler<RecordingValidator>>) {
        self.events
            .lock()
            .expect("annotation registry lock")
            .push("client");
    }

    fn register_client_id_secret_handler(
        &self,
        _: Arc<SaCheckClientIdSecretHandler<RecordingValidator>>,
    ) {
        self.events
            .lock()
            .expect("annotation registry lock")
            .push("credentials");
    }

    fn unregister_oauth2_handlers(&self) {
        self.events
            .lock()
            .expect("annotation registry lock")
            .push("destroy");
    }
}

struct DeterministicTokenGenerator;
struct TestGenerateHooks;
struct TestAuthorizationCodeChecker {
    code: String,
}
#[derive(Default)]
struct RecordingIdTokenGenerator {
    model: Mutex<Option<IdTokenModel>>,
}

impl SaOAuth2GenerateHooks for TestGenerateHooks {}

#[async_trait]
impl AuthorizationCodeParameterChecker for TestAuthorizationCodeChecker {
    async fn check(
        &self,
        candidate: &str,
        client_id: &str,
        secret: &str,
        redirect: Option<&str>,
    ) -> sa_token_core::exception::SaResult<()> {
        if candidate == self.code
            && client_id == "app-a"
            && secret == "secret"
            && redirect == Some("https://client.example/callback")
        {
            Ok(())
        } else {
            Err(sa_token_core::exception::SaTokenException::with_code(
                30110,
                "authorization-code parameters do not match",
            ))
        }
    }
}

impl SaOAuth2IdTokenGenerator for RecordingIdTokenGenerator {
    fn generate_id_token(&self, model: &IdTokenModel) -> Result<String, SaOAuth2Exception> {
        *self.model.lock().expect("ID-token model lock") = Some(model.clone());
        Ok("signed-id-token".into())
    }
}

impl SaOAuth2TokenGenerator for DeterministicTokenGenerator {
    type Error = &'static str;

    fn create_code(
        &self,
        client_id: &str,
        _: &serde_json::Value,
        _: &[String],
    ) -> Result<String, Self::Error> {
        Ok(format!("code-{client_id}"))
    }

    fn create_access_token(
        &self,
        client_id: &str,
        _: &serde_json::Value,
        _: &[String],
    ) -> Result<String, Self::Error> {
        Ok(format!("access-{client_id}"))
    }

    fn create_refresh_token(
        &self,
        client_id: &str,
        _: &serde_json::Value,
        _: &[String],
    ) -> Result<String, Self::Error> {
        Ok(format!("refresh-{client_id}"))
    }

    fn create_client_token(&self, client_id: &str, _: &[String]) -> Result<String, Self::Error> {
        Ok(format!("client-{client_id}"))
    }
}

#[test]
fn strategy_function_ports_delegate_and_keep_framework_state_explicit() {
    let generator = DeterministicTokenGenerator;
    let login_id = serde_json::json!(10001);
    let scopes = vec!["openid".into()];

    assert_eq!(
        SaOAuth2CreateCodeValueFunction::execute(&generator, "app-a", &login_id, &scopes),
        Ok("code-app-a".into())
    );
    assert_eq!(
        SaOAuth2CreateAccessTokenValueFunction::execute(&generator, "app-a", &login_id, &scopes),
        Ok("access-app-a".into())
    );
    assert_eq!(
        SaOAuth2CreateRefreshTokenValueFunction::execute(&generator, "app-a", &login_id, &scopes),
        Ok("refresh-app-a".into())
    );
    assert_eq!(
        SaOAuth2CreateClientTokenValueFunction::execute(&generator, "app-a", &scopes),
        Ok("client-app-a".into())
    );

    let grant = |request: &SaOAuth2Request| -> Result<AccessTokenModel, &'static str> {
        Ok(AccessTokenModel {
            client_id: request.param("client_id").map(str::to_owned),
            ..Default::default()
        })
    };
    let request = SaOAuth2Request {
        params: [("client_id".into(), "app-a".into())].into(),
        ..Default::default()
    };
    assert_eq!(
        SaOAuth2GrantTypeAuthFunction::execute(&grant, &request)
            .expect("grant function must return its model")
            .client_id
            .as_deref(),
        Some("app-a")
    );

    let access_hook = |model: &mut AccessTokenModel| {
        model.grant_type = Some(GrantType::AUTHORIZATION_CODE.into());
    };
    let client_hook = |model: &mut ClientTokenModel| {
        model.grant_type = Some(GrantType::CLIENT_CREDENTIALS.into());
    };
    let mut access = AccessTokenModel::default();
    let mut client = ClientTokenModel::default();
    SaOAuth2ScopeWorkAccessTokenFunction::accept(&access_hook, &mut access);
    SaOAuth2ScopeWorkClientTokenFunction::accept(&client_hook, &mut client);
    assert_eq!(
        access.grant_type.as_deref(),
        Some(GrantType::AUTHORIZATION_CODE)
    );
    assert_eq!(
        client.grant_type.as_deref(),
        Some(GrantType::CLIENT_CREDENTIALS)
    );
}

#[test]
fn authorization_server_view_and_login_functions_preserve_java_shapes() {
    let confirm = |client_id: &str, scopes: &[String]| serde_json::json!({"client_id": client_id, "scopes": scopes});
    let not_login = || serde_json::json!("当前会话在 OAuth-Server 认证中心尚未登录");
    let login = |name: &str, password: &str| serde_json::json!({"ok": name == "alice" && password == "correct"});

    assert_eq!(
        SaOAuth2ConfirmViewFunction::apply(&confirm, "app-a", &["openid".into()]),
        serde_json::json!({"client_id": "app-a", "scopes": ["openid"]})
    );
    assert_eq!(
        SaOAuth2NotLoginViewFunction::get(&not_login),
        serde_json::json!("当前会话在 OAuth-Server 认证中心尚未登录")
    );
    assert_eq!(
        SaOAuth2DoLoginHandleFunction::apply(&login, "alice", "correct"),
        serde_json::json!({"ok": true})
    );
}

impl RecordingValidator {
    fn fail_with(&self, message: &'static str) {
        *self.failure.lock().expect("failure lock must be available") = Some(message);
    }

    fn result(&self) -> Result<(), &'static str> {
        match *self.failure.lock().expect("failure lock must be available") {
            Some(message) => Err(message),
            None => Ok(()),
        }
    }
}

impl SaOAuth2AnnotationValidator for RecordingValidator {
    type Error = &'static str;

    fn check_access_token_scope(&self, scope: &[String]) -> Result<(), Self::Error> {
        self.calls
            .lock()
            .expect("calls lock must be available")
            .push(format!("access:{}", scope.join(",")));
        self.result()
    }

    fn check_client_token_scope(&self, scope: &[String]) -> Result<(), Self::Error> {
        self.calls
            .lock()
            .expect("calls lock must be available")
            .push(format!("client:{}", scope.join(",")));
        self.result()
    }

    fn check_client_id_secret(&self) -> Result<(), Self::Error> {
        self.calls
            .lock()
            .expect("calls lock must be available")
            .push("credentials".into());
        self.result()
    }
}

#[test]
fn oidc_defaults_and_serde_match_the_java_model() {
    let config = SaOAuth2OidcConfig::default();
    assert_eq!(config.iss, None);
    assert_eq!(config.id_token_timeout, 600);

    let json = serde_json::to_string(&config).expect("OIDC config must serialize");
    let decoded: SaOAuth2OidcConfig =
        serde_json::from_str(&json).expect("OIDC config must deserialize");
    assert_eq!(decoded, config);
}

#[test]
fn protocol_constants_match_the_java_baseline() {
    assert_eq!(GrantType::AUTHORIZATION_CODE, "authorization_code");
    assert_eq!(GrantType::REFRESH_TOKEN, "refresh_token");
    assert_eq!(GrantType::PASSWORD, "password");
    assert_eq!(GrantType::CLIENT_CREDENTIALS, "client_credentials");
    assert_eq!(GrantType::IMPLICIT, "implicit");

    assert_eq!(SaOAuth2Api::AUTHORIZE, "/oauth2/authorize");
    assert_eq!(SaOAuth2Api::TOKEN, "/oauth2/token");
    assert_eq!(SaOAuth2Api::REFRESH, "/oauth2/refresh");
    assert_eq!(SaOAuth2Api::REVOKE, "/oauth2/revoke");
    assert_eq!(SaOAuth2Api::CLIENT_TOKEN, "/oauth2/client_token");
    assert_eq!(SaOAuth2Api::DO_LOGIN, "/oauth2/doLogin");
    assert_eq!(SaOAuth2Api::DO_CONFIRM, "/oauth2/doConfirm");

    assert_eq!(SaOAuth2Param::RESPONSE_TYPE, "response_type");
    assert_eq!(SaOAuth2Param::CLIENT_ID, "client_id");
    assert_eq!(SaOAuth2Param::CLIENT_SECRET, "client_secret");
    assert_eq!(SaOAuth2Param::REDIRECT_URI, "redirect_uri");
    assert_eq!(SaOAuth2Param::SCOPE, "scope");
    assert_eq!(SaOAuth2Param::STATE, "state");
    assert_eq!(SaOAuth2Param::CODE, "code");
    assert_eq!(SaOAuth2Param::TOKEN, "token");
    assert_eq!(SaOAuth2Param::ACCESS_TOKEN, "access_token");
    assert_eq!(SaOAuth2Param::REFRESH_TOKEN, "refresh_token");
    assert_eq!(SaOAuth2Param::CLIENT_TOKEN, "client_token");
    assert_eq!(SaOAuth2Param::GRANT_TYPE, "grant_type");
    assert_eq!(SaOAuth2Param::USERNAME, "username");
    assert_eq!(SaOAuth2Param::PASSWORD, "password");
    assert_eq!(SaOAuth2Param::NAME, "name");
    assert_eq!(SaOAuth2Param::PWD, "pwd");
    assert_eq!(SaOAuth2Param::BUILD_REDIRECT_URI, "build_redirect_uri");
    assert_eq!(SaOAuth2Param::AUTHORIZATION, "Authorization");
    assert_eq!(SaOAuth2Param::NONCE, "nonce");

    assert_eq!(SaOAuth2ResponseType::CODE, "code");
    assert_eq!(SaOAuth2ResponseType::TOKEN, "token");
    assert_eq!(SaOAuth2TokenType::BASIC, "basic");
    assert_eq!(SaOAuth2TokenType::DIGEST, "digest");
    assert_eq!(SaOAuth2TokenType::BEARER, "bearer");
    assert_eq!(SaOAuth2TokenType::BASIC_TITLE, "Basic");
    assert_eq!(SaOAuth2TokenType::DIGEST_TITLE, "Digest");
    assert_eq!(SaOAuth2TokenType::BEARER_TITLE, "Bearer");
    assert_eq!(SaOAuth2ExtraField::UNION_ID, "unionid");
    assert_eq!(SaOAuth2ExtraField::OPEN_ID, "openid");
    assert_eq!(SaOAuth2ExtraField::USER_ID, "userid");
    assert_eq!(SaOAuth2ExtraField::ID_TOKEN, "id_token");

    assert_eq!(
        SaOAuth2Consts::OPENID_DEFAULT_DIGEST_PREFIX,
        "openid_default_digest_prefix"
    );
    assert_eq!(
        SaOAuth2Consts::UNIONID_DEFAULT_DIGEST_PREFIX,
        "unionid_default_digest_prefix"
    );
    assert_eq!(SaOAuth2Consts::OK, "ok");
    assert_eq!(SaOAuth2Consts::NOT_HANDLE, "{\"msg\": \"not handle\"}");
    assert_eq!(SaOAuth2Consts::FINALLY_WORK_SCOPE, "_FINALLY_WORK_SCOPE");
}

#[test]
fn error_codes_preserve_the_complete_java_code_set() {
    assert_eq!(
        [
            SaOAuth2ErrorCode::CODE_30101,
            SaOAuth2ErrorCode::CODE_30102,
            SaOAuth2ErrorCode::CODE_30103,
            SaOAuth2ErrorCode::CODE_30104,
            SaOAuth2ErrorCode::CODE_30105,
            SaOAuth2ErrorCode::CODE_30106,
            SaOAuth2ErrorCode::CODE_30107,
            SaOAuth2ErrorCode::CODE_30108,
            SaOAuth2ErrorCode::CODE_30109,
            SaOAuth2ErrorCode::CODE_30110,
            SaOAuth2ErrorCode::CODE_30111,
            SaOAuth2ErrorCode::CODE_30112,
            SaOAuth2ErrorCode::CODE_30113,
            SaOAuth2ErrorCode::CODE_30114,
            SaOAuth2ErrorCode::CODE_30115,
            SaOAuth2ErrorCode::CODE_30120,
            SaOAuth2ErrorCode::CODE_30122,
            SaOAuth2ErrorCode::CODE_30125,
            SaOAuth2ErrorCode::CODE_30126,
            SaOAuth2ErrorCode::CODE_30127,
            SaOAuth2ErrorCode::CODE_30131,
            SaOAuth2ErrorCode::CODE_30132,
            SaOAuth2ErrorCode::CODE_30133,
            SaOAuth2ErrorCode::CODE_30134,
            SaOAuth2ErrorCode::CODE_30141,
            SaOAuth2ErrorCode::CODE_30142,
            SaOAuth2ErrorCode::CODE_30151,
            SaOAuth2ErrorCode::CODE_30161,
            SaOAuth2ErrorCode::CODE_30191,
        ],
        [
            30101, 30102, 30103, 30104, 30105, 30106, 30107, 30108, 30109, 30110, 30111, 30112,
            30113, 30114, 30115, 30120, 30122, 30125, 30126, 30127, 30131, 30132, 30133, 30134,
            30141, 30142, 30151, 30161, 30191,
        ]
    );
}

#[test]
fn annotation_handlers_delegate_scopes_and_propagate_failures() {
    let validator = Arc::new(RecordingValidator::default());
    SaCheckAccessTokenHandler::new(Arc::clone(&validator))
        .check(&SaCheckAccessToken {
            scope: vec!["profile".into(), "email".into()],
        })
        .expect("access-token scope validation must pass");
    SaCheckClientTokenHandler::new(Arc::clone(&validator))
        .check(&SaCheckClientToken {
            scope: vec!["server".into()],
        })
        .expect("client-token scope validation must pass");
    SaCheckClientIdSecretHandler::new(Arc::clone(&validator))
        .check(&SaCheckClientIdSecret)
        .expect("client credential validation must pass");
    assert_eq!(
        *validator
            .calls
            .lock()
            .expect("calls lock must be available"),
        ["access:profile,email", "client:server", "credentials"]
    );

    validator.fail_with("invalid-token");
    assert_eq!(
        SaCheckAccessTokenHandler::new(Arc::clone(&validator))
            .check(&SaCheckAccessToken::default()),
        Err("invalid-token")
    );
    assert_eq!(
        SaCheckClientTokenHandler::new(Arc::clone(&validator))
            .check(&SaCheckClientToken::default()),
        Err("invalid-token")
    );
    assert_eq!(
        SaCheckClientIdSecretHandler::new(validator).check(&SaCheckClientIdSecret),
        Err("invalid-token")
    );
}

#[test]
fn oauth2_plugin_registers_handlers_once_and_cleans_up_its_registry() {
    let validator = Arc::new(RecordingValidator::default());
    let registry = Arc::new(RecordingAnnotationRegistry::default());
    let plugin = SaTokenPluginForOAuth2::new(validator, Arc::clone(&registry));

    plugin.install();
    plugin.install();
    assert!(plugin.is_installed());
    assert_eq!(
        *registry.events.lock().expect("annotation registry lock"),
        ["access", "client", "credentials"]
    );

    plugin.destroy();
    plugin.destroy();
    assert!(!plugin.is_installed());
    assert_eq!(
        *registry.events.lock().expect("annotation registry lock"),
        ["access", "client", "credentials", "destroy"]
    );
}

#[test]
fn structured_exceptions_preserve_context_without_displaying_credentials() {
    assert_eq!(SaOAuth2Exception::throw_by(false, "ignored", 30191), Ok(()));
    let base = SaOAuth2Exception::throw_by(true, "invalid request", 30191)
        .expect_err("true must create an OAuth2 error");
    assert_eq!(base.code, 30191);
    assert_eq!(base.message, "invalid request");

    let access = SaOAuth2AccessTokenException::new("invalid access token", 30106)
        .with_access_token("access-secret");
    assert_eq!(access.access_token.as_deref(), Some("access-secret"));
    assert!(!access.to_string().contains("access-secret"));

    let access_scope = SaOAuth2AccessTokenScopeException::new("missing scope", 30108)
        .with_access_token("access-secret")
        .with_scope("profile");
    assert_eq!(access_scope.scope.as_deref(), Some("profile"));
    assert!(!access_scope.to_string().contains("access-secret"));

    let authorization_code = SaOAuth2AuthorizationCodeException::throw_by(
        true,
        "invalid code",
        "authorization-secret",
        30110,
    )
    .expect_err("true must create an authorization-code error");
    assert_eq!(
        authorization_code.authorization_code.as_deref(),
        Some("authorization-secret")
    );
    assert!(
        !authorization_code
            .to_string()
            .contains("authorization-secret")
    );

    let client_model =
        SaOAuth2ClientModelException::new("invalid client", 30105).with_client_id("app-a");
    assert_eq!(client_model.client_id.as_deref(), Some("app-a"));
    let client_scope = SaOAuth2ClientModelScopeException::new("unsigned scope", 30112)
        .with_client_id("app-a")
        .with_scope("email");
    assert_eq!(client_scope.scope.as_deref(), Some("email"));

    let client_token = SaOAuth2ClientTokenException::new("invalid client token", 30107)
        .with_client_token("client-secret");
    assert_eq!(client_token.client_token.as_deref(), Some("client-secret"));
    assert!(!client_token.to_string().contains("client-secret"));
    let client_token_scope = SaOAuth2ClientTokenScopeException::new("missing client scope", 30109)
        .with_client_token("client-secret")
        .with_scope("server");
    assert_eq!(client_token_scope.scope.as_deref(), Some("server"));
    assert!(!client_token_scope.to_string().contains("client-secret"));

    let refresh = SaOAuth2RefreshTokenException::throw_by(
        true,
        "invalid refresh token",
        Some("refresh-secret".into()),
        30111,
    )
    .expect_err("true must create a refresh-token error");
    assert_eq!(refresh.refresh_token.as_deref(), Some("refresh-secret"));
    assert!(!refresh.to_string().contains("refresh-secret"));
}

#[test]
fn server_and_client_defaults_match_java_and_registration_is_explicit() {
    let mut server = SaOAuth2ServerConfig::default();
    assert!(server.enable_authorization_code);
    assert!(server.enable_implicit);
    assert!(server.enable_password);
    assert!(server.enable_client_credentials);
    assert_eq!(server.code_timeout, 300);
    assert_eq!(server.access_token_timeout, 7_200);
    assert_eq!(server.refresh_token_timeout, 2_592_000);
    assert_eq!(server.client_token_timeout, 7_200);
    assert_eq!(server.max_access_token_count, 12);
    assert_eq!(server.max_refresh_token_count, 12);
    assert_eq!(server.max_client_token_count, 12);
    assert!(!server.is_new_refresh);
    assert_eq!(server.higher_scope, None);
    assert_eq!(server.lower_scope, None);
    assert!(!server.mode4_return_access_token);
    assert!(!server.hide_status_field);

    let mut client = SaClientModel::from_server_config(&server);
    client.client_id = Some("app-a".into());
    client.client_secret = Some("client-secret".into());
    client
        .add_contract_scopes(["profile".into()])
        .add_allow_redirect_uris(["https://client.example/callback".into()])
        .add_allow_grant_types([GrantType::AUTHORIZATION_CODE.into()]);
    assert!(!format!("{client:?}").contains("client-secret"));
    assert_eq!(client.access_token_timeout, server.access_token_timeout);
    server
        .add_client(client)
        .expect("client with an ID must be registered");
    assert!(server.clients.contains_key("app-a"));

    let error = server
        .add_client(SaClientModel::default())
        .expect_err("client without an ID must be rejected");
    assert_eq!(error.base.code, 30101);
}

#[test]
fn persisted_models_match_java_fields_expiry_and_secret_redaction() {
    let now = 1_700_000_000_000_i64;
    let access = AccessTokenModel {
        access_token: Some("access-secret".into()),
        refresh_token: Some("refresh-secret".into()),
        expires_time: now + 5_999,
        refresh_expires_time: now,
        client_id: Some("app-a".into()),
        login_id: Some(serde_json::json!(10001)),
        scopes: Some(vec!["profile".into()]),
        ..Default::default()
    };
    assert_eq!(access.expires_in_at(now), 5);
    assert_eq!(access.refresh_expires_in_at(now), -2);
    assert!(!format!("{access:?}").contains("access-secret"));
    assert!(!format!("{access:?}").contains("refresh-secret"));

    let client = ClientTokenModel {
        client_token: Some("client-secret".into()),
        expires_time: now + 1_000,
        ..Default::default()
    };
    assert_eq!(client.expires_in_at(now), 1);
    assert!(!format!("{client:?}").contains("client-secret"));

    let refresh = RefreshTokenModel {
        refresh_token: Some("refresh-secret".into()),
        expires_time: now + 999,
        ..Default::default()
    };
    assert_eq!(refresh.expires_in_at(now), -2);
    assert!(!format!("{refresh:?}").contains("refresh-secret"));

    let code = CodeModel {
        code: Some("code-secret".into()),
        nonce: Some("nonce-secret".into()),
        ..Default::default()
    };
    assert!(!format!("{code:?}").contains("code-secret"));
    assert!(!format!("{code:?}").contains("nonce-secret"));

    for json in [
        serde_json::to_string(&access).expect("access token must serialize"),
        serde_json::to_string(&client).expect("client token must serialize"),
        serde_json::to_string(&refresh).expect("refresh token must serialize"),
        serde_json::to_string(&code).expect("code must serialize"),
    ] {
        assert!(json.contains("create_time"));
    }

    let credentials = ClientIdAndSecretModel::new("app-a", "client-secret");
    assert_eq!(credentials.client_id.as_deref(), Some("app-a"));
    assert!(!format!("{credentials:?}").contains("client-secret"));
}

#[test]
fn authorization_request_and_oidc_models_preserve_validation_and_claim_names() {
    let mut request = RequestAuthModel::default();
    assert_eq!(
        request
            .check_model()
            .expect_err("client ID is required")
            .code,
        30101
    );
    request.client_id = Some("app-a".into());
    assert_eq!(
        request.check_model().expect_err("scope is required").code,
        30102
    );
    request.scopes = Some(vec!["openid".into()]);
    assert_eq!(
        request
            .check_model()
            .expect_err("redirect URI is required")
            .code,
        30103
    );
    request.redirect_uri = Some("https://client.example/callback".into());
    assert_eq!(
        request
            .check_model()
            .expect_err("login ID is required")
            .code,
        30104
    );
    request.login_id = Some(serde_json::json!(10001));
    request.nonce = Some("nonce-secret".into());
    request.check_model().expect("complete request must pass");
    assert!(!format!("{request:?}").contains("nonce-secret"));

    let id_token = IdTokenModel {
        auth_time: 1_700_000_000,
        nonce: Some("nonce-secret".into()),
        ..Default::default()
    };
    let json = serde_json::to_value(&id_token).expect("ID token model must serialize");
    assert_eq!(json["auth_time"], 1_700_000_000);
    assert!(!format!("{id_token:?}").contains("nonce-secret"));
}

#[test]
fn default_converter_preserves_java_scope_and_model_transitions() {
    let converter = SaOAuth2DataConverterDefaultImpl::new(Arc::new(DeterministicTokenGenerator));
    assert_eq!(
        converter.convert_scope_string_to_list("openid profile,email%20phone+address"),
        ["openid", "profile", "email", "phone", "address"]
    );
    assert_eq!(
        converter.convert_scope_list_to_string(&["openid".into(), "profile".into()]),
        "openid,profile"
    );
    assert_eq!(
        converter.convert_redirect_uri_string_to_list(" https://a.example ,https://b.example "),
        ["https://a.example", "https://b.example"]
    );

    let request = RequestAuthModel {
        client_id: Some("app-a".into()),
        scopes: Some(vec!["openid".into()]),
        login_id: Some(serde_json::json!(10001)),
        redirect_uri: Some("https://client.example/callback".into()),
        nonce: Some("nonce".into()),
        ..Default::default()
    };
    let code = converter
        .convert_request_auth_to_code(&request)
        .expect("code conversion");
    assert_eq!(code.code.as_deref(), Some("code-app-a"));
    assert_eq!(code.nonce.as_deref(), Some("nonce"));

    let access = converter
        .convert_code_to_access_token(&code, 7_200)
        .expect("access conversion");
    assert_eq!(access.access_token.as_deref(), Some("access-app-a"));
    assert_eq!(
        access.grant_type.as_deref(),
        Some(GrantType::AUTHORIZATION_CODE)
    );
    assert_eq!(access.token_type.as_deref(), Some("Bearer"));

    let refresh = converter
        .convert_access_token_to_refresh_token(&access, 86_400)
        .expect("refresh conversion");
    assert_eq!(refresh.refresh_token.as_deref(), Some("refresh-app-a"));
    let refreshed_access = converter
        .convert_refresh_token_to_access_token(&refresh, 7_200)
        .expect("refreshed access conversion");
    assert_eq!(
        refreshed_access.grant_type.as_deref(),
        Some(GrantType::REFRESH_TOKEN)
    );
    assert_eq!(refreshed_access.refresh_token, refresh.refresh_token);

    let client = SaClientModel {
        client_id: Some("app-a".into()),
        ..Default::default()
    };
    let client_token = converter
        .convert_sa_client_to_client_token(&client, &["server".into()])
        .expect("client-token conversion");
    assert_eq!(client_token.client_token.as_deref(), Some("client-app-a"));
    assert_eq!(
        client_token.grant_type.as_deref(),
        Some(GrantType::CLIENT_CREDENTIALS)
    );
}

#[tokio::test]
async fn async_oauth2_dao_preserves_keys_crud_indexes_ttl_and_missing_values() {
    let storage = Arc::new(SaTokenDaoMemory::new());
    let dao = SaOAuth2Dao::new(storage.clone(), "satoken", 300);
    let login_id = serde_json::json!(10001);
    assert_eq!(dao.splicing_code_save_key("C"), "satoken:oauth2:code:C");
    assert_eq!(
        dao.splicing_code_index_key("app-a", &login_id),
        "satoken:oauth2:code-index:app-a:10001"
    );
    assert_eq!(
        dao.splicing_access_token_rsd_value("app-a", &login_id),
        "access-token:app-a:10001"
    );

    let code = CodeModel {
        code: Some("C".into()),
        client_id: Some("app-a".into()),
        login_id: Some(login_id.clone()),
        nonce: Some("N".into()),
        ..Default::default()
    };
    dao.save_code(&code).await.expect("save code");
    dao.save_code_index(&code).await.expect("save code index");
    dao.save_code_nonce_index(&code)
        .await
        .expect("save nonce index");
    assert_eq!(dao.get_code("C").await.expect("get code"), Some(code));
    assert_eq!(
        dao.get_code_value("app-a", &login_id)
            .await
            .expect("get code index")
            .as_deref(),
        Some("C")
    );
    assert_eq!(
        dao.get_nonce("C").await.expect("get nonce").as_deref(),
        Some("N")
    );

    let now = i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock must be after Unix epoch")
            .as_millis(),
    )
    .expect("current timestamp must fit i64");
    let first = AccessTokenModel {
        access_token: Some("A1".into()),
        client_id: Some("app-a".into()),
        login_id: Some(login_id.clone()),
        expires_time: now + 60_000,
        ..Default::default()
    };
    let second = AccessTokenModel {
        access_token: Some("A2".into()),
        ..first.clone()
    };
    dao.save_access_token(&first)
        .await
        .expect("save first token");
    dao.save_access_token_index_and_adjust(&first, 1)
        .await
        .expect("index first token");
    dao.save_access_token(&second)
        .await
        .expect("save second token");
    dao.save_access_token_index_and_adjust(&second, 1)
        .await
        .expect("index second token");
    assert_eq!(
        dao.get_access_token_value_list_from_adjust_after("app-a", &login_id)
            .await
            .expect("get adjusted index"),
        ["A2"]
    );
    assert_eq!(
        dao.get_access_token("A1").await.expect("overflow lookup"),
        None
    );
    assert!(
        dao.get_access_token_index_map_from_adjust_after("app-a", &login_id)
            .await
            .expect("access index map")
            .contains_key("A2")
    );

    let refresh = RefreshTokenModel {
        refresh_token: Some("R1".into()),
        client_id: Some("app-a".into()),
        login_id: Some(login_id.clone()),
        expires_time: now + 60_000,
        ..Default::default()
    };
    dao.save_refresh_token(&refresh)
        .await
        .expect("save refresh token");
    dao.save_refresh_token_index_and_adjust(&refresh, 12)
        .await
        .expect("index refresh token");
    assert_eq!(
        dao.get_refresh_token_value_list_from_adjust_after("app-a", &login_id)
            .await
            .expect("refresh index"),
        ["R1"]
    );

    let client_token = ClientTokenModel {
        client_token: Some("CT1".into()),
        client_id: Some("app-a".into()),
        expires_time: now + 60_000,
        ..Default::default()
    };
    dao.save_client_token(&client_token)
        .await
        .expect("save client token");
    dao.save_client_token_index_and_adjust(&client_token, 12)
        .await
        .expect("index client token");
    assert_eq!(
        dao.get_client_token_value_list_from_adjust_after("app-a")
            .await
            .expect("client index"),
        ["CT1"]
    );

    dao.save_grant_scope(
        "app-a",
        &login_id,
        &["openid".into(), "profile".into()],
        7_200,
    )
    .await
    .expect("save grant scope");
    assert_eq!(
        dao.get_grant_scope("app-a", &login_id)
            .await
            .expect("get scope"),
        Some(vec!["openid".into(), "profile".into()])
    );
    dao.save_state("state-a").await.expect("save state");
    assert_eq!(
        dao.get_state("state-a")
            .await
            .expect("get state")
            .as_deref(),
        Some("state-a")
    );
    dao.delete_state("state-a").await.expect("delete state");
    assert_eq!(dao.get_state("state-a").await.expect("missing state"), None);

    SaTokenDao::set_object(
        storage.as_ref(),
        &dao.splicing_access_token_save_key("BROKEN"),
        &serde_json::json!({"access_token": 42}),
        60,
    )
    .expect("inject malformed backend value");
    assert!(
        dao.get_access_token("BROKEN").await.is_err(),
        "deserialization errors must not be downgraded to missing values"
    );
}

#[test]
fn default_loader_is_isolated_and_matches_java_digest_and_scope_rules() {
    let mut config = SaOAuth2ServerConfig {
        higher_scope: Some("admin, audit".into()),
        lower_scope: Some("openid profile".into()),
        ..Default::default()
    };
    let mut client = SaClientModel::from_server_config(&config);
    client.client_id = Some("app-a".into());
    client.client_secret = Some("secret".into());
    config
        .add_client(client.clone())
        .expect("configured client must be accepted");
    let loader = SaOAuth2DataLoaderDefaultImpl::new(Arc::new(config));

    assert_eq!(loader.get_client_model("app-a"), Some(client));
    assert_eq!(
        loader
            .get_client_model_not_null("missing")
            .expect_err("unknown client must fail")
            .base
            .code,
        30105
    );
    assert_eq!(
        loader.get_openid("app-a", "10001"),
        "a5e35a7e0b1173d7cb4122b8453b392c"
    );
    assert_eq!(
        loader.get_unionid("subject-a", "10001"),
        "8b9eba0b1a7767284a656f8fde16c153"
    );
    assert_eq!(loader.get_higher_scope_list(), ["admin", "audit"]);
    assert_eq!(loader.get_lower_scope_list(), ["openid profile"]);
}

#[test]
fn common_scope_handlers_enrich_access_tokens_and_propagate_missing_clients() {
    assert_eq!(
        [
            CommonScope::OPENID,
            CommonScope::UNIONID,
            CommonScope::USERID,
            CommonScope::OIDC,
        ],
        ["openid", "unionid", "userid", "oidc"]
    );

    let mut config = SaOAuth2ServerConfig::default();
    let mut client = SaClientModel::from_server_config(&config);
    client.client_id = Some("app-a".into());
    client.subject_id = Some("subject-a".into());
    config
        .add_client(client)
        .expect("scope-handler client must be configured");
    let loader: Arc<dyn SaOAuth2DataLoader> =
        Arc::new(SaOAuth2DataLoaderDefaultImpl::new(Arc::new(config)));
    let openid = OpenIdScopeHandler::new(Arc::clone(&loader));
    let unionid = UnionIdScopeHandler::new(Arc::clone(&loader));
    let userid = UserIdScopeHandler;
    let mut access = AccessTokenModel {
        client_id: Some("app-a".into()),
        login_id: Some(serde_json::json!(10001)),
        ..Default::default()
    };

    openid
        .work_access_token(&mut access)
        .expect("openid handler");
    unionid
        .work_access_token(&mut access)
        .expect("unionid handler");
    userid
        .work_access_token(&mut access)
        .expect("userid handler");
    let extra = access.extra_data.as_ref().expect("extra data must exist");
    assert_eq!(
        extra[SaOAuth2ExtraField::OPEN_ID],
        serde_json::json!("a5e35a7e0b1173d7cb4122b8453b392c")
    );
    assert_eq!(
        extra[SaOAuth2ExtraField::UNION_ID],
        serde_json::json!("8b9eba0b1a7767284a656f8fde16c153")
    );
    assert_eq!(extra[SaOAuth2ExtraField::USER_ID], serde_json::json!(10001));
    assert!(!openid.refresh_access_token_is_work());

    let mut missing = AccessTokenModel {
        client_id: Some("missing".into()),
        login_id: Some(serde_json::json!(10001)),
        ..Default::default()
    };
    assert_eq!(
        unionid
            .work_access_token(&mut missing)
            .expect_err("unknown client must fail")
            .code,
        30105
    );
}

#[test]
fn oidc_scope_handler_builds_claims_through_explicit_context_and_signer_ports() {
    let config = Arc::new(SaOAuth2ServerConfig::default());
    let signer = Arc::new(RecordingIdTokenGenerator::default());
    let context = Arc::new(|_: &AccessTokenModel| {
        Ok(OidcScopeContext {
            issuer: "https://issuer.example".into(),
            audience: "app-a".into(),
            auth_time: 1_700_000_000,
            nonce: "nonce-a".into(),
        })
    });
    let handler = OidcScopeHandler::new(config, context, signer.clone());
    let mut access = AccessTokenModel {
        login_id: Some(serde_json::json!(10001)),
        ..Default::default()
    };

    handler
        .work_access_token(&mut access)
        .expect("OIDC scope must be generated");
    assert!(handler.refresh_access_token_is_work());
    assert_eq!(
        access.extra_data.as_ref().expect("extra data")["id_token"],
        serde_json::json!("signed-id-token")
    );
    let model = signer
        .model
        .lock()
        .expect("ID-token model lock")
        .clone()
        .expect("signer must receive claims");
    assert_eq!(model.iss.as_deref(), Some("https://issuer.example"));
    assert_eq!(model.sub, Some(serde_json::json!(10001)));
    assert_eq!(model.aud.as_deref(), Some("app-a"));
    assert_eq!(model.azp.as_deref(), Some("app-a"));
    assert_eq!(model.nonce.as_deref(), Some("nonce-a"));
    assert_eq!(model.auth_time, 1_700_000_000);
    assert_eq!(model.exp - model.iat, 600);
}

#[tokio::test]
async fn default_generator_covers_code_refresh_client_and_state_lifecycle() {
    let dao = Arc::new(SaOAuth2Dao::new(
        Arc::new(SaTokenDaoMemory::new()),
        "satoken",
        300,
    ));
    let converter = Arc::new(SaOAuth2DataConverterDefaultImpl::new(Arc::new(
        DeterministicTokenGenerator,
    )));
    let mut config = SaOAuth2ServerConfig::default();
    let mut client = SaClientModel::from_server_config(&config);
    client.client_id = Some("app-a".into());
    client.client_secret = Some("secret".into());
    client.contract_scopes = vec!["openid".into(), "profile".into()];
    config
        .add_client(client)
        .expect("test client must be configured");
    let config = Arc::new(config);
    let loader = Arc::new(SaOAuth2DataLoaderDefaultImpl::new(Arc::clone(&config)));
    let generator = Arc::new(SaOAuth2DataGenerateDefaultImpl::new(
        Arc::clone(&dao),
        Arc::clone(&converter),
        Arc::clone(&loader),
        Arc::new(TestGenerateHooks),
    ));
    let manager = SaOAuth2Manager::new(Arc::new(SaOAuth2Runtime::new(
        Arc::clone(&config),
        loader,
        Arc::new(SaOAuth2DataResolverDefaultImpl::new(false, false)),
        converter,
        generator.clone(),
        Arc::clone(&dao),
    )));
    assert_eq!(manager.server_config().code_timeout, 300);
    assert!(Arc::ptr_eq(manager.dao(), &dao));
    let request = RequestAuthModel {
        client_id: Some("app-a".into()),
        scopes: Some(vec!["openid".into()]),
        login_id: Some(serde_json::json!(10001)),
        redirect_uri: Some("https://client.example/callback".into()),
        nonce: Some("nonce-a".into()),
        ..Default::default()
    };

    let code = generator
        .generate_code(&request)
        .await
        .expect("generate code");
    let code_value = code
        .code
        .clone()
        .expect("generated code must contain a value");
    let access = generator
        .generate_access_token(&code_value)
        .await
        .expect("exchange code once");
    assert_eq!(access.access_token.as_deref(), Some("access-app-a"));
    assert_eq!(
        generator
            .generate_access_token(&code_value)
            .await
            .expect_err("authorization code must be single-use")
            .code(),
        30110
    );
    let refreshed = generator
        .refresh_access_token(
            access
                .refresh_token
                .as_deref()
                .expect("refresh token must be attached"),
        )
        .await
        .expect("refresh access token");
    assert_eq!(
        refreshed.grant_type.as_deref(),
        Some(GrantType::REFRESH_TOKEN)
    );
    let client_token = generator
        .generate_client_token("app-a", &["profile".into()])
        .await
        .expect("generate client token");
    assert_eq!(
        client_token.grant_type.as_deref(),
        Some(GrantType::CLIENT_CREDENTIALS)
    );

    assert_eq!(
        generator
            .build_redirect_uri(
                "https://client.example/callback?x=1",
                &code_value,
                Some("state-a"),
            )
            .await
            .expect("authorization redirect"),
        format!("https://client.example/callback?x=1&code={code_value}&state=state-a")
    );
    assert_eq!(
        generator
            .build_implicit_redirect_uri(
                "https://client.example/callback",
                "access-app-a",
                Some("state-b"),
            )
            .await
            .expect("implicit redirect"),
        "https://client.example/callback#token=access-app-a&state=state-b"
    );
    assert_eq!(
        generator
            .check_state("state-a")
            .await
            .expect_err("state replay must fail")
            .code(),
        30127
    );
}

#[tokio::test]
async fn async_grant_handlers_cover_code_password_refresh_and_error_codes() {
    let dao = Arc::new(SaOAuth2Dao::new(
        Arc::new(SaTokenDaoMemory::new()),
        "satoken",
        300,
    ));
    let converter = Arc::new(SaOAuth2DataConverterDefaultImpl::new(Arc::new(
        DeterministicTokenGenerator,
    )));
    let mut config = SaOAuth2ServerConfig::default();
    let mut client = SaClientModel::from_server_config(&config);
    client.client_id = Some("app-a".into());
    client.client_secret = Some("secret".into());
    client.contract_scopes = vec!["openid".into()];
    config.add_client(client).expect("grant test client");
    let loader = Arc::new(SaOAuth2DataLoaderDefaultImpl::new(Arc::new(config)));
    let concrete_generator = Arc::new(SaOAuth2DataGenerateDefaultImpl::new(
        Arc::clone(&dao),
        converter,
        Arc::clone(&loader),
        Arc::new(TestGenerateHooks),
    ));
    let generator: Arc<dyn SaOAuth2DataGenerate> = concrete_generator.clone();
    let resolver: Arc<dyn SaOAuth2DataResolver> =
        Arc::new(SaOAuth2DataResolverDefaultImpl::new(false, false));

    let request_auth = RequestAuthModel {
        client_id: Some("app-a".into()),
        scopes: Some(vec!["openid".into()]),
        login_id: Some(serde_json::json!(10001)),
        redirect_uri: Some("https://client.example/callback".into()),
        ..Default::default()
    };
    let code = concrete_generator
        .generate_code(&request_auth)
        .await
        .expect("authorization code")
        .code
        .expect("code value");
    let checker = Arc::new(TestAuthorizationCodeChecker { code: code.clone() });
    let authorization =
        AuthorizationCodeGrantTypeHandler::new(resolver, Arc::clone(&generator), checker);
    let authorization_request = SaOAuth2Request {
        params: [
            ("client_id".into(), "app-a".into()),
            ("client_secret".into(), "secret".into()),
            ("code".into(), code),
            (
                "redirect_uri".into(),
                "https://client.example/callback".into(),
            ),
        ]
        .into(),
        ..Default::default()
    };
    let access = authorization
        .get_access_token(&authorization_request, "app-a", &["openid".into()])
        .await
        .expect("authorization-code exchange");
    assert_eq!(
        authorization.handler_grant_type(),
        GrantType::AUTHORIZATION_CODE
    );

    let password = PasswordGrantTypeHandler::new(
        Arc::clone(&generator),
        Arc::new(|username: &str, password: &str| {
            if username == "alice" && password == "correct" {
                Ok(PasswordAuthResult::new(serde_json::json!(10001)))
            } else {
                Ok(PasswordAuthResult::default())
            }
        }),
    );
    let password_request = SaOAuth2Request {
        params: [
            ("username".into(), "alice".into()),
            ("password".into(), "correct".into()),
        ]
        .into(),
        ..Default::default()
    };
    let password_access = password
        .get_access_token(&password_request, "app-a", &["openid".into()])
        .await
        .expect("password exchange");
    assert_eq!(
        password_access.grant_type.as_deref(),
        Some(GrantType::PASSWORD)
    );
    let refresh_token = password_access
        .refresh_token
        .as_deref()
        .expect("password grant creates refresh token");
    let refresh = RefreshTokenGrantTypeHandler::new(Arc::clone(&dao), generator);
    let refresh_request = SaOAuth2Request {
        params: [("refresh_token".into(), refresh_token.into())].into(),
        ..Default::default()
    };
    assert_eq!(
        refresh
            .get_access_token(&refresh_request, "wrong-client", &[])
            .await
            .expect_err("refresh token client must match")
            .code(),
        30122
    );
    assert_eq!(
        refresh
            .get_access_token(&refresh_request, "app-a", &[])
            .await
            .expect("refresh exchange")
            .grant_type
            .as_deref(),
        Some(GrantType::REFRESH_TOKEN)
    );
    assert_eq!(
        PasswordGrantTypeHandler::new(
            concrete_generator.clone(),
            Arc::new(|_: &str, _: &str| Ok(PasswordAuthResult::default())),
        )
        .get_access_token(&password_request, "app-a", &["openid".into()])
        .await
        .expect_err("missing login ID must fail")
        .code(),
        30161
    );
    assert_eq!(access.client_id.as_deref(), Some("app-a"));

    let strategy_client = SaClientModel {
        client_id: Some("app-a".into()),
        client_secret: Some("secret".into()),
        allow_grant_types: vec![GrantType::PASSWORD.into()],
        ..Default::default()
    };
    let strategy = SaOAuth2Strategy::new(
        Arc::new(SaOAuth2ServerConfig::default()),
        Arc::new(SaOAuth2DataResolverDefaultImpl::new(false, false)),
        Arc::new(move |client_id: &str, secret: &str, scopes: &[String]| {
            if client_id == "app-a" && secret == "secret" && scopes == ["openid"] {
                Ok(strategy_client.clone())
            } else {
                Err(sa_token_core::exception::SaTokenException::with_code(
                    30115,
                    "invalid client credentials or scopes",
                ))
            }
        }),
    );
    let scope_loader: Arc<dyn SaOAuth2DataLoader> = loader;
    strategy
        .register_scope_handler(Arc::new(OpenIdScopeHandler::new(scope_loader)))
        .expect("register scope handler");
    strategy
        .register_grant_type_handler(Arc::new(PasswordGrantTypeHandler::new(
            concrete_generator,
            Arc::new(|_: &str, _: &str| Ok(PasswordAuthResult::new(serde_json::json!(10001)))),
        )))
        .expect("register grant handler");
    let strategy_request = SaOAuth2Request {
        params: [
            ("grant_type".into(), GrantType::PASSWORD.into()),
            ("client_id".into(), "app-a".into()),
            ("client_secret".into(), "secret".into()),
            ("scope".into(), "openid".into()),
            ("username".into(), "alice".into()),
            ("password".into(), "correct".into()),
        ]
        .into(),
        ..Default::default()
    };
    assert_eq!(
        strategy
            .grant_type_auth(&strategy_request)
            .await
            .expect("strategy grant dispatch")
            .grant_type
            .as_deref(),
        Some(GrantType::PASSWORD)
    );
    let mut scoped_access = AccessTokenModel {
        client_id: Some("app-a".into()),
        login_id: Some(serde_json::json!(10001)),
        scopes: Some(vec![CommonScope::OPENID.into()]),
        ..Default::default()
    };
    SaOAuth2GenerateHooks::work_access_token_by_scope(&strategy, &mut scoped_access)
        .expect("strategy scope dispatch");
    assert!(
        scoped_access
            .extra_data
            .as_ref()
            .is_some_and(|extra| extra.contains_key(SaOAuth2ExtraField::OPEN_ID))
    );
}

#[tokio::test]
async fn async_template_validates_clients_redirects_scopes_and_revokes_indexes() {
    let dao = Arc::new(SaOAuth2Dao::new(
        Arc::new(SaTokenDaoMemory::new()),
        "satoken",
        300,
    ));
    let mut config = SaOAuth2ServerConfig {
        higher_scope: Some("admin".into()),
        lower_scope: Some("openid".into()),
        ..Default::default()
    };
    let mut client = SaClientModel::from_server_config(&config);
    client.client_id = Some("app-a".into());
    client.client_secret = Some("secret".into());
    client.contract_scopes = vec!["openid".into(), "profile".into(), "admin".into()];
    client.allow_redirect_uris = vec![
        "https://client.example/callback".into(),
        "https://client.example/app/*".into(),
    ];
    config.add_client(client).expect("template test client");
    let loader: Arc<dyn SaOAuth2DataLoader> =
        Arc::new(SaOAuth2DataLoaderDefaultImpl::new(Arc::new(config)));
    let template = Arc::new(SaOAuth2Template::new(loader, Arc::clone(&dao)));
    let util = SaOAuth2Util::new(Arc::clone(&template));

    assert_eq!(
        util.check_client_secret_and_scope("app-a", "secret", &["profile".into()])
            .expect("client validation")
            .client_id
            .as_deref(),
        Some("app-a")
    );
    assert_eq!(
        util.check_client_secret_and_scope("app-a", "wrong", &[])
            .expect_err("wrong secret")
            .code(),
        30115
    );
    assert_eq!(
        util.check_client_secret_and_scope("app-a", "secret", &["unsigned".into()])
            .expect_err("unsigned scope")
            .code(),
        30112
    );
    template
        .check_redirect_uri("app-a", "https://client.example/callback?state=a")
        .expect("exact redirect");
    template
        .check_redirect_uri("app-a", "https://client.example/app/page")
        .expect("terminal wildcard redirect");
    assert_eq!(
        template
            .check_redirect_uri("app-a", "https://client.example@attacker.example/")
            .expect_err("userinfo-style bypass must fail")
            .code(),
        30113
    );
    assert_eq!(
        SaOAuth2Template::check_redirect_uri_list_normal(&["https://*.example/callback".into()])
            .expect_err("middle wildcard must fail")
            .code(),
        30114
    );

    let code = CodeModel {
        code: Some("code-a".into()),
        client_id: Some("app-a".into()),
        login_id: Some(serde_json::json!(10001)),
        scopes: Some(vec!["profile".into()]),
        redirect_uri: Some("https://client.example/callback".into()),
        ..Default::default()
    };
    dao.save_code(&code).await.expect("save code");
    template
        .check_gain_token_param(
            "code-a",
            "app-a",
            "secret",
            Some("https://client.example/callback"),
        )
        .await
        .expect("code parameters");
    assert_eq!(
        template
            .check_gain_token_param(
                "code-a",
                "app-a",
                "secret",
                Some("https://client.example/other"),
            )
            .await
            .expect_err("redirect mismatch")
            .code(),
        30120
    );

    let access = AccessTokenModel {
        access_token: Some("access-a".into()),
        expires_time: -1,
        client_id: Some("app-a".into()),
        login_id: Some(serde_json::json!(10001)),
        scopes: Some(vec!["profile".into()]),
        ..Default::default()
    };
    dao.save_access_token(&access).await.expect("save access");
    dao.save_access_token_index_and_adjust(&access, 12)
        .await
        .expect("save access index");
    util.check_access_token_scope("access-a", &["profile".into()])
        .await
        .expect("access scope");
    assert_eq!(
        util.check_access_token_scope("access-a", &["admin".into()])
            .await
            .expect_err("missing access scope")
            .code(),
        30108
    );
    template
        .revoke_access_token("access-a")
        .await
        .expect("revoke access");
    assert_eq!(
        util.check_access_token("access-a")
            .await
            .expect_err("revoked access")
            .code(),
        30106
    );

    dao.save_grant_scope("app-a", &serde_json::json!(10001), &["profile".into()], 300)
        .await
        .expect("save grant scope");
    assert!(
        !template
            .is_need_careful_confirm(
                &serde_json::json!(10001),
                "app-a",
                &["openid".into(), "profile".into()],
            )
            .await
            .expect("recent grant decision")
    );
    assert!(
        template
            .is_need_careful_confirm(&serde_json::json!(10001), "app-a", &["admin".into()])
            .await
            .expect("higher scope decision")
    );
}

#[tokio::test]
async fn framework_neutral_processor_runs_authorize_token_client_and_revoke_flows() {
    let dao = Arc::new(SaOAuth2Dao::new(
        Arc::new(SaTokenDaoMemory::new()),
        "satoken",
        300,
    ));
    let mut config = SaOAuth2ServerConfig {
        lower_scope: Some("openid".into()),
        ..Default::default()
    };
    let mut client = SaClientModel::from_server_config(&config);
    client.client_id = Some("app-a".into());
    client.client_secret = Some("secret".into());
    client.contract_scopes = vec!["openid".into()];
    client.allow_redirect_uris = vec!["https://client.example/callback".into()];
    client.allow_grant_types = vec![
        GrantType::AUTHORIZATION_CODE.into(),
        GrantType::IMPLICIT.into(),
        GrantType::CLIENT_CREDENTIALS.into(),
    ];
    config.add_client(client).expect("processor client");
    let config = Arc::new(config);
    let concrete_loader = Arc::new(SaOAuth2DataLoaderDefaultImpl::new(Arc::clone(&config)));
    let loader: Arc<dyn SaOAuth2DataLoader> = concrete_loader.clone();
    let resolver: Arc<dyn SaOAuth2DataResolver> =
        Arc::new(SaOAuth2DataResolverDefaultImpl::new(false, false));
    let template = Arc::new(SaOAuth2Template::new(Arc::clone(&loader), Arc::clone(&dao)));
    let converter = Arc::new(SaOAuth2DataConverterDefaultImpl::new(Arc::new(
        DeterministicTokenGenerator,
    )));
    let concrete_generator = Arc::new(SaOAuth2DataGenerateDefaultImpl::new(
        Arc::clone(&dao),
        converter,
        concrete_loader,
        Arc::new(TestGenerateHooks),
    ));
    let generator: Arc<dyn SaOAuth2DataGenerate> = concrete_generator;
    let strategy = Arc::new(SaOAuth2Strategy::new(
        Arc::clone(&config),
        Arc::clone(&resolver),
        template.clone(),
    ));
    strategy
        .register_grant_type_handler(Arc::new(AuthorizationCodeGrantTypeHandler::new(
            Arc::clone(&resolver),
            Arc::clone(&generator),
            template.clone(),
        )))
        .expect("authorization-code handler");
    let processor = SaOAuth2ServerProcessor::new(
        config,
        resolver,
        Arc::clone(&generator),
        Arc::clone(&template),
        strategy,
    );
    let authorize_request = SaOAuth2Request {
        params: [
            ("response_type".into(), "code".into()),
            ("client_id".into(), "app-a".into()),
            ("scope".into(), "openid".into()),
            (
                "redirect_uri".into(),
                "https://client.example/callback".into(),
            ),
            ("state".into(), "state-processor".into()),
        ]
        .into(),
        ..Default::default()
    };
    assert!(matches!(
        processor
            .authorize(&authorize_request, None)
            .await
            .expect("not-login view"),
        SaOAuth2ProcessorResponse::View(_)
    ));
    let redirect = match processor
        .authorize(&authorize_request, Some(serde_json::json!(10001)))
        .await
        .expect("authorize redirect")
    {
        SaOAuth2ProcessorResponse::Redirect(value) => value,
        other => panic!("expected redirect, got {other:?}"),
    };
    let parsed = url::Url::parse(&redirect).expect("redirect URL");
    let code = parsed
        .query_pairs()
        .find_map(|(key, value)| (key == "code").then(|| value.into_owned()))
        .expect("redirect code");
    let token_request = SaOAuth2Request {
        params: [
            ("grant_type".into(), GrantType::AUTHORIZATION_CODE.into()),
            ("client_id".into(), "app-a".into()),
            ("client_secret".into(), "secret".into()),
            ("code".into(), code),
            (
                "redirect_uri".into(),
                "https://client.example/callback".into(),
            ),
            ("scope".into(), "openid".into()),
        ]
        .into(),
        ..Default::default()
    };
    let token_data = match processor
        .token(&token_request)
        .await
        .expect("token exchange")
    {
        SaOAuth2ProcessorResponse::Data(value) => value,
        other => panic!("expected token data, got {other:?}"),
    };
    assert_eq!(
        token_data["access_token"],
        serde_json::json!("access-app-a")
    );

    let client_request = SaOAuth2Request {
        params: [
            ("grant_type".into(), GrantType::CLIENT_CREDENTIALS.into()),
            ("client_id".into(), "app-a".into()),
            ("client_secret".into(), "secret".into()),
            ("scope".into(), "openid".into()),
        ]
        .into(),
        ..Default::default()
    };
    let client_data = match processor
        .client_token(&client_request)
        .await
        .expect("client token")
    {
        SaOAuth2ProcessorResponse::Data(value) => value,
        other => panic!("expected client-token data, got {other:?}"),
    };
    assert_eq!(
        client_data["client_token"],
        serde_json::json!("client-app-a")
    );

    let revoke_request = SaOAuth2Request {
        params: [
            ("client_id".into(), "app-a".into()),
            ("client_secret".into(), "secret".into()),
            ("access_token".into(), "access-app-a".into()),
        ]
        .into(),
        ..Default::default()
    };
    assert!(matches!(
        processor
            .revoke(&revoke_request)
            .await
            .expect("revoke response"),
        SaOAuth2ProcessorResponse::Data(_)
    ));
    assert_eq!(
        template
            .check_access_token("access-app-a")
            .await
            .expect_err("revoked token")
            .code(),
        30106
    );
}

#[test]
fn default_resolver_preserves_request_precedence_basic_bearer_and_response_shape() {
    let resolver = SaOAuth2DataResolverDefaultImpl::new(false, true);
    let basic = SaBase64Util::encode(b"basic-app:basic-secret");
    let request = SaOAuth2Request {
        params: std::collections::BTreeMap::from([
            ("client_id".into(), "param-app".into()),
            ("access_token".into(), "param-access".into()),
            ("scope".into(), "openid profile,email".into()),
            ("response_type".into(), "code".into()),
            (
                "redirect_uri".into(),
                "https://client.example/callback".into(),
            ),
        ]),
        headers: std::collections::BTreeMap::from([(
            "Authorization".into(),
            format!("Basic {basic}"),
        )]),
    };
    let credentials = resolver
        .read_client_id_and_secret(&request)
        .expect("client ID alone is allowed when secret parameter is absent");
    assert_eq!(credentials.client_id.as_deref(), Some("basic-app"));
    assert_eq!(credentials.client_secret.as_deref(), Some("basic-secret"));
    assert_eq!(
        resolver.read_access_token(&request).as_deref(),
        Some("param-access")
    );
    let auth = resolver.read_request_auth_model(&request, serde_json::json!(10001));
    assert_eq!(
        auth.scopes,
        Some(vec!["openid".into(), "profile".into(), "email".into()])
    );

    let bearer_request = SaOAuth2Request {
        headers: std::collections::BTreeMap::from([(
            "Authorization".into(),
            "Bearer header-token".into(),
        )]),
        ..Default::default()
    };
    assert_eq!(
        resolver.read_client_token(&bearer_request).as_deref(),
        Some("header-token")
    );
    assert_eq!(
        resolver
            .read_client_id_and_secret(&SaOAuth2Request::default())
            .expect_err("missing credentials must fail")
            .code(),
        30191
    );

    let now = i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock after epoch")
            .as_millis(),
    )
    .expect("timestamp fits i64");
    let access = AccessTokenModel {
        access_token: Some("A".into()),
        refresh_token: Some("R".into()),
        expires_time: now + 5_000,
        refresh_expires_time: now + 10_000,
        client_id: Some("app-a".into()),
        scopes: Some(vec!["openid".into(), "profile".into()]),
        token_type: Some("Bearer".into()),
        extra_data: Some(std::collections::BTreeMap::from([(
            "openid".into(),
            serde_json::json!("OID"),
        )])),
        ..Default::default()
    };
    let response = resolver.build_access_token_return_value(&access);
    assert_eq!(response["code"], 200);
    assert_eq!(response["scope"], "openid,profile");
    assert_eq!(response["openid"], "OID");

    let hidden = SaOAuth2DataResolverDefaultImpl::new(true, true).build_client_token_return_value(
        &ClientTokenModel {
            client_token: Some("CT".into()),
            token_type: Some("Bearer".into()),
            expires_time: now + 5_000,
            ..Default::default()
        },
    );
    assert!(!hidden.contains_key("code"));
    assert_eq!(hidden["client_token"], "CT");
    assert_eq!(hidden["access_token"], "CT");
}
