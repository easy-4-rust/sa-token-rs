# Sa-Token-Rs 最终实施计划

> **版本**：v1.0（最终版，整合所有前期讨论与修正）
> **基线**：Sa-Token Java `dev` 分支 `89e47c12`
> **参考项目**：`easyexcel-rs`（同源 Java→Rust 移植，已验证 1315+ tests 全绿）
> **定位**：一比一复刻 Sa-Token Java 的 Rust 实现，命名、目录结构、API 语义严格对齐
> **状态**：待批准实施

---

## 目录

- [一、项目背景与目标](#一项目背景与目标)
- [二、核心设计原则](#二核心设计原则)
- [三、命名映射规则（严格对齐 Java）](#三命名映射规则严格对齐-java)
- [四、Workspace 总体结构](#四workspace-总体结构)
- [五、sa-token-core 内部模块（1:1 对齐 Java 包）](#五sa-token-core-内部模块11-对齐-java-包)
- [六、核心 Trait / 结构体签名](#六核心-trait--结构体签名)
- [七、注解（proc-macro）设计](#七注解proc-macro设计)
- [八、Web 框架适配层](#八web-框架适配层)
- [九、关键技术决策汇总](#九关键技术决策汇总)
- [十、分阶段实施计划](#十分阶段实施计划)
- [十一、依赖清单](#十一依赖清单)
- [十二、测试体系](#十二测试体系)
- [十三、迁移文档体系](#十三迁移文档体系)
- [十四、风险与缓解](#十四风险与缓解)
- [十五、Java ↔ Rust 完整文件对应关系](#十五java--rust-完整文件对应关系)
- [十六、Phase 1 立即执行清单](#十六phase-1-立即执行清单)
- [附录 A：与 Sa-Token Java 的关键差异速查](#附录-a与-sa-token-java-的关键差异速查)
- [附录 B：参考项目](#附录-b参考项目)

---

## 一、项目背景与目标

### 1.1 背景

Sa-Token 是 Java 生态最流行的轻量级权限认证框架，提供「登录认证 + 权限认证 + Session + 踢人下线 + SSO + 注解鉴权」全家桶能力。当前 Rust 生态**没有**能与之对标的单框架：

- `axum-login` + `tower-sessions`：仅覆盖登录与会话
- `casbin-rs`：仅权限引擎
- `jsonwebtoken` / `pasetors`：仅 Token
- `oauth2` / `openidconnect`：仅 OAuth2/OIDC 协议层

用户需要自行组合多个 crate，且无法直接复用 Sa-Token 的 API 心智模型。

### 1.2 目标

**一比一**复刻 Sa-Token Java 到 Rust：

| 维度 | 目标 |
|---|---|
| **API 语义** | `StpUtil::login(id)`、`StpUtil::is_login()`、`StpUtil::has_permission(p)` 等命名与 Java 一致 |
| **目录结构** | `sa-token-core/src/stp/stp_logic.rs` 对齐 `cn.dev33.satoken.stp.StpLogic` |
| **功能覆盖** | 登录、登出、踢人、顶替、权限、角色、会话、SSO、OAuth2、JWT、Sign、ApiKey 全覆盖 |
| **Rust 化** | 充分利用 Rust 类型系统、所有权、async/await、proc-macro |
| **生态对接** | axum / actix-web / salvo 三大框架；Redis / moka 存储 |

### 1.3 非目标

- ❌ 不追求 Java 字节码级 1:1（Rust 与 Java 语义差异大）
- ❌ 不复刻 Spring AOP 运行时反射（改用 proc-macro 编译期生成）
- ❌ 不复刻 Sa-Token 自带的弱权限引擎（改用 casbin/cedar 可插拔适配）

---

## 二、核心设计原则

本项目参考了 `easyexcel-rs`（同样是 Java→Rust 一比一移植项目，已验证成熟）的五条核心设计决策：

### 原则 1：核心同步 + 适配层 async（**重要修正**）

> ⚠️ **推翻了早期"全 async"的设想**。参考 easyexcel-rs 证明：核心同步 + facade 同步 + starter 层 async 包装 是更优解。

| 层 | 同步/异步 | 理由 |
|---|---|---|
| `sa-token-core`（StpLogic/SaSession/权限） | **同步** | 复刻 Java 阻塞语义；无 IO 时 async 无收益 |
| `sa-token-dao-memory` | 同步 | HashMap 操作 |
| `sa-token-dao-redis` | **async**（fred） | IO 必须 async |
| `sa-token-axum/actix/salvo` | **async** | Web 框架要求 |
| `sa-token-oauth2/sso` | **async** | 涉及 HTTP 调用 |

**实现策略**：定义 `SaTokenDao` 为**同步 trait**，Redis 实现内部用 `tokio::runtime::Handle::block_on` 或要求调用方在 async 上下文用 `block_in_place`。

### 原则 2：derive + helper attribute 优于 attribute 宏

```rust
// ❌ 早期设想：attribute 宏
#[sa_check_permission("user:add")]
async fn add_user() { ... }

// ✅ 最终方案：derive + helper attribute（参考 easyexcel-rs）
#[derive(SaHandler)]
#[sa_check(permission = "user:add", role = "admin")]
async fn add_user() { ... }
```

**理由**：单 derive 宏统一处理所有注解语义，`parse_nested_meta` 处理嵌套参数省 80% 代码。

### 原则 3：元数据落地为 `&'static [T]` 常量 + `const fn` builder

所有注解元数据**编译期烤进 `&'static [T]` 常量数组**，运行时零反射、零 HashMap 查找。

### 原则 4：全局状态用 `OnceLock<Arc<...>>`

```rust
static CONFIG: OnceLock<Arc<SaTokenConfig>> = OnceLock::new();
```

**修正**：不使用 `OnceLock<RwLock<T>>`（RwLock 冗余），只在需要运行期增删的字段（如 listener 列表）才在 OnceLock 内套 RwLock。

### 原则 5：错误处理单一 enum

Sa-Token Java 的 20+ 异常类折叠为单一 `thiserror` enum，实现 `Clone + Eq` 用于测试断言。

---

## 三、命名映射规则（严格对齐 Java）

| Java | Rust | 示例 |
|---|---|---|
| 包 `cn.dev33.satoken.stp` | crate + mod `sa_token_core::stp` | — |
| 类 `StpLogic` | 结构体 `StpLogic`（PascalCase 保持） | `StpLogic` |
| `StpUtil.login(id)` | `StpUtil::login(id)` | 静态方法→关联函数 |
| `getXxx()` | `xxx()` | `config()` / `login_type()` |
| `setXxx(v)` | `set_xxx(v)` | `set_config(cfg)` |
| `isXxx()` | `is_xxx()` | `is_login()` |
| `hasXxx()` | `has_xxx()` | `has_permission(p)` |
| `SaSession.USER` | `SaSession::USER` | 关联常量 |
| 接口 `SaTokenDao` | trait `SaTokenDao` | — |
| 文件 `StpLogic.java` | 文件 `stp_logic.rs` | snake_case 文件名 |
| Spring `@Bean` 注入 | `Arc<dyn Trait>` + `SaManager::set_xxx()` | — |
| Spring AOP 注解 | proc-macro `#[derive(SaHandler)]` + helper attr | — |
| `ThreadLocal` 上下文 | `thread_local!` 或显式 `&SaContext` 传递 | — |

> **原则**：类名/常量名 100% 对齐 Java；方法名按 Rust 习惯转 snake_case，但**动词与业务词汇保留 Java 原样**（`login` / `kickout` / `replaced` / `openSafe`→`open_safe`），保证阅读 Java 源码能 1:1 找到 Rust 对应实现。

---

## 四、Workspace 总体结构

```text
sa-token-rs/                                  # 仓库根（对应 Sa-Token 根 pom.xml）
├── Cargo.toml                                # [workspace] 根清单
├── README.md
├── CHANGELOG.md
├── docs/
│   ├── IMPLEMENTATION_PLAN.md                # 本文档
│   ├── ARCHITECTURE.md                       # 架构总览
│   ├── GUIDE.md                              # 使用指南
│   ├── compatibility.md                      # 兼容性说明
│   ├── ecosystem-roadmap.md                  # 生态路线图
│   └── migration/                            # 迁移审计文档
│       ├── MIGRATION_STATUS.md               # 分 Phase 迁移进度
│       ├── object-method-matrix.md           # Java 对象 × 方法矩阵
│       ├── CODEGRAPH_METHOD_MAP.md           # 方法级 1:1 审计
│       ├── java-tree-full.md                 # Java Sa-Token 完整目录树
│       ├── rust-tree-full.md                 # Rust Sa-Token-Rs 完整目录树
│       ├── project-tree-diff.md              # 两侧目录 diff
│       └── TEST_AUDIT_REPORT.md              # 测试审计
│
├── crates/
│   ├── sa-token/                             # ★ facade（用户只引这一个包）
│   ├── sa-token-core/                        # ★ 核心库（对应 sa-token-core/）
│   ├── sa-token-derive/                      # proc-macro（#[derive(SaHandler)] 等）
│   ├── sa-token-context-mock/                # Mock 上下文（测试/非 Web 场景）
│   │
│   ├── sa-token-dao-memory/                  # 默认 Memory DAO
│   ├── sa-token-dao-redis/                   # Redis DAO（async）
│   ├── sa-token-dao-moka/                    # moka 高性能缓存 DAO
│   │
│   ├── sa-token-starter/                     # Web 框架适配（对应 sa-token-starter/）
│   │   ├── sa-token-axum/                    # ★ axum 适配（首期优先）
│   │   ├── sa-token-actix-web/
│   │   └── sa-token-salvo/
│   │
│   ├── sa-token-plugin/                      # 业务插件（对应 sa-token-plugin/）
│   │   ├── sa-token-jwt/
│   │   ├── sa-token-sign/
│   │   ├── sa-token-oauth2/
│   │   ├── sa-token-sso/
│   │   └── sa-token-apikey/
│   │
│   ├── sa-token-test/                        # 集成测试
│   └── sa-token-demo/                        # 示例
│       ├── sa-token-demo-axum/
│       ├── sa-token-demo-actix-web/
│       └── sa-token-demo-salvo/
│
├── scripts/
│   ├── coverage.sh
│   ├── gap-check.sh
│   └── java-golden-export/                   # Java 生成 golden 期望文件
│
└── .github/workflows/ci.yml                  # CI: fmt + clippy(-D warnings) + test + coverage
```

### 依赖方向

```text
sa-token (facade) ──→ sa-token-core
                  ──→ sa-token-derive
                  ──→ sa-token-context-mock
                  ──→ sa-token-dao-memory
                  ──→ sa-token-starter/*
                  ──→ sa-token-plugin/*

sa-token-core ──→ (不依赖任何本 workspace crate)  ★底层基础

sa-token-derive ──→ sa-token-core (dev-deps)

sa-token-dao-* / sa-token-starter-* / sa-token-plugin-* ──→ sa-token-core
```

---

## 五、sa-token-core 内部模块（1:1 对齐 Java 包）

```text
sa-token-core/src/
├── lib.rs                                    # 对外 re-export
│
├── manager.rs                                # ← SaManager.java
│
├── annotation/                               # ← annotation/（注解本身在 sa-token-derive crate）
│   └── mod.rs                                #   仅放 SaMode 枚举等少量类型
│
├── application/                              # ← application/
│   ├── mod.rs
│   ├── application_info.rs                   # ← ApplicationInfo.java
│   ├── sa_application.rs                     # ← SaApplication.java
│   ├── sa_get_value_interface.rs             # ← SaGetValueInterface.java（trait）
│   └── sa_set_value_interface.rs             # ← SaSetValueInterface.java（trait）
│
├── config/                                   # ← config/
│   ├── mod.rs
│   ├── sa_token_config.rs                    # ← SaTokenConfig.java
│   ├── sa_cookie_config.rs                   # ← SaCookieConfig.java
│   └── sa_token_config_factory.rs            # ← SaTokenConfigFactory.java
│
├── context/                                  # ← context/
│   ├── mod.rs
│   ├── sa_holder.rs                          # ← SaHolder.java
│   ├── sa_token_context.rs                   # ← SaTokenContext.java（trait）
│   ├── sa_token_context_default_impl.rs      # ← SaTokenContextDefaultImpl.java
│   ├── sa_token_context_for_read_only.rs     # ← SaTokenContextForReadOnly.java
│   ├── sa_token_context_for_thread_local.rs  # ← SaTokenContextForThreadLocal.java
│   └── model/                                # ← context/model/
│       ├── mod.rs
│       ├── sa_cookie.rs                      # ← SaCookie.java
│       ├── sa_request.rs                     # ← SaRequest.java（trait）
│       ├── sa_response.rs                    # ← SaResponse.java（trait）
│       ├── sa_storage.rs                     # ← SaStorage.java（trait）
│       ├── sa_http_method.rs                 # ← router/SaHttpMethod.java（移入此处）
│       └── sa_token_context_model_box.rs     # ← SaTokenContextModelBox.java
│
├── dao/                                      # ← dao/
│   ├── mod.rs
│   ├── sa_token_dao.rs                       # ← SaTokenDao.java（trait，同步）
│   ├── sa_token_dao_default_impl.rs          # ← SaTokenDaoDefaultImpl.java（Memory）
│   ├── auto/                                 # ← dao/auto/
│   └── timed_cache/                          # ← dao/timedcache/
│
├── error/                                    # ← error/
│   └── mod.rs                                # ← SaErrorCode.java
│
├── exception/                                # ← exception/（20+ 异常类折叠为单一 enum）
│   └── mod.rs                                #   thiserror enum + SaResult<T>
│
├── filter/                                   # ← filter/
│   ├── mod.rs
│   ├── sa_filter.rs                          # ← SaFilter.java（trait）
│   ├── sa_filter_auth_strategy.rs            # ← SaFilterAuthStrategy.java
│   └── sa_filter_error_strategy.rs           # ← SaFilterErrorStrategy.java
│
├── fun/                                      # ← fun/（函数式接口 → Fn type alias）
│   ├── mod.rs
│   ├── strategy/                             # ← fun/strategy/
│   └── hooks/                                # ← fun/hooks/
│
├── http/                                     # ← http/
│   ├── mod.rs
│   ├── sa_http_template.rs                   # ← SaHttpTemplate.java（trait）
│   ├── sa_http_template_default_impl.rs      # ← SaHttpTemplateDefaultImpl.java
│   └── sa_http_util.rs                       # ← SaHttpUtil.java
│
├── httpauth/                                 # ← httpauth/
│   ├── basic/
│   │   ├── sa_http_basic_account.rs
│   │   ├── sa_http_basic_template.rs         # （trait）
│   │   └── sa_http_basic_util.rs
│   └── digest/
│       ├── sa_http_digest_model.rs
│       ├── sa_http_digest_template.rs
│       └── sa_http_digest_util.rs
│
├── json/                                     # ← json/
│   ├── mod.rs
│   ├── sa_json_template.rs                   # ← SaJsonTemplate.java（trait）
│   └── sa_json_template_default_impl.rs      #   默认基于 serde_json
│
├── listener/                                 # ← listener/
│   ├── mod.rs
│   ├── sa_token_listener.rs                  # ← SaTokenListener.java（trait）
│   ├── sa_token_event_center.rs              # ← SaTokenEventCenter.java
│   ├── sa_token_listener_for_log.rs          # ← SaTokenListenerForLog.java
│   └── sa_token_listener_for_simple.rs       # ← SaTokenListenerForSimple.java
│
├── log/                                      # ← log/
│   ├── mod.rs
│   ├── sa_log.rs                             # ← SaLog.java（trait）
│   └── sa_log_for_console.rs                 # ← SaLogForConsole.java（tracing 适配）
│
├── model/                                    # ← model/
│   └── wrapper_info/
│       └── sa_disable_wrapper_info.rs        # ← SaDisableWrapperInfo.java
│
├── plugin/                                   # ← plugin/
│   ├── mod.rs
│   ├── sa_token_plugin.rs                    # ← SaTokenPlugin.java（trait）
│   ├── sa_token_plugin_holder.rs             # ← SaTokenPluginHolder.java
│   └── sa_token_plugin_hook_model.rs         # ← SaTokenPluginHookModel.java
│
├── router/                                   # ← router/
│   ├── mod.rs
│   ├── sa_router.rs                          # ← SaRouter.java
│   └── sa_router_staff.rs                    # ← SaRouterStaff.java
│
├── same/                                     # ← same/
│   ├── mod.rs
│   ├── sa_same_template.rs                   # ← SaSameTemplate.java
│   └── sa_same_util.rs                       # ← SaSameUtil.java
│
├── secure/                                   # ← secure/
│   ├── mod.rs
│   ├── b_crypt.rs                            # ← BCrypt.java
│   ├── sa_base32_util.rs                     # ← SaBase32Util.java
│   ├── sa_base64_util.rs                     # ← SaBase64Util.java
│   ├── sa_secure_util.rs                     # ← SaSecureUtil.java
│   └── totp/
│       ├── sa_totp_template.rs
│       └── sa_totp_util.rs
│
├── serializer/                               # ← serializer/
│   ├── mod.rs
│   ├── sa_serializer_template.rs             # ← SaSerializerTemplate.java（trait）
│   └── impl/                                 # ← serializer/impl/
│       ├── sa_serializer_template_for_json.rs
│       └── *.rs
│
├── session/                                  # ← session/
│   ├── mod.rs
│   ├── sa_session.rs                         # ← SaSession.java
│   ├── sa_session_custom_util.rs             # ← SaSessionCustomUtil.java
│   ├── sa_terminal_info.rs                   # ← SaTerminalInfo.java
│   └── raw/
│       ├── sa_raw_session_delegator.rs       # ← SaRawSessionDelegator.java
│       └── sa_raw_session_util.rs            # ← SaRawSessionUtil.java
│
├── stp/                                      # ← stp/
│   ├── mod.rs
│   ├── stp_logic.rs                          # ← StpLogic.java ★核心
│   ├── stp_util.rs                           # ← StpUtil.java ★门面
│   ├── stp_interface.rs                      # ← StpInterface.java（trait）
│   ├── stp_interface_default_impl.rs         # ← StpInterfaceDefaultImpl.java
│   ├── sa_token_info.rs                      # ← SaTokenInfo.java
│   ├── sa_login_config.rs                    # ← SaLoginConfig.java
│   ├── sa_login_model.rs                     # ← SaLoginModel.java（旧版兼容）
│   └── parameter/                            # ← stp/parameter/
│       ├── mod.rs
│       ├── sa_login_parameter.rs             # ← SaLoginParameter.java
│       ├── sa_logout_parameter.rs            # ← SaLogoutParameter.java
│       └── enums/                            # ← stp/parameter/enums/
│           ├── sa_logout_mode.rs             # ← SaLogoutMode.java
│           ├── sa_logout_range.rs            # ← SaLogoutRange.java
│           ├── sa_replaced_login_exit_mode.rs
│           └── sa_replaced_range.rs
│
├── strategy/                                 # ← strategy/
│   ├── mod.rs
│   ├── sa_strategy.rs                        # ← SaStrategy.java
│   ├── sa_annotation_strategy.rs             # ← SaAnnotationStrategy.java
│   ├── sa_firewall_strategy.rs               # ← SaFirewallStrategy.java
│   └── hooks/                                # ← strategy/hooks/
│       └── sa_firewall_check_hook*.rs        # 9 个 Hook
│
├── temp/                                     # ← temp/
│   ├── mod.rs
│   ├── sa_temp_template.rs                   # ← SaTempTemplate.java
│   └── sa_temp_util.rs                       # ← SaTempUtil.java
│
└── util/                                     # ← util/
    ├── mod.rs
    ├── sa_fox_util.rs                        # ← SaFoxUtil.java ★工具集
    ├── sa_hex_util.rs                        # ← SaHexUtil.java
    ├── sa_result.rs                          # ← SaResult.java
    ├── sa_sugar.rs                           # ← SaSugar.java
    ├── sa_token_consts.rs                    # ← SaTokenConsts.java
    ├── sa_ttl_methods.rs                     # ← SaTtlMethods.java
    ├── sa_value2_box.rs                      # ← SaValue2Box.java
    └── str_formatter.rs                      # ← StrFormatter.java
```

---

## 六、核心 Trait / 结构体签名

### 6.1 `SaTokenDao`（存储层，**同步 trait**）

```rust
// sa-token-core/src/dao/sa_token_dao.rs
pub trait SaTokenDao: Send + Sync + 'static {
    const NEVER_EXPIRE: i64 = -1;
    const NOT_VALUE_EXPIRE: i64 = -2;

    // 字符串读写
    fn get(&self, key: &str) -> Option<String>;
    fn set(&self, key: &str, value: &str, timeout: i64);
    fn update(&self, key: &str, value: &str);
    fn delete(&self, key: &str);
    fn get_timeout(&self, key: &str) -> i64;
    fn update_timeout(&self, key: &str, timeout: i64);

    // 对象读写（基于 serde_json::Value）
    fn get_object(&self, key: &str) -> Option<serde_json::Value>;
    fn set_object(&self, key: &str, object: &serde_json::Value, timeout: i64);
    fn update_object(&self, key: &str, object: &serde_json::Value);
    fn delete_object(&self, key: &str);
    fn get_object_timeout(&self, key: &str) -> i64;
    fn update_object_timeout(&self, key: &str, timeout: i64);

    // SaSession 读写
    fn get_session(&self, session_id: &str) -> Option<SaSession>;
    fn set_session(&self, session: &SaSession, timeout: i64);
    fn update_session(&self, session: &SaSession);
    fn delete_session(&self, session_id: &str);
    fn get_session_timeout(&self, session_id: &str) -> i64;
    fn update_session_timeout(&self, session_id: &str, timeout: i64);

    // 搜索
    fn search_data(&self, prefix: &str, keyword: &str, start: i64, size: i64, sort_type: bool) -> Vec<String>;

    // 生命周期
    fn init(&self) {}
    fn destroy(&self) {}
}
```

默认实现：`SaTokenDaoDefaultImpl`（基于 `RwLock<HashMap + TTL>`）。

### 6.2 `SaTokenContext` + `SaHolder`

```rust
// sa-token-core/src/context/sa_token_context.rs
pub trait SaTokenContext: Send + Sync + 'static {
    fn set_context(&self, req: Arc<dyn SaRequest>, res: Arc<dyn SaResponse>, stg: Arc<dyn SaStorage>);
    fn clear_context(&self);
    fn is_valid(&self) -> bool;
    fn get_model_box(&self) -> SaTokenContextModelBox;
    fn request(&self) -> Arc<dyn SaRequest>;
    fn response(&self) -> Arc<dyn SaResponse>;
    fn storage(&self) -> Arc<dyn SaStorage>;
}

// sa-token-core/src/context/sa_holder.rs
// 基于 thread_local（同步核心）或 task_local（async starter）
thread_local! {
    pub static CURRENT_CONTEXT: RefCell<Option<Arc<dyn SaTokenContext>>> = const { RefCell::new(None) };
}

pub struct SaHolder;
impl SaHolder {
    pub fn current_context() -> Arc<dyn SaTokenContext> { ... }
    pub fn request() -> Arc<dyn SaRequest> { ... }
    pub fn response() -> Arc<dyn SaResponse> { ... }
    pub fn storage() -> Arc<dyn SaStorage> { ... }
}
```

### 6.3 `StpLogic`（核心逻辑，**同步**）

```rust
// sa-token-core/src/stp/stp_logic.rs
pub struct StpLogic {
    pub login_type: String,
    pub config: Arc<SaTokenConfig>,  // 不可变共享，配置变更走 SaManager
}

impl StpLogic {
    pub fn new(login_type: &str) -> Self { ... }
    pub fn login_type(&self) -> &str { ... }
    pub fn config(&self) -> Arc<SaTokenConfig> { ... }
    pub fn set_config(&self, config: Arc<SaTokenConfig>) { ... }

    // ─── 登录 ───
    pub fn login(&self, id: &str) -> SaResult<()>;
    pub fn login_with_device(&self, id: &str, device_type: &str) -> SaResult<()>;
    pub fn login_with_param(&self, id: &str, param: &SaLoginParameter) -> SaResult<()>;
    pub fn create_login_session(&self, id: &str, param: &SaLoginParameter) -> SaResult<String>;
    pub fn get_or_create_login_session(&self, id: &str) -> SaResult<String>;

    // ─── 登出 / 踢人 / 顶替 ───
    pub fn logout(&self) -> SaResult<()>;
    pub fn logout_with_param(&self, param: &SaLogoutParameter) -> SaResult<()>;
    pub fn logout_by_token_value(&self, token_value: &str) -> SaResult<()>;
    pub fn kickout_by_token_value(&self, token_value: &str) -> SaResult<()>;
    pub fn replaced_by_token_value(&self, token_value: &str) -> SaResult<()>;
    pub fn logout_by_login_id(&self, login_id: &str) -> SaResult<()>;
    pub fn kickout_by_login_id(&self, login_id: &str) -> SaResult<()>;
    pub fn replaced_by_login_id(&self, login_id: &str) -> SaResult<()>;

    // ─── Token 操作 ───
    pub fn token_name(&self) -> String;
    pub fn create_token_value(&self, login_id: &str, device_type: &str, timeout: i64, extra_data: &Value) -> String;
    pub fn set_token_value(&self, token_value: &str) -> SaResult<()>;
    pub fn set_token_value_with_param(&self, token_value: &str, param: &SaLoginParameter) -> SaResult<()>;
    pub fn get_token_value(&self) -> Option<String>;
    pub fn get_token_value_not_cut(&self) -> Option<String>;
    pub fn get_token_info(&self) -> SaResult<SaTokenInfo>;

    // ─── 登录状态 ───
    pub fn is_login(&self) -> bool;
    pub fn is_login_for(&self, login_id: &str) -> bool;
    pub fn check_login(&self) -> SaResult<()>;
    pub fn get_login_id(&self) -> SaResult<String>;
    pub fn get_login_id_default_null(&self) -> Option<String>;
    pub fn get_login_id_as_string(&self) -> SaResult<String>;
    pub fn get_login_id_as_i64(&self) -> SaResult<i64>;
    pub fn get_login_id_as_i32(&self) -> SaResult<i32>;
    pub fn get_login_id_by_token(&self, token_value: &str) -> Option<String>;

    // ─── 会话 ───
    pub fn get_session(&self) -> SaResult<SaSession>;
    pub fn get_session_by_login_id(&self, login_id: &str) -> SaResult<SaSession>;
    pub fn get_token_session(&self) -> SaResult<SaSession>;
    pub fn get_token_session_by_token(&self, token_value: &str) -> SaResult<SaSession>;
    pub fn delete_token_session(&self, token_value: &str) -> SaResult<()>;

    // ─── 权限 / 角色 ───
    pub fn get_role_list(&self) -> SaResult<Vec<String>>;
    pub fn get_role_list_for(&self, login_id: &str) -> SaResult<Vec<String>>;
    pub fn has_role(&self, role: &str) -> SaResult<bool>;
    pub fn has_role_and(&self, roles: &[&str]) -> SaResult<bool>;
    pub fn has_role_or(&self, roles: &[&str]) -> SaResult<bool>;
    pub fn check_role(&self, role: &str) -> SaResult<()>;
    pub fn check_role_and(&self, roles: &[&str]) -> SaResult<()>;
    pub fn check_role_or(&self, roles: &[&str]) -> SaResult<()>;
    pub fn get_permission_list(&self) -> SaResult<Vec<String>>;
    pub fn get_permission_list_for(&self, login_id: &str) -> SaResult<Vec<String>>;
    pub fn has_permission(&self, permission: &str) -> SaResult<bool>;
    pub fn has_permission_and(&self, perms: &[&str]) -> SaResult<bool>;
    pub fn has_permission_or(&self, perms: &[&str]) -> SaResult<bool>;
    pub fn check_permission(&self, permission: &str) -> SaResult<()>;
    pub fn check_permission_and(&self, perms: &[&str]) -> SaResult<()>;
    pub fn check_permission_or(&self, perms: &[&str]) -> SaResult<()>;

    // ─── 踢人 / 终端 ───
    pub fn get_terminal_list_by_login_id(&self, login_id: &str) -> SaResult<Vec<SaTerminalInfo>>;
    pub fn get_terminal_info(&self) -> SaResult<SaTerminalInfo>;
    pub fn get_terminal_info_by_token(&self, token_value: &str) -> SaResult<SaTerminalInfo>;

    // ─── 设备 ───
    pub fn get_login_device_type(&self) -> SaResult<String>;
    pub fn get_login_device_id(&self) -> SaResult<String>;
    pub fn is_trust_device_id(&self, user_id: &str, device_id: &str) -> bool;

    // ─── 禁用 ───
    pub fn disable(&self, login_id: &str, time: i64) -> SaResult<()>;
    pub fn is_disable(&self, login_id: &str) -> bool;
    pub fn check_disable(&self, login_id: &str) -> SaResult<()>;
    pub fn get_disable_time(&self, login_id: &str) -> i64;
    pub fn untie_disable(&self, login_id: &str) -> SaResult<()>;
    // 业务级 / level 级略

    // ─── 安全认证 ───
    pub fn open_safe(&self, safe_time: i64) -> SaResult<()>;
    pub fn open_safe_with_service(&self, service: &str, safe_time: i64) -> SaResult<()>;
    pub fn is_safe(&self) -> bool;
    pub fn check_safe(&self) -> SaResult<()>;
    pub fn close_safe(&self) -> SaResult<()>;

    // ─── 切换账号 ───
    pub fn switch_to(&self, login_id: &str) -> SaResult<()>;
    pub fn end_switch(&self) -> SaResult<()>;
    pub fn is_switch(&self) -> bool;
    pub fn get_switch_login_id(&self) -> Option<String>;

    // ─── Token 生命周期 ───
    pub fn update_last_active_to_now(&self) -> SaResult<()>;
    pub fn check_active_timeout(&self) -> SaResult<()>;
    pub fn get_token_timeout(&self) -> i64;
    pub fn renew_timeout(&self, timeout: i64) -> SaResult<()>;

    // ─── Key 拼接（对齐 Java） ───
    pub fn splicing_key_token_name(&self) -> String;
    pub fn splicing_key_token_value(&self, token_value: &str) -> String;
    pub fn splicing_key_session(&self, login_id: &str) -> String;
    pub fn splicing_key_token_session(&self, token_value: &str) -> String;
    pub fn splicing_key_last_active_time(&self, token_value: &str) -> String;

    // ... 其余按 Java StpLogic 一一实现（共 ~150 个同步方法）
}
```

### 6.4 `StpUtil`（全局门面）

```rust
// sa-token-core/src/stp/stp_util.rs
use std::sync::OnceLock;

static DEFAULT_STP_LOGIC: OnceLock<StpLogic> = OnceLock::new();
static STP_LOGIC_MAP: OnceLock<RwLock<HashMap<String, StpLogic>>> = OnceLock::new();

pub struct StpUtil;
impl StpUtil {
    pub const TYPE: &'static str = "login";

    pub fn init(login_type: &str) { ... }
    pub fn set_stp_logic(logic: StpLogic) { ... }
    pub fn stp_logic() -> &'static StpLogic { ... }
    pub fn get_login_type() -> &'static str { Self::TYPE }

    // 所有 StpLogic 方法的静态转发（同步，无 async）
    pub fn login(id: &str) -> SaResult<()> { Self::stp_logic().login(id) }
    pub fn login_with_param(id: &str, p: &SaLoginParameter) -> SaResult<()> { ... }
    pub fn logout() -> SaResult<()> { ... }
    pub fn is_login() -> bool { ... }
    pub fn check_login() -> SaResult<()> { ... }
    pub fn get_login_id() -> SaResult<String> { ... }
    pub fn get_login_id_as_string() -> SaResult<String> { ... }
    pub fn has_permission(p: &str) -> SaResult<bool> { ... }
    pub fn check_permission(p: &str) -> SaResult<()> { ... }
    pub fn has_role(r: &str) -> SaResult<bool> { ... }
    pub fn check_role(r: &str) -> SaResult<()> { ... }
    pub fn get_session() -> SaResult<SaSession> { ... }
    pub fn get_token_session() -> SaResult<SaSession> { ... }
    pub fn kickout(login_id: &str) -> SaResult<()> { ... }
    pub fn disable(login_id: &str, time: i64) -> SaResult<()> { ... }
    pub fn open_safe(time: i64) -> SaResult<()> { ... }
    pub fn check_safe() -> SaResult<()> { ... }
    // ... 所有 StpLogic 方法静态转发
}
```

> 多账号体系（`StpUserUtil`、`StpAdminUtil`）：提供 `sa_token::stp::define_stp_util!` 宏，一行定义新的 `StpUtil` 子类型。

### 6.5 `SaManager`（全局组件注册中心，`OnceLock<Arc<...>>` 模式）

```rust
// sa-token-core/src/manager.rs
use std::sync::OnceLock;
use tokio::sync::RwLock;  // 仅 listener 列表需要

pub struct SaManager;

static CONFIG: OnceLock<Arc<SaTokenConfig>> = OnceLock::new();
static DAO: OnceLock<Arc<dyn SaTokenDao>> = OnceLock::new();
static STP_INTERFACE: OnceLock<Arc<dyn StpInterface>> = OnceLock::new();
static CONTEXT: OnceLock<Arc<dyn SaTokenContext>> = OnceLock::new();
static JSON_TEMPLATE: OnceLock<Arc<dyn SaJsonTemplate>> = OnceLock::new();
static SERIALIZER_TEMPLATE: OnceLock<Arc<dyn SaSerializerTemplate>> = OnceLock::new();
static HTTP_TEMPLATE: OnceLock<Arc<dyn SaHttpTemplate>> = OnceLock::new();
static TEMP_TEMPLATE: OnceLock<Arc<dyn SaTempTemplate>> = OnceLock::new();
static SAME_TEMPLATE: OnceLock<Arc<dyn SaSameTemplate>> = OnceLock::new();
static TOTP_TEMPLATE: OnceLock<Arc<dyn SaTotpTemplate>> = OnceLock::new();
static LOG: OnceLock<Arc<dyn SaLog>> = OnceLock::new();
static STP_LOGIC_MAP: OnceLock<RwLock<HashMap<String, StpLogic>>> = OnceLock::new();
static LISTENER_LIST: OnceLock<RwLock<Vec<Arc<dyn SaTokenListener>>>> = OnceLock::new();

impl SaManager {
    pub fn set_config(c: Arc<SaTokenConfig>) { let _ = CONFIG.set(c); }
    pub fn config() -> Arc<SaTokenConfig> {
        CONFIG.get_or_init(|| Arc::new(SaTokenConfig::default())).clone()
    }
    pub fn set_sa_token_dao(dao: Arc<dyn SaTokenDao>) { let _ = DAO.set(dao); }
    pub fn sa_token_dao() -> Arc<dyn SaTokenDao> {
        DAO.get_or_init(|| Arc::new(SaTokenDaoDefaultImpl::new())).clone()
    }
    pub fn set_stp_interface(i: Arc<dyn StpInterface>) { let _ = STP_INTERFACE.set(i); }
    pub fn stp_interface() -> Arc<dyn StpInterface> {
        STP_INTERFACE.get_or_init(|| Arc::new(StpInterfaceDefaultImpl)).clone()
    }
    pub fn set_sa_token_context(ctx: Arc<dyn SaTokenContext>) { let _ = CONTEXT.set(ctx); }
    pub fn sa_token_context() -> Arc<dyn SaTokenContext> {
        CONTEXT.get_or_init(|| Arc::new(SaTokenContextDefaultImpl)).clone()
    }
    // ... 各 template getter/setter（懒加载默认实现）
    pub fn put_stp_logic(logic: StpLogic) {
        STP_LOGIC_MAP.get_or_init(|| RwLock::new(HashMap::new()))
            .write().unwrap().insert(logic.login_type().to_string(), logic);
    }
    pub fn get_stp_logic(login_type: &str) -> Option<StpLogic> {
        STP_LOGIC_MAP.get()?.read().unwrap().get(login_type).cloned()
    }
    pub fn listeners() -> &'static RwLock<Vec<Arc<dyn SaTokenListener>>> {
        LISTENER_LIST.get_or_init(|| RwLock::new(vec![Arc::new(SaTokenListenerForLog)]))
    }
}
```

> **修正**：放弃 `OnceLock<RwLock<T>>` 组合（RwLock 冗余）。OnceLock 保证"设置一次后只读"，只有需要运行期增删的 listener 列表才在 OnceLock 内套 RwLock。

### 6.6 `SaSession` + `SaTerminalInfo`

```rust
// sa-token-core/src/session/sa_session.rs
#[derive(Clone, Serialize, Deserialize)]
pub struct SaSession {
    id: String,
    r#type: String,
    login_type: String,
    login_id: serde_json::Value,
    token: String,
    create_time: i64,
    history_terminal_count: AtomicI32,
    data_map: Arc<RwLock<HashMap<String, Value>>>,
    terminal_list: Arc<RwLock<Vec<SaTerminalInfo>>>,
}

impl SaSession {
    pub const USER: &'static str = "USER";
    pub const ROLE_LIST: &'static str = "ROLE_LIST";
    pub const PERMISSION_LIST: &'static str = "PERMISSION_LIST";

    pub fn new(id: &str) -> Self;
    pub fn id(&self) -> &str;
    pub fn set_id(&self, id: String);
    // ... 链式 getter/setter
    pub fn get_terminal_list(&self) -> Vec<SaTerminalInfo>;
    pub fn add_terminal(&self, info: SaTerminalInfo);
    pub fn remove_terminal(&self, token_value: &str);
    pub fn get_terminal(&self, token_value: &str) -> Option<SaTerminalInfo>;
    pub fn get(&self, key: &str) -> Option<Value>;
    pub fn set(&self, key: &str, value: Value);
    pub fn delete(&self, key: &str);
    pub fn update(&self);          // 写回 SaTokenDao
    pub fn logout(&self);
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SaTerminalInfo {
    pub index: i32,
    pub token_value: String,
    pub device_type: String,
    pub device_id: String,
    pub extra_data: HashMap<String, Value>,
    pub create_time: i64,
    pub auth_hash: String,  // ★ 新增：用于踢下线（借鉴 axum-login 的 session_auth_hash）
}
```

### 6.7 错误处理（单一 enum）

```rust
// sa-token-core/src/exception/mod.rs
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum SaTokenException {
    #[error("未登录: {message}, login_type={login_type}")]
    NotLogin { message: String, login_type: String },

    #[error("缺少权限: {permission}, login_type={login_type}")]
    NotPermission { permission: String, login_type: String },

    #[error("缺少角色: {role}, login_type={login_type}")]
    NotRole { role: String, login_type: String },

    #[error("未通过二级认证: service={service}, login_type={login_type}")]
    NotSafe { service: String, login_type: String },

    #[error("账号已被封禁: login_id={login_id}, service={service}, 剩余={disable_time}s")]
    DisableService { login_id: String, service: String, disable_time: i64 },

    #[error("Same-Token 无效")]
    SameTokenInvalid,

    #[error("上下文无效")]
    InvalidContext,

    #[error("非 Web 上下文")]
    NotWebContext,

    #[error("防火墙拦截: {message}")]
    FirewallCheck { message: String },

    #[error("请求路径无效: {path}")]
    RequestPathInvalid { path: String },

    #[error("插件错误: {message}")]
    Plugin { message: String },

    #[error("API 被禁用")]
    ApiDisabled,

    #[error("HTTP Basic 认证失败")]
    NotHttpBasicAuth,

    #[error("HTTP Digest 认证失败")]
    NotHttpDigestAuth,

    #[error("JSON 转换失败: {message}")]
    JsonConvert { message: String },

    #[error("停止匹配")]
    StopMatch,

    #[error("TOTP 认证失败")]
    TotpAuth,

    #[error("其他错误: {message}")]
    Other { message: String },
}

pub type SaResult<T> = std::result::Result<T, SaTokenException>;
```

---

## 七、注解（proc-macro）设计

### 7.1 方案：derive + helper attribute（借鉴 easyexcel-rs）

```rust
// sa-token-derive/src/lib.rs
use proc_macro::TokenStream;

mod implementation;

/// 为 handler 函数生成鉴权检查代码
#[proc_macro_attribute]
pub fn sa_check_login(args: TokenStream, input: TokenStream) -> TokenStream { ... }

#[proc_macro_attribute]
pub fn sa_check_permission(args: TokenStream, input: TokenStream) -> TokenStream { ... }

#[proc_macro_attribute]
pub fn sa_check_role(args: TokenStream, input: TokenStream) -> TokenStream { ... }

#[proc_macro_attribute]
pub fn sa_check_safe(args: TokenStream, input: TokenStream) -> TokenStream { ... }

#[proc_macro_attribute]
pub fn sa_check_disable(args: TokenStream, input: TokenStream) -> TokenStream { ... }

#[proc_macro_attribute]
pub fn sa_check_or(args: TokenStream, input: TokenStream) -> TokenStream { ... }

#[proc_macro_attribute]
pub fn sa_check_http_basic(args: TokenStream, input: TokenStream) -> TokenStream { ... }

#[proc_macro_attribute]
pub fn sa_check_http_digest(args: TokenStream, input: TokenStream) -> TokenStream { ... }

#[proc_macro_attribute]
pub fn sa_ignore(args: TokenStream, input: TokenStream) -> TokenStream { ... }
```

### 7.2 用法

```rust
use sa_token::prelude::*;

#[sa_check_login]
fn current_login_id() -> SaResult<String> {
    StpUtil::get_login_id()
}

#[sa_check_permission("user:add")]
fn add_user() -> SaResult<()> { ... }

#[sa_check_role("admin")]
fn admin_only() -> SaResult<()> { ... }

#[sa_check_or(roles = ["admin", "super"], permissions = ["system:config"])]
fn admin_or_config_perm() -> SaResult<()> { ... }

#[sa_ignore]
fn public_api() -> SaResult<()> { ... }
```

### 7.3 宏内部实现要点（借鉴 easyexcel-rs）

- 用 `syn::Attribute::parse_nested_meta` 解析参数（而非手写解析）
- 错误处理：`syn::Error::into_compile_error` 生成 `compile_error!`
- 保留原始字面量 token 的 span，编译器错误定位到用户源码
- 复杂参数（枚举值）用「字符串查表」映射

---

## 八、Web 框架适配层

### 8.1 统一 `tower::Layer` 实现

所有 Web 框架适配统一基于 `tower::Layer`，三个 starter 各自只需薄胶水代码：

```rust
// sa-token-starter/sa-token-axum/src/lib.rs
pub struct AxumRequest(pub axum::http::Request<Body>);
impl SaRequest for AxumRequest { ... }

pub struct AxumResponse { ... }
impl SaResponse for AxumResponse { ... }

pub struct AxumStorage { ... }
impl SaStorage for AxumStorage { ... }

pub struct AxumContext { ... }
impl SaTokenContext for AxumContext { ... }

// 中间件：把 request/response 包装后注入 thread_local
pub async fn sa_token_middleware(
    State(state): State<Arc<SaTokenState>>,
    req: Request,
    next: Next,
) -> Response {
    let ctx = Arc::new(AxumContext::from_request(req));
    CURRENT_CONTEXT.with(|cell| *cell.borrow_mut() = Some(ctx));
    next.run(req).await
}

// Extractor：在 handler 里直接取登录态
pub struct CurrentLoginId(pub String);
#[async_trait]
impl FromRequestParts<S> for CurrentLoginId {
    type Rejection = SaTokenException;
    async fn from_request_parts(_parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 在 async 上下文中调用同步核心（block_in_place）
        let login_id = tokio::task::block_in_place(|| StpUtil::get_login_id())?;
        Ok(CurrentLoginId(login_id))
    }
}
```

### 8.2 路由保护：四层 API

| 层级 | API | 场景 |
|---|---|---|
| 宏（对齐 Java） | `#[sa_check_permission("user:add")]` | Sa-Token 用户无缝迁移 |
| Extractor（对齐 axum） | `RequirePermission::new("user:add")` | Rust idiomatic |
| Layer（路由级） | `SaTokenLayer::require_permission("admin")` | 批量保护路由组 |
| 策略文件（可选） | `SaPolicyLayer::from_file("policy.ron")` | 复杂 ABAC，对接 casbin/cedar |

---

## 九、关键技术决策汇总

| 决策点 | 方案 | 原因 / 借鉴 |
|---|---|---|
| 异步模型 | **核心同步 + starter async** | 借鉴 easyexcel-rs 全同步；核心无 IO 时 async 无收益 |
| 全局状态 | `OnceLock<Arc<T>>`（仅 listener 列表套 RwLock） | 借鉴 easyexcel-rs 的 file_utils 模式 |
| 请求上下文 | `thread_local!`（核心）+ `task_local!`（async starter） | 对齐 Java ThreadLocal |
| 组件注入 | `Arc<dyn Trait>` + `SaManager::set_xxx()` | 对齐 Java SaManager |
| 错误处理 | `thiserror` 单一 enum + `SaResult<T>` | 借鉴 easyexcel-rs ExcelError，实现 Clone+Eq 用于测试 |
| 序列化 | `serde` + `serde_json::Value` | 对齐 Java Object |
| JSON 模板 | 默认 `serde_json`，trait 可替换 | 对齐 Java SaJsonTemplate |
| 链式 setter | builder 返回 `&mut Self` | 对齐 Java builder |
| 注解 AOP | `#[proc_macro_attribute]` + 可选 `#[derive]` | Rust 无反射，编译期生成 |
| 登录 ID 类型 | 内部 `String`，提供 `as_i32/as_i64` | 简化泛型 |
| 多账号体系 | `define_stp_util!` 宏 | 对齐 Java StpUserUtil |
| 文件命名 | snake_case 文件 + PascalCase 结构体 | Rust 习惯 + Java 类名对齐 |
| 模块路径 | `sa_token_core::stp::StpLogic` | 对齐 Java `cn.dev33.satoken.stp.StpLogic` |
| Edition | **2024** | 借鉴 easyexcel-rs |
| Resolver | **3** | 借鉴 easyexcel-rs |
| Lint | `unsafe_code=forbid` + clippy `all+pedantic` | 借鉴 easyexcel-rs |
| 踢下线 | `auth_hash` 字段 + Redis Pub/Sub 推送 | 借鉴 axum-login session_auth_hash |
| 权限引擎 | core 只留 trait，默认空实现；可插拔 casbin/cedar | Rust 生态已有更强方案 |
| Redis 客户端 | `fred`（而非 `redis`） | 集群/Pipeline/Pub/Sub 更强 |
| 跨服务认证 | same-token + 可选 biscuit | 兼容 Java + Rust 增强 |

---

## 十、分阶段实施计划

### **Phase 1：MVP 核心** ⭐（2~3 周）

**目标**：跑通 `StpUtil::login("10001")` → `is_login` → `get_login_id` → `logout`。

**范围**：
- ✅ `sa-token-core` 完整骨架（所有目录创建，部分文件先留 stub）
- ✅ 完整实现：
  - `config/` - SaTokenConfig / SaCookieConfig（serde + 默认值）
  - `context/` - 全部 trait + `SaHolder` + `SaTokenContextForThreadLocal`（基于 `thread_local!`）+ `context/model/*`
  - `dao/` - SaTokenDao trait + `SaTokenDaoDefaultImpl`（Memory + TTL）
  - `exception/` - 单一 enum（thiserror）
  - `session/` - SaSession / SaTerminalInfo
  - `stp/StpLogic` - 登录、登出、Token、会话、登录状态、踢人、顶替（约 80 个同步方法）
  - `stp/StpUtil` - 静态门面
  - `stp/StpInterface` - trait + 默认实现
  - `stp/SaTokenInfo` / `stp/parameter/*`
  - `listener/` - SaTokenListener + SaTokenEventCenter
  - `log/` - SaLog + SaLogForConsole（tracing 适配）
  - `manager.rs` - SaManager
  - `util/` - SaFoxUtil / SaResult / SaHexUtil / StrFormatter
  - `json/` - SaJsonTemplate trait + 默认（serde_json）
  - `serializer/` - SaSerializerTemplate trait + 默认
- ✅ `sa-token-context-mock` - 完整 Mock 上下文（测试用）
- ✅ `sa-token` facade crate
- ✅ `sa-token-test` - 基础集成测试（覆盖 `getOrCreateLoginSession` 流）
- ✅ `docs/migration/object-method-matrix.md` - 初版方法对照表

**验收标准**：

```rust
#[test]
fn test_login_and_is_login() {
    SaManager::set_config(Arc::new(SaTokenConfig::default()));
    SaManager::set_sa_token_dao(Arc::new(SaTokenDaoDefaultImpl::new()));
    SaTokenContextMockUtil::set_mock_context();

    StpUtil::login("10001").unwrap();
    assert!(StpUtil::is_login());
    assert_eq!(StpUtil::get_login_id().unwrap(), "10001");
    StpUtil::logout().unwrap();
    assert!(!StpUtil::is_login());
}
```

### **Phase 2：权限/角色/注解宏**（1~2 周）

- ✅ 完善 `StpLogic` 剩余方法：权限、角色、禁用（含业务级/等级）、安全认证、切换账号、设备、Token 续签/活跃超时、搜索
- ✅ `strategy/` - SaStrategy / SaAnnotationStrategy / SaFirewallStrategy（防火墙 9 个 Hook）
- ✅ `sa-token-derive` crate - 9 个 proc-macro 注解
- ✅ `httpauth/` - SaHttpBasicTemplate / SaHttpDigestTemplate
- ✅ `same/` / `temp/` / `router/` / `filter/` / `plugin/` 等剩余子模块

### **Phase 3：Web 框架适配** ⭐（1~2 周）

按优先级：
- ✅ `sa-token-starter/sa-token-axum/` - Layer + Extractor + `FromRequest` 实现 + 完整示例
- ✅ `sa-token-starter/sa-token-actix-web/` - 中间件 + FromRequest
- ✅ `sa-token-starter/sa-token-salvo/` - Depot 中间件

### **Phase 4：存储扩展**（1 周）

- ✅ `sa-token-dao-redis/` - 基于 `fred` crate 的 async 实现
- ✅ `sa-token-dao-moka/` - 基于 `moka` 的高性能进程内缓存（类似 Caffeine）

### **Phase 5：插件生态**（3~4 周）

- ✅ `sa-token-plugin/sa-token-jwt/` - JWT Style Token（`jsonwebtoken` crate）
- ✅ `sa-token-plugin/sa-token-sign/` - API 参数签名校验
- ✅ `sa-token-plugin/sa-token-oauth2/` - OAuth2 完整实现
- ✅ `sa-token-plugin/sa-token-sso/` - SSO 单点登录
- ✅ `sa-token-plugin/sa-token-apikey/` - API Key 管理

### **Phase 6：示例与文档**（持续）

- ✅ `sa-token-demo/sa-token-demo-axum/` - 完整功能演示
- ✅ `sa-token-demo/sa-token-demo-actix-web/`
- ✅ `sa-token-demo/sa-token-demo-salvo/`
- ✅ 完善所有 `docs/migration/` 审计文档
- ✅ Golden 测试 + Parity 测试
- ✅ CHANGELOG.md / README.md / crate 级文档测试

---

## 十一、依赖清单

### 11.1 Workspace 根 Cargo.toml

```toml
[workspace]
members = [
    "crates/sa-token",
    "crates/sa-token-core",
    "crates/sa-token-derive",
    "crates/sa-token-context-mock",
    "crates/sa-token-dao-memory",
    "crates/sa-token-dao-redis",
    "crates/sa-token-dao-moka",
    "crates/sa-token-starter/sa-token-axum",
    "crates/sa-token-starter/sa-token-actix-web",
    "crates/sa-token-starter/sa-token-salvo",
    "crates/sa-token-plugin/sa-token-jwt",
    "crates/sa-token-plugin/sa-token-sign",
    "crates/sa-token-plugin/sa-token-oauth2",
    "crates/sa-token-plugin/sa-token-sso",
    "crates/sa-token-plugin/sa-token-apikey",
    "crates/sa-token-test",
]
resolver = "3"

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.88"
license = "Apache-2.0"
repository = "https://github.com/dromara/sa-token-rs"

[workspace.dependencies]
# 本 workspace crate
sa-token = { path = "crates/sa-token" }
sa-token-core = { path = "crates/sa-token-core" }
sa-token-derive = { path = "crates/sa-token-derive" }
sa-token-context-mock = { path = "crates/sa-token-context-mock" }
sa-token-dao-memory = { path = "crates/sa-token-dao-memory" }
sa-token-dao-redis = { path = "crates/sa-token-dao-redis" }

# 核心依赖
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tracing = "0.1"
chrono = "0.4"
uuid = { version = "1", features = ["v4"] }
base64 = "0.22"
hex = "0.4"
sha2 = "0.10"
hmac = "0.12"
md-5 = "0.10"
rand = "0.8"
regex = "1"
url = "2"
async-trait = "0.1"
proc-macro2 = "1"
quote = "1"
syn = { version = "2", features = ["full"] }
proc-macro-crate = "3"

# 异步运行时（仅 starter/plugin 用）
tokio = { version = "1", features = ["full"] }

# 存储后端
fred = "9"          # Redis 客户端
moka = { version = "0.12", features = ["future"] }

# Web 框架
axum = "0.7"
actix-web = "4"
salvo = "0.68"
tower = "0.5"

# 可选插件
jsonwebtoken = "9"
casbin = "2"

[workspace.lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"

[workspace.lints.clippy]
all = "warn"
pedantic = "warn"
```

### 11.2 sa-token-core Cargo.toml（核心，无 async）

```toml
[package]
name = "sa-token-core"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
description = "Core types and extension points for sa-token-rs"

[dependencies]
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
tracing.workspace = true
chrono.workspace = true
uuid.workspace = true
base64.workspace = true
hex.workspace = true
sha2.workspace = true
hmac.workspace = true
md-5.workspace = true
rand.workspace = true
regex.workspace = true
url.workspace = true

[lints]
workspace = true
```

### 11.3 sa-token-derive Cargo.toml（proc-macro）

```toml
[package]
name = "sa-token-derive"
version.workspace = true
edition.workspace = true
proc-macro = true
description = "Derive macros for sa-token-rs"

[lib]
proc-macro = true

[dependencies]
proc-macro2.workspace = true
quote.workspace = true
syn.workspace = true
proc-macro-crate.workspace = true

[dev-dependencies]
sa-token-core.workspace = true

[lints]
workspace = true
```

---

## 十二、测试体系

**四层测试，完全借鉴 easyexcel-rs**：

### 12.1 测试分层

| 层级 | 文件位置 | 数量目标 | 说明 |
|---|---|---|---|
| 单元测试 | `src/tests.rs` + `src/missing_tests.rs` | 每 crate | 追踪未移植的 Java 测试 |
| 1:1 方法测试 | `tests/1to1/*_1to1_tests.rs` | ~150 | 每个 Java StpLogic 方法对应一个 Rust 测试 |
| Golden 测试 | `tests/golden/*.expected.json` | ~50 | Java Sa-Token 跑出 token/session 快照 → Rust 字节级比对 |
| Parity 测试 | `tests/parity/*_parity_tests.rs` | ~100 | 端到端行为对等 |

### 12.2 Golden 测试生成

```bash
# scripts/java-golden-export/ 是一个 Java Maven 子项目
# 跑 Java Sa-Token 生成 token、session、权限快照
./scripts/export-java-golden.sh
# 输出 tests/golden/*.expected.json

# Rust 侧逐字节比对
cargo test --test java_golden_tests
```

### 12.3 Phase 1 测试示例

```rust
// sa-token-test/tests/login_test.rs
use sa_token::prelude::*;
use sa_token_context_mock::SaTokenContextMockUtil;
use std::sync::Arc;

#[test]
fn test_login_and_is_login() {
    // 初始化全局组件
    SaManager::set_config(Arc::new(SaTokenConfig::default()));
    SaManager::set_sa_token_dao(Arc::new(SaTokenDaoDefaultImpl::new()));
    SaTokenContextMockUtil::set_mock_context();

    // 登录
    StpUtil::login("10001").unwrap();
    assert!(StpUtil::is_login());
    assert_eq!(StpUtil::get_login_id().unwrap(), "10001");

    // 登出
    StpUtil::logout().unwrap();
    assert!(!StpUtil::is_login());
}

#[test]
fn test_login_with_device() {
    SaManager::set_config(Arc::new(SaTokenConfig::default()));
    SaManager::set_sa_token_dao(Arc::new(SaTokenDaoDefaultImpl::new()));
    SaTokenContextMockUtil::set_mock_context();

    let param = SaLoginParameter::create()
        .set_device_type("PC")
        .set_device_id("device-001");

    StpUtil::login_with_param("10001", &param).unwrap();
    assert_eq!(StpUtil::get_login_device_type().unwrap(), "PC");
}

#[test]
fn test_kickout() {
    SaManager::set_config(Arc::new(SaTokenConfig::default()));
    SaManager::set_sa_token_dao(Arc::new(SaTokenDaoDefaultImpl::new()));
    SaTokenContextMockUtil::set_mock_context();

    StpUtil::login("10001").unwrap();
    assert!(StpUtil::is_login());

    StpUtil::kickout("10001").unwrap();
    assert!(!StpUtil::is_login());
}
```

---

## 十三、迁移文档体系

完全借鉴 easyexcel-rs 的文档组织：

```text
docs/
├── IMPLEMENTATION_PLAN.md                # 本文档（最终实施计划）
├── ARCHITECTURE.md                       # 架构总览 + Core Traits Java 映射表
├── GUIDE.md                              # 使用指南
├── compatibility.md                      # 兼容性说明
├── ecosystem-roadmap.md                  # 生态路线图
├── benchmarks.md                         # 性能基准
└── migration/
    ├── MIGRATION_STATUS.md               # 分 Phase 迁移进度（Phase 0/1/2/3/4/5/6）
    ├── object-method-matrix.md           # Java 对象 × 方法矩阵
    ├── CODEGRAPH_METHOD_MAP.md           # 方法级 1:1 审计
    ├── java-tree-full.md                 # Java Sa-Token 完整目录树
    ├── rust-tree-full.md                 # Rust Sa-Token-Rs 完整目录树
    ├── project-tree-diff.md              # 两侧目录 diff
    ├── codegraph-gap-audit.md            # 缺口审计
    └── TEST_AUDIT_REPORT.md              # 测试审计（golden + parity）
```

---

## 十四、风险与缓解

| 风险 | 缓解 |
|---|---|
| `StpLogic` 方法极多（~150 个）翻译量大 | Phase 1 先做 MVP 最短路径（登录/会话/Token），其余迭代推进 |
| 核心同步 + starter async 的桥接复杂（`block_in_place`） | 提供明确的 starter 文档；Redis DAO 单独 async 实现 |
| `thread_local!` 在 async 上下文行为需注意 | 核心同步无此问题；starter 用 `tokio::task::block_in_place` 显式桥接 |
| 全局状态测试隔离困难 | 提供 `SaManager::reset()` 测试辅助方法；每个 `#[test]` 独立 runtime |
| `Object` 在 Rust 中无对应 | 统一 `serde_json::Value` + 泛型 `FromValue`；登录 ID 内部统一 `String` |
| Java 反射注解 → Rust 宏心智模型差异 | 提供 `docs/migration/object-method-matrix.md` 详尽对照表 |
| Golden 测试依赖 Java 环境 | `scripts/java-golden-export/` 独立 Maven 子项目；CI 可选跳过 |

---

## 十五、Java ↔ Rust 完整文件对应关系

| Java 源文件 | Rust 源文件 |
|---|---|
| `cn/dev33/satoken/SaManager.java` | `sa-token-core/src/manager.rs` |
| `cn/dev33/satoken/stp/StpLogic.java` | `sa-token-core/src/stp/stp_logic.rs` |
| `cn/dev33/satoken/stp/StpUtil.java` | `sa-token-core/src/stp/stp_util.rs` |
| `cn/dev33/satoken/stp/StpInterface.java` | `sa-token-core/src/stp/stp_interface.rs` |
| `cn/dev33/satoken/stp/SaTokenInfo.java` | `sa-token-core/src/stp/sa_token_info.rs` |
| `cn/dev33/satoken/stp/parameter/SaLoginParameter.java` | `sa-token-core/src/stp/parameter/sa_login_parameter.rs` |
| `cn/dev33/satoken/stp/parameter/SaLogoutParameter.java` | `sa-token-core/src/stp/parameter/sa_logout_parameter.rs` |
| `cn/dev33/satoken/stp/parameter/enums/SaLogoutMode.java` | `sa-token-core/src/stp/parameter/enums/sa_logout_mode.rs` |
| `cn/dev33/satoken/stp/parameter/enums/SaLogoutRange.java` | `sa-token-core/src/stp/parameter/enums/sa_logout_range.rs` |
| `cn/dev33/satoken/config/SaTokenConfig.java` | `sa-token-core/src/config/sa_token_config.rs` |
| `cn/dev33/satoken/config/SaCookieConfig.java` | `sa-token-core/src/config/sa_cookie_config.rs` |
| `cn/dev33/satoken/config/SaTokenConfigFactory.java` | `sa-token-core/src/config/sa_token_config_factory.rs` |
| `cn/dev33/satoken/context/SaHolder.java` | `sa-token-core/src/context/sa_holder.rs` |
| `cn/dev33/satoken/context/SaTokenContext.java` | `sa-token-core/src/context/sa_token_context.rs` |
| `cn/dev33/satoken/context/SaTokenContextDefaultImpl.java` | `sa-token-core/src/context/sa_token_context_default_impl.rs` |
| `cn/dev33/satoken/context/SaTokenContextForThreadLocal.java` | `sa-token-core/src/context/sa_token_context_for_thread_local.rs` |
| `cn/dev33/satoken/context/model/SaRequest.java` | `sa-token-core/src/context/model/sa_request.rs` |
| `cn/dev33/satoken/context/model/SaResponse.java` | `sa-token-core/src/context/model/sa_response.rs` |
| `cn/dev33/satoken/context/model/SaStorage.java` | `sa-token-core/src/context/model/sa_storage.rs` |
| `cn/dev33/satoken/context/model/SaCookie.java` | `sa-token-core/src/context/model/sa_cookie.rs` |
| `cn/dev33/satoken/dao/SaTokenDao.java` | `sa-token-core/src/dao/sa_token_dao.rs` |
| `cn/dev33/satoken/dao/SaTokenDaoDefaultImpl.java` | `sa-token-core/src/dao/sa_token_dao_default_impl.rs` |
| `cn/dev33/satoken/session/SaSession.java` | `sa-token-core/src/session/sa_session.rs` |
| `cn/dev33/satoken/session/SaTerminalInfo.java` | `sa-token-core/src/session/sa_terminal_info.rs` |
| `cn/dev33/satoken/session/SaSessionCustomUtil.java` | `sa-token-core/src/session/sa_session_custom_util.rs` |
| `cn/dev33/satoken/listener/SaTokenListener.java` | `sa-token-core/src/listener/sa_token_listener.rs` |
| `cn/dev33/satoken/listener/SaTokenEventCenter.java` | `sa-token-core/src/listener/sa_token_event_center.rs` |
| `cn/dev33/satoken/listener/SaTokenListenerForLog.java` | `sa-token-core/src/listener/sa_token_listener_for_log.rs` |
| `cn/dev33/satoken/log/SaLog.java` | `sa-token-core/src/log/sa_log.rs` |
| `cn/dev33/satoken/log/SaLogForConsole.java` | `sa-token-core/src/log/sa_log_for_console.rs` |
| `cn/dev33/satoken/json/SaJsonTemplate.java` | `sa-token-core/src/json/sa_json_template.rs` |
| `cn/dev33/satoken/json/SaJsonTemplateDefaultImpl.java` | `sa-token-core/src/json/sa_json_template_default_impl.rs` |
| `cn/dev33/satoken/serializer/SaSerializerTemplate.java` | `sa-token-core/src/serializer/sa_serializer_template.rs` |
| `cn/dev33/satoken/util/SaFoxUtil.java` | `sa-token-core/src/util/sa_fox_util.rs` |
| `cn/dev33/satoken/util/SaResult.java` | `sa-token-core/src/util/sa_result.rs` |
| `cn/dev33/satoken/util/SaTokenConsts.java` | `sa-token-core/src/util/sa_token_consts.rs` |
| `cn/dev33/satoken/exception/SaTokenException.java` | `sa-token-core/src/exception/mod.rs`（单一 enum） |
| `cn/dev33/satoken/exception/NotLoginException.java` | 折叠为 `SaTokenException::NotLogin` variant |
| `cn/dev33/satoken/exception/NotPermissionException.java` | 折叠为 `SaTokenException::NotPermission` variant |
| `cn/dev33/satoken/exception/NotRoleException.java` | 折叠为 `SaTokenException::NotRole` variant |
| `cn/dev33/satoken/router/SaRouter.java` | `sa-token-core/src/router/sa_router.rs` |
| `cn/dev33/satoken/router/SaHttpMethod.java` | `sa-token-core/src/context/model/sa_http_method.rs` |
| `cn/dev33/satoken/annotation/SaMode.java` | `sa-token-core/src/annotation/mod.rs` |
| `cn/dev33/satoken/annotation/handler/SaCheckLoginHandler.java` | `sa-token-derive/src/lib.rs`（proc-macro `sa_check_login`） |
| `cn/dev33/satoken/annotation/handler/SaCheckPermissionHandler.java` | `sa-token-derive/src/lib.rs`（proc-macro `sa_check_permission`） |
| `cn/dev33/satoken/annotation/handler/SaCheckRoleHandler.java` | `sa-token-derive/src/lib.rs`（proc-macro `sa_check_role`） |
| `cn/dev33/satoken/strategy/SaStrategy.java` | `sa-token-core/src/strategy/sa_strategy.rs` |
| `cn/dev33/satoken/strategy/SaFirewallStrategy.java` | `sa-token-core/src/strategy/sa_firewall_strategy.rs` |
| `cn/dev33/satoken/httpauth/basic/SaHttpBasicTemplate.java` | `sa-token-core/src/httpauth/basic/sa_http_basic_template.rs` |
| `cn/dev33/satoken/same/SaSameTemplate.java` | `sa-token-core/src/same/sa_same_template.rs` |
| `cn/dev33/satoken/temp/SaTempTemplate.java` | `sa-token-core/src/temp/sa_temp_template.rs` |
| `cn/dev33/satoken/secure/SaSecureUtil.java` | `sa-token-core/src/secure/sa_secure_util.rs` |
| `cn/dev33/satoken/secure/BCrypt.java` | `sa-token-core/src/secure/b_crypt.rs` |
| `cn/dev33/satoken/plugin/sa-token-oauth2/...` | `sa-token-plugin/sa-token-oauth2/src/...` |
| `cn/dev33/satoken/plugin/sa-token-sso/...` | `sa-token-plugin/sa-token-sso/src/...` |
| `cn/dev33/satoken/plugin/sa-token-jwt/...` | `sa-token-plugin/sa-token-jwt/src/...` |
| `cn/dev33/satoken/plugin/sa-token-sign/...` | `sa-token-plugin/sa-token-sign/src/...` |
| `cn/dev33/satoken/starter/sa-token-spring-boot-starter/...` | `sa-token-starter/sa-token-axum/src/...`（或其他 starter） |

---

## 十六、Phase 1 立即执行清单

批准本计划后，Phase 1 的具体执行步骤如下：

### Step 1：创建 workspace（半天）

```bash
mkdir -p /Users/wandl/workspaces/workspace-github/sa-token-rs/crates
cd /Users/wandl/workspaces/workspace-github/sa-token-rs
# 创建根 Cargo.toml（按第 11.1 节）
# 创建 README.md / CHANGELOG.md / .gitignore
```

### Step 2：创建 sa-token-core 骨架（半天）

```bash
mkdir -p crates/sa-token-core/src/{config,context/model,dao,error,exception,session,stp/parameter/enums,listener,log,json,util,strategy,secure,router,...}
# 创建所有 mod.rs（空文件或基本导出）
# 创建 lib.rs（re-export）
```

### Step 3：实现"最短路径跑通 login 流"的核心文件（1 周）

按依赖顺序：

1. `util/sa_fox_util.rs` - 基础工具（随机串、时间、URL）
2. `util/sa_result.rs` - 统一返回结构
3. `exception/mod.rs` - SaTokenException enum
4. `config/sa_token_config.rs` - 配置结构体
5. `config/sa_cookie_config.rs` - Cookie 配置
6. `context/model/{sa_request,sa_response,sa_storage,sa_cookie}.rs` - 上下文模型 trait
7. `context/sa_token_context.rs` - 上下文 trait
8. `context/sa_token_context_for_thread_local.rs` - thread_local 实现
9. `context/sa_holder.rs` - 上下文门面
10. `dao/sa_token_dao.rs` - 存储 trait
11. `dao/sa_token_dao_default_impl.rs` - Memory 默认实现
12. `session/sa_terminal_info.rs` - 终端信息
13. `session/sa_session.rs` - 会话
14. `stp/parameter/{sa_login_parameter,sa_logout_parameter}.rs` - 参数对象
15. `stp/stp_interface.rs` - 权限数据源 trait
16. `stp/sa_token_info.rs` - Token 信息
17. `stp/stp_logic.rs` - 核心逻辑（login / logout / is_login / get_login_id / get_session）
18. `stp/stp_util.rs` - 静态门面
19. `listener/{sa_token_listener,sa_token_event_center}.rs` - 事件
20. `log/{sa_log,sa_log_for_console}.rs` - 日志
21. `json/{sa_json_template,sa_json_template_default_impl}.rs` - JSON
22. `manager.rs` - 全局组件管理

### Step 4：创建 sa-token-context-mock crate（半天）

```rust
// crates/sa-token-context-mock/src/lib.rs
pub struct SaTokenContextMockUtil;
impl SaTokenContextMockUtil {
    pub fn set_mock_context() { ... }
}
```

### Step 5：创建 sa-token facade crate（半天）

```rust
// crates/sa-token/src/lib.rs
pub use sa_token_core::*;
pub use sa_token_context_mock::*;

pub mod prelude {
    pub use crate::*;
}
```

### Step 6：跑通测试（半天）

```bash
cd crates/sa-token-test
cargo test --test login_test
# 期望：3 个测试全绿
```

### Step 7：输出初版迁移文档（半天）

- `docs/migration/object-method-matrix.md` - Phase 1 已实现方法的对照表
- `docs/ARCHITECTURE.md` - 架构总览
- `docs/GUIDE.md` - 5 分钟快速开始

---

## 附录 A：与 Sa-Token Java 的关键差异速查

| 差异点 | Java 做法 | Rust 做法 |
|---|---|---|
| 依赖注入 | Spring Boot `@Bean` | 手动构造 + `Arc` 共享 |
| 反射注解 | Spring AOP | proc-macro 编译期生成 |
| 类型擦除 | `Object` 通用 | `String` 或 `serde_json::Value` + 泛型 |
| 上下文隔离 | `ThreadLocal` | `thread_local!`（核心）/ `task_local!`（async） |
| 异步 | 同步为主 | 核心同步 + starter async |
| 错误处理 | 抛出异常 | `Result<T, SaTokenException>` |
| 配置读取 | 反射注入字段 | `Deserialize` 从 YAML/TOML/ENV |
| 异常层次 | 20+ 异常类 | 单一 enum + variant |
| 元数据 | 运行时反射 HashMap | `&'static [T]` 编译期常量 |
| 踢下线 | 维护 token 列表主动删除 | `auth_hash` + Redis Pub/Sub |

## 附录 B：参考项目

- **Sa-Token Java**：`/Users/wandl/workspaces/workspace-github/Sa-Token`（基线）
- **easyexcel-rs**：`/Users/wandl/workspaces/workspace-github/easyexcel-rs`（Java→Rust 移植参考）
- **axum-login**：https://github.com/maxcountryman/axum-login（`session_auth_hash` 借鉴）
- **casbin-rs**：https://github.com/casbin/casbin-rs（权限引擎适配）
- **fred**：https://github.com/aembke/fred.rs（Redis 客户端）

---

**文档结束**

本计划已整合：
1. ✅ Sa-Token Java 完整代码结构分析（基于 code-review-graph）
2. ✅ Sa-Token 核心 API 摘要（StpLogic/StpUtil/SaManager/SaTokenDao/SaSession/SaTerminalInfo/SaTokenListener）
3. ✅ Rust 生态现状调研（axum-login/casbin-rs/jsonwebtoken 等组合方案）
4. ✅ easyexcel-rs 移植经验借鉴（同步核心/OnceLock 模式/derive 注解/单一 enum 错误/测试分层）
5. ✅ 五大核心设计修正（同步核心、OnceLock<Arc<>>、derive+helper attr、单一 enum、四层测试）

**下一步**：批准后立即执行 Phase 1 Step 1。
