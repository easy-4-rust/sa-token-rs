//! `SaTokenDialect` —— 1:1 对应 Java
//! `cn.dev33.satoken.thymeleaf.dialect.SaTokenDialect`
//!
//! Java 注册 `sa:login` / `sa:hasRole` 等属性处理器。
//! Askama 为编译期模板，本类型提供同等断言 API，供视图模型预计算布尔字段。

use std::sync::Arc;

use sa_token_core::stp::stp_logic::StpLogic;

use super::sa_token_tag_processor::{SaTokenTagProcessor, TagPredicate};

/// 默认方言名/前缀（对应 Java `super("sa", ...)`）
pub const DEFAULT_DIALECT_NAME: &str = "sa";

/// 默认优先级（对应 Java 构造参数 `1000`）
pub const DEFAULT_PRECEDENCE: i32 = 1000;

/// Sa-Token Askama/Thymeleaf 方言（对应 Java `SaTokenDialect`）。
pub struct SaTokenDialect {
    /// 方言名称（对应 Java 父类 name / prefix）
    pub name: String,
    /// 优先级（对应 Java `precedence`）
    pub precedence: i32,
    /// 底层 StpLogic（对应 Java `stpLogic`）
    pub stp_logic: Arc<StpLogic>,
}

impl SaTokenDialect {
    /// 使用默认参数注册方言（对应 Java 无参构造）。
    pub fn new(stp_logic: Arc<StpLogic>) -> Self {
        Self::with_params(DEFAULT_DIALECT_NAME, DEFAULT_PRECEDENCE, stp_logic)
    }

    /// 自定义方言名、优先级与 StpLogic（对应 Java `(name, precedence, stpLogic)`）。
    pub fn with_params(name: impl Into<String>, precedence: i32, stp_logic: Arc<StpLogic>) -> Self {
        Self {
            name: name.into(),
            precedence,
            stp_logic,
        }
    }

    /// String 转列表（对应 Java `toArray` / `SaFoxUtil.convertStringToList`）。
    pub fn to_array(str: &str) -> Vec<String> {
        str.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// 构建全部标签处理器（对应 Java `getProcessors`）。
    pub fn processors(&self) -> Vec<SaTokenTagProcessor> {
        let prefix = self.name.clone();
        let logic = Arc::clone(&self.stp_logic);

        // 登录判断
        let login = Self::processor(&prefix, "login", {
            let logic = Arc::clone(&logic);
            Arc::new(move |_v| logic.is_login().unwrap_or(false))
        });
        let not_login = Self::processor(&prefix, "notLogin", {
            let logic = Arc::clone(&logic);
            Arc::new(move |_v| !logic.is_login().unwrap_or(false))
        });

        // 角色判断
        let has_role = Self::processor(&prefix, "hasRole", {
            let logic = Arc::clone(&logic);
            Arc::new(move |v| {
                let Some(role) = v else {
                    return false;
                };
                logic.has_role(role).unwrap_or(false)
            })
        });
        let has_role_and = Self::processor(&prefix, "hasRoleAnd", {
            let logic = Arc::clone(&logic);
            Arc::new(move |v| {
                let list = Self::to_array(v.unwrap_or(""));
                let refs: Vec<&str> = list.iter().map(String::as_str).collect();
                logic.has_role_and(&refs).unwrap_or(false)
            })
        });
        let has_role_or = Self::processor(&prefix, "hasRoleOr", {
            let logic = Arc::clone(&logic);
            Arc::new(move |v| {
                let list = Self::to_array(v.unwrap_or(""));
                let refs: Vec<&str> = list.iter().map(String::as_str).collect();
                logic.has_role_or(&refs).unwrap_or(false)
            })
        });
        let not_role = Self::processor(&prefix, "notRole", {
            let logic = Arc::clone(&logic);
            Arc::new(move |v| {
                let Some(role) = v else {
                    return true;
                };
                !logic.has_role(role).unwrap_or(false)
            })
        });
        let lack_role = Self::processor(&prefix, "lackRole", {
            let logic = Arc::clone(&logic);
            Arc::new(move |v| {
                let Some(role) = v else {
                    return true;
                };
                !logic.has_role(role).unwrap_or(false)
            })
        });

        // 权限判断
        let has_permission = Self::processor(&prefix, "hasPermission", {
            let logic = Arc::clone(&logic);
            Arc::new(move |v| {
                let Some(p) = v else {
                    return false;
                };
                logic.has_permission(p).unwrap_or(false)
            })
        });
        let has_permission_and = Self::processor(&prefix, "hasPermissionAnd", {
            let logic = Arc::clone(&logic);
            Arc::new(move |v| {
                let list = Self::to_array(v.unwrap_or(""));
                let refs: Vec<&str> = list.iter().map(String::as_str).collect();
                logic.has_permission_and(&refs).unwrap_or(false)
            })
        });
        let has_permission_or = Self::processor(&prefix, "hasPermissionOr", {
            let logic = Arc::clone(&logic);
            Arc::new(move |v| {
                let list = Self::to_array(v.unwrap_or(""));
                let refs: Vec<&str> = list.iter().map(String::as_str).collect();
                logic.has_permission_or(&refs).unwrap_or(false)
            })
        });
        let not_permission = Self::processor(&prefix, "notPermission", {
            let logic = Arc::clone(&logic);
            Arc::new(move |v| {
                let Some(p) = v else {
                    return true;
                };
                !logic.has_permission(p).unwrap_or(false)
            })
        });
        let lack_permission = Self::processor(&prefix, "lackPermission", {
            let logic = Arc::clone(&logic);
            Arc::new(move |v| {
                let Some(p) = v else {
                    return true;
                };
                !logic.has_permission(p).unwrap_or(false)
            })
        });

        vec![
            login,
            not_login,
            has_role,
            has_role_and,
            has_role_or,
            not_role,
            lack_role,
            has_permission,
            has_permission_and,
            has_permission_or,
            not_permission,
            lack_permission,
        ]
    }

    /// 按属性名查找处理器并求值（Askama 预计算辅助）。
    pub fn evaluate(&self, attr_name: &str, value: Option<&str>) -> bool {
        self.processors()
            .into_iter()
            .find(|p| p.attr_name == attr_name)
            .map(|p| p.evaluate(value))
            .unwrap_or(false)
    }

    /// 当前是否登录（对应模板中 `stp.isLogin()` / `sa:login`）。
    pub fn is_login(&self) -> bool {
        self.stp_logic.is_login().unwrap_or(false)
    }

    /// 构造单个处理器。
    fn processor(prefix: &str, attr_name: &str, fun: TagPredicate) -> SaTokenTagProcessor {
        SaTokenTagProcessor::new(prefix, attr_name, fun)
    }
}
