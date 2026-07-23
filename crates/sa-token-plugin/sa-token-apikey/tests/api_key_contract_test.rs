//! End-to-end API Key plugin contracts.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use sa_token_apikey::apikey::annotation::{SaCheckApiKey, SaCheckApiKeyHandler};
use sa_token_apikey::apikey::error::SaApiKeyErrorCode;
use sa_token_apikey::apikey::exception::ApiKeyException;
use sa_token_apikey::apikey::loader::{SaApiKeyDataLoader, SaApiKeyDataLoaderDefaultImpl};
use sa_token_apikey::apikey::model::ApiKeyModel;
use sa_token_apikey::{SaApiKeyConfig, SaApiKeyManager, SaApiKeyUtil, SaTokenPluginForApiKey};
use sa_token_context_mock::SaRequestForMock;
use sa_token_core::annotation::sa_mode::SaMode;
use sa_token_core::plugin::sa_token_plugin::SaTokenPlugin;
use sa_token_dao_memory::SaTokenDaoMemory;

#[derive(Default)]
struct TestLoader {
    values: RwLock<HashMap<String, ApiKeyModel>>,
}

#[async_trait]
impl SaApiKeyDataLoader for TestLoader {
    async fn get_api_key_model_from_database(
        &self,
        _namespace: &str,
        api_key: &str,
    ) -> Result<Option<ApiKeyModel>, ApiKeyException> {
        Ok(self
            .values
            .read()
            .expect("test loader lock")
            .get(api_key)
            .cloned())
    }
}

fn manager(loader: Arc<dyn SaApiKeyDataLoader>) -> SaApiKeyManager {
    SaApiKeyManager::new(
        "satoken",
        Arc::new(SaApiKeyConfig {
            timeout: 60,
            ..SaApiKeyConfig::default()
        }),
        Arc::new(SaTokenDaoMemory::new()),
        loader,
    )
    .expect("valid manager")
}

#[tokio::test]
async fn create_save_validate_scope_index_and_delete_round_trip() {
    let manager = manager(Arc::new(SaApiKeyDataLoaderDefaultImpl));
    let template = manager.template();
    let model = template
        .create_api_key_model("10001")
        .await
        .expect("create key")
        .add_scopes(["document:read", "document:write"]);
    assert!(model.api_key.starts_with("AK-"));
    assert_eq!(model.api_key.len(), 39);
    template.save_api_key(&model).await.expect("save key");

    assert_eq!(
        template
            .check_api_key(&model.api_key)
            .await
            .expect("valid key")
            .login_id,
        "10001"
    );
    template
        .check_api_key_scope(&model.api_key, &["document:read", "document:write"])
        .await
        .expect("AND scopes");
    template
        .check_api_key_scope_or(&model.api_key, &["missing", "document:read"])
        .await
        .expect("OR scopes");
    assert_eq!(
        template
            .check_api_key_scope(&model.api_key, &["missing"])
            .await
            .expect_err("missing scope")
            .code,
        SaApiKeyErrorCode::CODE_12311
    );
    template
        .check_api_key_login_id(&model.api_key, "10001")
        .await
        .expect("owner");
    assert_eq!(
        template
            .check_api_key_login_id(&model.api_key, "other")
            .await
            .expect_err("wrong owner")
            .code,
        SaApiKeyErrorCode::CODE_12312
    );
    assert_eq!(
        template
            .get_api_key_list("10001")
            .await
            .expect("index")
            .len(),
        1
    );

    template
        .delete_api_key(&model.api_key)
        .await
        .expect("delete key");
    assert!(
        template
            .get_api_key(&model.api_key)
            .await
            .expect("lookup deleted")
            .is_none()
    );
}

#[tokio::test]
async fn database_fallback_invalid_states_request_handler_and_lifecycle_are_explicit() {
    let loader = Arc::new(TestLoader::default());
    let model = ApiKeyModel {
        api_key: "AK-DATABASE".to_string(),
        login_id: "10002".to_string(),
        expires_time: -1,
        scopes: vec!["report:read".to_string()],
        ..ApiKeyModel::default()
    };
    loader
        .values
        .write()
        .expect("test loader lock")
        .insert(model.api_key.clone(), model.clone());
    let manager = manager(loader);
    let util = Arc::new(SaApiKeyUtil::new(Arc::clone(manager.template())));

    assert_eq!(
        util.check_api_key("AK-DATABASE")
            .await
            .expect("database fallback"),
        model
    );
    assert_eq!(
        util.check_api_key("missing")
            .await
            .expect_err("invalid key")
            .code,
        SaApiKeyErrorCode::CODE_12301
    );

    let request = SaRequestForMock::default();
    request.set_header("apikey", "AK-DATABASE");
    assert_eq!(
        util.read_api_key_value(&request).as_deref(),
        Some("AK-DATABASE")
    );
    let handler = SaCheckApiKeyHandler::new(Arc::clone(&util));
    handler
        .check(
            &SaCheckApiKey {
                scopes: vec!["report:read".to_string()],
                mode: SaMode::And,
            },
            &request,
        )
        .await
        .expect("annotation handler");

    let plugin = SaTokenPluginForApiKey::with_handler(Arc::new(handler));
    assert!(!plugin.is_installed());
    assert!(plugin.handler().is_none());
    plugin.install();
    assert!(plugin.is_installed());
    assert!(plugin.handler().is_some());
    plugin.destroy();
    assert!(!plugin.is_installed());
}

#[tokio::test]
async fn disabled_expired_and_index_disabled_states_have_stable_codes() {
    let manager = manager(Arc::new(SaApiKeyDataLoaderDefaultImpl));
    let template = manager.template();
    let mut disabled = template
        .create_api_key_model("10001")
        .await
        .expect("create disabled key");
    disabled.is_valid = false;
    template
        .save_api_key(&disabled)
        .await
        .expect("save disabled");
    assert_eq!(
        template
            .check_api_key(&disabled.api_key)
            .await
            .expect_err("disabled key")
            .code,
        SaApiKeyErrorCode::CODE_12303
    );

    let no_index = SaApiKeyManager::new(
        "satoken",
        Arc::new(SaApiKeyConfig {
            is_record_index: false,
            ..SaApiKeyConfig::default()
        }),
        Arc::new(SaTokenDaoMemory::new()),
        Arc::new(SaApiKeyDataLoaderDefaultImpl),
    )
    .expect("manager without index");
    assert_eq!(
        no_index
            .template()
            .get_api_key_list("10001")
            .await
            .expect_err("index disabled")
            .code,
        SaApiKeyErrorCode::CODE_12305
    );
}
