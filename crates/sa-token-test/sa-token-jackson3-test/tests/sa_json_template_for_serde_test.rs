//! 对应 Java：`sa-token-jackson3-test` / `SaJsonTemplateForJackson3Test.java`
//!
//! 映射：Jackson3 → `serde_json`（与项目 Jackson → serde 约定一致）。

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// 对应 Java 测试模型 `SysUser`。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SysUser {
    /// 用户 id
    id: i64,
    /// 用户名
    name: String,
}

/// 对应 Java 测试模型 `SysRole`。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SysRole {
    /// 角色 id
    id: i64,
    /// 角色名
    name: String,
}

/// 对应 `SaJsonTemplateForJackson3Test`：对象 ↔ JSON 往返。
#[test]
fn serde_json_roundtrip_sys_user() {
    let user = SysUser {
        id: 10001,
        name: "zhang".into(),
    };
    let text = serde_json::to_string(&user).expect("serialize");
    let back: SysUser = serde_json::from_str(&text).expect("deserialize");
    assert_eq!(back, user);
}

/// 嵌套结构序列化（Jackson ObjectMapper 行为对齐 serde）。
#[test]
fn nested_role_in_value() {
    let role = SysRole {
        id: 1,
        name: "admin".into(),
    };
    let v: Value = json!({ "userId": 10001, "role": role });
    assert_eq!(v["role"]["name"], json!("admin"));
}
