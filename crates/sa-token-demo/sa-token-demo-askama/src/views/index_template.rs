//! 首页 Askama 模板（对应 Java `templates/index.html` + `sa:*` 方言标签）。

use askama::Template;

/// 首页视图模型：布尔字段对应 Thymeleaf `sa:login` / `sa:hasRole` 等预计算结果。
#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    /// 当前是否登录（对应 `stp.isLogin()`）
    pub is_login: bool,
    /// Session 中的 name（对应 `stp.getSession().get("name")`）
    pub session_name: String,
    /// 对应 `sa:login`
    pub sa_login: bool,
    /// 对应 `sa:notLogin`
    pub sa_not_login: bool,
    /// 对应 `sa:hasRole="admin"`
    pub sa_has_role_admin: bool,
    /// 对应 `sa:hasRoleAnd="admin, ceo, cto"`
    pub sa_has_role_and: bool,
    /// 对应 `sa:hasRoleOr="admin, ceo, cto"`
    pub sa_has_role_or: bool,
    /// 对应 `sa:notRole="admin"`
    pub sa_not_role_admin: bool,
    /// 对应 `sa:hasPermission="user-add"`
    pub sa_has_permission_user_add: bool,
    /// 对应 `sa:hasPermissionAnd="user-add, user-delete, user-get"`
    pub sa_has_permission_and: bool,
    /// 对应 `sa:hasPermissionOr="user-add, user-delete, user-get"`
    pub sa_has_permission_or: bool,
    /// 对应 `sa:notPermission="user-add"`
    pub sa_not_permission_user_add: bool,
}
