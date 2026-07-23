# Sa-Token Java 对象 × 方法矩阵

> 本文档列出 Sa-Token Java `sa-token-core` 中所有核心类/接口的 public 方法，
> 以及在 Sa-Token-Rs 中的对应 Rust 方法签名。
> 参考 easyexcel-rs 的 `object-method-matrix.md` 组织方式。

---

## 1. SaManager（`sa-token-core/src/manager.rs`）

| # | Java 方法 | Rust 方法 | Phase |
|---|---|---|---|
| 1 | `setConfig(SaTokenConfig)` | `set_config(Arc<SaTokenConfig>)` | 1 |
| 2 | `getConfig()` → `SaTokenConfig` | `config()` → `Arc<SaTokenConfig>` | 1 |
| 3 | `setSaTokenDao(SaTokenDao)` | `set_sa_token_dao(Arc<dyn SaTokenDao>)` | 1 |
| 4 | `getSaTokenDao()` → `SaTokenDao` | `sa_token_dao()` → `Arc<dyn SaTokenDao>` | 1 |
| 5 | `setStpInterface(StpInterface)` | `set_stp_interface(Arc<dyn StpInterface>)` | 1 |
| 6 | `getStpInterface()` → `StpInterface` | `stp_interface()` → `Arc<dyn StpInterface>` | 1 |
| 7 | `setSaTokenContext(SaTokenContext)` | `set_sa_token_context(Arc<dyn SaTokenContext>)` | 1 |
| 8 | `getSaTokenContext()` → `SaTokenContext` | `sa_token_context()` → `Arc<dyn SaTokenContext>` | 1 |
| 9 | `setSaTempTemplate(SaTempTemplate)` | `set_sa_temp_template(Arc<dyn SaTempTemplate>)` | 2 |
| 10 | `getSaTempTemplate()` → `SaTempTemplate` | `sa_temp_template()` → `Arc<dyn SaTempTemplate>` | 2 |
| 11 | `setSaJsonTemplate(SaJsonTemplate)` | `set_sa_json_template(Arc<dyn SaJsonTemplate>)` | 1 |
| 12 | `getSaJsonTemplate()` → `SaJsonTemplate` | `sa_json_template()` → `Arc<dyn SaJsonTemplate>` | 1 |
| 13 | `setSaHttpTemplate(SaHttpTemplate)` | `set_sa_http_template(Arc<dyn SaHttpTemplate>)` | 2 |
| 14 | `getSaHttpTemplate()` → `SaHttpTemplate` | `sa_http_template()` → `Arc<dyn SaHttpTemplate>` | 2 |
| 15 | `setSaSerializerTemplate(SaSerializerTemplate)` | `set_sa_serializer_template(Arc<dyn SaSerializerTemplate>)` | 1 |
| 16 | `getSaSerializerTemplate()` → `SaSerializerTemplate` | `sa_serializer_template()` → `Arc<dyn SaSerializerTemplate>` | 1 |
| 17 | `setSaSameTemplate(SaSameTemplate)` | `set_sa_same_template(Arc<dyn SaSameTemplate>)` | 2 |
| 18 | `getSaSameTemplate()` → `SaSameTemplate` | `sa_same_template()` → `Arc<dyn SaSameTemplate>` | 2 |
| 19 | `setLog(SaLog)` | `set_log(Arc<dyn SaLog>)` | 1 |
| 20 | `getLog()` → `SaLog` | `log()` → `Arc<dyn SaLog>` | 1 |
| 21 | `setSaTotpTemplate(SaTotpTemplate)` | `set_sa_totp_template(Arc<dyn SaTotpTemplate>)` | 2 |
| 22 | `getSaTotpTemplate()` → `SaTotpTemplate` | `sa_totp_template()` → `Arc<dyn SaTotpTemplate>` | 2 |
| 23 | `putStpLogic(StpLogic)` | `put_stp_logic(StpLogic)` | 1 |
| 24 | `removeStpLogic(String)` | `remove_stp_logic(login_type)` | 1 |
| 25 | `getStpLogic(String)` → `StpLogic` | `get_stp_logic(login_type)` → `Option<StpLogic>` | 1 |
| 26 | `getStpLogic(String, boolean)` → `StpLogic` | `get_stp_logic(login_type)` → `Option<StpLogic>` | 1 |

---

## 2. StpLogic（`sa-token-core/src/stp/stp_logic.rs`）

> StpLogic 是核心类，~196 个 public 方法。此处按功能分组列出关键方法，完整矩阵见 [MIGRATION_STATUS.md](./MIGRATION_STATUS.md)。

### 2.1 构造与基础

| # | Java 方法 | Rust 方法 | Phase |
|---|---|---|---|
| 1 | `StpLogic(String loginType)` | `StpLogic::new(login_type)` | 1 |
| 2 | `getLoginType()` → `String` | `login_type()` → `&str` | 1 |
| 3 | `getConfig()` → `SaTokenConfig` | `config()` → `Arc<SaTokenConfig>` | 1 |
| 4 | `setConfig(SaTokenConfig)` | `set_config(Arc<SaTokenConfig>)` | 1 |

### 2.2 登录

| # | Java 方法 | Rust 方法 | Phase |
|---|---|---|---|
| 5 | `login(Object id)` | `login(id: &str)` | 1 |
| 6 | `login(Object id, String deviceType)` | `login_with_device(id, device_type)` | 1 |
| 7 | `login(Object id, SaLoginParameter)` | `login_with_param(id, param)` | 1 |
| 8 | `createLoginSession(Object id)` → `String` | `create_login_session(id, param)` → `SaResult<String>` | 1 |
| 9 | `getOrCreateLoginSession(Object id)` → `String` | `get_or_create_login_session(id)` → `SaResult<String>` | 1 |

### 2.3 登出 / 踢 / 顶

| # | Java 方法 | Rust 方法 | Phase |
|---|---|---|---|
| 10 | `logout()` | `logout()` | 1 |
| 11 | `logoutByTokenValue(String)` | `logout_by_token_value(token)` | 1 |
| 12 | `kickout(Object)` | `kickout_by_login_id(id)` | 1 |
| 13 | `kickoutByTokenValue(String)` | `kickout_by_token_value(token)` | 1 |
| 14 | `replaced(Object)` | `replaced_by_login_id(id)` | 1 |
| 15 | `replacedByTokenValue(String)` | `replaced_by_token_value(token)` | 1 |

### 2.4 登录状态

| # | Java 方法 | Rust 方法 | Phase |
|---|---|---|---|
| 16 | `isLogin()` → `boolean` | `is_login()` → `bool` | 1 |
| 17 | `checkLogin()` | `check_login()` → `SaResult<()>` | 1 |
| 18 | `getLoginId()` → `Object` | `get_login_id()` → `SaResult<String>` | 1 |
| 19 | `getLoginIdDefaultNull()` → `Object` | `get_login_id_default_null()` → `Option<String>` | 1 |
| 20 | `getLoginIdAsString()` → `String` | `get_login_id_as_string()` → `SaResult<String>` | 1 |
| 21 | `getLoginIdAsInt()` → `int` | `get_login_id_as_i32()` → `SaResult<i32>` | 1 |
| 22 | `getLoginIdAsLong()` → `long` | `get_login_id_as_i64()` → `SaResult<i64>` | 1 |
| 23 | `getLoginIdByToken(String)` → `Object` | `get_login_id_by_token(token)` → `Option<String>` | 1 |

### 2.5 Token

| # | Java 方法 | Rust 方法 | Phase |
|---|---|---|---|
| 24 | `getTokenName()` → `String` | `token_name()` → `String` | 1 |
| 25 | `getTokenValue()` → `String` | `get_token_value()` → `Option<String>` | 1 |
| 26 | `getTokenValueNotCut()` → `String` | `get_token_value_not_cut()` → `Option<String>` | 1 |
| 27 | `setTokenValue(String)` | `set_token_value(token)` | 1 |
| 28 | `getTokenInfo()` → `SaTokenInfo` | `get_token_info()` → `SaResult<SaTokenInfo>` | 1 |

### 2.6 会话

| # | Java 方法 | Rust 方法 | Phase |
|---|---|---|---|
| 29 | `getSession()` → `SaSession` | `get_session()` → `SaResult<SaSession>` | 1 |
| 30 | `getSessionByLoginId(Object)` → `SaSession` | `get_session_by_login_id(id)` → `SaResult<SaSession>` | 1 |
| 31 | `getTokenSession()` → `SaSession` | `get_token_session()` → `SaResult<SaSession>` | 1 |
| 32 | `getTokenSessionByToken(String)` → `SaSession` | `get_token_session_by_token(token)` → `SaResult<SaSession>` | 1 |
| 33 | `deleteTokenSession(String)` | `delete_token_session(token)` | 1 |

### 2.7 权限 / 角色

| # | Java 方法 | Rust 方法 | Phase |
|---|---|---|---|
| 34 | `getRoleList()` → `List<String>` | `get_role_list()` → `SaResult<Vec<String>>` | 2 |
| 35 | `hasRole(String)` → `boolean` | `has_role(role)` → `SaResult<bool>` | 2 |
| 36 | `hasRoleAnd(String...)` → `boolean` | `has_role_and(roles)` → `SaResult<bool>` | 2 |
| 37 | `hasRoleOr(String...)` → `boolean` | `has_role_or(roles)` → `SaResult<bool>` | 2 |
| 38 | `checkRole(String)` | `check_role(role)` | 2 |
| 39 | `checkRoleAnd(String...)` | `check_role_and(roles)` | 2 |
| 40 | `checkRoleOr(String...)` | `check_role_or(roles)` | 2 |
| 41 | `getPermissionList()` → `List<String>` | `get_permission_list()` → `SaResult<Vec<String>>` | 2 |
| 42 | `hasPermission(String)` → `boolean` | `has_permission(perm)` → `SaResult<bool>` | 2 |
| 43 | `hasPermissionAnd(String...)` → `boolean` | `has_permission_and(perms)` → `SaResult<bool>` | 2 |
| 44 | `hasPermissionOr(String...)` → `boolean` | `has_permission_or(perms)` → `SaResult<bool>` | 2 |
| 45 | `checkPermission(String)` | `check_permission(perm)` | 2 |
| 46 | `checkPermissionAnd(String...)` | `check_permission_and(perms)` | 2 |
| 47 | `checkPermissionOr(String...)` | `check_permission_or(perms)` | 2 |

### 2.8 禁用 / 安全 / 切换

| # | Java 方法 | Rust 方法 | Phase |
|---|---|---|---|
| 48 | `disable(Object, long)` | `disable(login_id, time)` | 2 |
| 49 | `isDisable(Object)` → `boolean` | `is_disable(login_id)` → `bool` | 2 |
| 50 | `checkDisable(Object)` | `check_disable(login_id)` | 2 |
| 51 | `openSafe(long)` | `open_safe(safe_time)` | 2 |
| 52 | `isSafe()` → `boolean` | `is_safe()` → `bool` | 2 |
| 53 | `checkSafe()` | `check_safe()` | 2 |
| 54 | `switchTo(Object)` | `switch_to(login_id)` | 2 |
| 55 | `endSwitch()` | `end_switch()` | 2 |
| 56 | `isSwitch()` → `boolean` | `is_switch()` → `bool` | 2 |

### 2.9 续签 / 超时

| # | Java 方法 | Rust 方法 | Phase |
|---|---|---|---|
| 57 | `getTokenTimeout()` → `long` | `get_token_timeout()` → `i64` | 1 |
| 58 | `renewTimeout(long)` | `renew_timeout(timeout)` | 1 |
| 59 | `updateLastActiveToNow()` | `update_last_active_to_now()` | 1 |
| 60 | `checkActiveTimeout()` | `check_active_timeout()` | 1 |

### 2.10 设备 / 终端

| # | Java 方法 | Rust 方法 | Phase |
|---|---|---|---|
| 61 | `getLoginDeviceType()` → `String` | `get_login_device_type()` | 1 |
| 62 | `getLoginDeviceId()` → `String` | `get_login_device_id()` | 1 |
| 63 | `getTerminalListByLoginId(Object)` → `List<SaTerminalInfo>` | `get_terminal_list_by_login_id(id)` | 1 |
| 64 | `getTerminalInfo()` → `SaTerminalInfo` | `get_terminal_info()` | 1 |
| 65 | `isTrustDeviceId(Object, String)` → `boolean` | `is_trust_device_id(user_id, device_id)` | 1 |

### 2.11 Key 拼接

| # | Java 方法 | Rust 方法 | Phase |
|---|---|---|---|
| 66 | `splicingKeyTokenName()` → `String` | `splicing_key_token_name()` | 1 |
| 67 | `splicingKeyTokenValue(String)` → `String` | `splicing_key_token_value(token)` | 1 |
| 68 | `splicingKeySession(Object)` → `String` | `splicing_key_session(login_id)` | 1 |
| 69 | `splicingKeyTokenSession(String)` → `String` | `splicing_key_token_session(token)` | 1 |
| 70 | `splicingKeyLastActiveTime(String)` → `String` | `splicing_key_last_active_time(token)` | 1 |

---

## 3. StpUtil（`sa-token-core/src/stp/stp_util.rs`）

StpUtil 是 StpLogic 的静态代理，所有方法与 StpLogic 一一对应（去掉了 `&self`）。

| # | Java 方法 | Rust 方法 | Phase |
|---|---|---|---|
| 1 | `setStpLogic(StpLogic)` | `set_stp_logic(logic)` | 1 |
| 2 | `getStpLogic()` → `StpLogic` | `stp_logic()` → `&'static StpLogic` | 1 |
| 3 | `getLoginType()` → `String` | `get_login_type()` → `&'static str` | 1 |
| 4+ | （其余方法 = StpLogic 方法的静态转发） | （对应 StpUtil::xxx()） | 1 |

---

## 4. SaTokenConfig（`sa-token-core/src/config/sa_token_config.rs`）

| # | Java 字段/方法 | Rust 字段/方法 | Phase |
|---|---|---|---|
| 1 | `tokenName` / `getTokenName()` | `token_name: String` | 1 |
| 2 | `timeout` / `getTimeout()` | `timeout: i64` | 1 |
| 3 | `activeTimeout` / `getActiveTimeout()` | `active_timeout: i64` | 1 |
| 4 | `isConcurrent` / `getIsConcurrent()` | `is_concurrent: bool` | 1 |
| 5 | `isShare` / `getIsShare()` | `is_share: bool` | 1 |
| 6 | `maxLoginCount` / `getMaxLoginCount()` | `max_login_count: i32` | 1 |
| 7 | `maxTryTimes` / `getMaxTryTimes()` | `max_try_times: i32` | 1 |
| 8 | `isReadBody` / `getIsReadBody()` | `is_read_body: bool` | 1 |
| 9 | `isReadHeader` / `getIsReadHeader()` | `is_read_header: bool` | 1 |
| 10 | `isReadCookie` / `getIsReadCookie()` | `is_read_cookie: bool` | 1 |
| 11 | `isLastingCookie` / `getIsLastingCookie()` | `is_lasting_cookie: bool` | 1 |
| 12 | `isWriteHeader` / `getIsWriteHeader()` | `is_write_header: bool` | 1 |
| 13 | `tokenStyle` / `getTokenStyle()` | `token_style: SaTokenStyle` | 1 |
| 14 | `tokenPrefix` / `getTokenPrefix()` | `token_prefix: String` | 1 |
| 15 | `isLog` / `getIsLog()` | `is_log: bool` | 1 |
| 16 | `jwtSecretKey` / `getJwtSecretKey()` | `jwt_secret_key: String` | 1 |
| 17 | `cookie` / `getCookie()` | `cookie: SaCookieConfig` | 1 |

---

## 5. SttpInterface（`sa-token-core/src/stp/stp_interface.rs`）

| # | Java 方法 | Rust 方法 | Phase |
|---|---|---|---|
| 1 | `getPermissionList(Object loginId, String loginType)` → `List<String>` | `get_permission_list(login_id, login_type)` → `Vec<String>` | 1 |
| 2 | `getRoleList(Object loginId, String loginType)` → `List<String>` | `get_role_list(login_id, login_type)` → `Vec<String>` | 1 |
| 3 | `isDisabled(Object loginId, String service)` → `SaDisableWrapperInfo` | `is_disabled(login_id, service)` → `SaDisableWrapperInfo` | 2 |

---

## 6. SaTokenDao（`sa-token-core/src/dao/sa_token_dao.rs`）

| # | Java 方法 | Rust 方法 | Phase |
|---|---|---|---|
| 1 | `get(String)` → `String` | `get(key)` → `Option<String>` | 1 |
| 2 | `set(String, String, long)` | `set(key, value, timeout)` | 1 |
| 3 | `update(String, String)` | `update(key, value)` | 1 |
| 4 | `delete(String)` | `delete(key)` | 1 |
| 5 | `getTimeout(String)` → `long` | `get_timeout(key)` → `i64` | 1 |
| 6 | `updateTimeout(String, long)` | `update_timeout(key, timeout)` | 1 |
| 7 | `getObject(String)` → `Object` | `get_object(key)` → `Option<Value>` | 1 |
| 8 | `setObject(String, Object, long)` | `set_object(key, obj, timeout)` | 1 |
| 9 | `updateObject(String, Object)` | `update_object(key, obj)` | 1 |
| 10 | `deleteObject(String)` | `delete_object(key)` | 1 |
| 11 | `getSession(String)` → `SaSession` | `get_session(id)` → `Option<SaSession>` | 1 |
| 12 | `setSession(SaSession, long)` | `set_session(session, timeout)` | 1 |
| 13 | `updateSession(SaSession)` | `update_session(session)` | 1 |
| 14 | `deleteSession(String)` | `delete_session(id)` | 1 |
| 15 | `getSessionTimeout(String)` → `long` | `get_session_timeout(id)` → `i64` | 1 |
| 16 | `updateSessionTimeout(String, long)` | `update_session_timeout(id, timeout)` | 1 |
| 17 | `searchData(String, String, int, int, boolean)` → `List<String>` | `search_data(prefix, kw, start, size, sort)` → `Vec<String>` | 1 |

---

## 7. SaSession（`sa-token-core/src/session/sa_session.rs`）

| # | Java 方法 | Rust 方法 | Phase |
|---|---|---|---|
| 1 | `SaSession(String id)` | `SaSession::new(id)` | 1 |
| 2 | `getId()` → `String` | `id()` → `&str` | 1 |
| 3 | `getType()` → `String` | `r#type()` → `&str` | 1 |
| 4 | `getLoginId()` → `Object` | `login_id()` → `&Value` | 1 |
| 5 | `getToken()` → `String` | `token()` → `&str` | 1 |
| 6 | `getTerminalList()` → `List<SaTerminalInfo>` | `get_terminal_list()` → `Vec<SaTerminalInfo>` | 1 |
| 7 | `addTerminal(SaTerminalInfo)` | `add_terminal(info)` | 1 |
| 8 | `removeTerminal(String)` | `remove_terminal(token)` | 1 |
| 9 | `getTerminal(String)` → `SaTerminalInfo` | `get_terminal(token)` → `Option<SaTerminalInfo>` | 1 |
| 10 | `get(String)` → `Object` | `get(key)` → `Option<Value>` | 1 |
| 11 | `set(String, Object)` | `set(key, value)` | 1 |
| 12 | `delete(String)` | `delete(key)` | 1 |
| 13 | `update()` | `update()` | 1 |
| 14 | `logout()` | `logout()` | 1 |
| 15 | `timeout()` → `long` | `timeout()` → `i64` | 1 |

---

## 8. SaTerminalInfo（`sa-token-core/src/session/sa_terminal_info.rs`）

| # | Java 字段 | Rust 字段 | Phase |
|---|---|---|---|
| 1 | `index: int` | `index: i32` | 1 |
| 2 | `tokenValue: String` | `token_value: String` | 1 |
| 3 | `deviceType: String` | `device_type: String` | 1 |
| 4 | `deviceId: String` | `device_id: String` | 1 |
| 5 | `extraData: Map<String, Object>` | `extra_data: HashMap<String, Value>` | 1 |
| 6 | `createTime: long` | `create_time: i64` | 1 |
| 7 | （Rust 新增） | `auth_hash: String` | 1 |

---

## 参考

- [MIGRATION_STATUS.md](./MIGRATION_STATUS.md) - 迁移进度（含每方法状态追踪）
- [CODEGRAPH_METHOD_MAP.md](./CODEGRAPH_METHOD_MAP.md) - 方法级 1:1 审计
- **easyexcel-rs 对照表**：<https://github.com/easy-4-rust/easyexcel-rs/blob/main/docs/migration/object-method-matrix.md>
