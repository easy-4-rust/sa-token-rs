# Sa-Token-Rs 生态路线图

> 本文档描述 Sa-Token-Rs 的长期发展规划和生态建设方向。

---

## 一、版本规划

### 1.1 里程碑路线图

```text
v0.1.0 (Phase 1) ─── MVP 核心
    ├── 登录 / 登出 / 会话 / Token 基础闭环
    ├── Memory DAO
    └── Mock 上下文 + 基础测试

v0.2.0 (Phase 2) ─── 权限与注解
    ├── 权限 / 角色 / 禁用 / 安全认证 / 切换账号
    ├── proc-macro 注解（9 个）
    ├── 防火墙策略
    └── HttpBasic / HttpDigest / SameToken / TempToken

v0.3.0 (Phase 3) ─── Web 框架适配
    ├── sa-token-axum
    ├── sa-token-actix-web
    ├── sa-token-salvo
    └── 四层路由保护 API

v0.4.0 (Phase 4) ─── 存储扩展
    ├── sa-token-dao-redis（fred）
    ├── sa-token-dao-moka
    └── Redis Pub/Sub 实时踢下线

v0.5.0 (Phase 5) ─── 插件生态
    ├── sa-token-jwt
    ├── sa-token-sign
    ├── sa-token-oauth2
    ├── sa-token-sso
    └── sa-token-apikey

v1.0.0 (Phase 6) ─── 正式发布
    ├── 完整示例（axum/actix/salvo）
    ├── Golden + Parity 测试 100% 通过
    ├── 完整文档
    └── 性能基准
```

### 1.2 发布节奏

| 版本 | 目标时间 | 主要内容 |
|---|---|---|
| v0.1.0 | 2~3 周 | Phase 1 MVP 核心 |
| v0.2.0 | +2 周 | Phase 2 权限注解 |
| v0.3.0 | +2 周 | Phase 3 Web 适配 |
| v0.4.0 | +1 周 | Phase 4 存储扩展 |
| v0.5.0 | +4 周 | Phase 5 插件生态 |
| v1.0.0 | +2 周 | Phase 6 正式发布 |

---

## 二、生态对接

### 2.1 Web 框架（Phase 3）

| 框架 | crate | 状态 |
|---|---|---|
| axum | `sa-token-axum` | Phase 3 |
| actix-web | `sa-token-actix-web` | Phase 3 |
| salvo | `sa-token-salvo` | Phase 3 |
| poem | `sa-token-poem` | 🔜 未来 |
| rocket | `sa-token-rocket` | 🔜 未来 |
| warp | `sa-token-warp` | 🔜 未来 |

### 2.2 存储后端（Phase 4）

| 存储 | crate | 状态 |
|---|---|---|
| Memory | 内置 `sa-token-core` | Phase 1 |
| Redis（fred） | `sa-token-dao-redis` | Phase 4 |
| moka 缓存 | `sa-token-dao-moka` | Phase 4 |
| PostgreSQL | `sa-token-dao-postgres` | 🔜 未来 |
| MySQL | `sa-token-dao-mysql` | 🔜 未来 |
| MongoDB | `sa-token-dao-mongo` | 🔜 未来 |

### 2.3 权限引擎（可插拔）

| 引擎 | crate | 状态 |
|---|---|---|
| 默认（Sa-Token 自带） | 内置 | Phase 2 |
| Casbin | `sa-token-casbin` | 🔜 未来 |
| AWS Cedar | `sa-token-cedar` | 🔜 未来 |
| OPA（Open Policy Agent） | `sa-token-opa` | 🔜 未来 |

### 2.4 序列化

| 库 | crate | 状态 |
|---|---|---|
| serde_json（默认） | 内置 | Phase 1 |
|simd-json | `sa-token-json-simdjson` | 🔜 未来 |

---

## 三、进阶能力规划

### 3.1 Sa-Token-Rs 独有增强

以下能力超出 Sa-Token Java 原版范围，是 Rust 版本的增强：

#### 实时踢下线（Redis Pub/Sub）

```text
Java 版：被踢用户下次请求时才发现
Rust 版：通过 Redis Pub/Sub 主动推送，前端 SSE/WebSocket 实时感知
```

#### 强类型 JWT Claims

```rust
// Java 版：extraData: Map<String, Object>（弱类型）
// Rust 版：泛型 Claims
#[derive(Serialize, Deserialize)]
struct MyClaims {
    user_id: String,
    tenant_id: String,
    roles: Vec<String>,
}

let token = StpUtil::login_with_claims("10001", &MyClaims { ... })?;
let claims: MyClaims = StpUtil::get_extra()?;
```

#### 编译期路由保护

```rust
// 路由元数据编译期烤进二进制，运行时零开销
#[derive(SaRouteTable)]
enum Routes {
    #[sa_route(path = "/user/add", permission = "user:add")]
    AddUser,
    #[sa_route(path = "/user/list", permission = "user:list")]
    ListUsers,
}

// 生成的代码：
impl SaRouteTable for Routes {
    fn routes() -> &'static [SaRoute] { ... }  // &'static，编译期常量
}
```

#### biscuit 能力令牌（可选）

```text
Java 版：Same-Token 是简单签名，能力有限
Rust 版：可选 biscuit-auth 作为 Same-Token 后端，支持衰减权限、第三方签发、离线验证
```

### 3.2 可观测性

| 能力 | 规划 |
|---|---|
| 结构化日志 | `tracing` 集成（Phase 1） |
| Metrics | `prometheus` 指标导出（未来） |
| 分布式追踪 | `opentelemetry` 集成（未来） |
| 审计日志 | 登录/登出/权限变更审计（未来） |

### 3.3 性能优化

| 方向 | 规划 |
|---|---|
| `const fn` 元数据 | 编译期常量（Phase 2，借鉴 easyexcel-rs） |
| 零拷贝序列化 | `serde` + `bytes`（未来） |
| 连接池 | `fred` 内置（Phase 4） |
| 异步 IO | starter 层 async（Phase 3） |

---

## 四、社区与治理

### 4.1 仓库结构

```text
sa-token-rs/                      主仓库
├── crates/                       所有 crate
├── docs/                         文档
├── scripts/                      辅助脚本
├── .github/workflows/            CI
└── Cargo.toml                    workspace
```

### 4.2 CI/CD

```yaml
# .github/workflows/ci.yml
jobs:
  quality:
    steps:
      - cargo fmt --all -- --check
      - cargo clippy --workspace --all-targets --all-features -- -D warnings
      - cargo test --workspace --all-features
      - ./scripts/coverage.sh
```

### 4.3 版本策略

- 遵循 [Semantic Versioning](https://semver.org/)
- MSRV（最低支持 Rust 版本）：1.88
- 每个 Phase 结束发布一个 minor 版本
- v1.0.0 后保证 API 稳定性

### 4.4 贡献指南（未来）

- Fork → Branch → PR
- 所有 PR 必须通过 CI（fmt + clippy + test）
- 新功能必须有测试
- 公共 API 必须有文档注释（`///`）

---

## 五、与 Java 版的协同

### 5.1 功能同步策略

- Sa-Token Java 发布新版本后，1~2 周内评估移植
- 仅移植稳定特性，实验性特性暂缓
- Rust 独有增强不回溯到 Java 版

### 5.2 数据互通

- Redis 数据格式完全兼容（Phase 4）
- 可与 Java 版混用（如 Java 版写、Rust 版读）

### 5.3 文档对照

- [migration/object-method-matrix.md](./migration/object-method-matrix.md) 维护方法级对照
- [migration/CODEGRAPH_METHOD_MAP.md](./migration/CODEGRAPH_METHOD_MAP.md) 维护审计记录

---

## 六、长期愿景

### 6.1 短期（6 个月）

- 完成 Phase 1~5，覆盖 Sa-Token Java 核心功能
- axum / actix / salvo 三大框架适配
- Redis 存储支持
- Golden + Parity 测试 100% 通过

### 6.2 中期（1 年）

- v1.0.0 正式发布
- 接入 casbin / Cedar 权限引擎
- 完善可观测性（tracing / metrics）
- 性能基准发布

### 6.3 长期（2 年+）

- Rust 生态最完整的鉴权框架
- 支持 GraphQL / gRPC
- 微服务网关集成
- 企业级多租户支持

---

## 七、参考

- [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md) - 完整实施计划
- [migration/MIGRATION_STATUS.md](./migration/MIGRATION_STATUS.md) - 迁移进度
- **Sa-Token Java 官方路线图**：https://sa-token.cc
- **easyexcel-rs 生态路线图**：`/Users/wandl/workspaces/workspace-github/easyexcel-rs/docs/ecosystem-roadmap.md`
