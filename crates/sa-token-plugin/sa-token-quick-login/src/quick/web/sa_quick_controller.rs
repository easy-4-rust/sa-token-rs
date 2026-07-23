//! Login endpoints (Java `SaQuickController`).

use sa_token_core::util::sa_result::SaResultData;
use serde_json::Value;

use crate::quick::config::sa_quick_config::SaQuickConfig;
use crate::quick::sa_quick_manager::SaQuickManager;

/// Quick login HTTP handlers (framework-agnostic).
pub struct SaQuickController;

impl SaQuickController {
    /// Builds login page HTML using current configuration.
    pub fn sa_login_html(cfg: &SaQuickConfig) -> String {
        let copyright = if cfg.copr {
            r#"<footer>Sa-Token Quick Login</footer>"#
        } else {
            ""
        };
        format!(
            r#"<!DOCTYPE html>
<html lang="zh-CN">
<head><meta charset="utf-8"><title>{title}</title></head>
<body>
  <h1>{title}</h1>
  <form method="post" action="/doLogin">
    <p>账号：<input name="name" value="{name}"></p>
    <p>密码：<input name="pwd" type="password" value="{pwd}"></p>
    <button type="submit">登录</button>
  </form>
  {copyright}
</body>
</html>"#,
            title = cfg.title,
            name = cfg.name,
            pwd = cfg.pwd,
            copyright = copyright,
        )
    }

    /// Handles `/doLogin` using configured handler.
    pub fn do_login(name: &str, pwd: &str) -> SaResultData<Value> {
        SaQuickManager::get_config().do_login(name, pwd)
    }

    /// Convenience wrapper using global config for login page.
    pub fn sa_login_page() -> String {
        Self::sa_login_html(&SaQuickManager::get_config())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_page_contains_title_and_form() {
        let html = SaQuickController::sa_login_html(&SaQuickConfig::default());
        assert!(html.contains("Sa-Token 登录"));
        assert!(html.contains("/doLogin"));
    }
}
