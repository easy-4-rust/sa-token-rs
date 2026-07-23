//! Non-IoC `sa-token.properties` configuration loader.

use std::collections::HashMap;
use std::io::ErrorKind;
use std::path::Path;

use crate::config::sa_token_config::{SaTokenConfig, SaTokenStyle};
use crate::exception::{SaResult, SaTokenException};
use crate::stp::parameter::enums::sa_logout_mode::SaLogoutMode;
use crate::stp::parameter::enums::sa_logout_range::SaLogoutRange;
use crate::stp::parameter::enums::sa_replaced_login_exit_mode::SaReplacedLoginExitMode;
use crate::stp::parameter::enums::sa_replaced_range::SaReplacedRange;

/// Default properties path.
pub const DEFAULT_CONFIG_PATH: &str = "sa-token.properties";

/// Builds [`SaTokenConfig`] from Java-compatible properties.
pub struct SaTokenConfigFactory;

impl SaTokenConfigFactory {
    /// Loads [`DEFAULT_CONFIG_PATH`], or returns defaults when it does not exist.
    ///
    /// # Errors
    ///
    /// Returns code 10021 for I/O failures and 10022 for invalid values.
    pub fn create_config() -> SaResult<SaTokenConfig> {
        Self::create_config_from_path(DEFAULT_CONFIG_PATH)
    }

    /// Loads a properties file, or returns defaults when it does not exist.
    ///
    /// # Errors
    ///
    /// Returns code 10021 for I/O failures and 10022 for invalid values.
    pub fn create_config_from_path(path: impl AsRef<Path>) -> SaResult<SaTokenConfig> {
        let content = match std::fs::read_to_string(path.as_ref()) {
            Ok(content) => content,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                return Ok(SaTokenConfig::default());
            }
            Err(error) => {
                return Err(SaTokenException::with_code(
                    10021,
                    format!("配置文件({})加载失败: {error}", path.as_ref().display()),
                ));
            }
        };
        Self::create_config_from_properties(&content)
    }

    /// Parses Java properties text into [`SaTokenConfig`].
    ///
    /// # Errors
    ///
    /// Returns code 10022 when a recognized property has an invalid value.
    pub fn create_config_from_properties(content: &str) -> SaResult<SaTokenConfig> {
        let properties = parse_properties(content);
        let mut config = SaTokenConfig::default();

        assign_string(&properties, "tokenName", &mut config.token_name);
        assign(&properties, "timeout", &mut config.timeout)?;
        assign(&properties, "activeTimeout", &mut config.active_timeout)?;
        assign(
            &properties,
            "dynamicActiveTimeout",
            &mut config.dynamic_active_timeout,
        )?;
        assign(&properties, "isConcurrent", &mut config.is_concurrent)?;
        assign(&properties, "isShare", &mut config.is_share)?;
        assign_enum(
            &properties,
            "replacedLoginExitMode",
            &mut config.replaced_login_exit_mode,
            |value| match value {
                "OLD_DEVICE" => Some(SaReplacedLoginExitMode::OldDeviceOffline),
                "NEW_DEVICE" => Some(SaReplacedLoginExitMode::NewDeviceNotLogin),
                _ => None,
            },
        )?;
        assign_enum(
            &properties,
            "replacedRange",
            &mut config.replaced_range,
            |value| match value {
                "CURR_DEVICE_TYPE" => Some(SaReplacedRange::CurrDeviceType),
                "ALL_DEVICE_TYPE" => Some(SaReplacedRange::AllDeviceType),
                _ => None,
            },
        )?;
        assign(&properties, "maxLoginCount", &mut config.max_login_count)?;
        assign_enum(
            &properties,
            "overflowLogoutMode",
            &mut config.overflow_logout_mode,
            |value| match value {
                "LOGOUT" => Some(SaLogoutMode::Logout),
                "KICKOUT" => Some(SaLogoutMode::Kickout),
                "REPLACED" => Some(SaLogoutMode::Replaced),
                _ => None,
            },
        )?;
        assign(&properties, "maxTryTimes", &mut config.max_try_times)?;
        assign(&properties, "isReadBody", &mut config.is_read_body)?;
        assign(&properties, "isReadHeader", &mut config.is_read_header)?;
        assign(&properties, "isReadCookie", &mut config.is_read_cookie)?;
        assign(
            &properties,
            "isLastingCookie",
            &mut config.is_lasting_cookie,
        )?;
        assign(&properties, "isWriteHeader", &mut config.is_write_header)?;
        assign_enum(
            &properties,
            "logoutRange",
            &mut config.logout_range,
            |value| match value {
                "TOKEN" => Some(SaLogoutRange::Token),
                "ACCOUNT" => Some(SaLogoutRange::Account),
                _ => None,
            },
        )?;
        assign(
            &properties,
            "isLogoutKeepFreezeOps",
            &mut config.is_logout_keep_freeze_ops,
        )?;
        assign(
            &properties,
            "isLogoutKeepTokenSession",
            &mut config.is_logout_keep_token_session,
        )?;
        assign(
            &properties,
            "rightNowCreateTokenSession",
            &mut config.right_now_create_token_session,
        )?;
        assign_enum(
            &properties,
            "tokenStyle",
            &mut config.token_style,
            |value| match value {
                "uuid" => Some(SaTokenStyle::Uuid),
                "simple-uuid" => Some(SaTokenStyle::SimpleUuid),
                "random-32" => Some(SaTokenStyle::Random32),
                "random-64" => Some(SaTokenStyle::Random64),
                "random-128" => Some(SaTokenStyle::Random128),
                "base64" => Some(SaTokenStyle::Base64),
                "jwt" => Some(SaTokenStyle::Jwt),
                "tik" => Some(SaTokenStyle::Tik),
                _ => None,
            },
        )?;
        assign(
            &properties,
            "dataRefreshPeriod",
            &mut config.data_refresh_period,
        )?;
        assign(
            &properties,
            "tokenSessionCheckLogin",
            &mut config.token_session_check_login,
        )?;
        assign(&properties, "autoRenew", &mut config.auto_renew)?;
        assign_string(&properties, "tokenPrefix", &mut config.token_prefix);
        assign(
            &properties,
            "cookieAutoFillPrefix",
            &mut config.cookie_auto_fill_prefix,
        )?;
        assign(&properties, "isPrint", &mut config.is_print)?;
        assign(&properties, "isLog", &mut config.is_log)?;
        assign_string(&properties, "logLevel", &mut config.log_level);
        assign(&properties, "logLevelInt", &mut config.log_level_int)?;
        assign_optional(&properties, "isColorLog", &mut config.is_color_log)?;
        assign_string(&properties, "jwtSecretKey", &mut config.jwt_secret_key);
        assign_string(&properties, "httpBasic", &mut config.http_basic);
        assign_string(&properties, "httpDigest", &mut config.http_digest);
        if let Some(value) = properties.get("currDomain") {
            config.curr_domain = Some(value.clone());
        }
        assign(
            &properties,
            "sameTokenTimeout",
            &mut config.same_token_timeout,
        )?;
        assign(&properties, "checkSameToken", &mut config.check_same_token)?;
        Ok(config)
    }
}

fn parse_properties(content: &str) -> HashMap<String, String> {
    content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
                return None;
            }
            line.split_once('=')
                .or_else(|| line.split_once(':'))
                .map(|(key, value)| (key.trim().to_owned(), value.trim().to_owned()))
        })
        .collect()
}

fn assign_string(properties: &HashMap<String, String>, key: &str, target: &mut String) {
    if let Some(value) = properties.get(key) {
        *target = value.clone();
    }
}

fn assign<T>(properties: &HashMap<String, String>, key: &str, target: &mut T) -> SaResult<()>
where
    T: std::str::FromStr,
{
    if let Some(value) = properties.get(key) {
        *target = value.parse().map_err(|_| invalid_property(key, value))?;
    }
    Ok(())
}

fn assign_optional<T>(
    properties: &HashMap<String, String>,
    key: &str,
    target: &mut Option<T>,
) -> SaResult<()>
where
    T: std::str::FromStr,
{
    if let Some(value) = properties.get(key) {
        *target = Some(value.parse().map_err(|_| invalid_property(key, value))?);
    }
    Ok(())
}

fn assign_enum<T>(
    properties: &HashMap<String, String>,
    key: &str,
    target: &mut T,
    parse: impl FnOnce(&str) -> Option<T>,
) -> SaResult<()> {
    if let Some(value) = properties.get(key) {
        *target = parse(value).ok_or_else(|| invalid_property(key, value))?;
    }
    Ok(())
}

fn invalid_property(key: &str, value: &str) -> SaTokenException {
    SaTokenException::with_code(10022, format!("属性赋值出错: {key}={value}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::sa_cookie_config::SaCookieConfig;

    #[test]
    fn defaults_and_properties_match_the_java_configuration_contract() {
        let defaults = SaTokenConfig::default();
        assert!(!defaults.is_share);
        assert_eq!(defaults.max_login_count, 12);
        assert!(!defaults.is_write_header);
        assert!(!defaults.is_log);
        assert_eq!(defaults.max_try_times, 12);
        assert_eq!(defaults.data_refresh_period, 30);
        assert!(defaults.token_session_check_login);
        assert!(defaults.auto_renew);
        assert_eq!(defaults.cookie, Default::default());

        let mut cookie = SaCookieConfig::default();
        cookie
            .set_domain("example.com")
            .set_path("/app")
            .set_secure(true)
            .set_http_only(true)
            .set_same_site("Strict")
            .add_extra_attr("Priority", "High")
            .add_extra_flag("Partitioned")
            .remove_extra_attr("Priority");
        assert_eq!(cookie.domain.as_deref(), Some("example.com"));
        assert_eq!(cookie.path.as_deref(), Some("/app"));
        assert!(cookie.secure);
        assert!(cookie.http_only);
        assert_eq!(cookie.same_site.as_deref(), Some("Strict"));
        assert_eq!(cookie.extra_attrs.get("Partitioned"), Some(&None));

        let parsed = SaTokenConfigFactory::create_config_from_properties(
            "tokenName=test-token\n\
             timeout=60\n\
             isShare=true\n\
             maxLoginCount=3\n\
             tokenStyle=tik\n\
             logoutRange=ACCOUNT\n\
             isColorLog=true\n",
        )
        .expect("valid Java-style properties");
        assert_eq!(parsed.token_name, "test-token");
        assert_eq!(parsed.timeout, 60);
        assert!(parsed.is_share);
        assert_eq!(parsed.max_login_count, 3);
        assert_eq!(parsed.token_style, SaTokenStyle::Tik);
        assert_eq!(parsed.logout_range, SaLogoutRange::Account);
        assert_eq!(parsed.is_color_log, Some(true));

        let error = SaTokenConfigFactory::create_config_from_properties("timeout=nope")
            .expect_err("invalid numeric property");
        assert_eq!(error.code(), 10022);
    }
}
