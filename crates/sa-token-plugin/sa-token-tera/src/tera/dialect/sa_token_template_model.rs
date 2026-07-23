//! `SaTokenTemplateModel` —— 1:1 对应 Java
//! `cn.dev33.satoken.freemarker.dialect.SaTokenTemplateModel`
//!
//! Java 继承 FreeMarker `SimpleHash`，注册 `login` / `hasRole` 等标签。
//! Rust 使用 Tera `register_function`，函数名加 `sa_` 前缀避免冲突。

use std::sync::Arc;

use sa_token_core::stp::stp_logic::StpLogic;
use tera::Tera;

use super::sa_token_template_directive_model::SaTokenTemplateDirectiveModel;

/// 默认属性名（对应 Java `DEFAULT_ATTR_NAME = "value"`）
pub const DEFAULT_ATTR_NAME: &str = "value";

/// Sa-Token 模板模型（对应 Java `SaTokenTemplateModel`）
pub struct SaTokenTemplateModel {
    /// 底层 StpLogic（对应 Java `stpLogic` 字段）
    pub stp_logic: Arc<StpLogic>,
    /// 属性名（对应 Java 构造参数 `attrName`）
    pub attr_name: String,
}

impl SaTokenTemplateModel {
    /// 使用默认属性名创建（对应 Java 无参 / `StpLogic` 构造）。
    pub fn new(stp_logic: Arc<StpLogic>) -> Self {
        Self::with_attr(DEFAULT_ATTR_NAME, stp_logic)
    }

    /// 指定属性名与 StpLogic（对应 Java `(attrName, stpLogic)` 构造）。
    pub fn with_attr(attr_name: impl Into<String>, stp_logic: Arc<StpLogic>) -> Self {
        Self {
            stp_logic,
            attr_name: attr_name.into(),
        }
    }

    /// 将逗号分隔字符串转为列表（对应 Java `toArray` / `SaFoxUtil.convertStringToList`）。
    pub fn to_array(str: &str) -> Vec<String> {
        str.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// 注册全部鉴权函数到 Tera（对应 Java `put("login", ...)` 等）。
    ///
    /// 函数命名：`sa_login`、`sa_has_role`、`sa_has_permission_and` …
    pub fn register_into(&self, tera: &mut Tera) {
        let attr = self.attr_name.clone();
        let logic = Arc::clone(&self.stp_logic);

        // ---------- 登录 ----------
        // 对应 Java put("login", ...)
        {
            let logic = Arc::clone(&logic);
            let d = SaTokenTemplateDirectiveModel::new(
                attr.clone(),
                Arc::new(move |_v| logic.is_login().unwrap_or(false)),
            );
            tera.register_function("sa_login", d.into_tera_function());
        }
        // 对应 Java put("notLogin", ...)
        {
            let logic = Arc::clone(&logic);
            let d = SaTokenTemplateDirectiveModel::new(
                attr.clone(),
                Arc::new(move |_v| !logic.is_login().unwrap_or(false)),
            );
            tera.register_function("sa_not_login", d.into_tera_function());
        }

        // ---------- 角色 ----------
        {
            let logic = Arc::clone(&logic);
            let d = SaTokenTemplateDirectiveModel::new(
                attr.clone(),
                Arc::new(move |v| {
                    let Some(role) = v else {
                        return false;
                    };
                    logic.has_role(role).unwrap_or(false)
                }),
            );
            tera.register_function("sa_has_role", d.into_tera_function());
        }
        {
            let logic = Arc::clone(&logic);
            let d = SaTokenTemplateDirectiveModel::new(
                attr.clone(),
                Arc::new(move |v| {
                    let list = Self::to_array(v.unwrap_or(""));
                    let refs: Vec<&str> = list.iter().map(String::as_str).collect();
                    logic.has_role_and(&refs).unwrap_or(false)
                }),
            );
            tera.register_function("sa_has_role_and", d.into_tera_function());
        }
        {
            let logic = Arc::clone(&logic);
            let d = SaTokenTemplateDirectiveModel::new(
                attr.clone(),
                Arc::new(move |v| {
                    let list = Self::to_array(v.unwrap_or(""));
                    let refs: Vec<&str> = list.iter().map(String::as_str).collect();
                    logic.has_role_or(&refs).unwrap_or(false)
                }),
            );
            tera.register_function("sa_has_role_or", d.into_tera_function());
        }
        {
            let logic = Arc::clone(&logic);
            let d = SaTokenTemplateDirectiveModel::new(
                attr.clone(),
                Arc::new(move |v| {
                    let Some(role) = v else {
                        return true;
                    };
                    !logic.has_role(role).unwrap_or(false)
                }),
            );
            tera.register_function("sa_not_role", d.into_tera_function());
        }

        // ---------- 权限 ----------
        {
            let logic = Arc::clone(&logic);
            let d = SaTokenTemplateDirectiveModel::new(
                attr.clone(),
                Arc::new(move |v| {
                    let Some(p) = v else {
                        return false;
                    };
                    logic.has_permission(p).unwrap_or(false)
                }),
            );
            tera.register_function("sa_has_permission", d.into_tera_function());
        }
        {
            let logic = Arc::clone(&logic);
            let d = SaTokenTemplateDirectiveModel::new(
                attr.clone(),
                Arc::new(move |v| {
                    let list = Self::to_array(v.unwrap_or(""));
                    let refs: Vec<&str> = list.iter().map(String::as_str).collect();
                    logic.has_permission_and(&refs).unwrap_or(false)
                }),
            );
            tera.register_function("sa_has_permission_and", d.into_tera_function());
        }
        {
            let logic = Arc::clone(&logic);
            let d = SaTokenTemplateDirectiveModel::new(
                attr.clone(),
                Arc::new(move |v| {
                    let list = Self::to_array(v.unwrap_or(""));
                    let refs: Vec<&str> = list.iter().map(String::as_str).collect();
                    logic.has_permission_or(&refs).unwrap_or(false)
                }),
            );
            tera.register_function("sa_has_permission_or", d.into_tera_function());
        }
        {
            let logic = Arc::clone(&logic);
            let d = SaTokenTemplateDirectiveModel::new(
                attr.clone(),
                Arc::new(move |v| {
                    let Some(p) = v else {
                        return true;
                    };
                    !logic.has_permission(p).unwrap_or(false)
                }),
            );
            tera.register_function("sa_not_permission", d.into_tera_function());
        }
    }
}
