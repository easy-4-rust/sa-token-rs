# Sa-Token-Rs

> 轻量级权限认证框架 — Rust 实现版

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![Tests](https://img.shields.io/badge/tests-48%20passed-brightgreen.svg)](#测试)

**Sa-Token-Rs** 是 [Sa-Token](https://sa-token.cc/) 的 Rust 一比一移植版，提供登录认证、权限认证、会话管理、踢人下线等全家桶能力。

> 仓库地址：<https://github.com/easy-4-rust/sa-token-rs>
> 文档站：<https://easy-4-rust.github.io/sa-token-rs/>

## 特性

- 🚀 **登录认证** — 一行代码完成登录
- 🔐 **权限认证** — 注解式权限校验
- 👥 **角色认证** — 角色管理与校验
- 💾 **会话管理** — 多设备 Session 管理
- 🚪 **踢人下线** — 主动踢人/顶替下线
- 🔒 **二级认证** — 敏感操作二次验证
- 🔄 **身份切换** — 临时切换账号身份
- 📦 **插件生态** — JWT、API 签名等插件
- 🌐 **Web 适配** — axum 框架支持

## 快速开始

### 安装

```toml
[dependencies]
sa-token = "0.1"
tokio = { version = "1", features = ["full"] }
```

### 基础用法

```rust
use sa_token::prelude::*;

fn main() {
    // 初始化
    SaManager::set_config(Arc::new(SaTokenConfig::default()));
    SaManager::set_sa_token_dao(Arc::new(SaTokenDaoMemory::new()));
    SaTokenContextMockUtil::set_mock_context();
    SaManager::put_stp_logic(Arc::new(StpLogic::new("login")));

    // 登录
    StpUtil::login("10001").unwrap();
    assert!(StpUtil::is_login());
    assert_eq!(StpUtil::get_login_id().unwrap(), "10001");

    // 登出
    StpUtil::logout().unwrap();
    assert!(!StpUtil::is_login());
}
```

### Axum Web 示例

```rust
use axum::{routing::get, Router, Json};
use sa_token::prelude::*;
use sa_token_web_axum::{SaTokenLayer, CurrentLoginId};

async fn user_info(login_id: CurrentLoginId) -> Json<String> {
    Json(format!("Hello, {}!", login_id.0))
}

#[tokio::main]
async fn main() {
    // 初始化 Sa-Token
    SaManager::set_config(Arc::new(SaTokenConfig::default()));
    SaManager::set_sa_token_dao(Arc::new(SaTokenDaoMemory::new()));
    SaManager::put_stp_logic(Arc::new(StpLogic::new("login")));

    // 创建应用
    let app = Router::new()
        .route("/user/info", get(user_info))
        .layer(SaTokenLayer::new());

    // 启动服务器
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

### 权限校验

```rust
use sa_token::prelude::*;

// 实现权限数据源
struct MyStpInterface;

impl StpInterface for MyStpInterface {
    fn get_permission_list(&self, login_id: &str, _login_type: &str) -> Vec<String> {
        match login_id {
            "10001" => vec!["user:add".into(), "user:list".into()],
            _ => vec![],
        }
    }

    fn get_role_list(&self, login_id: &str, _login_type: &str) -> Vec<String> {
        match login_id {
            "10001" => vec!["admin".into()],
            _ => vec![],
        }
    }
}

// 注册
SaManager::set_stp_interface(Arc::new(MyStpInterface));

// 使用
StpUtil::login("10001").unwrap();
assert!(StpUtil::has_permission("user:add").unwrap());
assert!(StpUtil::has_role("admin").unwrap());

StpUtil::check_permission("user:add").unwrap(); // 通过
StpUtil::check_permission("user:delete").unwrap_err(); // 抛异常
```

### JWT Token

```toml
[dependencies]
sa-token = "0.1"
sa-token-jwt = "0.1"
```

```rust
use sa_token_jwt::{JwtConfig, SaJwtTemplate};

let config = JwtConfig::new("my-secret-key");
let jwt = SaJwtTemplate::new(config);

// 生成 Token
let token = jwt.create_token("10001", "login", 3600).unwrap();

// 解析 Token
let claims = jwt.parse_token(&token).unwrap();
assert_eq!(claims.sub, "10001");
```

## 项目结构

```text
sa-token-rs/
├── crates/
│   ├── sa-token/                 # 用户入口（门面）
│   ├── sa-token-core/            # 核心库
│   ├── sa-token-web/             # Web 框架适配
│   │   ├── sa-token-web-axum/
│   │   ├── sa-token-web-actix/
│   │   └── sa-token-web-salvo/
│   ├── sa-token-dao/             # 存储适配
│   │   ├── sa-token-dao-memory/
│   │   └── sa-token-dao-redis/
│   ├── sa-token-support/         # core 附属
│   │   ├── sa-token-derive/
│   │   └── sa-token-context-mock/
│   ├── sa-token-plugin/          # 能力插件
│   │   ├── sa-token-jwt/
│   │   └── sa-token-sign/
│   ├── sa-token-test/            # 对齐 Java 测试合集
│   │   ├── sa-token-easy-test/
│   │   ├── sa-token-springboot-test/  # Spring Boot→axum
│   │   ├── sa-token-jwt-test/
│   │   ├── sa-token-temp-jwt-test/
│   │   ├── sa-token-json-test/
│   │   ├── sa-token-jackson3-test/
│   │   └── sa-token-serializer-test/
│   └── sa-token-demo/
└── docs/
```

## 与 Java Sa-Token 的对应关系

| 维度 | Java Sa-Token | Sa-Token-Rs |
|---|---|---|
| 包坐标 | `cn.dev33.satoken:sa-token-core` | `sa-token = "0.1"` |
| 静态门面 | `StpUtil.login(id)` | `StpUtil::login("10001")?` |
| 布尔查询 | `StpUtil.isLogin()` | `StpUtil::is_login()` |
| 异常检查 | `StpUtil.checkPermission(p)` | `StpUtil::check_permission(p)?` |
| 注解 | `@SaCheckLogin` | `#[sa_check_login]` |
| 注解（带参） | `@SaCheckPermission("x")` | `#[sa_check_permission("x")]` |
| 会话模型 | `SaSession` | `SaSession` |
| 持久化接口 | `SaTokenDao` | `SaTokenDao` trait |
| 配置载体 | `application.yml` | `config.toml`（可选）或 `SaTokenConfig::default()` |
| 请求上下文 | `SaHolder.getRequest()` | `SaHolder::request()?` |
| 异常类型 | `NotLoginException` 等 20+ | `SaTokenException::NotLogin` 等单一 enum |
| 启动器 | `sa-token-spring-boot-starter` | `sa-token-web-axum = "0.1"` |

### 双侧示例：登录与权限校验

**Java Sa-Token：**

```java
// application.yml
sa-token:
  token-name: satoken
  timeout: 2592000
  is-concurrent: true

// 业务代码
@RestController
public class UserController {
    @PostMapping("/login")
    public String login(String id) {
        StpUtil.login(id);                 // 登录
        return StpUtil.getTokenValue();    // 返回 Token
    }

    @SaCheckPermission("user:add")
    @PostMapping("/user/add")
    public void addUser() {
        // 注解自动校验权限，未通过抛 NotPermissionException
    }
}
```

**Sa-Token-Rs：**

```rust
// config.toml（可选，与 application.yml 等价）
[sa_token]
token_name = "satoken"
timeout = 2592000
is_concurrent = true

// 业务代码（axum 0.8）
use axum::{routing::post, Router, Json};
use sa_token::prelude::*;
use sa_token_web_axum::SaTokenLayer;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/login", post(login))
        .route("/user/add", post(add_user))
        .layer(SaTokenLayer::new());
    // ...
}

async fn login(Json(id): Json<String>) -> SaResult<Json<String>> {
    StpUtil::login(&id)?;
    Ok(Json(StpUtil::get_token_value().unwrap_or_default()))
}

#[sa_check_permission("user:add")]
async fn add_user() -> SaResult<Json<&'static str>> {
    Ok(Json("ok"))
}
```

### Cargo.toml 完整配置

**最小依赖**（仅核心 + Mock）：

```toml
[dependencies]
sa-token = "0.1"
tokio = { version = "1", features = ["full"] }
```

**完整依赖**（核心 + Web 适配 + Redis DAO + 插件）：

```toml
[dependencies]
sa-token = "0.1"                          # 核心门面
sa-token-web-axum = "0.1"                 # axum 0.8 适配
sa-token-dao-redis = "0.1"                # 生产级 Redis DAO（async，基于 fred）
sa-token-jwt = "0.1"                      # JWT Token 插件
sa-token-sign = "0.1"                     # API 参数签名校验
sa-token-sso = "0.1"                      # SSO 单点登录
sa-token-oauth2 = "0.1"                   # OAuth2（含 OIDC）
sa-token-apikey = "0.1"                   # API Key 鉴权
tokio = { version = "1", features = ["full"] }
axum = "0.8"
fred = "9"                                # Redis 客户端（sa-token-dao-redis 依赖）
```

**Java vs Rust 配置文件对照：**

| 维度 | Java | Rust |
|---|---|---|
| 依赖声明 | `pom.xml` / `build.gradle` | `Cargo.toml` `[dependencies]` |
| 配置文件 | `application.yml` / `application.properties` | `config.toml`（可选） |
| 加载方式 | Spring Boot 自动绑定 | `toml::from_str` + `SaManager::set_config` |
| 配置对象 | `SaTokenConfig`（Spring Bean） | `SaTokenConfig`（`Arc<SaTokenConfig>`） |
| 热更新 | Spring Cloud Refresh | 需手动 `SaManager::set_config` |
| Profile 切换 | `spring.profiles.active` | `[features]` + `--features` |

> **说明**：Sa-Token-Rs 的 `SaTokenConfig` 已实现 `serde::Deserialize`，可通过 TOML/JSON 反序列化加载；如需 YAML，可由用户自行加上 `serde_yaml` crate。

## 测试

```bash
# 运行所有测试
cargo test -- --test-threads=1

# 运行特定测试
cargo test -p sa-token-test --test login_test
cargo test -p sa-token-test --test phase2_test
cargo test -p sa-token-web-axum --test axum_test
cargo test -p sa-token-jwt
```

## 文档

- [架构设计](docs/ARCHITECTURE.md)
- [使用指南](docs/GUIDE.md)
- [兼容性说明](docs/compatibility.md)
- [实施计划](docs/IMPLEMENTATION_PLAN.md)
- [迁移文档](docs/migration/)

## 许可证

Apache License 2.0

## 致谢

- [Sa-Token](https://sa-token.cc/) — Java 原版
- [easyexcel-rs](https://github.com/easy-4-rust/easyexcel-rs) — Java→Rust 移植参考
