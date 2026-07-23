# 更新日志

> 本文档历史上是 [Sa-Token Java](https://github.com/dromara/Sa-Token) 的版本日志（v1.0.0 – v1.45.0）。
> Sa-Token-Rs 作为独立项目，不再沿用 Java 版本号。
>
> 当前 Sa-Token-Rs 发布版本：**v0.1.0**（首个 MVP 阶段）。
> Rust 侧增量更新日志位于仓库根目录的 [`CHANGELOG.md`](https://github.com/easy-4-rust/sa-token-rs/blob/main/CHANGELOG.md)。

## Sa-Token-Rs 历史

### v0.1.0 — 2026-07-21（首个 MVP）

**核心（`sa-token-core`）**

- 登录 / 登出 / Token 管理
- 会话管理（`SaSession`）
- 终端管理（`SaTerminalInfo`）
- 全局配置（`SaTokenConfig`）
- 存储接口（`SaTokenDao` trait）
- 事件监听（`SaTokenListener`）
- 日志接口（`SaLog`）
- 上下文抽象（`SaTokenContext`）

**内存存储**

- `sa-token-dao-memory` — 内存 DAO 实现
- `sa-token-context-mock` — Mock 上下文（测试用）
- `sa-token` — Facade crate

**权限与角色**

- 权限认证（`has_permission` / `check_permission`）
- 角色认证（`has_role` / `check_role`）
- 账号封禁（`disable` / `is_disable` / `untie_disable`）
- 二级认证（`open_safe` / `check_safe` / `close_safe`）
- 身份切换（`switch_to` / `end_switch`）

**注解宏（`sa-token-derive`）**

- `#[sa_check_login]`
- `#[sa_check_permission("x")]`
- `#[sa_check_role("x")]`
- `#[sa_check_safe]`
- `#[sa_check_disable]`
- `#[sa_ignore]`

**Web 适配（`sa-token-web-axum`）**

- `SaTokenLayer` 中间件
- `CurrentLoginId` / `OptionalLoginId` Extractor
- `RequirePermission` / `RequireRole` Layer

**存储扩展**

- `sa-token-dao-redis` — Redis DAO（基于 fred）

**插件**

- `sa-token-jwt` — JWT Token 生成与验证
- `sa-token-sign` — API 参数签名校验

**文档**

- 13 个文档文件 + GitHub Pages 自动部署

**测试**

- 48 个测试全部通过（Phase 1: 13 + Phase 2: 21 + Phase 3: 7 + Phase 5: 7）