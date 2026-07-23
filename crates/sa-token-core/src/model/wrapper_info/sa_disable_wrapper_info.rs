//! Account disable-state response model.

use crate::util::sa_token_consts::NOT_DISABLE_LEVEL;

use serde::{Deserialize, Serialize};

/// Describes whether an account is disabled and for how long.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaDisableWrapperInfo {
    /// Whether the account is disabled.
    pub is_disable: bool,
    /// Remaining disable time in seconds.
    pub disable_time: i64,
    /// Disable level; zero means not disabled.
    pub disable_level: i32,
}

impl SaDisableWrapperInfo {
    /// Creates a response with explicit state.
    pub const fn new(is_disable: bool, disable_time: i64, disable_level: i32) -> Self {
        Self {
            is_disable,
            disable_time,
            disable_level,
        }
    }

    /// Creates a disabled response.
    pub const fn create_disabled(disable_time: i64, disable_level: i32) -> Self {
        Self::new(true, disable_time, disable_level)
    }

    /// Creates a non-disabled response.
    pub const fn create_not_disabled() -> Self {
        Self::new(false, 0, NOT_DISABLE_LEVEL)
    }

    /// Creates a cacheable non-disabled response.
    pub const fn create_not_disabled_with_cache(cache_time: i64) -> Self {
        Self::new(false, cache_time, NOT_DISABLE_LEVEL)
    }

    /// Sets the disabled marker.
    pub fn set_is_disable(&mut self, is_disable: bool) -> &mut Self {
        self.is_disable = is_disable;
        self
    }

    /// Sets the remaining disable time.
    pub fn set_disable_time(&mut self, disable_time: i64) -> &mut Self {
        self.disable_time = disable_time;
        self
    }

    /// Sets the disable level.
    pub fn set_disable_level(&mut self, disable_level: i32) -> &mut Self {
        self.disable_level = disable_level;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::SaDisableWrapperInfo;
    use crate::util::sa_token_consts::NOT_DISABLE_LEVEL;

    #[test]
    fn constructors_and_serialized_fields_match_java() {
        assert_eq!(
            SaDisableWrapperInfo::create_disabled(120, 3),
            SaDisableWrapperInfo::new(true, 120, 3)
        );
        assert_eq!(
            SaDisableWrapperInfo::create_not_disabled(),
            SaDisableWrapperInfo::new(false, 0, NOT_DISABLE_LEVEL)
        );
        assert_eq!(
            SaDisableWrapperInfo::create_not_disabled_with_cache(30),
            SaDisableWrapperInfo::new(false, 30, NOT_DISABLE_LEVEL)
        );
        let value = serde_json::to_value(SaDisableWrapperInfo::create_disabled(120, 3))
            .expect("serialize disable wrapper");
        assert_eq!(
            value,
            serde_json::json!({
                "isDisable": true,
                "disableTime": 120,
                "disableLevel": 3
            })
        );
    }
}
