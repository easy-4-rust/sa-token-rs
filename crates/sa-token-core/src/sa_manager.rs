//! 全局组件管理器（对应 Java `cn.dev33.satoken.SaManager`）。
use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

use crate::config::sa_token_config::SaTokenConfig;
use crate::context::sa_token_context::SaTokenContext;
use crate::context::sa_token_context_for_thread_local::SaTokenContextForThreadLocal;
use crate::dao::sa_token_dao::SaTokenDao;
use crate::json::sa_json_template::SaJsonTemplate;
use crate::json::sa_json_template_default_impl::SaJsonTemplateDefaultImpl;
use crate::listener::SaListenerTrait as SaTokenListener;
use crate::listener::sa_token_listener_for_log::SaTokenListenerForLog;
use crate::log::sa_log::SaLog;
use crate::log::sa_log_for_console::SaLogForConsole;
use crate::same::sa_same_template::SaSameTemplate;
use crate::stp::stp_interface::StpInterface;
use crate::stp::stp_interface_default_impl::StpInterfaceDefaultImpl;
use crate::stp::stp_logic::StpLogic;
use crate::temp::sa_temp_template::{SaTempTemplate, SaTempTemplateDefault};

/// 全局组件管理器
///
/// 对应 Java `SaManager`，持有所有全局组件的引用。
pub struct SaManager;

static CONFIG: OnceLock<RwLock<Arc<SaTokenConfig>>> = OnceLock::new();
static DAO: OnceLock<RwLock<Arc<dyn SaTokenDao>>> = OnceLock::new();
static STP_INTERFACE: OnceLock<RwLock<Arc<dyn StpInterface>>> = OnceLock::new();
static CONTEXT: OnceLock<RwLock<Arc<dyn SaTokenContext>>> = OnceLock::new();
static JSON_TEMPLATE: OnceLock<RwLock<Arc<dyn SaJsonTemplate>>> = OnceLock::new();
static LOG: OnceLock<RwLock<Arc<dyn SaLog>>> = OnceLock::new();
static STP_LOGIC_MAP: OnceLock<RwLock<HashMap<String, Arc<StpLogic>>>> = OnceLock::new();
static LISTENERS: OnceLock<RwLock<Vec<Arc<dyn SaTokenListener>>>> = OnceLock::new();
static TEMP_TEMPLATE: OnceLock<RwLock<Arc<dyn SaTempTemplate>>> = OnceLock::new();
static SAME_TEMPLATE: OnceLock<RwLock<Arc<SaSameTemplate>>> = OnceLock::new();

impl SaManager {
    // ==================== Config ====================

    /// 设置全局配置
    pub fn set_config(config: Arc<SaTokenConfig>) {
        *CONFIG
            .get_or_init(|| RwLock::new(config.clone()))
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = config.clone();
    }

    /// 获取全局配置
    pub fn config() -> Arc<SaTokenConfig> {
        CONFIG
            .get_or_init(|| RwLock::new(Arc::new(SaTokenConfig::default())))
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    // ==================== DAO ====================

    /// 设置持久化组件
    pub fn set_sa_token_dao(dao: Arc<dyn SaTokenDao>) {
        *DAO.get_or_init(|| RwLock::new(dao.clone()))
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = dao.clone();
    }

    /// 获取持久化组件
    pub fn sa_token_dao() -> Arc<dyn SaTokenDao> {
        DAO.get()
            .expect("SaTokenDao not initialized. Call SaManager::set_sa_token_dao() first.")
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    /// 尝试获取持久化组件
    pub fn try_sa_token_dao() -> Option<Arc<dyn SaTokenDao>> {
        DAO.get().map(|dao| {
            dao.read()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .clone()
        })
    }

    // ==================== StpInterface ====================

    /// 设置权限数据源
    pub fn set_stp_interface(stp_interface: Arc<dyn StpInterface>) {
        *STP_INTERFACE
            .get_or_init(|| RwLock::new(stp_interface.clone()))
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = stp_interface.clone();
    }

    /// 获取权限数据源
    pub fn stp_interface() -> Arc<dyn StpInterface> {
        STP_INTERFACE
            .get_or_init(|| RwLock::new(Arc::new(StpInterfaceDefaultImpl)))
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    // ==================== Context ====================

    /// 设置上下文实现
    pub fn set_sa_token_context(context: Arc<dyn SaTokenContext>) {
        *CONTEXT
            .get_or_init(|| RwLock::new(context.clone()))
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = context.clone();
    }

    /// 获取上下文实现
    pub fn sa_token_context() -> Arc<dyn SaTokenContext> {
        CONTEXT
            .get_or_init(|| RwLock::new(Arc::new(SaTokenContextForThreadLocal)))
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    // ==================== JSON Template ====================

    /// 设置 JSON 模板
    pub fn set_sa_json_template(template: Arc<dyn SaJsonTemplate>) {
        *JSON_TEMPLATE
            .get_or_init(|| RwLock::new(template.clone()))
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = template.clone();
    }

    /// 获取 JSON 模板
    pub fn sa_json_template() -> Arc<dyn SaJsonTemplate> {
        JSON_TEMPLATE
            .get_or_init(|| RwLock::new(Arc::new(SaJsonTemplateDefaultImpl)))
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    // ==================== Log ====================

    /// 设置日志实现
    pub fn set_log(log: Arc<dyn SaLog>) {
        *LOG.get_or_init(|| RwLock::new(log.clone()))
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = log.clone();
    }

    /// 获取日志实现
    pub fn log() -> Arc<dyn SaLog> {
        LOG.get_or_init(|| RwLock::new(Arc::new(SaLogForConsole)))
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    // ==================== StpLogic ====================

    /// 注册 StpLogic
    pub fn put_stp_logic(stp_logic: Arc<StpLogic>) {
        let map = STP_LOGIC_MAP.get_or_init(|| RwLock::new(HashMap::new()));
        map.write()
            .unwrap()
            .insert(stp_logic.login_type().to_string(), stp_logic);
    }

    /// 移除 StpLogic
    pub fn remove_stp_logic(login_type: &str) {
        if let Some(map) = STP_LOGIC_MAP.get() {
            map.write().unwrap().remove(login_type);
        }
    }

    /// 获取 StpLogic
    pub fn get_stp_logic(login_type: &str) -> Option<Arc<StpLogic>> {
        STP_LOGIC_MAP
            .get()?
            .read()
            .unwrap()
            .get(login_type)
            .cloned()
    }

    // ==================== Listeners ====================

    /// 获取监听器列表
    pub fn listeners() -> &'static RwLock<Vec<Arc<dyn SaTokenListener>>> {
        LISTENERS.get_or_init(|| RwLock::new(vec![Arc::new(SaTokenListenerForLog)]))
    }

    /// 注册监听器
    pub fn register_listener(listener: Arc<dyn SaTokenListener>) {
        Self::listeners().write().unwrap().push(listener);
    }

    // ==================== TempTemplate ====================

    /// 设置临时 token 模板
    pub fn set_sa_temp_template(template: Arc<dyn SaTempTemplate>) {
        *TEMP_TEMPLATE
            .get_or_init(|| RwLock::new(template.clone()))
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = template.clone();
    }

    /// 获取临时 token 模板
    pub fn sa_temp_template() -> Arc<dyn SaTempTemplate> {
        TEMP_TEMPLATE
            .get_or_init(|| RwLock::new(Arc::new(SaTempTemplateDefault::default())))
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    // ==================== SameTemplate ====================

    /// Replaces the default Same-Token template.
    pub fn set_sa_same_template(template: Arc<SaSameTemplate>) {
        *SAME_TEMPLATE
            .get_or_init(|| RwLock::new(Arc::clone(&template)))
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Arc::clone(&template);
    }

    /// Returns the default Same-Token template.
    pub fn sa_same_template() -> Arc<SaSameTemplate> {
        SAME_TEMPLATE
            .get_or_init(|| RwLock::new(Arc::new(SaSameTemplate::default())))
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    // ==================== Rust 风格兼容门面 ====================

    /// Java `SaManager.getConfig()`
    pub fn get_config() -> Arc<SaTokenConfig> {
        Self::config()
    }
    /// Java `SaManager.getSaTokenDao()`
    pub fn get_sa_token_dao() -> Arc<dyn SaTokenDao> {
        Self::sa_token_dao()
    }

    /// Java `SaManager.getStpInterface()`
    pub fn get_stp_interface() -> Arc<dyn StpInterface> {
        Self::stp_interface()
    }

    /// Java `SaManager.getSaTokenContext()`
    pub fn get_sa_token_context() -> Arc<dyn SaTokenContext> {
        Self::sa_token_context()
    }

    /// Java `SaManager.getSaJsonTemplate()`
    pub fn get_sa_json_template() -> Arc<dyn SaJsonTemplate> {
        Self::sa_json_template()
    }

    /// Java `SaManager.getLog()`
    pub fn get_log() -> Arc<dyn SaLog> {
        Self::log()
    }

    // ==================== 初始化 ====================

    /// 初始化默认组件（测试用）
    pub fn init_defaults() {
        Self::set_config(Arc::new(SaTokenConfig::default()));
        Self::set_sa_json_template(Arc::new(SaJsonTemplateDefaultImpl));
        Self::set_log(Arc::new(SaLogForConsole));
        Self::put_stp_logic(Arc::new(StpLogic::new("login")));
    }

    /// 重置所有组件（测试用）
    pub fn reset() {
        if let Some(map) = STP_LOGIC_MAP.get() {
            map.write().unwrap().clear();
        }
        if let Some(listeners) = LISTENERS.get() {
            listeners.write().unwrap().clear();
            listeners
                .write()
                .unwrap()
                .push(Arc::new(SaTokenListenerForLog));
        }
    }
}
