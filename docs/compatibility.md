# Sa-Token-Rs 兼容性说明

> 本文档说明 Sa-Token-Rs 与 Sa-Token Java 的兼容性，以及不同版本间的迁移注意事项。

---

## 一、与 Sa-Token Java 的兼容性

### 1.1 API 兼容性矩阵

| 功能模块 | 兼容性 | 说明 |
|---|---|---|
| **登录认证** | ✅ 100% | `login` / `logout` / `is_login` / `check_login` 等完全对齐 |
| **Token 管理** | ✅ 100% | Token 生成、续签、超时管理完全对齐 |
| **会话管理** | ✅ 100% | `SaSession` / `SaTerminalInfo` 完全对齐 |
| **权限认证** | ✅ 100% | `has_permission` / `check_permission` 等完全对齐 |
| **角色认证** | ✅ 100% | `has_role` / `check_role` 等完全对齐 |
| **踢人下线** | ✅ 100% | `kickout` / `replaced` 完全对齐 |
| **账号封禁** | ✅ 100% | `disable` / `is_disable` 等完全对齐 |
| **二级认证** | ✅ 100% | `open_safe` / `check_safe` 等完全对齐 |
| **身份切换** | ✅ 100% | `switch_to` / `end_switch` 完全对齐 |
| **设备管理** | ✅ 100% | 设备类型、设备 ID 管理完全对齐 |
| **注解鉴权** | 🟡 90% | 注解改为 proc-macro，语义对齐但语法略异 |
| **SSO** | ✅ 100% | Phase 5 实现 |
| **OAuth2** | ✅ 100% | Phase 5 实现 |
| **JWT** | ✅ 100% | Phase 5 实现 |
| **Sign 签名** | ✅ 100% | Phase 5 实现 |
| **ApiKey** | ✅ 100% | Phase 5 实现 |

### 1.2 不兼容/差异点

#### 1.2.1 异步模型差异

| 方面 | Java | Rust |
|---|---|---|
| 核心调用 | 同步阻塞 | **同步阻塞**（核心保持同步） |
| Web 集成 | Servlet 同步 | axum/actix async（用 `block_in_place` 桥接） |
| Redis 操作 | 同步 | **async**（fred） |

**迁移影响**：在 async Web handler 中调用核心 API 时，需用 `block_in_place` 或 `spawn_blocking`。

#### 1.2.2 反射 → 宏

| 方面 | Java | Rust |
|---|---|---|
| 注解实现 | Spring AOP 运行时反射 | **proc-macro 编译期生成** |
| 注解扫描 | 启动时扫描 ClassPath | **编译期一次性生成** |
| 运行时开销 | 有反射开销 | **零运行时开销** |

**迁移影响**：注解语法略有不同（`#[sa_check_login]` vs `@SaCheckLogin`），但语义一致。

#### 1.2.3 异常处理

| 方面 | Java | Rust |
|---|---|---|
| 异常类型 | 20+ RuntimeException 子类 | **单一 enum** |
| 错误传播 | `throw` 异常 | **`Result<T, SaTokenException>`** |
| 错误恢复 | `try-catch` | **`match` / `?` 运算符** |

**迁移影响**：Java 的 `try-catch` 改为 Rust 的 `?` 或 `match`。

#### 1.2.4 类型系统

| 方面 | Java | Rust |
|---|---|---|
| loginId 类型 | `Object`（任意类型） | **`String`**（内部统一） |
| Session 数据 | `Map<String, Object>` | **`HashMap<String, serde_json::Value>`** |
| 泛型擦除 | 运行时擦除 | **编译期单态化** |

**Java 类型访问示例：**

```java
// Object 类型强转
Integer id = (Integer) StpUtil.getLoginId();
String idStr = (String) StpUtil.getLoginId();

// Session 数据按类型反序列化
session.set("user", userObject);
User user = session.getModel("user", User.class);  // Sa-Token 内置反序列化
```

**Sa-Token-Rs 反序列化示例：**

```rust
// loginId 一律 String
let id: String = StpUtil::get_login_id()?;

// Session 数据用 serde_json::from_value 反序列化
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct User { id: u32, name: String }

let session = StpUtil::get_session()?;
session.set("user", serde_json::json!({"id": 10001, "name": "Alice"}));

let user: Option<User> = session
    .get("user")
    .and_then(|v| serde_json::from_value(v).ok());
```

**迁移影响**：
- Java `StpUtil.login(10001)` → Rust `StpUtil::login("10001")`（数字需转字符串）
- Java `session.get("key", User.class)` → Rust `session.get("key").and_then(|v| serde_json::from_value(v).ok())`

#### 1.2.5 依赖注入

| 方面 | Java | Rust |
|---|---|---|
| 组件注册 | Spring `@Bean` 自动注入 | **手动 `SaManager::set_xxx()`** |
| 自动装配 | `@Autowired` | **`Arc<dyn Trait>` 显式传递** |
| 循环依赖 | Spring 自动解决 | **Rust 所有权禁止循环依赖** |

**迁移影响**：需在应用启动时手动注册所有组件。

---

## 二、Web 框架兼容性

### 2.1 支持的框架

| 框架 | 状态 | 版本要求 |
|---|---|---|
| **axum** | ✅ Phase 3 | axum 0.7+ |
| **actix-web** | ✅ Phase 3 | actix-web 4+ |
| **salvo** | ✅ Phase 3 | salvo 0.68+ |
| Spring Boot（Rust 移植） | 🔜 未来 | - |

### 2.2 从 Java Spring Boot 迁移

| Java 写法 | Rust 等价 |
|---|---|
| `@Bean public SaTokenDao dao() { ... }` | `SaManager::set_sa_token_dao(Arc::new(MyDao));` |
| `@Bean public StpInterface stpInterface() { ... }` | `SaManager::set_stp_interface(Arc::new(MyStpInterface));` |
| `application.yml` 配置 | TOML/JSON 反序列化到 `SaTokenConfig` |
| `WebMvcConfigurer.addInterceptors` | `.layer(SaTokenLayer::new())` |
| `@ControllerAdvice` 异常处理 | axum 的错误响应 Handler |

---

## 三、存储后端兼容性

### 3.1 支持的存储

| 存储 | Java 版 | Rust 版 | 状态 |
|---|---|---|---|
| **Memory** | ✅ 内置 | ✅ `sa-token-core` 内置 | Phase 1 |
| **Redis（Lettuce）** | ✅ 插件 | 🔜 `sa-token-dao-redis`（fred） | Phase 4 |
| **Redisson** | ✅ 插件 | 🔜 同上（fred 覆盖） | Phase 4 |
| **moka 缓存** | ❌ | ✅ `sa-token-dao-moka` | Phase 4 |
| **Caffeine** | ✅ 插件 | ✅ 用 moka 替代 | Phase 4 |

### 3.2 Redis 数据格式兼容性

Sa-Token-Rs 的 Redis 数据格式与 Java 版**完全兼容**，可直接读写 Java 版写入的数据：

- Token-LoginId 映射：`satoken:login:token:{tokenValue}` → `{loginId}`
- Session：`satoken:login:session:{loginId}` → JSON
- 最后活跃时间：`satoken:login:last-active:{tokenValue}` → `{timestamp}`

**注意**：JSON 序列化格式需对齐（`SaJsonTemplate` 默认用 `serde_json`）。

---

## 四、Token 风格兼容性

| 风格 | Java 枚举 | Rust 枚举 | 兼容 |
|---|---|---|---|
| UUID | `SaTokenStyle::UUID` | `SaTokenStyle::Uuid` | ✅ |
| 简单 UUID（无连字符） | `SaTokenStyle::SIMPLE_UUID` | `SaTokenStyle::SimpleUuid` | ✅ |
| 随机 32 位 | `SaTokenStyle::RANDOM_32` | `SaTokenStyle::Random32` | ✅ |
| 随机 64 位 | `SaTokenStyle::RANDOM_64` | `SaTokenStyle::Random64` | ✅ |
| 随机 128 位 | `SaTokenStyle::RANDOM_128` | `SaTokenStyle::Random128` | ✅ |
| Base64 编码 | `SaTokenStyle::BASE_64` | `SaTokenStyle::Base64` | ✅ |
| JWT | `SaTokenStyle::JWT` | `SaTokenStyle::Jwt`（需 sa-token-jwt 插件） | ✅ |

---

## 五、配置项兼容性

### 5.1 完全兼容的配置

| 配置项 | Java | Rust | 默认值 |
|---|---|---|---|
| tokenName | `token_name` | `token_name` | `"satoken"` |
| timeout | `timeout` | `timeout` | `2592000`（30 天） |
| activeTimeout | `active_timeout` | `active_timeout` | `-1`（不检查） |
| isConcurrent | `is_concurrent` | `is_concurrent` | `true` |
| isShare | `is_share` | `is_share` | `true` |
| maxLoginCount | `max_login_count` | `max_login_count` | `12` |
| isReadBody | `is_read_body` | `is_read_body` | `true` |
| isReadHeader | `is_read_header` | `is_read_header` | `true` |
| isReadCookie | `is_read_cookie` | `is_read_cookie` | `true` |
| isLastingCookie | `is_lasting_cookie` | `is_lasting_cookie` | `true` |
| tokenPrefix | `token_prefix` | `token_prefix` | `""` |
| isPrint | `is_print` | `is_print` | `true` |
| isLog | `is_log` | `is_log` | `true` |

### 5.2 Rust 独有配置

| 配置项 | 说明 |
|---|---|
| `auth_hash_enabled` | 是否启用 auth_hash 踢下线（Rust 增强） |
| `redis_pubsub_enabled` | 是否启用 Redis Pub/Sub 实时踢下线推送 |

### 5.3 Java application.yml ↔ Rust config.toml 配置对照

**Java application.yml：**

```yaml
sa-token:
  token-name: satoken
  token-prefix: ''
  token-style: uuid
  timeout: 2592000
  active-timeout: -1
  is-concurrent: true
  is-share: true
  max-login-count: 12
  is-read-cookie: true
  is-read-header: true
  is-read-body: true
  is-lasting-cookie: true
  is-log: true
  is-print: false
  is-write-header: true
  jwt:
    secret-key: my-secret-key
    algorithm: HS256
    issuer: sa-token-rs
```

**Sa-Token-Rs config.toml（等价）：**

```toml
[sa_token]
token_name = "satoken"
token_prefix = ""
token_style = "uuid"
timeout = 2592000
active_timeout = -1
is_concurrent = true
is_share = true
max_login_count = 12
is_read_cookie = true
is_read_header = true
is_read_body = true
is_lasting_cookie = true
is_log = true
is_print = false
is_write_header = true

[sa_token.jwt]
secret_key = "my-secret-key"
algorithm = "HS256"
issuer = "sa-token-rs"
```

**字段名映射：**

| Java | Rust | 说明 |
|---|---|---|
| `token-name` | `token_name` | kebab-case → snake_case |
| `is-concurrent` | `is_concurrent` | 保留 `is_` 前缀 |
| `max-login-count` | `max_login_count` | 数值类型 |
| `jwt.secret-key` | `jwt.secret_key` | 嵌套表用 `.` 分隔 |
| `is-write-header` | `is_write_header` | — |

**Rust 加载方式：**

```rust
use sa_token::prelude::*;
use sa_token::config::SaTokenConfig;
use sa_token::SaManager;
use std::sync::Arc;

let text = std::fs::read_to_string("config.toml")?;
let cfg: SaTokenConfig = toml::from_str(&text)?;
SaManager::set_config(Arc::new(cfg));
```

> **YAML 支持**：Sa-Token-Rs 不内置 `serde_yaml`，但因 `SaTokenConfig` 实现 `Deserialize`，加上 `serde_yaml` crate 后可用 `serde_yaml::from_str` 直接解析。

---

## 六、从 Java 版迁移的检查清单

### 6.1 代码迁移

- [ ] `StpUtil.login(id)` → `StpUtil::login(&id.to_string())?`
- [ ] `StpUtil.getLoginId()` → `StpUtil::get_login_id()?`
- [ ] `StpUtil.checkLogin()` → `StpUtil::check_login()?`
- [ ] `@SaCheckLogin` → `#[sa_check_login]`
- [ ] `@SaCheckPermission("x")` → `#[sa_check_permission("x")]`
- [ ] `implements StpInterface` → `impl StpInterface for ...`
- [ ] `try { ... } catch (NotLoginException e)` → `match StpUtil::get_login_id() { Ok(id) => ..., Err(SaTokenException::NotLogin { .. }) => ... }`

### 6.2 配置迁移

- [ ] `application.yml` 的 `sa-token:` 节 → TOML/JSON `SaTokenConfig`
- [ ] Spring `@Bean` 组件注册 → `SaManager::set_xxx()` 手动注册
- [ ] Redis 连接配置 → `fred::Client::init(url)`

### 6.3 测试迁移

- [ ] JUnit `@Test` → Rust `#[test]`
- [ ] `@BeforeEach setupSaToken()` → `SaManager::reset()` + 手动初始化
- [ ] `MockMvc` → axum `TestServer` + 测试客户端

---

## 七、参考

- [ARCHITECTURE.md](./ARCHITECTURE.md) - 架构设计
- [migration/object-method-matrix.md](./migration/object-method-matrix.md) - 方法对照表
- **Sa-Token Java 官方文档**：https://sa-token.cc
