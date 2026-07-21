# Sa-Token-Rs 使用指南

> 本文档面向使用者，介绍 Sa-Token-Rs 的安装、配置、基础用法和进阶功能。
> 架构设计请参阅 [ARCHITECTURE.md](./ARCHITECTURE.md)。

---

## 一、5 分钟快速开始

### 1.1 添加依赖

```toml
# Cargo.toml
[dependencies]
sa-token = "0.1"              # 核心门面（必选）
sa-token-axum = "0.1"         # Web 框架适配（任选其一）
tokio = { version = "1", features = ["full"] }
```

### 1.2 最小示例（非 Web）

```rust
use sa_token::prelude::*;
use sa_token::SaManager;
use std::sync::Arc;

fn main() -> SaResult<()> {
    // 1. 初始化全局配置（应用启动时调用一次）
    SaManager::set_config(Arc::new(SaTokenConfig::default()));
    SaManager::set_sa_token_dao(Arc::new(SaTokenDaoDefaultImpl::new()));

    // 2. 设置 Mock 上下文（非 Web 场景）
    SaTokenContextMockUtil::set_mock_context();

    // 3. 登录
    StpUtil::login("10001")?;
    println!("当前登录 ID: {}", StpUtil::get_login_id()?);

    // 4. 鉴权检查
    if StpUtil::is_login() {
        println!("已登录");
    }

    // 5. 登出
    StpUtil::logout()?;

    Ok(())
}
```

### 1.3 axum Web 示例

```rust
use axum::{routing::get, Router, Json};
use sa_token::prelude::*;
use sa_token_axum::SaTokenLayer;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/login", get(login_handler))
        .route("/user/info", get(user_info_handler))
        .layer(SaTokenLayer::new());  // ← 加载 Sa-Token 中间件

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn login_handler() -> SaResult<Json<String>> {
    StpUtil::login("10001")?;  // ← 在 async 上下文调用同步核心
    Ok(Json("登录成功".to_string()))
}

async fn user_info_handler() -> SaResult<Json<String>> {
    StpUtil::check_login()?;  // ← 未登录会抛 NotLogin 异常
    let login_id = StpUtil::get_login_id()?;
    Ok(Json(format!("用户: {}", login_id)))
}
```

---

## 二、核心 API 速查

### 2.1 登录相关

```rust
// 基础登录
StpUtil::login("10001")?;

// 指定设备类型
StpUtil::login_with_device("10001", "PC")?;

// 完整参数登录
let param = SaLoginParameter::create()
    .set_device_type("PC")
    .set_device_id("device-001")
    .set_timeout(3600);  // 1 小时
StpUtil::login_with_param("10001", &param)?;

// 检查登录状态
StpUtil::check_login()?;  // 未登录抛 NotLogin 异常
let is_login: bool = StpUtil::is_login();
let login_id: String = StpUtil::get_login_id()?;
let login_id_opt: Option<String> = StpUtil::get_login_id_default_null();
```

### 2.2 登出 / 踢人 / 顶替

```rust
// 当前会话登出
StpUtil::logout()?;

// 按 login_id 登出（所有设备）
StpUtil::logout_by_login_id("10001")?;

// 踢人下线（对方会立即失效）
StpUtil::kickout("10001")?;

// 顶人下线（被顶的设备在新登录时自动失效）
StpUtil::replaced("10001")?;

// 按 Token 值登出
let token = StpUtil::get_token_value().unwrap();
StpUtil::logout_by_token_value(&token)?;
```

### 2.3 权限 / 角色

```rust
// 实现 StpInterface 提供权限数据
struct MyStpInterface;
impl StpInterface for MyStpInterface {
    fn get_permission_list(&self, login_id: &str, login_type: &str) -> Vec<String> {
        match login_id {
            "10001" => vec!["user:add".into(), "user:list".into(), "user:delete".into()],
            _ => vec![],
        }
    }

    fn get_role_list(&self, login_id: &str, login_type: &str) -> Vec<String> {
        match login_id {
            "10001" => vec!["admin".into()],
            _ => vec!["user".into()],
        }
    }
}

// 注册（应用启动时）
SaManager::set_stp_interface(Arc::new(MyStpInterface));

// 检查权限
StpUtil::check_permission("user:add")?;          // 缺权限抛异常
let has: bool = StpUtil::has_permission("user:add")?;

// 检查角色
StpUtil::check_role("admin")?;
let has_admin: bool = StpUtil::has_role("admin")?;

// AND / OR 组合
StpUtil::check_permission_and(&["user:add", "user:delete"])?;  // 必须同时具有
StpUtil::check_permission_or(&["user:add", "user:list"])?;     // 具有任一即可
```

### 2.4 会话

```rust
// 获取当前账号 Session
let session = StpUtil::get_session()?;

// 存取数据
session.set("key", json!("value"));
let value: Option<Value> = session.get("key");

// 获取指定账号 Session
let session = StpUtil::get_session_by_login_id("10001")?;

// Token-Session（每个 Token 独立的 Session）
let token_session = StpUtil::get_token_session()?;
```

### 2.5 Token 操作

```rust
// 获取当前 Token
let token: Option<String> = StpUtil::get_token_value()?;

// Token 详情
let info = StpUtil::get_token_info()?;
println!("token: {}, login_id: {}", info.token_value, info.login_id);

// Token 有效期
let timeout: i64 = StpUtil::get_token_timeout();

// 续签
StpUtil::renew_timeout(3600)?;
```

### 2.6 设备管理

```rust
// 获取当前登录设备
let device: String = StpUtil::get_login_device_type()?;
let device_id: String = StpUtil::get_login_device_id()?;

// 获取账号所有终端
let terminals = StpUtil::get_terminal_list_by_login_id("10001")?;
for t in terminals {
    println!("token={}, device={}", t.token_value, t.device_type);
}
```

---

## 三、注解鉴权

### 3.1 基本用法

```rust
use sa_token_derive::*;

#[sa_check_login]
fn need_login() -> SaResult<()> { ... }

#[sa_check_permission("user:add")]
fn add_user() -> SaResult<()> { ... }

#[sa_check_role("admin")]
fn admin_only() -> SaResult<()> { ... }

#[sa_check_safe]
fn sensitive_op() -> SaResult<()> { ... }

#[sa_check_or(roles = ["admin", "super"], permissions = ["system:config"])]
fn admin_or_config_perm() -> SaResult<()> { ... }

#[sa_ignore]  // 跳过全局拦截器
fn public_api() -> SaResult<()> { ... }
```

### 3.2 在 axum 中使用

```rust
use sa_token_derive::*;

#[sa_check_permission("user:add")]
async fn add_user() -> SaResult<Json<User>> {
    // 宏会在函数入口处自动插入权限检查代码
    // 未通过则返回 403
    Ok(Json(User { ... }))
}
```

---

## 四、配置

### 4.1 通过代码配置

```rust
use sa_token::config::SaTokenConfig;
use std::sync::Arc;

let config = SaTokenConfig {
    token_name: "satoken".into(),
    timeout: 30 * 24 * 3600,        // 30 天
    active_timeout: -1,              // 不检查活跃超时
    is_concurrent: true,             // 允许同账号多端登录
    is_share: true,                  // 多人登录共用 Token
    max_login_count: 12,
    token_style: SaTokenStyle::Uuid,
    is_log: true,
    ..Default::default()
};

SaManager::set_config(Arc::new(config));
```

### 4.2 通过 TOML/YAML 配置（未来支持）

```toml
# config.toml
[sa-token]
token_name = "satoken"
timeout = 2592000
active_timeout = -1
is_concurrent = true
is_share = true
token_style = "uuid"
is_log = true
```

```rust
let config: SaTokenConfig = toml::from_str(&fs::read_to_string("config.toml")?)?;
SaManager::set_config(Arc::new(config));
```

---

## 五、存储后端

### 5.1 默认 Memory（单机/测试）

```rust
use sa_token::SaManager;
use std::sync::Arc;

SaManager::set_sa_token_dao(Arc::new(SaTokenDaoDefaultImpl::new()));
```

### 5.2 Redis（生产推荐）

```toml
[dependencies]
sa-token-dao-redis = "0.1"
fred = "9"
```

```rust
use sa_token_dao_redis::SaTokenDaoRedis;
use fred::Client;

#[tokio::main]
async fn main() {
    let client = Client::init("redis://127.0.0.1:6379").await.unwrap();
    let dao = SaTokenDaoRedis::new(client);
    SaManager::set_sa_token_dao(Arc::new(dao));
}
```

### 5.3 moka 高性能缓存

```toml
[dependencies]
sa-token-dao-moka = "0.1"
```

```rust
use sa_token_dao_moka::SaTokenDaoMoka;

let dao = SaTokenDaoMoka::builder()
    .max_capacity(100_000)
    .time_to_live(Duration::from_secs(3600))
    .build();
SaManager::set_sa_token_dao(Arc::new(dao));
```

---

## 六、多账号体系

Sa-Token 支持多账号类型（如 User / Admin），每种类型独立登录态。

### 6.1 使用 `define_stp_util!` 宏

```rust
use sa_token::define_stp_util;

define_stp_util!(StpUserUtil, "user");      // 用户账号体系
define_stp_util!(StpAdminUtil, "admin");    // 管理员账号体系

// 使用
StpUserUtil::login("10001")?;
StpAdminUtil::login("admin001")?;

StpUserUtil::check_login()?;
StpAdminUtil::check_login()?;
```

---

## 七、事件监听

### 7.1 实现自定义监听器

```rust
use sa_token::listener::SaTokenListener;

struct MyListener;

impl SaTokenListener for MyListener {
    fn do_login(&self, login_type: &str, login_id: &str, token_value: &str, param: &SaLoginParameter) {
        println!("[登录] type={}, id={}, token={}", login_type, login_id, token_value);
    }

    fn do_logout(&self, login_type: &str, login_id: &str, token_value: &str) {
        println!("[登出] type={}, id={}, token={}", login_type, login_id, token_value);
    }

    fn do_kickout(&self, login_type: &str, login_id: &str, token_value: &str) {
        println!("[踢下线] type={}, id={}, token={}", login_type, login_id, token_value);
    }

    // ... 其他事件可选实现
}

// 注册
SaManager::listeners().write().unwrap().push(Arc::new(MyListener));
```

---

## 八、进阶：SSO 单点登录（Phase 5）

### 8.1 SSO Server 端

```rust
use sa_token_sso::SaSsoServerTemplate;

let sso_server = SaSsoServerTemplate::new();
// 处理 /sso/auth 请求
let redirect_url = sso_server.sso_auth(req).await?;
```

### 8.2 SSO Client 端

```rust
use sa_token_sso::SaSsoClientTemplate;

let sso_client = SaSsoClientTemplate::new();
// 处理 ticket 回调
let login_id = sso_client.check_ticket(ticket).await?;
StpUtil::login(&login_id)?;
```

---

## 九、常见问题

### Q1: 在 async 上下文调用同步 API 报错？

Sa-Token 核心是同步的，在 async handler 中直接调用同步阻塞 API 可能有问题。两种方案：

```rust
// 方案 1：用 block_in_place（推荐，需 multi_thread runtime）
let login_id = tokio::task::block_in_place(|| StpUtil::get_login_id())?;

// 方案 2：用 spawn_blocking
let login_id = tokio::task::spawn_blocking(|| StpUtil::get_login_id())
    .await
    .unwrap()?;
```

### Q2: 测试如何隔离全局状态？

```rust
#[test]
fn test_xxx() {
    SaManager::reset();  // 重置全局状态（测试辅助方法）
    // ... 测试代码
}
```

### Q3: Java Sa-Token 代码怎么对应到 Rust？

参考 [migration/object-method-matrix.md](./migration/object-method-matrix.md)。

---

## 十、参考

- [ARCHITECTURE.md](./ARCHITECTURE.md) - 架构设计
- [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md) - 完整实施计划
- [migration/object-method-matrix.md](./migration/object-method-matrix.md) - 方法对照表
- **Sa-Token Java 官方文档**：https://sa-token.cc
