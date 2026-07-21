//! 全局组件管理器（对应 Java `cn.dev33.satoken.SaManager`）。
use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

use crate::config::sa_token_config::SaTokenConfig;
use crate::context::sa_token_context::SaTokenContext;
use crate::context::sa_token_context_for_thread_local::SaTokenContextForThreadLocal;
use crate::dao::sa_token_dao::SaTokenDao;
use crate::json::sa_json_template::{SaJsonTemplate, SaJsonTemplateDefaultImpl};
use crate::listener::sa_token_listener::SaTokenListenerForLog;
use crate::listener::SaTokenListener;
use crate::log::sa_log::{SaLog, SaLogForConsole};
use crate::stp::stp_interface::{StpInterface, StpInterfaceDefaultImpl};
use crate::stp::stp_logic::StpLogic;

/// 全局组件管理器
///
/// 对应 Java `SaManager`，持有所有全局组件的引用。
pub struct SaManager;

static CONFIG: OnceLock<Arc<SaTokenConfig>> = OnceLock::new();
static DAO: OnceLock<Arc<dyn SaTokenDao>> = OnceLock::new();
static STP_INTERFACE: OnceLock<Arc<dyn StpInterface>> = OnceLock::new();
static CONTEXT: OnceLock<Arc<dyn SaTokenContext>> = OnceLock::new();
static JSON_TEMPLATE: OnceLock<Arc<dyn SaJsonTemplate>> = OnceLock::new();
static LOG: OnceLock<Arc<dyn SaLog>> = OnceLock::new();
static STP_LOGIC_MAP: OnceLock<RwLock<HashMap<String, Arc<StpLogic>>>> = OnceLock::new();
static LISTENERS: OnceLock<RwLock<Vec<Arc<dyn SaTokenListener>>>> = OnceLock::new();

impl SaManager {
    // ==================== Config ====================

    /// 设置全局配置
    pub fn set_config(config: Arc<SaTokenConfig>) {
        let _ = CONFIG.set(config);
    }

    /// 获取全局配置
    pub fn config() -> Arc<SaTokenConfig> {
        CONFIG
            .get_or_init(|| Arc::new(SaTokenConfig::default()))
            .clone()
    }

    // ==================== DAO ====================

    /// 设置持久化组件
    pub fn set_sa_token_dao(dao: Arc<dyn SaTokenDao>) {
        let _ = DAO.set(dao);
    }

    /// 获取持久化组件
    pub fn sa_token_dao() -> Arc<dyn SaTokenDao> {
        DAO.get()
            .expect("SaTokenDao not initialized. Call SaManager::set_sa_token_dao() first.")
            .clone()
    }

    /// 尝试获取持久化组件
    pub fn try_sa_token_dao() -> Option<Arc<dyn SaTokenDao>> {
        DAO.get().cloned()
    }

    // ==================== StpInterface ====================

    /// 设置权限数据源
    pub fn set_stp_interface(stp_interface: Arc<dyn StpInterface>) {
        let _ = STP_INTERFACE.set(stp_interface);
    }

    /// 获取权限数据源
    pub fn stp_interface() -> Arc<dyn StpInterface> {
        STP_INTERFACE
            .get_or_init(|| Arc::new(StpInterfaceDefaultImpl))
            .clone()
    }

    // ==================== Context ====================

    /// 设置上下文实现
    pub fn set_sa_token_context(context: Arc<dyn SaTokenContext>) {
        let _ = CONTEXT.set(context);
    }

    /// 获取上下文实现
    pub fn sa_token_context() -> Arc<dyn SaTokenContext> {
        CONTEXT
            .get_or_init(|| Arc::new(SaTokenContextForThreadLocal))
            .clone()
    }

    // ==================== JSON Template ====================

    /// 设置 JSON 模板
    pub fn set_sa_json_template(template: Arc<dyn SaJsonTemplate>) {
        let _ = JSON_TEMPLATE.set(template);
    }

    /// 获取 JSON 模板
    pub fn sa_json_template() -> Arc<dyn SaJsonTemplate> {
        JSON_TEMPLATE
            .get_or_init(|| Arc::new(SaJsonTemplateDefaultImpl))
            .clone()
    }

    // ==================== Log ====================

    /// 设置日志实现
    pub fn set_log(log: Arc<dyn SaLog>) {
        let _ = LOG.set(log);
    }

    /// 获取日志实现
    pub fn log() -> Arc<dyn SaLog> {
        LOG.get_or_init(|| Arc::new(SaLogForConsole)).clone()
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
