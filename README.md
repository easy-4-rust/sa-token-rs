# Sa-Token-Rs

> 轻量级权限认证框架 — Rust 实现版

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![Tests](https://img.shields.io/badge/tests-48%20passed-brightgreen.svg)](#测试)

**Sa-Token-Rs** 是 [Sa-Token](https://sa-token.cc/) 的 Rust 一比一移植版，提供登录认证、权限认证、会话管理、踢人下线等全家桶能力。

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

| Java Sa-Token | Sa-Token-Rs |
|---|---|
| `StpUtil.login(id)` | `StpUtil::login(id)` |
| `StpUtil.isLogin()` | `StpUtil::is_login()` |
| `StpUtil.checkPermission(p)` | `StpUtil::check_permission(p)` |
| `@SaCheckLogin` | `#[sa_check_login]` |
| `@SaCheckPermission("x")` | `#[sa_check_permission("x")]` |
| `SaSession` | `SaSession` |
| `SaTokenDao` | `SaTokenDao` trait |

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
- [easyexcel-rs](https://github.com/hiwepy/easyexcel-rs) — Java→Rust 移植参考
