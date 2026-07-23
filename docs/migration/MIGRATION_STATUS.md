# Sa-Token-Rs 迁移状态

> 本文档追踪 Sa-Token Java → Sa-Token-Rs 的迁移进度，按 Phase 划分。
> 参考 easyexcel-rs 的 `MIGRATION_STATUS.md` 组织方式。

---

## 总体进度

| Phase | 名称 | 状态 | 目标时间 | 已实现方法数 |
|---|---|---|---|---|
| Phase 0 | 基础设施（workspace/骨架） | ⬜ 未开始 | 第 1 天 | — |
| Phase 1 | MVP 核心（登录/会话/Token） | ⬜ 未开始 | 2~3 周 | ~80 |
| Phase 2 | 权限/角色/注解宏 | ⬜ 未开始 | +2 周 | ~70 |
| Phase 3 | Web 框架适配 | ⬜ 未开始 | +2 周 | — |
| Phase 4 | 存储扩展 | ⬜ 未开始 | +1 周 | — |
| Phase 5 | 插件生态 | ⬜ 未开始 | +4 周 | ~200 |
| Phase 6 | 示例/文档/测试 | ⬜ 未开始 | +2 周 | — |

---

## Phase 0：基础设施

**目标**：创建 workspace、所有 crate 骨架、CI 配置。

| 任务 | 状态 | 文件 |
|---|---|---|
| 创建 workspace Cargo.toml | ⬜ | `Cargo.toml` |
| 创建 sa-token-core 骨架（所有目录 + mod.rs） | ⬜ | `crates/sa-token-core/` |
| 创建 sa-token-derive 骨架 | ⬜ | `crates/sa-token-support/sa-token-derive/` |
| 创建 sa-token-context-mock 骨架 | ⬜ | `crates/sa-token-support/sa-token-context-mock/` |
| 创建 sa-token facade 骨架 | ⬜ | `crates/sa-token/` |
| 创建 sa-token-dao-memory 骨架 | ⬜ | `crates/sa-token-dao/sa-token-dao-memory/` |
| 创建 sa-token-test 骨架 | ⬜ | `crates/sa-token-test/` |
| 创建 CI 配置 | ⬜ | `.github/workflows/ci.yml` |
| 创建 README.md | ⬜ | `README.md` |

---

## Phase 1：MVP 核心

**目标**：跑通 `login → is_login → get_login_id → logout`。

### 1.1 工具与基础设施

| Java 方法 | Rust 方法 | 状态 | 测试 |
|---|---|---|---|
| `SaFoxUtil.getRandomString(int)` | `SaFoxUtil::random_string(len)` | ⬜ | ⬜ |
| `SaFoxUtil.isEmpty(Object)` | `SaFoxUtil::is_empty(s)` | ⬜ | ⬜ |
| `SaFoxUtil.isNotEmpty(Object)` | `SaFoxUtil::is_not_empty(s)` | ⬜ | ⬜ |
| `SaFoxUtil.equals(Object, Object)` | `SaFoxUtil::equals(a, b)` | ⬜ | ⬜ |
| `SaFoxUtil.joinParam(...)` | `SaFoxUtil::join_param(...)` | ⬜ | ⬜ |
| `SaTokenConsts.*` | `SaTokenConsts::*` | ⬜ | ⬜ |
| `SaResult` | `SaResult` struct | ⬜ | ⬜ |
| `SaTokenException` (基类) | `SaTokenException` enum | ⬜ | ⬜ |
| `NotLoginException` | `SaTokenException::NotLogin` | ⬜ | ⬜ |

### 1.2 配置

| Java 方法 | Rust 方法 | 状态 | 测试 |
|---|---|---|---|
| `SaTokenConfig.getTokenName()` | `SaTokenConfig::token_name()` | ⬜ | ⬜ |
| `SaTokenConfig.getTimeout()` | `SaTokenConfig::timeout()` | ⬜ | ⬜ |
| `SaTokenConfig.getActiveTimeout()` | `SaTokenConfig::active_timeout()` | ⬜ | ⬜ |
| `SaTokenConfig.getIsConcurrent()` | `SaTokenConfig::is_concurrent()` | ⬜ | ⬜ |
| `SaTokenConfig.getIsShare()` | `SaTokenConfig::is_share()` | ⬜ | ⬜ |
| `SaTokenConfig.getMaxLoginCount()` | `SaTokenConfig::max_login_count()` | ⬜ | ⬜ |
| `SaTokenConfig.getTokenStyle()` | `SaTokenConfig::token_style()` | ⬜ | ⬜ |
| `SaTokenConfig.getIsLog()` | `SaTokenConfig::is_log()` | ⬜ | ⬜ |
| `SaCookieConfig.getDomain()` | `SaCookieConfig::domain()` | ⬜ | ⬜ |
| `SaCookieConfig.getPath()` | `SaCookieConfig::path()` | ⬜ | ⬜ |
| `SaCookieConfig.getSecure()` | `SaCookieConfig::secure()` | ⬜ | ⬜ |
| `SaCookieConfig.getHttpOnly()` | `SaCookieConfig::http_only()` | ⬜ | ⬜ |
| `SaCookieConfig.getSameSite()` | `SaCookieConfig::same_site()` | ⬜ | ⬜ |

### 1.3 上下文

| Java 方法 | Rust 方法 | 状态 | 测试 |
|---|---|---|---|
| `SaHolder.getContext()` | `SaHolder::current_context()` | ⬜ | ⬜ |
| `SaHolder.getRequest()` | `SaHolder::request()` | ⬜ | ⬜ |
| `SaHolder.getResponse()` | `SaHolder::response()` | ⬜ | ⬜ |
| `SaHolder.getStorage()` | `SaHolder::storage()` | ⬜ | ⬜ |
| `SaTokenContext.setContext(...)` | `SaTokenContext::set_context(...)` | ⬜ | ⬜ |
| `SaTokenContext.clearContext()` | `SaTokenContext::clear_context()` | ⬜ | ⬜ |
| `SaTokenContext.isValid()` | `SaTokenContext::is_valid()` | ⬜ | ⬜ |
| `SaRequest.getParam(String)` | `SaRequest::get_param(name)` | ⬜ | ⬜ |
| `SaRequest.getHeader(String)` | `SaRequest::get_header(name)` | ⬜ | ⬜ |
| `SaRequest.getCookieValue(String)` | `SaRequest::get_cookie_value(name)` | ⬜ | ⬜ |
| `SaRequest.getRequestPath()` | `SaRequest::get_request_path()` | ⬜ | ⬜ |
| `SaResponse.addCookie(SaCookie)` | `SaResponse::add_cookie(cookie)` | ⬜ | ⬜ |
| `SaResponse.deleteCookie(String)` | `SaResponse::delete_cookie(name)` | ⬜ | ⬜ |
| `SaResponse.setStatus(int)` | `SaResponse::set_status(code)` | ⬜ | ⬜ |
| `SaResponse.setHeader(...)` | `SaResponse::set_header(...)` | ⬜ | ⬜ |
| `SaStorage.get(String)` | `SaStorage::get(key)` | ⬜ | ⬜ |
| `SaStorage.set(String, Object)` | `SaStorage::set(key, value)` | ⬜ | ⬜ |
| `SaCookie` | `SaCookie` struct | ⬜ | ⬜ |

### 1.4 存储层（DAO）

| Java 方法 | Rust 方法 | 状态 | 测试 |
|---|---|---|---|
| `SaTokenDao.get(String)` | `SaTokenDao::get(key)` | ⬜ | ⬜ |
| `SaTokenDao.set(String, String, long)` | `SaTokenDao::set(key, value, timeout)` | ⬜ | ⬜ |
| `SaTokenDao.update(String, String)` | `SaTokenDao::update(key, value)` | ⬜ | ⬜ |
| `SaTokenDao.delete(String)` | `SaTokenDao::delete(key)` | ⬜ | ⬜ |
| `SaTokenDao.getTimeout(String)` | `SaTokenDao::get_timeout(key)` | ⬜ | ⬜ |
| `SaTokenDao.updateTimeout(String, long)` | `SaTokenDao::update_timeout(key, timeout)` | ⬜ | ⬜ |
| `SaTokenDao.getObject(String)` | `SaTokenDao::get_object(key)` | ⬜ | ⬜ |
| `SaTokenDao.setObject(...)` | `SaTokenDao::set_object(...)` | ⬜ | ⬜ |
| `SaTokenDao.getSession(String)` | `SaTokenDao::get_session(id)` | ⬜ | ⬜ |
| `SaTokenDao.setSession(SaSession, long)` | `SaTokenDao::set_session(session, timeout)` | ⬜ | ⬜ |
| `SaTokenDao.updateSession(SaSession)` | `SaTokenDao::update_session(session)` | ⬜ | ⬜ |
| `SaTokenDao.deleteSession(String)` | `SaTokenDao::delete_session(id)` | ⬜ | ⬜ |
| `SaTokenDao.searchData(...)` | `SaTokenDao::search_data(...)` | ⬜ | ⬜ |

### 1.5 会话

| Java 方法 | Rust 方法 | 状态 | 测试 |
|---|---|---|---|
| `SaSession(String)` constructor | `SaSession::new(id)` | ⬜ | ⬜ |
| `SaSession.getId()` | `SaSession::id()` | ⬜ | ⬜ |
| `SaSession.getType()` | `SaSession::r#type()` | ⬜ | ⬜ |
| `SaSession.getLoginId()` | `SaSession::login_id()` | ⬜ | ⬜ |
| `SaSession.getToken()` | `SaSession::token()` | ⬜ | ⬜ |
| `SaSession.getTerminalList()` | `SaSession::get_terminal_list()` | ⬜ | ⬜ |
| `SaSession.addTerminal(SaTerminalInfo)` | `SaSession::add_terminal(info)` | ⬜ | ⬜ |
| `SaSession.removeTerminal(String)` | `SaSession::remove_terminal(token)` | ⬜ | ⬜ |
| `SaSession.getTerminal(String)` | `SaSession::get_terminal(token)` | ⬜ | ⬜ |
| `SaSession.get(String)` | `SaSession::get(key)` | ⬜ | ⬜ |
| `SaSession.set(String, Object)` | `SaSession::set(key, value)` | ⬜ | ⬜ |
| `SaSession.delete(String)` | `SaSession::delete(key)` | ⬜ | ⬜ |
| `SaSession.update()` | `SaSession::update()` | ⬜ | ⬜ |
| `SaSession.logout()` | `SaSession::logout()` | ⬜ | ⬜ |
| `SaSession.timeout()` | `SaSession::timeout()` | ⬜ | ⬜ |
| `SaTerminalInfo` fields | `SaTerminalInfo` struct | ⬜ | ⬜ |

### 1.6 核心逻辑（StpLogic Phase 1）

| Java 方法 | Rust 方法 | 状态 | 测试 |
|---|---|---|---|
| `StpLogic(String)` constructor | `StpLogic::new(login_type)` | ⬜ | ⬜ |
| `StpLogic.getLoginType()` | `StpLogic::login_type()` | ⬜ | ⬜ |
| `StpLogic.getConfig()` | `StpLogic::config()` | ⬜ | ⬜ |
| `StpLogic.login(Object)` | `StpLogic::login(id)` | ⬜ | ⬜ |
| `StpLogic.login(Object, SaLoginParameter)` | `StpLogic::login_with_param(id, param)` | ⬜ | ⬜ |
| `StpLogic.createLoginSession(Object, SaLoginParameter)` | `StpLogic::create_login_session(id, param)` | ⬜ | ⬜ |
| `StpLogic.getOrCreateLoginSession(Object)` | `StpLogic::get_or_create_login_session(id)` | ⬜ | ⬜ |
| `StpLogic.logout()` | `StpLogic::logout()` | ⬜ | ⬜ |
| `StpLogic.logoutByTokenValue(String)` | `StpLogic::logout_by_token_value(token)` | ⬜ | ⬜ |
| `StpLogic.kickout(Object)` | `StpLogic::kickout_by_login_id(id)` | ⬜ | ⬜ |
| `StpLogic.kickoutByTokenValue(String)` | `StpLogic::kickout_by_token_value(token)` | ⬜ | ⬜ |
| `StpLogic.replaced(Object)` | `StpLogic::replaced_by_login_id(id)` | ⬜ | ⬜ |
| `StpLogic.replacedByTokenValue(String)` | `StpLogic::replaced_by_token_value(token)` | ⬜ | ⬜ |
| `StpLogic.isLogin()` | `StpLogic::is_login()` | ⬜ | ⬜ |
| `StpLogic.checkLogin()` | `StpLogic::check_login()` | ⬜ | ⬜ |
| `StpLogic.getLoginId()` | `StpLogic::get_login_id()` | ⬜ | ⬜ |
| `StpLogic.getLoginIdDefaultNull()` | `StpLogic::get_login_id_default_null()` | ⬜ | ⬜ |
| `StpLogic.getLoginIdAsString()` | `StpLogic::get_login_id_as_string()` | ⬜ | ⬜ |
| `StpLogic.getLoginIdAsInt()` | `StpLogic::get_login_id_as_i32()` | ⬜ | ⬜ |
| `StpLogic.getLoginIdAsLong()` | `StpLogic::get_login_id_as_i64()` | ⬜ | ⬜ |
| `StpLogic.getLoginIdByToken(String)` | `StpLogic::get_login_id_by_token(token)` | ⬜ | ⬜ |
| `StpLogic.getTokenValue()` | `StpLogic::get_token_value()` | ⬜ | ⬜ |
| `StpLogic.getTokenValueNotCut()` | `StpLogic::get_token_value_not_cut()` | ⬜ | ⬜ |
| `StpLogic.setTokenValue(String)` | `StpLogic::set_token_value(token)` | ⬜ | ⬜ |
| `StpLogic.getTokenName()` | `StpLogic::token_name()` | ⬜ | ⬜ |
| `StpLogic.getTokenInfo()` | `StpLogic::get_token_info()` | ⬜ | ⬜ |
| `StpLogic.getSession()` | `StpLogic::get_session()` | ⬜ | ⬜ |
| `StpLogic.getSessionByLoginId(Object)` | `StpLogic::get_session_by_login_id(id)` | ⬜ | ⬜ |
| `StpLogic.getTokenSession()` | `StpLogic::get_token_session()` | ⬜ | ⬜ |
| `StpLogic.getTokenSessionByToken(String)` | `StpLogic::get_token_session_by_token(token)` | ⬜ | ⬜ |
| `StpLogic.getTokenTimeout()` | `StpLogic::get_token_timeout()` | ⬜ | ⬜ |
| `StpLogic.renewTimeout(long)` | `StpLogic::renew_timeout(timeout)` | ⬜ | ⬜ |
| `StpLogic.updateLastActiveToNow()` | `StpLogic::update_last_active_to_now()` | ⬜ | ⬜ |
| `StpLogic.checkActiveTimeout()` | `StpLogic::check_active_timeout()` | ⬜ | ⬜ |
| `StpLogic.getTokenValueByLoginId(Object)` | `StpLogic::get_token_value_by_login_id(id)` | ⬜ | ⬜ |
| `StpLogic.getTokenValueListByLoginId(Object)` | `StpLogic::get_token_value_list_by_login_id(id)` | ⬜ | ⬜ |
| `StpLogic.getTerminalListByLoginId(Object)` | `StpLogic::get_terminal_list_by_login_id(id)` | ⬜ | ⬜ |
| `StpLogic.getTerminalInfo()` | `StpLogic::get_terminal_info()` | ⬜ | ⬜ |
| `StpLogic.getTerminalInfoByToken(String)` | `StpLogic::get_terminal_info_by_token(token)` | ⬜ | ⬜ |
| `StpLogic.getLoginDeviceType()` | `StpLogic::get_login_device_type()` | ⬜ | ⬜ |
| `StpLogic.getLoginDeviceId()` | `StpLogic::get_login_device_id()` | ⬜ | ⬜ |
| `StpLogic.isTrustDeviceId(Object, String)` | `StpLogic::is_trust_device_id(user_id, device_id)` | ⬜ | ⬜ |
| `StpLogic.searchTokenValue(...)` | `StpLogic::search_token_value(...)` | ⬜ | ⬜ |
| `StpLogic.searchSessionId(...)` | `StpLogic::search_session_id(...)` | ⬜ | ⬜ |
| `StpLogic.searchTokenSessionId(...)` | `StpLogic::search_token_session_id(...)` | ⬜ | ⬜ |
| `StpLogic.isFreeze(String)` | `StpLogic::is_freeze(token)` | ⬜ | ⬜ |
| `StpLogic.getExtra(String)` | `StpLogic::get_extra(key)` | ⬜ | ⬜ |

### 1.7 StpUtil（门面）

| Java 方法 | Rust 方法 | 状态 | 测试 |
|---|---|---|---|
| 所有 StpLogic 方法的静态转发 | 对应的 `StpUtil::xxx()` | ⬜ | ⬜ |

### 1.8 事件与日志

| Java 方法 | Rust 方法 | 状态 | 测试 |
|---|---|---|---|
| `SaTokenListener.doLogin(...)` | `SaTokenListener::do_login(...)` | ⬜ | ⬜ |
| `SaTokenListener.doLogout(...)` | `SaTokenListener::do_logout(...)` | ⬜ | ⬜ |
| `SaTokenListener.doKickout(...)` | `SaTokenListener::do_kickout(...)` | ⬜ | ⬜ |
| `SaTokenEventCenter.registerListener(...)` | `SaTokenEventCenter::register_listener(...)` | ⬜ | ⬜ |
| `SaTokenEventCenter.doLogin(...)` | `SaTokenEventCenter::do_login(...)` | ⬜ | ⬜ |
| `SaLog.trace/debug/info/warn/error/fatal` | 对应方法 | ⬜ | ⬜ |

### 1.9 Manager

| Java 方法 | Rust 方法 | 状态 | 测试 |
|---|---|---|---|
| `SaManager.setConfig(...)` | `SaManager::set_config(...)` | ⬜ | ⬜ |
| `SaManager.getConfig()` | `SaManager::config()` | ⬜ | ⬜ |
| `SaManager.setSaTokenDao(...)` | `SaManager::set_sa_token_dao(...)` | ⬜ | ⬜ |
| `SaManager.getSaTokenDao()` | `SaManager::sa_token_dao()` | ⬜ | ⬜ |
| `SaManager.setStpInterface(...)` | `SaManager::set_stp_interface(...)` | ⬜ | ⬜ |
| `SaManager.getStpInterface()` | `SaManager::stp_interface()` | ⬜ | ⬜ |
| `SaManager.putStpLogic(...)` | `SaManager::put_stp_logic(...)` | ⬜ | ⬜ |
| `SaManager.getStpLogic(String)` | `SaManager::get_stp_logic(login_type)` | ⬜ | ⬜ |

---

## Phase 2：权限/角色/注解宏

**目标**：权限、角色、禁用、安全认证、切换账号、设备、续签、搜索。

### 2.1 权限与角色

| Java 方法 | Rust 方法 | 状态 | 测试 |
|---|---|---|---|
| `StpLogic.getRoleList()` | `StpLogic::get_role_list()` | ⬜ | ⬜ |
| `StpLogic.hasRole(String)` | `StpLogic::has_role(role)` | ⬜ | ⬜ |
| `StpLogic.hasRoleAnd(String...)` | `StpLogic::has_role_and(roles)` | ⬜ | ⬜ |
| `StpLogic.hasRoleOr(String...)` | `StpLogic::has_role_or(roles)` | ⬜ | ⬜ |
| `StpLogic.checkRole(String)` | `StpLogic::check_role(role)` | ⬜ | ⬜ |
| `StpLogic.checkRoleAnd(String...)` | `StpLogic::check_role_and(roles)` | ⬜ | ⬜ |
| `StpLogic.checkRoleOr(String...)` | `StpLogic::check_role_or(roles)` | ⬜ | ⬜ |
| `StpLogic.getPermissionList()` | `StpLogic::get_permission_list()` | ⬜ | ⬜ |
| `StpLogic.hasPermission(String)` | `StpLogic::has_permission(perm)` | ⬜ | ⬜ |
| `StpLogic.hasPermissionAnd(String...)` | `StpLogic::has_permission_and(perms)` | ⬜ | ⬜ |
| `StpLogic.hasPermissionOr(String...)` | `StpLogic::has_permission_or(perms)` | ⬜ | ⬜ |
| `StpLogic.checkPermission(String)` | `StpLogic::check_permission(perm)` | ⬜ | ⬜ |
| `StpLogic.checkPermissionAnd(String...)` | `StpLogic::check_permission_and(perms)` | ⬜ | ⬜ |
| `StpLogic.checkPermissionOr(String...)` | `StpLogic::check_permission_or(perms)` | ⬜ | ⬜ |

### 2.2 禁用

| Java 方法 | Rust 方法 | 状态 | 测试 |
|---|---|---|---|
| `StpLogic.disable(Object, long)` | `StpLogic::disable(login_id, time)` | ⬜ | ⬜ |
| `StpLogic.isDisable(Object)` | `StpLogic::is_disable(login_id)` | ⬜ | ⬜ |
| `StpLogic.checkDisable(Object)` | `StpLogic::check_disable(login_id)` | ⬜ | ⬜ |
| `StpLogic.getDisableTime(Object)` | `StpLogic::get_disable_time(login_id)` | ⬜ | ⬜ |
| `StpLogic.untieDisable(Object)` | `StpLogic::untie_disable(login_id)` | ⬜ | ⬜ |

### 2.3 安全认证

| Java 方法 | Rust 方法 | 状态 | 测试 |
|---|---|---|---|
| `StpLogic.openSafe(long)` | `StpLogic::open_safe(safe_time)` | ⬜ | ⬜ |
| `StpLogic.isSafe()` | `StpLogic::is_safe()` | ⬜ | ⬜ |
| `StpLogic.checkSafe()` | `StpLogic::check_safe()` | ⬜ | ⬜ |
| `StpLogic.closeSafe()` | `StpLogic::close_safe()` | ⬜ | ⬜ |

### 2.4 切换账号

| Java 方法 | Rust 方法 | 状态 | 测试 |
|---|---|---|---|
| `StpLogic.switchTo(Object)` | `StpLogic::switch_to(login_id)` | ⬜ | ⬜ |
| `StpLogic.endSwitch()` | `StpLogic::end_switch()` | ⬜ | ⬜ |
| `StpLogic.isSwitch()` | `StpLogic::is_switch()` | ⬜ | ⬜ |

### 2.5 注解宏

| Java 注解 | Rust 宏 | 状态 | 测试 |
|---|---|---|---|
| `@SaCheckLogin` | `#[sa_check_login]` | ⬜ | ⬜ |
| `@SaCheckPermission("x")` | `#[sa_check_permission("x")]` | ⬜ | ⬜ |
| `@SaCheckRole("x")` | `#[sa_check_role("x")]` | ⬜ | ⬜ |
| `@SaCheckSafe` | `#[sa_check_safe]` | ⬜ | ⬜ |
| `@SaCheckDisable` | `#[sa_check_disable]` | ⬜ | ⬜ |
| `@SaCheckOr(...)` | `#[sa_check_or(...)]` | ⬜ | ⬜ |
| `@SaIgnore` | `#[sa_ignore]` | ⬜ | ⬜ |

### 2.6 剩余子模块

| 模块 | 状态 | 说明 |
|---|---|---|
| `strategy/` (SaStrategy, SaAnnotationStrategy, SaFirewallStrategy) | ⬜ | |
| `filter/` (SaFilter) | ⬜ | |
| `router/` (SaRouter, SaRouterStaff) | ⬜ | |
| `httpauth/basic/` (SaHttpBasicTemplate) | ⬜ | |
| `httpauth/digest/` (SaHttpDigestTemplate) | ⬜ | |
| `same/` (SaSameTemplate, SaSameUtil) | ⬜ | |
| `temp/` (SaTempTemplate, SaTempUtil) | ⬜ | |
| `application/` (SaApplication) | ⬜ | |
| `plugin/` (SaTokenPlugin) | ⬜ | |
| `secure/` (SaSecureUtil, BCrypt, TOTP) | ⬜ | |

---

## Phase 3~6

Phase 3~6 的详细方法矩阵将在对应 Phase 启动时补充。

---

## 统计

| 指标 | 数值 |
|---|---|
| Java StpLogic public 方法总数 | ~196 |
| Phase 1 已规划方法数 | ~80 |
| Phase 2 已规划方法数 | ~70 |
| Phase 1+2 总计已规划 | ~150 |
| 剩余（Phase 5 插件等） | ~200+ |

---

## 参考

- [IMPLEMENTATION_PLAN.md](../IMPLEMENTATION_PLAN.md) - 完整实施计划
- [object-method-matrix.md](./object-method-matrix.md) - 方法级对照表
- **easyexcel-rs 迁移状态**：`/Users/wandl/workspaces/workspace-github/easyexcel-rs/docs/migration/MIGRATION_STATUS.md`
