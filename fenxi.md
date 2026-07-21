基于刚才对 Sa-Token 的深度解析，如果要 **一比一在 Rust 生态中实现 Sa-Token-Rs**，核心思路是：**保留 Sa-Token 的 API 语义与模块边界，把 Java 的“类+接口+Spring Bean 注入”映射为 Rust 的“trait + 泛型参数 + 生态适配器”**。下面是我建议的整体架构设计。

---

## 1. 设计目标

| 目标 | 说明 |
|---|---|
| API 语义一致 | `StpUtil::login(id)`、`StpUtil::is_login()`、`StpUtil::has_permission(...)` 等命名与 Sa-Token 尽量对齐 |
| 框架无关核心 | `sa-token-rs-core` 不依赖任何 Web 框架 |
| 多 Web 框架适配 | 提供 `axum`、`actix-web`、`salvo`、`rocket` 等适配层 |
| 存储可插拔 | 默认 Memory，可选 Redis（`redis`、`fred`）、分布式缓存 |
| 异步优先 | 核心接口基于 `async`/`await`，但提供同步兼容 API |
| 无反射 | 用宏/derive 替代 Spring AOP 注解（`#[sa_check_login]` 等） |
| 类型安全 | 利用 Rust 类型系统消除 `Object` 转换和 NPE 风险 |

---

## 2. Workspace 模块结构

```text
sa-token-rs/                          # 根 workspace
├── Cargo.toml
├── sa-token-rs-core/                 # 核心：会话、登录、权限、Token
├── sa-token-rs-context/              # 上下文抽象：Request/Response/Storage
├── sa-token-rs-dao/                  # 存储接口与默认 Memory 实现
├── sa-token-rs-dao-redis/            # Redis 适配
├── sa-token-rs-annotation/           # 鉴权注解宏（proc-macro）
├── sa-token-rs-plugin-oauth2/        # OAuth2 插件
├── sa-token-rs-plugin-sso/            # SSO 插件
├── sa-token-rs-plugin-sign/           # 参数签名插件
├── sa-token-rs-plugin-apikey/         # API Key 插件
├── sa-token-rs-axum/                 # axum 适配层
├── sa-token-rs-actix-web/             # actix-web 适配层
├── sa-token-rs-salvo/                 # salvo 适配层
├── sa-token-rs-spring-boot-starter/   # 如果需要给 Spring Cloud Rust 端用（可选）
├── sa-token-rs-test/                 # 集成测试
└── sa-token-rs-demo-axum/             # 官方示例
```

> 对应 Sa-Token 的 `sa-token-core` / `sa-token-context` / `sa-token-dao` / `sa-token-plugin-*` / `sa-token-starter-*` 模块。

---

## 3. 核心类型映射

| Sa-Token Java | Sa-Token-Rs Rust | 说明 |
|---|---|---|
| `StpLogic` | `StpLogic<Cfg, Dao, Ctx>` | 泛型结构体，核心逻辑 |
| `StpUtil` | `StpUtil` | 全局静态门面（通过 `OnceLock` / `RwLock` 持有默认 `StpLogic`） |
| `SaManager` | `SaManager` | 全局组件注册中心 |
| `SaTokenConfig` | `SaTokenConfig` | 配置结构体 |
| `SaTokenContext` | `SaTokenContext` trait | 上下文抽象 |
| `SaRequest` / `SaResponse` / `SaStorage` | `SaRequest` / `SaResponse` / `SaStorage` traits | 请求/响应/存储抽象 |
| `SaTokenDao` | `SaTokenDao` trait | 数据持久化 |
| `SaSession` | `SaSession` | 会话模型 |
| `SaTerminalInfo` | `SaTerminalInfo` | 终端信息 |
| `SaLoginParameter` | `SaLoginParameter` | 登录参数 |
| `SaLogoutParameter` | `SaLogoutParameter` | 登出参数 |
| `SaTokenEventCenter` | `SaTokenEventCenter` / `EventBus` | 事件中心 |
| `SaAnnotationHandler` | `SaAuthChecker` | 注解检查器 |
| `@SaCheckLogin` 等 | `#[sa_check_login]` 等 proc-macro | 宏注解 |

---

## 4. 核心 trait 设计

### 4.1 存储层：`SaTokenDao`

```rust
// sa-token-rs-dao/src/lib.rs
#[async_trait]
pub trait SaTokenDao: Send + Sync + 'static {
    async fn get(&self, key: &str) -> Option<String>;
    async fn set(&self, key: &str, value: &str, timeout: i64);
    async fn delete(&self, key: &str);
    async fn get_session(&self, session_id: &str) -> Option<SaSession>;
    async fn set_session(&self, session_id: &str, session: &SaSession, timeout: i64);
    async fn update_session(&self, session_id: &str, session: &SaSession);
    async fn get_session_timeout(&self, session_id: &str) -> i64;
    async fn update_session_timeout(&self, session_id: &str, timeout: i64);
    // ...
}
```

默认提供 `MemoryDao`，Redis 通过 `sa-token-rs-dao-redis` 实现。

### 4.2 上下文抽象：`SaTokenContext`

```rust
// sa-token-rs-context/src/lib.rs
#[async_trait]
pub trait SaTokenContext: Send + Sync + 'static {
    fn request(&self) -> &dyn SaRequest;
    fn response(&self) -> &dyn SaResponse;
    fn storage(&self) -> &dyn SaStorage;
    fn is_valid(&self) -> bool;
}

pub trait SaRequest {
    fn get_param(&self, name: &str) -> Option<String>;
    fn get_header(&self, name: &str) -> Option<String>;
    fn get_cookie_value(&self, name: &str) -> Option<String>;
    fn get_request_path(&self) -> String;
    fn is_path(&self, path: &str) -> bool;
    fn is_ajax(&self) -> bool;
    fn forward(&self, path: &str);
}

pub trait SaResponse {
    fn delete_cookie(&self, name: &str);
    fn add_cookie(&self, cookie: &SaCookie);
    fn set_status(&self, status: u16);
    fn set_header(&self, name: &str, value: &str);
    fn redirect(&self, url: &str);
}
```

### 4.3 核心逻辑：`StpLogic`

```rust
// sa-token-rs-core/src/stp_logic.rs
pub struct StpLogic<Cfg, Dao, Ctx, CtxGetter>
where
    Cfg: SaTokenConfigAccess,
    Dao: SaTokenDao,
    Ctx: SaTokenContext,
    CtxGetter: ContextGetter<Ctx>,
{
    pub login_type: String,
    pub config: Arc<Cfg>,
    pub dao: Arc<Dao>,
    pub context_getter: CtxGetter,
    // 其他可替换组件：StpInterface, PathMatcher, Listener, etc.
}

impl<Cfg, Dao, Ctx, CtxGetter> StpLogic<Cfg, Dao, Ctx, CtxGetter> {
    pub async fn login(&self, login_id: &str) -> SaTokenInfo { ... }
    pub async fn login_with_param(&self, param: &SaLoginParameter) -> SaTokenInfo { ... }
    pub async fn logout(&self) { ... }
    pub async fn is_login(&self) -> bool { ... }
    pub async fn get_login_id(&self) -> Option<String> { ... }
    pub async fn has_permission(&self, permission: &str) -> bool { ... }
    pub async fn has_role(&self, role: &str) -> bool { ... }
    pub async fn get_session(&self) -> SaSession { ... }
    pub async fn get_token_session(&self) -> SaSession { ... }
    // ...
}
```

> 注意：为了避免泛型爆炸，也可以提供一个 `StpLogicDyn` 类型擦除版本，用 `Arc<dyn SaTokenDao>` 等。

---

## 5. 全局门面：`StpUtil`

Sa-Token 的 `StpUtil` 是静态方法门面，Rust 中可通过 **全局静态** + **类型擦除** 实现：

```rust
// sa-token-rs-core/src/stp_util.rs
use std::sync::OnceLock;

static DEFAULT_STP_LOGIC: OnceLock<Arc<dyn StpLogicTrait>> = OnceLock::new();

pub struct StpUtil;

impl StpUtil {
    pub fn init(logic: Arc<dyn StpLogicTrait>) {
        let _ = DEFAULT_STP_LOGIC.set(logic);
    }

    pub async fn login(login_id: &str) -> SaTokenInfo {
        DEFAULT_STP_LOGIC.get().unwrap().login(login_id).await
    }

    pub async fn is_login() -> bool {
        DEFAULT_STP_LOGIC.get().unwrap().is_login().await
    }

    pub async fn has_permission(permission: &str) -> bool {
        DEFAULT_STP_LOGIC.get().unwrap().has_permission(permission).await
    }
    // ...
}
```

多账号体系（如 `StpUserUtil`）可以通过不同的 `login_type` 注册多个全局 `StpLogic` 实例实现。

---

## 6. 上下文隔离方案

Sa-Token 用 `ThreadLocal` 实现请求上下文隔离。Rust 中：

- **单线程异步运行时（tokio）**：使用 `tokio::task_local!` 或 `async` 参数传递。
- **更 Rust  idiomatic 的做法**：不依赖全局 ThreadLocal，而是把 `StpLogic` 注入到 Handler 的参数中（如 `axum::Extension<Arc<StpLogic>>`）。

但为了兼容 Sa-Token 的 API 风格（`StpUtil::get_login_id()` 随处可用），推荐：

```rust
// 使用 task-local 存储当前请求的上下文
tokio::task_local! {
    static CURRENT_CONTEXT: Arc<dyn SaTokenContext>;
}
```

或更通用的 `thread_local!` 同步版本：

```rust
thread_local! {
    static CURRENT_CONTEXT: RefCell<Option<Arc<dyn SaTokenContext>>> = const { RefCell::new(None) };
}
```

---

## 7. 鉴权注解宏

Sa-Token 的 `@SaCheckLogin`、`@SaCheckPermission` 依赖 Spring AOP。Rust 中没有运行时反射，用 **proc-macro** 生成中间件代码：

```rust
// sa-token-rs-annotation/src/lib.rs
#[proc_macro_attribute]
pub fn sa_check_login(args: TokenStream, input: TokenStream) -> TokenStream { ... }

#[proc_macro_attribute]
pub fn sa_check_permission(args: TokenStream, input: TokenStream) -> TokenStream { ... }

#[proc_macro_attribute]
pub fn sa_check_role(args: TokenStream, input: TokenStream) -> TokenStream { ... }
```

使用示例：

```rust
#[sa_check_permission("user:add")]
async fn add_user(Json(body): Json<AddUserReq>) -> Json<SaResult> { ... }
```

宏内部会生成：
1. 检查当前请求是否已登录
2. 调用 `StpUtil::has_permission("user:add")`
3. 不满足时返回 `403` 或自定义异常

---

## 8. Web 框架适配层

### 8.1 axum 示例

```rust
// sa-token-rs-axum/src/lib.rs
pub struct SaTokenAxumLayer;

impl<S> Layer<S> for SaTokenAxumLayer { ... }

// 提供 Request/Response/Storage 的 axum 实现
pub struct AxumRequest(pub Parts);
pub struct AxumResponse(pub Response<Body>);

impl SaRequest for AxumRequest { ... }
impl SaResponse for AxumResponse { ... }
```

### 8.2 路由中间件

```rust
pub async fn sa_token_middleware(
    State(stp_logic): State<Arc<dyn StpLogicTrait>>,
    mut request: Request,
    next: Next,
) -> Response {
    // 1. 构建上下文
    let ctx = AxumContext::new(&request);
    // 2. 设置到 task-local
    CURRENT_CONTEXT.scope(ctx, async {
        next.run(request).await
    }).await
}
```

---

## 9. 与 Sa-Token 的关键差异

| 差异点 | Java 做法 | Rust 做法 |
|---|---|---|
| 依赖注入 | Spring Boot `@Bean` | 手动构造 + `Arc` 共享；或 `salvo`/`axum` 的 State |
| 反射注解 | Spring AOP | proc-macro 生成中间件 |
| 类型擦除 | `Object` 通用 | `String` 或 `serde_json::Value` + 泛型 |
| 上下文隔离 | `ThreadLocal` | `tokio::task_local!` 或显式传递 |
| 异步 | 同步为主 | `async`/`.await` 为主 |
| 错误处理 | 抛出异常 | `Result<T, SaTokenError>` |
| 配置读取 | 反射注入字段 | `Deserialize` 从 YAML/TOML/ENV 读取 |

---

## 10. 实现阶段建议

### Phase 1：最小可用核心（MVP）
- `sa-token-rs-core`：登录、登出、Token 获取、会话读写
- `sa-token-rs-dao`：Memory 实现
- `sa-token-rs-context`：Mock 上下文
- `sa-token-rs-axum`：基础适配
- 目标：能跑通 `StpUtil::login("10001")` 和 `StpUtil::is_login()`

### Phase 2：权限与注解
- 角色、权限、禁用账号、安全认证
- `#[sa_check_login]` / `#[sa_check_permission]` 宏
- 多账号类型 `StpUserUtil`

### Phase 3：存储扩展
- Redis DAO
- 分布式场景下的会话共享

### Phase 4：插件生态
- OAuth2
- SSO
- Sign 参数签名
- API Key

### Phase 5：完善示例与文档
- 多框架 demo
- 集成测试
- 与 Sa-Token Java 的 API 对照表

---

## 11. 关键文件对应关系

如果你后续要参考 Sa-Token Java 源码逐项实现，可以按以下对应关系：

| Sa-Token Java 文件 | Sa-Token-Rs 建议文件 |
|---|---|
| `cn/dev33/satoken/SaManager.java` | `sa-token-rs-core/src/manager.rs` |
| `cn/dev33/satoken/stp/StpLogic.java` | `sa-token-rs-core/src/stp_logic.rs` |
| `cn/dev33/satoken/stp/StpUtil.java` | `sa-token-rs-core/src/stp_util.rs` |
| `cn/dev33/satoken/config/SaTokenConfig.java` | `sa-token-rs-core/src/config.rs` |
| `cn/dev33/satoken/context/SaTokenContext.java` | `sa-token-rs-context/src/context.rs` |
| `cn/dev33/satoken/context/SaHolder.java` | `sa-token-rs-context/src/holder.rs` |
| `cn/dev33/satoken/session/SaSession.java` | `sa-token-rs-core/src/session.rs` |
| `cn/dev33/satoken/dao/SaTokenDao.java` | `sa-token-rs-dao/src/lib.rs` |
| `cn/dev33/satoken/listener/SaTokenEventCenter.java` | `sa-token-rs-core/src/event.rs` |
| `cn/dev33/satoken/annotation/handler/SaCheckLoginHandler.java` | `sa-token-rs-annotation/src/lib.rs` |

---

## 12. 下一步

如果你认可这个架构方向，我可以进入 **Plan Mode** 进一步细化：

1. 确定第一阶段 MVP 的范围和 crate 划分
2. 设计核心 trait 的精确签名
3. 选择默认 Web 框架适配器（建议 `axum`）
4. 输出可执行的目录结构、Cargo.toml workspace、Phase 1 代码骨架
