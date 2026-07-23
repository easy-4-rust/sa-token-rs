//! Quick login registration and auth hook (Java `SaQuickRegister`).

use sa_token_core::context::sa_holder::SaHolder;
use sa_token_core::exception::SaResult;
use sa_token_core::http_auth::basic::sa_http_basic_account::SaHttpBasicAccount;
use sa_token_core::http_auth::basic::sa_http_basic_template::SaHttpBasicTemplate;
use sa_token_core::router::sa_router::SaRouter;
use sa_token_core::stp::stp_util::StpUtil;
use sa_token_core::util::sa_fox_util;

use crate::quick::config::sa_quick_config::SaQuickConfig;
use crate::quick::sa_quick_manager::SaQuickManager;

/// Quick login bean registration helpers.
pub struct SaQuickRegister;

impl SaQuickRegister {
    /// Short prefix used by Java `@ConfigurationProperties`.
    pub const CONFIG_VERSION: &'static str = "sa";

    /// Creates default quick-login configuration.
    pub fn default_config() -> SaQuickConfig {
        SaQuickConfig::default()
    }

    /// Paths excluded from quick-login auth, matching Java filter defaults.
    pub fn default_exclude_paths() -> &'static [&'static str] {
        &["/favicon.ico", "/saLogin", "/doLogin", "/sa-res/**"]
    }

    /// Parses HTTP Basic credentials from the current request, if present.
    pub fn get_http_basic_account() -> Option<SaHttpBasicAccount> {
        let auth = SaHolder::get_request().get_header("Authorization");
        let auth = auth.as_deref()?;
        let (username, password) = SaHttpBasicTemplate::parse_authorization(auth)?;
        Some(SaHttpBasicAccount::new(username, password))
    }

    /// Runs quick-login auth logic for the current request.
    ///
    /// Returns `Ok(())` when the request may continue. Returns
    /// [`SaTokenException::BackResult`] when the caller should respond with login HTML
    /// or a JSON [`SaResultData`] payload.
    pub fn auth_check() -> SaResult<()> {
        let cfg = SaQuickManager::get_config();
        let include = sa_fox_util::convert_string_to_list(&cfg.include);
        let mut exclude = sa_fox_util::convert_string_to_list(&cfg.exclude);
        for path in Self::default_exclude_paths() {
            if !exclude.iter().any(|item| item == path) {
                exclude.push(path.to_string());
            }
        }
        let include_patterns: Vec<&str> = include.iter().map(String::as_str).collect();
        let exclude_patterns: Vec<&str> = exclude.iter().map(String::as_str).collect();

        SaRouter::match_paths(&include_patterns)
            .not_match(&exclude_patterns)
            .free(|_| {
                if !cfg.auth {
                    return Ok(());
                }
                if let Some(hba) = Self::get_http_basic_account() {
                    let res = cfg.do_login(&hba.username, &hba.password);
                    if !res.is_ok() {
                        let payload = serde_json::to_string(&res)
                            .unwrap_or_else(|_| res.message.clone());
                        return SaRouter::back_with(payload);
                    }
                    return Ok(());
                }
                if !StpUtil::is_login()? {
                    return SaRouter::back_with("forward:/saLogin");
                }
                Ok(())
            })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sa_token_core::config::sa_token_config::SaTokenConfig;
    use sa_token_core::context::mock::sa_request_for_mock::SaRequestForMock;
    use sa_token_core::context::mock::sa_response_for_mock::SaResponseForMock;
    use sa_token_core::context::mock::sa_storage_for_mock::SaStorageForMock;
    use sa_token_core::context::model::sa_token_context_model_box::SaTokenContextModelBox;
    use sa_token_core::context::model::sa_request::SaRequest;
    use sa_token_core::context::model::sa_response::SaResponse;
    use sa_token_core::context::model::sa_storage::SaStorage;
    use sa_token_core::context::sa_token_context::SaTokenContext;
    use sa_token_core::exception::SaTokenException;
    use sa_token_core::sa_manager::SaManager;
    use sa_token_core::stp::stp_logic::StpLogic;
    use sa_token_dao_memory::SaTokenDaoMemory;
    use std::sync::{Arc, Mutex};

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    struct TestContext {
        request: Arc<dyn SaRequest>,
        response: Arc<dyn SaResponse>,
        storage: Arc<dyn SaStorage>,
    }

    impl SaTokenContext for TestContext {
        fn set_context(
            &self,
            _request: Arc<dyn SaRequest>,
            _response: Arc<dyn SaResponse>,
            _storage: Arc<dyn SaStorage>,
        ) {
        }

        fn clear_context(&self) {}

        fn is_valid(&self) -> bool {
            true
        }

        fn request(&self) -> Arc<dyn SaRequest> {
            Arc::clone(&self.request)
        }

        fn response(&self) -> Arc<dyn SaResponse> {
            Arc::clone(&self.response)
        }

        fn storage(&self) -> Arc<dyn SaStorage> {
            Arc::clone(&self.storage)
        }

        fn model_box(&self) -> SaTokenContextModelBox {
            SaTokenContextModelBox::new(
                Arc::clone(&self.request),
                Arc::clone(&self.response),
                Arc::clone(&self.storage),
            )
        }
    }

    fn setup(path: &str) -> std::sync::MutexGuard<'static, ()> {
        let guard = TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        SaManager::reset();
        SaManager::set_config(Arc::new(SaTokenConfig::default()));
        SaManager::set_sa_token_dao(Arc::new(SaTokenDaoMemory::new()));
        let req = SaRequestForMock::default().with_url(path);
        let ctx = TestContext {
            request: Arc::new(req),
            response: Arc::new(SaResponseForMock::new()),
            storage: Arc::new(SaStorageForMock::new()),
        };
        SaManager::set_sa_token_context(Arc::new(ctx));
        SaManager::put_stp_logic(Arc::new(StpLogic::new("login")));
        SaQuickManager::set_config(SaQuickConfig::default());
        guard
    }

    #[test]
    fn unauthenticated_request_is_blocked() {
        let _guard = setup("/user/list");
        let err = SaQuickRegister::auth_check().expect_err("should redirect login");
        assert!(matches!(err, SaTokenException::BackResult { .. }));
    }

    #[test]
    fn login_route_is_excluded_from_auth() {
        let _guard = setup("/doLogin");
        SaQuickRegister::auth_check().expect("login route excluded");
    }
}
