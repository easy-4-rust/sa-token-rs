# Sa-Token-Rs 架构总览

> 本文档描述 Sa-Token-Rs 的整体架构、核心组件、以及与 Java Sa-Token 的对应关系。
> 实施细节请参阅 [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md)。

---

## 一、架构总览

### 1.1 分层架构

```text
┌─────────────────────────────────────────────────────────────┐
│  应用层（用户代码）                                            │
│    use sa_token::prelude::*;                                │
│    StpUtil::login("10001")?;                                │
└─────────────────────────┬───────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────┐
│  Facade 层：sa-token crate                                    │
│    对外统一入口，re-export 所有子 crate                        │
└─────────────────────────┬───────────────────────────────────┘
                          │
        ┌─────────────────┼─────────────────┐
        │                 │                 │
┌───────▼───────┐  ┌──────▼──────┐  ┌──────▼──────┐
│  核心层 Core   │  │  宏层 Derive │  │  适配层     │
│  sa-token-core│  │  sa-token-   │  │  starter/*  │
│  (同步)       │  │  derive      │  │  plugin/*   │
│               │  │              │  │  dao-*      │
│  StpLogic     │  │  #[sa_check] │  │  (async)    │
│  StpUtil      │  │  #[derive]   │  │             │
│  SaManager    │  │              │  │  AxumLayer  │
│  SaSession    │  └──────────────┘  │  Redis      │
│  SaTokenDao   │                    │  OAuth2     │
│  (trait)      │                    │  SSO        │
└───────────────┘                    └─────────────┘
```

### 1.2 核心设计原则

| 原则 | 决策 | 原因 |
|---|---|---|
| 异步模型 | **核心同步 + starter async** | 复刻 Java 阻塞语义；核心无 IO 时 async 无收益 |
| 全局状态 | `OnceLock<Arc<T>>` | 借鉴 easyexcel-rs；OnceLock 设置一次后只读 |
| 请求上下文 | `thread_local!`（核心）/ `task_local!`（async） | 对齐 Java ThreadLocal |
| 组件注入 | `Arc<dyn Trait>` + `SaManager::set_xxx()` | 对齐 Java SaManager |
| 错误处理 | `thiserror` 单一 enum | 折叠 Java 20+ 异常类；实现 Clone+Eq 用于测试 |
| 注解 AOP | proc-macro 编译期生成 | Rust 无运行时反射 |
| 元数据存储 | `&'static [T]` 常量 + `const fn` | 编译期烤进二进制，运行时零开销 |

---

## 二、Core Traits Java 映射

### 2.1 核心 Trait / 结构体映射表

| Java 类/接口 | Rust Trait/结构体 | 所在 crate | 说明 |
|---|---|---|---|
| `SaManager` | `SaManager` | `sa-token-core` | 全局组件注册中心 |
| `StpLogic` | `StpLogic` | `sa-token-core` | 核心逻辑（~150 方法） |
| `StpUtil` | `StpUtil` | `sa-token-core` | 静态门面 |
| `StpInterface` | `StpInterface` (trait) | `sa-token-core` | 权限数据源 |
| `SaTokenInfo` | `SaTokenInfo` | `sa-token-core` | Token 详情 |
| `SaLoginParameter` | `SaLoginParameter` | `sa-token-core` | 登录参数 |
| `SaLogoutParameter` | `SaLogoutParameter` | `sa-token-core` | 登出参数 |
| `SaTokenConfig` | `SaTokenConfig` | `sa-token-core` | 全局配置 |
| `SaCookieConfig` | `SaCookieConfig` | `sa-token-core` | Cookie 配置 |
| `SaHolder` | `SaHolder` | `sa-token-core` | 上下文门面 |
| `SaTokenContext` | `SaTokenContext` (trait) | `sa-token-core` | 上下文抽象 |
| `SaRequest` | `SaRequest` (trait) | `sa-token-core` | 请求抽象 |
| `SaResponse` | `SaResponse` (trait) | `sa-token-core` | 响应抽象 |
| `SaStorage` | `SaStorage` (trait) | `sa-token-core` | 存储抽象 |
| `SaCookie` | `SaCookie` | `sa-token-core` | Cookie 模型 |
| `SaTokenDao` | `SaTokenDao` (trait) | `sa-token-core` | 持久化接口 |
| `SaTokenDaoDefaultImpl` | `SaTokenDaoDefaultImpl` | `sa-token-core` | Memory 默认实现 |
| `SaSession` | `SaSession` | `sa-token-core` | 会话模型 |
| `SaTerminalInfo` | `SaTerminalInfo` | `sa-token-core` | 终端信息 |
| `SaTokenListener` | `SaTokenListener` (trait) | `sa-token-core` | 事件监听器 |
| `SaTokenEventCenter` | `SaTokenEventCenter` | `sa-token-core` | 事件分发器 |
| `SaLog` | `SaLog` (trait) | `sa-token-core` | 日志接口 |
| `SaJsonTemplate` | `SaJsonTemplate` (trait) | `sa-token-core` | JSON 模板 |
| `SaSerializerTemplate` | `SaSerializerTemplate` (trait) | `sa-token-core` | 序列化模板 |
| `SaStrategy` | `SaStrategy` | `sa-token-core` | 全局策略 |
| `SaFirewallStrategy` | `SaFirewallStrategy` | `sa-token-core` | 防火墙策略 |
| `SaRouter` | `SaRouter` | `sa-token-core` | 路由匹配 |
| `SaFoxUtil` | `SaFoxUtil` | `sa-token-core` | 工具集 |
| `SaResult` | `SaResult` | `sa-token-core` | 统一返回 |
| `SaTokenException` | `SaTokenException` (enum) | `sa-token-core` | 统一异常 |
| `SaOAuth2Manager` | `SaOAuth2Manager` | `sa-token-oauth2` | OAuth2 管理 |
| `SaSsoManager` | `SaSsoManager` | `sa-token-sso` | SSO 管理 |

### 2.2 注解映射

| Java 注解 | Rust 宏 | 所在 crate |
|---|---|---|
| `@SaCheckLogin` | `#[sa_check_login]` | `sa-token-derive` |
| `@SaCheckPermission` | `#[sa_check_permission("...")]` | `sa-token-derive` |
| `@SaCheckRole` | `#[sa_check_role("...")]` | `sa-token-derive` |
| `@SaCheckSafe` | `#[sa_check_safe]` | `sa-token-derive` |
| `@SaCheckDisable` | `#[sa_check_disable]` | `sa-token-derive` |
| `@SaCheckOr` | `#[sa_check_or(...)]` | `sa-token-derive` |
| `@SaCheckHttpBasic` | `#[sa_check_http_basic]` | `sa-token-derive` |
| `@SaCheckHttpDigest` | `#[sa_check_http_digest]` | `sa-token-derive` |
| `@SaIgnore` | `#[sa_ignore]` | `sa-token-derive` |

### 2.3 异常映射

Java 的 20+ 异常类折叠为单一 Rust enum：

| Java 异常类 | Rust enum variant |
|---|---|
| `SaTokenException` | `SaTokenException`（基类 enum） |
| `NotLoginException` | `SaTokenException::NotLogin` |
| `NotPermissionException` | `SaTokenException::NotPermission` |
| `NotRoleException` | `SaTokenException::NotRole` |
| `NotSafeException` | `SaTokenException::NotSafe` |
| `DisableServiceException` | `SaTokenException::DisableService` |
| `SameTokenInvalidException` | `SaTokenException::SameTokenInvalid` |
| `InvalidContextException` | `SaTokenException::InvalidContext` |
| `NotWebContextException` | `SaTokenException::NotWebContext` |
| `FirewallCheckException` | `SaTokenException::FirewallCheck` |
| `RequestPathInvalidException` | `SaTokenException::RequestPathInvalid` |
| `SaTokenPluginException` | `SaTokenException::Plugin` |
| `ApiDisabledException` | `SaTokenException::ApiDisabled` |
| `NotHttpBasicAuthException` | `SaTokenException::NotHttpBasicAuth` |
| `NotHttpDigestAuthException` | `SaTokenException::NotHttpDigestAuth` |
| `SaJsonConvertException` | `SaTokenException::JsonConvert` |
| `StopMatchException` | `SaTokenException::StopMatch` |
| `TotpAuthException` | `SaTokenException::TotpAuth` |

---

## 三、核心数据流

### 3.1 登录流程

```text
StpUtil::login("10001")
  │
  ▼
StpLogic::login(id)
  │
  ├──► createSaLoginParameter()          // 构造登录参数
  │
  ├──► createLoginSession(id, param)     // 创建登录会话
  │      │
  │      ├──► distUsableToken()          // 分配可用 Token
  │      │      │
  │      │      └──► getTokenValueByLoginId()  // 复用现有 Token（is_share=true）
  │      │
  │      ├──► saveTokenToIdMapping()     // 保存 token→loginId 映射
  │      │
  │      ├──► getSessionByLoginId()      // 获取账号 Session
  │      │      │
  │      │      └──► SaTokenDao::get_session()
  │      │
  │      ├──► session.addTerminal()      // 添加终端信息
  │      │
  │      ├──► session.update()           // 更新 Session 到持久层
  │      │      │
  │      │      └──► SaTokenDao::update_session()
  │      │
  │      └──► SaTokenEventCenter::doLogin()  // 触发登录事件
  │
  ├──► setTokenValue(token)              // 写入 Token 到当前请求
  │      │
  │      ├──► setTokenValueToStorage()   // 写入 Storage
  │      ├──► setTokenValueToCookie()    // 写入 Cookie
  │      └──► setTokenValueToResponseHeader()  // 写入响应头
  │
  └──► setLastActiveToNow()              // 更新最后活跃时间
```

### 3.2 请求鉴权流程

```text
HTTP 请求进入
  │
  ▼
SaTokenMiddleware（Layer）
  │
  ├──► 构造 AxumContext { request, response, storage }
  ├──► CURRENT_CONTEXT.with(|ctx| ctx.set(context))  // 注入 thread_local
  │
  ▼
Handler 函数
  │
  ├──► #[sa_check_login] 宏生成的检查代码
  │      │
  │      └──► StpUtil::check_login()
  │             │
  │             ├──► get_token_value()      // 从请求读 Token
  │             ├──► get_login_id_by_token() // 查询 loginId
  │             └──► 校验通过 → 继续；失败 → 返回 NotLogin 异常
  │
  └──► 业务逻辑
```

---

## 四、Crate 依赖关系

```text
                         ┌─────────────┐
                         │  sa-token   │ ← 用户唯一依赖
                         │  (facade)   │
                         └──────┬──────┘
                                │
        ┌───────────┬───────────┼───────────┬───────────┐
        │           │           │           │           │
┌───────▼──────┐ ┌──▼──────┐ ┌──▼────────┐ ┌▼────────┐ ┌▼──────────┐
│sa-token-core│ │derive   │ │context-mock│ │dao-memory│ │dao-redis  │
│   (基础)     │ │(proc-macro)│ │(测试用)   │ │(默认)    │ │(async)    │
└──────┬───────┘ └────┬────┘ └─────┬──────┘ └────┬────┘ └─────┬─────┘
       │              │            │              │            │
       │              │            └──────────────┼────────────┘
       │              │                           │
       │              │     ┌─────────────────────┘
       │              │     │
       │              │  ┌──▼──────────────────────────────────┐
       │              │  │  Web 框架适配 / 插件                  │
       │              │  │  web/sa-token-web-axum                   │
       │              │  │  web/sa-token-web-actix              │
       │              │  │  web/sa-token-web-salvo                  │
       │              │  │  plugin/sa-token-jwt                 │
       │              │  │  plugin/sa-token-sign                │
       │              │  │  plugin/sa-token-oauth2              │
       │              │  │  plugin/sa-token-sso                 │
       │              │  │  plugin/sa-token-apikey              │
       │              │  └──────────────────────────────────────┘
       │              │
       └──────────────┘
```

**依赖方向原则**：
- `sa-token-core` 是底层基础，**不依赖**任何其他 workspace crate
- `sa-token-derive` 仅在 dev-dependencies 引 core
- 所有 dao / web / plugin 都依赖 core
- facade `sa-token` 依赖全部

---

## 五、Web 框架适配设计

### 5.1 统一 Layer 抽象

所有 Web 框架适配基于 `tower::Layer`：

```rust
// sa-token-web-axum
pub struct SaTokenAxumLayer { /* config */ }
impl<S> Layer<S> for SaTokenAxumLayer { ... }

// 中间件职责：
// 1. 把 Request/Response 包装为 SaRequest/SaResponse
// 2. 注入到 thread_local CURRENT_CONTEXT
// 3. 调用 next.run(req)
```

### 5.2 四层路由保护 API

| 层级 | API | 场景 |
|---|---|---|
| **宏** | `#[sa_check_permission("user:add")]` | 对齐 Java 注解，Sa-Token 用户无缝迁移 |
| **Extractor** | `RequirePermission<"user:add">` | Rust idiomatic，类型安全 |
| **Layer** | `.layer(RequirePermissionLayer::new("admin"))` | 批量保护路由组 |
| **策略文件** | `SaPolicyLayer::from_file("policy.ron")` | 复杂 ABAC，对接 casbin/cedar |

---

## 六、存储层设计

### 6.1 SaTokenDao Trait 层次

```text
SaTokenDao (trait, 同步)
  │
  ├──► SaTokenDaoDefaultImpl     // Memory + TTL，core 内置
  │
  ├──► SaTokenDaoMemory          // sa-token-dao-memory crate（独立）
  │
  ├──► SaTokenDaoRedis           // sa-token-dao-redis crate（async，基于 fred）
  │      │
  │      └──► 内部用 block_on 适配同步 trait
  │
  └──► SaTokenDaoMoka            // sa-token-dao-moka crate（基于 moka 缓存）
```

### 6.2 Redis Pub/Sub 踢下线

```text
踢下线触发                        其他节点实时感知
─────────────────                ─────────────────
StpUtil::kickout("10001")        Redis 订阅 sa:kickout 频道
  │                                │
  ├─► 更新 auth_hash             ├─► 收到 KickoutEvent
  └─► publish sa:kickout event     └─► 清理本地 Session 缓存
                                     └─► 推送 SSE/WebSocket 给前端
```

---

## 七、参考

- [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md) - 完整实施计划
- [GUIDE.md](./GUIDE.md) - 使用指南
- [migration/object-method-matrix.md](./migration/object-method-matrix.md) - 方法级对照表
- **easyexcel-rs** 架构参考：`/Users/wandl/workspaces/workspace-github/easyexcel-rs/docs/ARCHITECTURE.md`
