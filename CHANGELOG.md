# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-07-21

### Added

#### Phase 1: MVP 核心
- `sa-token-core` — 核心库
  - 登录/登出/Token 管理
  - 会话管理（SaSession）
  - 终端管理（SaTerminalInfo）
  - 全局配置（SaTokenConfig）
  - 存储接口（SaTokenDao trait）
  - 事件监听（SaTokenListener）
  - 日志接口（SaLog）
  - 上下文抽象（SaTokenContext）
- `sa-token-dao-memory` — 内存 DAO 实现
- `sa-token-context-mock` — Mock 上下文（测试用）
- `sa-token` — Facade crate

#### Phase 2: 权限/角色/注解
- 权限认证（has_permission/check_permission）
- 角色认证（has_role/check_role）
- 账号封禁（disable/is_disable/untie_disable）
- 二级认证（open_safe/check_safe/close_safe）
- 身份切换（switch_to/end_switch）
- `sa-token-derive` — proc-macro 注解
  - `#[sa_check_login]`
  - `#[sa_check_permission("x")]`
  - `#[sa_check_role("x")]`
  - `#[sa_check_safe]`
  - `#[sa_check_disable]`
  - `#[sa_ignore]`

#### Phase 3: Web 框架适配
- `sa-token-axum` — axum 集成
  - SaTokenLayer 中间件
  - CurrentLoginId Extractor
  - OptionalLoginId Extractor
  - RequirePermission/RequireRole Layer

#### Phase 4: 存储扩展
- `sa-token-dao-redis` — Redis DAO 实现

#### Phase 5: 插件生态
- `sa-token-jwt` — JWT Token 生成与验证
- `sa-token-sign` — API 参数签名校验

#### Phase 6: 文档
- 12 个文档文件（4634 行）
- README.md
- CHANGELOG.md

### Tests

- 48 个测试全部通过
- Phase 1: 13 个测试（登录/登出/Token/会话）
- Phase 2: 21 个测试（权限/角色/禁用/安全/切换）
- Phase 3: 7 个测试（axum 适配）
- Phase 5: 7 个测试（JWT）

[0.1.0]: https://github.com/dromara/sa-token-rs/releases/tag/v0.1.0
