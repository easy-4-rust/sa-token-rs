//! Wave 4 契约测试：dao/auto、timed_cache、util、secure、json、log。
//!
//! 作为 `file-map.csv` 中 Wave 4 条目的 `test_evidence`。

use sa_token_core::dao::auto::sa_token_dao_by_object_follow_string::{
    get_object as object_get, set_object as object_set,
};
use sa_token_core::dao::auto::sa_token_dao_by_session_follow_object::{
    get_session, set_session,
};
use sa_token_core::dao::auto::sa_token_dao_by_string_follow_object::{
    get as string_get, set as string_set,
};
use sa_token_core::dao::sa_token_dao::SaTokenDao;
use sa_token_core::dao::sa_token_dao_default_impl::SaTokenDaoDefaultImpl;
use sa_token_core::dao::timed_cache::sa_map_package::SaMapPackage;
use sa_token_core::dao::timed_cache::sa_map_package_for_concurrent_hash_map::SaMapPackageForConcurrentHashMap;
use sa_token_core::dao::timed_cache::sa_timed_cache::SaTimedCache;
use sa_token_core::json::sa_json_template::SaJsonTemplate;
use sa_token_core::json::sa_json_template_default_impl::SaJsonTemplateDefaultImpl;
use sa_token_core::log::sa_log::{SaLog, SaLogLevel};
use sa_token_core::log::sa_log_for_console::SaLogForConsole;
use sa_token_core::secure::bcrypt::BCrypt;
use sa_token_core::secure::sa_base32_util::SaBase32Util;
use sa_token_core::secure::sa_base64_util::SaBase64Util;
use sa_token_core::secure::sa_secure_util::SaSecureUtil;
use sa_token_core::secure::totp::sa_totp_template::SaTotpTemplate;
use sa_token_core::secure::totp::sa_totp_util::SaTotpUtil;
use sa_token_core::serializer::r#impl::SaSerializerTemplateForJson;
use sa_token_core::session::sa_session::SaSession;
use sa_token_core::util::sa_fox_util::SaFoxUtil;
use sa_token_core::util::sa_hex_util::{decode, encode};
use sa_token_core::util::sa_result::SaResultData;
use sa_token_core::util::sa_sugar::SaSugar;
use sa_token_core::util::sa_token_consts::{DEFAULT_TOKEN_NAME, NEVER_EXPIRE};
use sa_token_core::util::sa_ttl_methods::SaTtlMethods;
use sa_token_core::util::sa_value2_box::SaValue2Box;
use sa_token_core::util::str_formatter::format;

/// SaFoxUtil 核心方法契约
#[test]
fn wave4_sa_fox_util_contract() {
    assert_eq!(SaFoxUtil::random_string(8).len(), 8);
    assert!(SaFoxUtil::is_empty(""));
    assert!(SaFoxUtil::equals("a", "a"));
    assert_eq!(
        SaFoxUtil::join_param("http://x.com", "id=1"),
        "http://x.com?id=1"
    );
    assert!(SaFoxUtil::vague_match("api*", "api/user"));
}

/// util 其余条目契约
#[test]
fn wave4_util_contract() {
    assert_eq!(encode(b"ab"), "6162");
    assert_eq!(decode("6162").unwrap(), b"ab");
    let ok: SaResultData<i32> = SaResultData::ok(1);
    assert!(ok.is_ok());
    assert!(SaSugar::empty_list().is_empty());
    assert_eq!(DEFAULT_TOKEN_NAME, "satoken");
    assert_eq!(SaTtlMethods::NEVER_EXPIRE, NEVER_EXPIRE);
    let boxv: SaValue2Box<u32> = SaValue2Box::new();
    assert!(boxv.set(1));
    assert_eq!(boxv.get(), Some(1));
    let world: &dyn std::fmt::Display = &"world";
    assert_eq!(format("hi {0}", &[world]), "hi world");
}

/// dao auto 三层跟随契约
#[test]
fn wave4_dao_auto_contract() {
    let dao = SaTokenDaoDefaultImpl::new();
    let serializer = SaSerializerTemplateForJson;
    let value = serde_json::json!({"k": "v"});
    object_set(&dao, &serializer, "obj", &value, -1).expect("object set");
    assert_eq!(
        object_get(&dao, &serializer, "obj").expect("object get"),
        Some(value)
    );
    string_set(&dao, "str", "text", -1).expect("string set");
    assert_eq!(
        string_get(&dao, &serializer, "str").expect("string get"),
        Some("text".to_string())
    );
    let session = SaSession::new("sid-1");
    set_session(&dao, &session, -1).expect("session set");
    assert_eq!(
        get_session(&dao, "sid-1")
            .expect("session get")
            .expect("exists")
            .id(),
        "sid-1"
    );
}

/// timed_cache 契约
#[test]
fn wave4_timed_cache_contract() {
    let mut cache = SaTimedCache::new();
    cache.set_object("k", "v", NEVER_EXPIRE);
    assert_eq!(cache.get_object("k"), Some("v".to_string()));
    let mut concurrent = SaMapPackageForConcurrentHashMap::<String>::new();
    concurrent.put("a", "1".to_string());
    assert_eq!(concurrent.get("a"), Some("1".to_string()));
}

/// secure + totp 契约
#[test]
fn wave4_secure_contract() {
    assert_eq!(
        SaSecureUtil::md5("123456"),
        "e10adc3949ba59abbe56e057f20f883e"
    );
    let enc = SaBase64Util::encode(b"hi");
    assert_eq!(SaBase64Util::decode(&enc).unwrap(), b"hi");
    assert_eq!(SaBase32Util::encode(b"foobar"), "MZXW6YTBOI");
    let hash = BCrypt::hashpw("pwd", &BCrypt::gensalt());
    assert!(BCrypt::checkpw("pwd", &hash));
    let tpl = SaTotpTemplate::new();
    let key = tpl.generate_secret_key();
    let code = tpl.generate_totp(&key);
    assert!(tpl.validate_totp(&key, &code, 1));
    assert!(SaTotpUtil::validate_totp(&key, &code, 1));
}

/// json + log 契约
#[test]
fn wave4_json_log_contract() {
    let json_tpl = SaJsonTemplateDefaultImpl;
    let value = serde_json::json!({"x": 1});
    let text = json_tpl.to_json(&value);
    assert_eq!(json_tpl.parse_json(&text), Some(value));
    let logger = SaLogForConsole;
    logger.log(SaLogLevel::Info, "wave4", "ok");
}
