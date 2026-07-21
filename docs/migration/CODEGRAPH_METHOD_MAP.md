# Sa-Token 方法级 1:1 审计

> 本文档用于审计 Sa-Token Java → Sa-Token-Rs 的方法级迁移完整性。
> 参考 easyexcel-rs 的 `CODEGRAPH_METHOD_MAP.md` 组织方式。
> 每个方法记录：Java 签名 → Rust 签名 → 迁移状态 → 备注。

---

## 审计规则

| 标记 | 含义 |
|---|---|
| ✅ | 已实现 |
| ⬜ | 待实现 |
| 🔄 | 签名调整（语义对齐但 Rust 化） |
| ➖ | 不适用（Java 特有，Rust 不需要） |
| ➕ | Rust 新增（Java 无对应） |

### 签名映射规则

| Java 类型 | Rust 类型 | 说明 |
|---|---|---|
| `Object` | `String` / `&str` | 登录 ID 统一为 String |
| `String` | `String` / `&str` | — |
| `boolean` | `bool` | — |
| `int` | `i32` | — |
| `long` | `i64` | — |
| `List<String>` | `Vec<String>` | — |
| `Map<String, Object>` | `HashMap<String, Value>` | serde_json::Value |
| `void` | `SaResult<()>` | 可能抛异常的方法返回 Result |
| `T`（返回值可能为 null） | `Option<T>` | — |
| `T`（返回值不为 null） | `SaResult<T>` | 未登录/无权限时返回 Err |

---

## 1. StpLogic 方法审计

### 1.1 构造与基础

| # | Java 签名 | Rust 签名 | 状态 | 备注 |
|---|---|---|---|---|
| 1 | `StpLogic(String loginType)` | `StpLogic::new(login_type: &str)` | ⬜ | |
| 2 | `String getLoginType()` | `fn login_type(&self) -> &str` | ⬜ | 🔄 返回引用 |
| 3 | `StpLogic setLoginType(String)` | `fn set_login_type(&mut self, t: String)` | ⬜ | |
| 4 | `SaTokenConfig getConfig()` | `fn config(&self) -> Arc<SaTokenConfig>` | ⬜ | 🔄 Arc 包装 |
| 5 | `StpLogic setConfig(SaTokenConfig)` | `fn set_config(&self, c: Arc<SaTokenConfig>)` | ⬜ | |
| 6 | `SaTokenConfig getConfigOrGlobal()` | `fn config_or_global(&self) -> Arc<SaTokenConfig>` | ⬜ | |

### 1.2 登录

| # | Java 签名 | Rust 签名 | 状态 | 备注 |
|---|---|---|---|---|
| 7 | `void login(Object id)` | `fn login(&self, id: &str) -> SaResult<()>` | ⬜ | 🔄 |
| 8 | `void login(Object id, String deviceType)` | `fn login_with_device(&self, id: &str, device_type: &str) -> SaResult<()>` | ⬜ | |
| 9 | `void login(Object id, boolean isLastingCookie)` | `fn login_with_lasting_cookie(&self, id: &str, is_lasting: bool) -> SaResult<()>` | ⬜ | |
| 10 | `void login(Object id, long timeout)` | `fn login_with_timeout(&self, id: &str, timeout: i64) -> SaResult<()>` | ⬜ | |
| 11 | `void login(Object id, SaLoginParameter param)` | `fn login_with_param(&self, id: &str, param: &SaLoginParameter) -> SaResult<()>` | ⬜ | |
| 12 | `String createLoginSession(Object id)` | `fn create_login_session(&self, id: &str, param: &SaLoginParameter) -> SaResult<String>` | ⬜ | 🔄 合并为带参数版本 |
| 13 | `String getOrCreateLoginSession(Object id)` | `fn get_or_create_login_session(&self, id: &str) -> SaResult<String>` | ⬜ | |

### 1.3 登出 / 踢 / 顶

| # | Java 签名 | Rust 签名 | 状态 | 备注 |
|---|---|---|---|---|
| 14 | `void logout()` | `fn logout(&self) -> SaResult<()>` | ⬜ | |
| 15 | `void logout(SaLogoutParameter)` | `fn logout_with_param(&self, p: &SaLogoutParameter) -> SaResult<()>` | ⬜ | |
| 16 | `void logoutByTokenValue(String)` | `fn logout_by_token_value(&self, t: &str) -> SaResult<()>` | ⬜ | |
| 17 | `void kickout(Object)` | `fn kickout_by_login_id(&self, id: &str) -> SaResult<()>` | ⬜ | |
| 18 | `void kickoutByTokenValue(String)` | `fn kickout_by_token_value(&self, t: &str) -> SaResult<()>` | ⬜ | |
| 19 | `void replaced(Object)` | `fn replaced_by_login_id(&self, id: &str) -> SaResult<()>` | ⬜ | |
| 20 | `void replacedByTokenValue(String)` | `fn replaced_by_token_value(&self, t: &str) -> SaResult<()>` | ⬜ | |

### 1.4 登录状态

| # | Java 签名 | Rust 签名 | 状态 | 备注 |
|---|---|---|---|---|
| 21 | `boolean isLogin()` | `fn is_login(&self) -> bool` | ⬜ | |
| 22 | `void checkLogin()` | `fn check_login(&self) -> SaResult<()>` | ⬜ | |
| 23 | `Object getLoginId()` | `fn get_login_id(&self) -> SaResult<String>` | ⬜ | 🔄 返回 Result |
| 24 | `<T> T getLoginId(T defaultValue)` | `fn get_login_id_or(&self, default: &str) -> String` | ⬜ | |
| 25 | `Object getLoginIdDefaultNull()` | `fn get_login_id_default_null(&self) -> Option<String>` | ⬜ | |
| 26 | `String getLoginIdAsString()` | `fn get_login_id_as_string(&self) -> SaResult<String>` | ⬜ | |
| 27 | `int getLoginIdAsInt()` | `fn get_login_id_as_i32(&self) -> SaResult<i32>` | ⬜ | |
| 28 | `long getLoginIdAsLong()` | `fn get_login_id_as_i64(&self) -> SaResult<i64>` | ⬜ | |
| 29 | `Object getLoginIdByToken(String)` | `fn get_login_id_by_token(&self, t: &str) -> Option<String>` | ⬜ | |
| 30 | `boolean isValidLoginId(Object)` | `fn is_valid_login_id(&self, id: &str) -> bool` | ⬜ | |
| 31 | `boolean isValidToken(String)` | `fn is_valid_token(&self, t: &str) -> bool` | ⬜ | |

### 1.5 Token 操作

| # | Java 签名 | Rust 签名 | 状态 | 备注 |
|---|---|---|---|---|
| 32 | `String getTokenName()` | `fn token_name(&self) -> String` | ⬜ | |
| 33 | `String createTokenValue(Object, String, long, Map)` | `fn create_token_value(&self, id, device, timeout, extra) -> String` | ⬜ | |
| 34 | `void setTokenValue(String)` | `fn set_token_value(&self, t: &str) -> SaResult<()>` | ⬜ | |
| 35 | `void setTokenValue(String, int cookieTimeout)` | `fn set_token_value_with_cookie_timeout(&self, t, ct) -> SaResult<()>` | ⬜ | |
| 36 | `void setTokenValue(String, SaLoginParameter)` | `fn set_token_value_with_param(&self, t, p) -> SaResult<()>` | ⬜ | |
| 37 | `void setTokenValueToStorage(String)` | `fn set_token_value_to_storage(&self, t) -> SaResult<()>` | ⬜ | |
| 38 | `void setTokenValueToCookie(String, int)` | `fn set_token_value_to_cookie(&self, t, ct) -> SaResult<()>` | ⬜ | |
| 39 | `void setTokenValueToResponseHeader(String)` | `fn set_token_value_to_response_header(&self, t) -> SaResult<()>` | ⬜ | |
| 40 | `String getTokenValue()` | `fn get_token_value(&self) -> Option<String>` | ⬜ | |
| 41 | `String getTokenValue(boolean)` | `fn get_token_value_ex(&self, no_prefix_throw: bool) -> Option<String>` | ⬜ | |
| 42 | `String getTokenValueNotCut()` | `fn get_token_value_not_cut(&self) -> Option<String>` | ⬜ | |
| 43 | `String getTokenValueNotNull()` | `fn get_token_value_not_null(&self) -> SaResult<String>` | ⬜ | |
| 44 | `SaTokenInfo getTokenInfo()` | `fn get_token_info(&self) -> SaResult<SaTokenInfo>` | ⬜ | |

### 1.6 权限 / 角色

| # | Java 签名 | Rust 签名 | 状态 | 备注 |
|---|---|---|---|---|
| 45 | `List<String> getRoleList()` | `fn get_role_list(&self) -> SaResult<Vec<String>>` | ⬜ | |
| 46 | `List<String> getRoleList(Object loginId)` | `fn get_role_list_for(&self, id: &str) -> SaResult<Vec<String>>` | ⬜ | |
| 47 | `boolean hasRole(String)` | `fn has_role(&self, role: &str) -> SaResult<bool>` | ⬜ | |
| 48 | `boolean hasRoleAnd(String...)` | `fn has_role_and(&self, roles: &[&str]) -> SaResult<bool>` | ⬜ | |
| 49 | `boolean hasRoleOr(String...)` | `fn has_role_or(&self, roles: &[&str]) -> SaResult<bool>` | ⬜ | |
| 50 | `void checkRole(String)` | `fn check_role(&self, role: &str) -> SaResult<()>` | ⬜ | |
| 51 | `void checkRoleAnd(String...)` | `fn check_role_and(&self, roles: &[&str]) -> SaResult<()>` | ⬜ | |
| 52 | `void checkRoleOr(String...)` | `fn check_role_or(&self, roles: &[&str]) -> SaResult<()>` | ⬜ | |
| 53 | `List<String> getPermissionList()` | `fn get_permission_list(&self) -> SaResult<Vec<String>>` | ⬜ | |
| 54 | `List<String> getPermissionList(Object loginId)` | `fn get_permission_list_for(&self, id: &str) -> SaResult<Vec<String>>` | ⬜ | |
| 55 | `boolean hasPermission(String)` | `fn has_permission(&self, p: &str) -> SaResult<bool>` | ⬜ | |
| 56 | `boolean hasPermissionAnd(String...)` | `fn has_permission_and(&self, ps: &[&str]) -> SaResult<bool>` | ⬜ | |
| 57 | `boolean hasPermissionOr(String...)` | `fn has_permission_or(&self, ps: &[&str]) -> SaResult<bool>` | ⬜ | |
| 58 | `void checkPermission(String)` | `fn check_permission(&self, p: &str) -> SaResult<()>` | ⬜ | |
| 59 | `void checkPermissionAnd(String...)` | `fn check_permission_and(&self, ps: &[&str]) -> SaResult<()>` | ⬜ | |
| 60 | `void checkPermissionOr(String...)` | `fn check_permission_or(&self, ps: &[&str]) -> SaResult<()>` | ⬜ | |

### 1.7 会话

| # | Java 签名 | Rust 签名 | 状态 | 备注 |
|---|---|---|---|---|
| 61 | `SaSession getSession()` | `fn get_session(&self) -> SaResult<SaSession>` | ⬜ | |
| 62 | `SaSession getSessionByLoginId(Object)` | `fn get_session_by_login_id(&self, id: &str) -> SaResult<SaSession>` | ⬜ | |
| 63 | `SaSession getSessionBySessionId(String)` | `fn get_session_by_session_id(&self, sid: &str) -> SaResult<SaSession>` | ⬜ | |
| 64 | `SaSession getTokenSession()` | `fn get_token_session(&self) -> SaResult<SaSession>` | ⬜ | |
| 65 | `SaSession getTokenSessionByToken(String)` | `fn get_token_session_by_token(&self, t: &str) -> SaResult<SaSession>` | ⬜ | |
| 66 | `SaSession getAnonTokenSession()` | `fn get_anon_token_session(&self) -> SaResult<SaSession>` | ⬜ | |
| 67 | `void deleteTokenSession(String)` | `fn delete_token_session(&self, t: &str) -> SaResult<()>` | ⬜ | |

### 1.8 禁用

| # | Java 签名 | Rust 签名 | 状态 | 备注 |
|---|---|---|---|---|
| 68 | `void disable(Object, long)` | `fn disable(&self, id: &str, time: i64) -> SaResult<()>` | ⬜ | |
| 69 | `boolean isDisable(Object)` | `fn is_disable(&self, id: &str) -> bool` | ⬜ | |
| 70 | `void checkDisable(Object)` | `fn check_disable(&self, id: &str) -> SaResult<()>` | ⬜ | |
| 71 | `long getDisableTime(Object)` | `fn get_disable_time(&self, id: &str) -> i64` | ⬜ | |
| 72 | `void untieDisable(Object)` | `fn untie_disable(&self, id: &str) -> SaResult<()>` | ⬜ | |

### 1.9 安全认证

| # | Java 签名 | Rust 签名 | 状态 | 备注 |
|---|---|---|---|---|
| 73 | `void openSafe(long)` | `fn open_safe(&self, t: i64) -> SaResult<()>` | ⬜ | |
| 74 | `boolean isSafe()` | `fn is_safe(&self) -> bool` | ⬜ | |
| 75 | `void checkSafe()` | `fn check_safe(&self) -> SaResult<()>` | ⬜ | |
| 76 | `long getSafeTime()` | `fn get_safe_time(&self) -> i64` | ⬜ | |
| 77 | `void closeSafe()` | `fn close_safe(&self) -> SaResult<()>` | ⬜ | |

### 1.10 切换账号

| # | Java 签名 | Rust 签名 | 状态 | 备注 |
|---|---|---|---|---|
| 78 | `void switchTo(Object)` | `fn switch_to(&self, id: &str) -> SaResult<()>` | ⬜ | |
| 79 | `void endSwitch()` | `fn end_switch(&self) -> SaResult<()>` | ⬜ | |
| 80 | `boolean isSwitch()` | `fn is_switch(&self) -> bool` | ⬜ | |
| 81 | `Object getSwitchLoginId()` | `fn get_switch_login_id(&self) -> Option<String>` | ⬜ | |

---

## 2. 统计

| 指标 | 数值 |
|---|---|
| StpLogic 已列出方法数 | 81 |
| 状态 ✅ | 0 |
| 状态 ⬜ | 81 |
| 状态 🔄 | ~15（签名调整但语义对齐） |
| 状态 ➖ | 0 |
| 状态 ➕ | 1（auth_hash） |

---

## 3. 审计说明

1. **签名调整原则**：Rust 签名不需要与 Java 字面一致，但语义必须一致。
   - Java `Object` → Rust `String`（登录 ID 统一）
   - Java `void` → Rust `SaResult<()>`（可能抛异常）
   - Java 返回 null → Rust 返回 `Option<T>`
   - Java 参数可变长 → Rust 参数切片 `&[&str]`

2. **Phase 追踪**：每个方法标注所属 Phase，参见 [MIGRATION_STATUS.md](./MIGRATION_STATUS.md)。

3. **测试覆盖**：每个 ✅ 方法必须有对应测试，参见 [TEST_AUDIT_REPORT.md](./TEST_AUDIT_REPORT.md)。

---

## 参考

- [MIGRATION_STATUS.md](./MIGRATION_STATUS.md) - 迁移进度
- [object-method-matrix.md](./object-method-matrix.md) - 方法矩阵
- **easyexcel-rs 审计**：`/Users/wandl/workspaces/workspace-github/easyexcel-rs/docs/migration/CODEGRAPH_METHOD_MAP.md`
