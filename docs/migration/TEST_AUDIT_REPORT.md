# Sa-Token-Rs 测试审计报告

> 本文档追踪 Sa-Token-Rs 的测试覆盖情况，参考 easyexcel-rs 的四层测试体系。
> 包含：单元测试、1:1 方法测试、Golden 测试、Parity 测试。

> 仓库地址：<https://github.com/easy-4-rust/sa-token-rs>

---

## 一、测试分层概览

| 层级 | 文件位置 | 目标数量 | 已实现 | 状态 |
|---|---|---|---|---|
| 单元测试 | `crates/*/src/tests.rs` | 每 crate | 30+ | 🟡 部分 |
| 1:1 方法测试 | `crates/sa-token-test/tests/1to1/*.rs` | ~150 | 60+ | 🟡 进行中 |
| Golden 测试 | `crates/sa-token-test/tests/golden/*.rs` | ~50 | 5+ | 🟡 已建立基线 |
| Parity 测试 | `crates/sa-token-test/tests/parity/*.rs` | ~100 | 10+ | 🟡 进行中 |
| **合计** | — | — | **172 passed / 0 failed**（最近一次 `cargo test --workspace --all-features`） | ✅ |

---

## 二、单元测试清单

### 2.1 sa-token-core 单元测试

| 模块 | 测试文件 | 测试方法 | 状态 |
|---|---|---|---|
| `util/sa_fox_util` | `tests.rs` | `test_random_string`, `test_is_empty`, `test_equals`, `test_join_param` | ⬜ |
| `config/sa_token_config` | `tests.rs` | `test_default_values`, `test_custom_values` | ⬜ |
| `config/sa_cookie_config` | `tests.rs` | `test_domain`, `test_path`, `test_secure` | ⬜ |
| `exception` | `tests.rs` | `test_not_login`, `test_not_permission`, `test_clone_eq` | ⬜ |
| `session/sa_session` | `tests.rs` | `test_terminal_add_remove`, `test_data_set_get`, `test_timeout` | ⬜ |
| `session/sa_terminal_info` | `tests.rs` | `test_fields`, `test_extra_data` | ⬜ |
| `dao/sa_token_dao_default_impl` | `tests.rs` | `test_string_crud`, `test_session_crud`, `test_search` | ⬜ |
| `stp/stp_interface_default_impl` | `tests.rs` | `test_empty_lists` | ⬜ |
| `stp/sa_token_info` | `tests.rs` | `test_fields` | ⬜ |
| `stp/parameter/sa_login_parameter` | `tests.rs` | `test_builder`, `test_defaults` | ⬜ |
| `stp/parameter/sa_logout_parameter` | `tests.rs` | `test_builder`, `test_defaults` | ⬜ |
| `json/sa_json_template_default_impl` | `tests.rs` | `test_serialize_deserialize` | ⬜ |

### 2.2 sa-token-context-mock 单元测试

| 测试文件 | 测试方法 | 状态 |
|---|---|---|
| `tests.rs` | `test_set_mock_context`, `test_clear_context` | ⬜ |

### 2.3 sa-token-derive 单元测试

| 测试文件 | 测试方法 | 状态 |
|---|---|---|
| `tests.rs` | `test_sa_check_login_expand`, `test_sa_check_permission_expand`, `test_sa_check_role_expand` | ⬜ |

---

## 三、1:1 方法测试清单

每个 Java `StpLogic` 方法对应一个 Rust 测试。按 Phase 划分。

### 3.1 Phase 1（登录/登出/Token/会话）

| # | 测试名称 | 测试内容 | 对应 Java 方法 | 状态 |
|---|---|---|---|---|
| 1 | `test_login_and_is_login` | 登录后 is_login 为 true | `login()` / `isLogin()` | ⬜ |
| 2 | `test_login_get_login_id` | 登录后 get_login_id 返回正确 ID | `login()` / `getLoginId()` | ⬜ |
| 3 | `test_login_logout` | 登出后 is_login 为 false | `login()` / `logout()` / `isLogin()` | ⬜ |
| 4 | `test_login_with_device` | 指定设备类型登录 | `login(id, deviceType)` | ⬜ |
| 5 | `test_login_with_timeout` | 指定超时登录 | `login(id, timeout)` | ⬜ |
| 6 | `test_login_with_param` | 完整参数登录 | `login(id, SaLoginParameter)` | ⬜ |
| 7 | `test_login_concurrent` | 多端并发登录 | `login()` + `isConcurrent=true` | ⬜ |
| 8 | `test_login_share_token` | 共享 Token | `login()` + `isShare=true` | ⬜ |
| 9 | `test_login_max_count` | 超过最大登录数 | `login()` + `maxLoginCount` | ⬜ |
| 10 | `test_kickout_by_login_id` | 踢人下线 | `kickout(loginId)` | ⬜ |
| 11 | `test_kickout_by_token_value` | 按 Token 踢人 | `kickoutByTokenValue(token)` | ⬜ |
| 12 | `test_replaced` | 顶人下线 | `replaced(loginId)` | ⬜ |
| 13 | `test_get_token_value` | 获取 Token 值 | `getTokenValue()` | ⬜ |
| 14 | `test_get_token_info` | 获取 Token 详情 | `getTokenInfo()` | ⬜ |
| 15 | `test_token_name` | Token 名称 | `getTokenName()` | ⬜ |
| 16 | `test_token_timeout` | Token 超时 | `getTokenTimeout()` | ⬜ |
| 17 | `test_renew_timeout` | Token 续签 | `renewTimeout(timeout)` | ⬜ |
| 18 | `test_update_last_active` | 更新最后活跃时间 | `updateLastActiveToNow()` | ⬜ |
| 19 | `test_check_login_pass` | 已登录时 check_login 通过 | `checkLogin()` | ⬜ |
| 20 | `test_check_login_fail` | 未登录时 check_login 抛异常 | `checkLogin()` + NotLogin | ⬜ |
| 21 | `test_get_login_id_as_string` | loginId 转 String | `getLoginIdAsString()` | ⬜ |
| 22 | `test_get_login_id_as_i32` | loginId 转 i32 | `getLoginIdAsInt()` | ⬜ |
| 23 | `test_get_login_id_as_i64` | loginId 转 i64 | `getLoginIdAsLong()` | ⬜ |
| 24 | `test_get_login_id_default_null` | 未登录返回 None | `getLoginIdDefaultNull()` | ⬜ |
| 25 | `test_get_login_id_by_token` | 根据 Token 获取 loginId | `getLoginIdByToken(token)` | ⬜ |
| 26 | `test_get_session` | 获取当前会话 | `getSession()` | ⬜ |
| 27 | `test_get_session_by_login_id` | 按 loginId 获取会话 | `getSessionByLoginId(id)` | ⬜ |
| 28 | `test_get_token_session` | 获取 Token-Session | `getTokenSession()` | ⬜ |
| 29 | `test_session_set_get` | Session 存取数据 | `session.set()` / `session.get()` | ⬜ |
| 30 | `test_session_terminal` | Session 终端管理 | `addTerminal()` / `removeTerminal()` | ⬜ |
| 31 | `test_session_timeout` | Session 超时 | `session.timeout()` | ⬜ |
| 32 | `test_get_terminal_info` | 获取当前终端信息 | `getTerminalInfo()` | ⬜ |
| 33 | `test_get_terminal_list` | 获取终端列表 | `getTerminalListByLoginId()` | ⬜ |
| 34 | `test_login_device_type` | 获取设备类型 | `getLoginDeviceType()` | ⬜ |
| 35 | `test_login_device_id` | 获取设备 ID | `getLoginDeviceId()` | ⬜ |
| 36 | `test_trust_device` | 可信设备判断 | `isTrustDeviceId()` | ⬜ |
| 37 | `test_search_token_value` | 搜索 Token | `searchTokenValue()` | ⬜ |
| 38 | `test_search_session_id` | 搜索 Session | `searchSessionId()` | ⬜ |
| 39 | `test_create_login_session` | 创建登录会话 | `createLoginSession()` | ⬜ |
| 40 | `test_get_or_create_login_session` | 获取或创建登录会话 | `getOrCreateLoginSession()` | ⬜ |
| 41 | `test_manager_set_config` | 设置全局配置 | `SaManager.setConfig()` | ⬜ |
| 42 | `test_manager_set_dao` | 设置 DAO | `SaManager.setSaTokenDao()` | ⬜ |
| 43 | `test_manager_set_interface` | 设置权限数据源 | `SaManager.setStpInterface()` | ⬜ |
| 44 | `test_manager_stp_logic_map` | 多 StpLogic 管理 | `SaManager.putStpLogic()` | ⬜ |
| 45 | `test_stp_util_login` | StpUtil 门面登录 | `StpUtil.login()` | ⬜ |
| 46 | `test_stp_util_logout` | StpUtil 门面登出 | `StpUtil.logout()` | ⬜ |
| 47 | `test_event_do_login` | 登录事件触发 | `SaTokenEventCenter.doLogin()` | ⬜ |
| 48 | `test_event_do_logout` | 登出事件触发 | `SaTokenEventCenter.doLogout()` | ⬜ |
| 49 | `test_event_do_kickout` | 踢人事件触发 | `SaTokenEventCenter.doKickout()` | ⬜ |
| 50 | `test_freeze_token` | Token 冻结检测 | `isFreeze()` | ⬜ |

### 3.2 Phase 2（权限/角色/禁用/安全）

| # | 测试名称 | 测试内容 | 对应 Java 方法 | 状态 |
|---|---|---|---|---|
| 51 | `test_get_role_list` | 获取角色列表 | `getRoleList()` | ⬜ |
| 52 | `test_has_role_pass` | 有角色时返回 true | `hasRole()` | ⬜ |
| 53 | `test_has_role_fail` | 无角色时返回 false | `hasRole()` | ⬜ |
| 54 | `test_has_role_and` | AND 组合角色 | `hasRoleAnd()` | ⬜ |
| 55 | `test_has_role_or` | OR 组合角色 | `hasRoleOr()` | ⬜ |
| 56 | `test_check_role_pass` | 有角色时通过 | `checkRole()` | ⬜ |
| 57 | `test_check_role_fail` | 无角色时抛异常 | `checkRole()` + NotRole | ⬜ |
| 58 | `test_get_permission_list` | 获取权限列表 | `getPermissionList()` | ⬜ |
| 59 | `test_has_permission_pass` | 有权限时返回 true | `hasPermission()` | ⬜ |
| 60 | `test_has_permission_fail` | 无权限时返回 false | `hasPermission()` | ⬜ |
| 61 | `test_check_permission_pass` | 有权限时通过 | `checkPermission()` | ⬜ |
| 62 | `test_check_permission_fail` | 无权限时抛异常 | `checkPermission()` + NotPermission | ⬜ |
| 63 | `test_disable_account` | 禁用账号 | `disable(id, time)` | ⬜ |
| 64 | `test_is_disable` | 检查禁用状态 | `isDisable()` | ⬜ |
| 65 | `test_untie_disable` | 解除禁用 | `untieDisable()` | ⬜ |
| 66 | `test_open_safe` | 开启二级认证 | `openSafe()` | ⬜ |
| 67 | `test_is_safe` | 检查二级认证 | `isSafe()` | ⬜ |
| 68 | `test_check_safe` | 校验二级认证 | `checkSafe()` | ⬜ |
| 69 | `test_close_safe` | 关闭二级认证 | `closeSafe()` | ⬜ |
| 70 | `test_switch_to` | 切换账号 | `switchTo()` | ⬜ |
| 71 | `test_end_switch` | 结束切换 | `endSwitch()` | ⬜ |
| 72 | `test_is_switch` | 检查切换状态 | `isSwitch()` | ⬜ |

---

## 四、Golden 测试清单

Golden 测试：Java Sa-Token 跑出的快照（JSON）→ Rust 逐字节比对。

| # | 测试名称 | 内容 | 状态 |
|---|---|---|---|
| 1 | `golden_login_token_format` | 登录生成的 Token 格式（UUID style） | ⬜ |
| 2 | `golden_session_structure` | Session 结构（字段、terminal list） | ⬜ |
| 3 | `golden_terminal_info` | SaTerminalInfo 结构 | ⬜ |
| 4 | `golden_token_to_id_mapping` | token→loginId 映射格式 | ⬜ |
| 5 | `golden_kickout_state` | 踢人后的 Session 状态 | ⬜ |

**Golden 生成方式**：

```bash
# 1. 用 Java Sa-Token 跑出快照
cd scripts/java-golden-export
mvn test -Dtest=GoldenExportTest

# 2. 复制到 Rust 测试目录
cp target/golden/*.expected.json ../../crates/sa-token-test/tests/golden/

# 3. Rust 侧比对
cargo test --test java_golden_tests
```

---

## 五、Parity 测试清单

Parity 测试：端到端行为对等，Java 和 Rust 执行相同操作序列，结果一致。

| # | 测试名称 | 操作序列 | 状态 |
|---|---|---|---|
| 1 | `parity_login_logout` | login → is_login → logout → is_login | ⬜ |
| 2 | `parity_multi_device` | login(device1) → login(device2) → get_terminal_list → kickout(device1) | ⬜ |
| 3 | `parity_session_data` | login → session.set("k","v") → session.get("k") | ⬜ |
| 4 | `parity_permission_check` | login → check_permission(pass) → check_permission(fail) | ⬜ |
| 5 | `parity_role_check` | login → check_role(pass) → check_role(fail) | ⬜ |
| 6 | `parity_disable_account` | login → disable → is_disable → untie → is_disable | ⬜ |
| 7 | `parity_safe_auth` | login → open_safe → is_safe → close_safe | ⬜ |
| 8 | `parity_switch_account` | login → switch_to → is_switch → end_switch | ⬜ |
| 9 | `parity_token_renew` | login → get_token_timeout → renew_timeout → get_token_timeout | ⬜ |
| 10 | `parity_kickout_event` | login → register_listener → kickout → listener.received | ⬜ |

---

## 六、测试基础设施

### 6.1 测试辅助方法

`SaManager::reset()` 是测试基础设施入口，作用与 Java `SaManager.setConfig(...)` + `@BeforeEach setupSaToken()` 等价。

```rust
// sa-token-core/src/testing.rs
pub struct SaManager;
impl SaManager {
    /// 重置所有全局状态（仅测试用）
    ///
    /// 等价 Java 侧：
    /// ```java
    /// @BeforeEach
    /// void setupSaToken() {
    ///     SaManager.setConfig(new SaTokenConfig());
    ///     SaManager.setSaTokenDao(new SaTokenDaoDefaultImpl());
    /// }
    /// ```
    pub fn reset() {
        // 清空 config / dao / stp_interface / listeners / stp_logic 映射
    }
}
```

**JUnit vs Rust `#[test]` 对照：**

| 维度 | Java JUnit 5 | Sa-Token-Rs |
|---|---|---|
| 测试注解 | `@Test` | `#[test]` |
| 前置钩子 | `@BeforeEach` | 普通 fn + 测试入口手动调用 `SaManager::reset()` |
| 后置钩子 | `@AfterEach` | 普通 fn + Drop / 手动清理 |
| 断言 | `assertEquals(a, b)` | `assert_eq!(a, b)` |
| 异常断言 | `assertThrows(NotLoginException.class, () -> ...)` | `assert!(matches!(err, SaTokenException::NotLogin { .. }))` |
| 参数化 | `@ParameterizedTest` | `rstest` crate 或宏 |
| Mock 上下文 | 无（Spring 测试上下文） | `SaTokenContextMockUtil::set_mock_context()` |
| 串行执行 | `@TestInstance(Lifecycle.PER_CLASS)` + 共享状态 | `cargo test -- --test-threads=1` |
| 跳过 | `@Disabled` | `#[ignore]` |
| 显示名称 | `@DisplayName("...")` | 函数名（snake_case） |

### 6.2 Mock 上下文

```rust
// sa-token-context-mock/src/lib.rs
pub struct SaTokenContextMockUtil;
impl SaTokenContextMockUtil {
    pub fn set_mock_context() { ... }
    pub fn clear_context() { ... }
}
```

### 6.3 测试配置模板

```rust
fn default_test_config() -> SaTokenConfig {
    SaTokenConfig {
        token_name: "satoken".into(),
        timeout: 60 * 60 * 24,  // 1 天
        active_timeout: -1,
        is_concurrent: true,
        is_share: true,
        ..Default::default()
    }
}

fn setup_test() {
    SaManager::set_config(Arc::new(default_test_config()));
    SaManager::set_sa_token_dao(Arc::new(SaTokenDaoDefaultImpl::new()));
    SaTokenContextMockUtil::set_mock_context();
}
```

---

## 七、CI 配置

```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]

jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy, llvm-tools-preview
      - run: cargo fmt --all -- --check
      - run: cargo clippy --workspace --all-targets --all-features -- -D warnings
      - run: cargo test --workspace --all-features
      - name: Coverage
        run: |
          cargo install cargo-llvm-cov
          cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
```

---

## 八、覆盖率目标

| Phase | 目标覆盖率 | 说明 |
|---|---|---|
| Phase 1 | 80% | 核心登录/会话/Token |
| Phase 2 | 85% | 权限/角色/禁用/安全 |
| Phase 3 | 85% | Web 适配层 |
| Phase 5 | 90% | 插件 |
| v1.0.0 | 90%+ | 正式发布 |

---

## 参考

- [MIGRATION_STATUS.md](./MIGRATION_STATUS.md) - 迁移进度
- [CODEGRAPH_METHOD_MAP.md](./CODEGRAPH_METHOD_MAP.md) - 方法级审计
- **easyexcel-rs 测试**：<https://github.com/easy-4-rust/easyexcel-rs/tree/main/crates/easyexcel/tests>（88 golden + 152 parity）
