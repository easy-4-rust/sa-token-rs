//! 工具模块（对应 Java `cn.dev33.satoken.util`）。

pub mod sa_fox_util;
pub mod sa_hex_util;
pub mod sa_result;
pub mod sa_sugar;
pub mod sa_token_consts;
pub mod sa_ttl_methods;
pub mod sa_value2_box;
pub mod str_formatter;

// ---------- re-exports ----------
// Selected utility facade exports.
pub use sa_hex_util::SaHexUtil;
pub use sa_result::SaResultData;
pub use sa_sugar::SaSugar;
pub use sa_token_consts::{
    DEFAULT_ACTIVE_TIMEOUT, DEFAULT_IS_CONCURRENT, DEFAULT_IS_SHARE, DEFAULT_MAX_LOGIN_COUNT,
    DEFAULT_TIMEOUT, DEFAULT_TOKEN_NAME, DEFAULT_TOKEN_STYLE, NEVER_EXPIRE, NOT_VALUE_EXPIRE,
    REDIS_KEY_PREFIX, SESSION_KEY_PERMISSION_LIST, SESSION_KEY_ROLE_LIST, SESSION_KEY_USER,
    SESSION_TYPE_LOGIN, SESSION_TYPE_TOKEN,
};
pub use sa_ttl_methods::{
    DEFAULT_TIMEOUT as TTL_DEFAULT_TIMEOUT, NEVER_EXPIRE as TTL_NEVER_EXPIRE, SaTtlMethods,
};
pub use sa_value2_box::SaValue2Box;
pub use str_formatter::format;
