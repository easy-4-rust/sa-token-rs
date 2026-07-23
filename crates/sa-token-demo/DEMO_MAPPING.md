# Java `sa-token-demo` ↔ Rust 对照表

> 框架映射：Spring Boot / WebFlux / SSM 等 → **axum**；第二 Web 栈（Quarkus 风格）→ **actix-web**。  
> Jackson → **serde**（`AjaxJson`）。  
> 版本号变体合并到同一 Rust demo，在「备注」列说明。

## 状态说明

| 状态 | 含义 |
|------|------|
| covered | 已有对应 Rust demo |
| skipped | Rust 无对等能力，不移植 |

## Wave 7 账本收口（2026-07-23）

按场景将 `docs/migration/file-map.csv` 中可映射条目由 `planned` 标为 `complete`（更新 `rust_file` / `test_evidence`），**不**硬搬 `sa-token-demo-suite` 356 文件。

| 优先级 | Java 场景 | Rust 双轨 crate | 账本条目 |
|--------|-----------|-----------------|----------|
| P0 | case | `sa-token-demo-axum-case` / `actix-case` | 42 |
| P0 | oauth2（server + client） | `axum-oauth2` / `actix-oauth2` + client | 18 |
| P0 | sso（server + 多 client 变体） | `axum-sso` / `actix-sso` + client | 32 |
| P1 | jwt | `axum-jwt` / `actix-jwt` | 5 |
| P1 | apikey | `axum-apikey` / `actix-apikey` | 8 |
| P1 | device-lock | `axum-device-lock` / `actix-device-lock` | 8 |
| P1 | sse | `axum-sse` / `actix-sse` | 7 |
| P1 | alone-redis（含 sb4 变体） | `axum-alone-redis` / `actix-alone-redis` | 5 |
| P1 | remember-me | `axum-remember-me` / `actix-remember-me` | 2 |
| — | sign | `axum-sign` / `actix-sign` | Java 无独立模块；Rust 独有 demo |

测试证据：`crates/sa-token-demo/sa-token-demo-axum-*/tests/demo_compile_test.rs`（各场景 compile contract）。

仍保持 **planned**：dubbo、grpc、solon、ssm、caffeine、beetl、cluster、springboot/webflux 等未双轨化的 Java 模块。

## 全量对照（按 Java 模块）

| Java 模块 | Rust crate(s) | 状态 | 备注 |
|-----------|---------------|------|------|
| `sa-token-demo-springboot` | `sa-token-demo-axum`, `sa-token-demo-actix-web` | covered | 主示例 |
| `sa-token-demo-springboot-low-version` | 同上 | covered | 版本变体 |
| `sa-token-demo-webflux` | 同上 | covered | 响应式变体 → axum/actix |
| `sa-token-demo-webflux-springboot3` | 同上 | covered | 版本变体 |
| `sa-token-demo-webflux-springboot4` | 同上 | covered | 版本变体 |
| `sa-token-demo-test` | 同上 | covered | 核心登录/权限与 springboot 同类 |
| `sa-token-demo-springboot-redis` | `sa-token-demo-axum-redis`, `sa-token-demo-actix-redis` | covered | |
| `sa-token-demo-springboot3-redis` | 同上 | covered | 版本变体 |
| `sa-token-demo-springboot4-redis` | 同上 | covered | 版本变体 |
| `sa-token-demo-springboot-redisson` | 同上 | covered | Redisson → redis-rs |
| `sa-token-demo-alone-redis` | `sa-token-demo-axum-alone-redis`, `sa-token-demo-actix-alone-redis` | covered | Sa-Token 独立 Redis + 业务 Redis |
| `sa-token-demo-alone-redis-sb4` | 同上 | covered | 版本变体 |
| `sa-token-demo-alone-redis-cluster` | — | skipped | `sa-token-dao-redis` 未暴露 cluster 专用适配 |
| `sa-token-demo-jwt` | `sa-token-demo-axum-jwt`, `sa-token-demo-actix-jwt` | covered | |
| `sa-token-demo-apikey` | `sa-token-demo-axum-apikey`, `sa-token-demo-actix-apikey` | covered | |
| `sa-token-demo-oauth2/oauth2-server` | `sa-token-demo-axum-oauth2`, `sa-token-demo-actix-oauth2` | covered | |
| `sa-token-demo-oauth2/oauth2-client` | `sa-token-demo-axum-oauth2-client`, `sa-token-demo-actix-oauth2-client` | covered | |
| `sa-token-demo-oauth2-*-h5` | — | skipped | 前端静态资源 |
| `sa-token-demo-sso/sso-server` | `sa-token-demo-axum-sso`, `sa-token-demo-actix-sso` | covered | |
| `sa-token-demo-sso/sso3-client` | `sa-token-demo-axum-sso-client`, `sa-token-demo-actix-sso-client` | covered | Mode3 代表 |
| `sa-token-demo-sso/sso1-client` | 同上 | covered | 映射到 sso-client |
| `sa-token-demo-sso/sso2-client` | 同上 | covered | 映射到 sso-client |
| `sa-token-demo-sso/sso3-client-anon` | 同上 | covered | 映射到 sso-client |
| `sa-token-demo-sso/sso3-client-nosdk` | 同上 | covered | 映射到 sso-client |
| `sa-token-demo-sso/sso3-client-resdk` | 同上 | covered | 映射到 sso-client |
| `sa-token-demo-sso-*-h5` / `*-vue*` | — | skipped | 前端 |
| `sa-token-demo-sso-for-solon/*` | — | skipped | Solon 专用 |
| `sa-token-demo-websocket` | `sa-token-demo-axum-websocket`, `sa-token-demo-actix-websocket` | covered | |
| `sa-token-demo-websocket-spring` | 同上 | covered | 变体 |
| `sa-token-demo-case` | `sa-token-demo-axum-case`, `sa-token-demo-actix-case` | covered | 综合场景 |
| `sa-token-demo-async` | `sa-token-demo-axum-async`, `sa-token-demo-actix-async` | covered | |
| `sa-token-demo-remember-me/*` | `sa-token-demo-axum-remember-me`, `sa-token-demo-actix-remember-me` | covered | |
| `sa-token-demo-device-lock` | `sa-token-demo-axum-device-lock`, `sa-token-demo-actix-device-lock` | covered | 业务层模拟信任设备（无 `isTrustDeviceId` API） |
| `sa-token-demo-device-lock-h5` | — | skipped | 前端 |
| `sa-token-demo-quick-login` | `sa-token-demo-axum-quick-login`, `sa-token-demo-actix-quick-login` | covered | |
| `sa-token-demo-quick-login-sb3` | 同上 | covered | 版本变体 |
| `sa-token-demo-sse` | `sa-token-demo-axum-sse`, `sa-token-demo-actix-sse` | covered | |
| （签名能力 / Secure） | `sa-token-demo-axum-sign`, `sa-token-demo-actix-sign` | covered | Java 无独立模块；Rust 有 `sa-token-sign` |
| `sa-token-demo-caffeine` | — | skipped | 无 `sa-token-dao-moka` |
| `sa-token-demo-hutool-timed-cache` | — | skipped | 无 `sa-token-dao-moka` |
| `sa-token-demo-beetl` | — | skipped | 无 Beetl 引擎适配 |
| `sa-token-demo-freemarker` | `sa-token-demo-tera`（+ `sa-token-tera` 插件） | covered | Freemarker 标签 → Tera；独立于本计划双轨 Web demo |
| `sa-token-demo-thymeleaf` | — | skipped | 无 Thymeleaf 标签集成 |
| `sa-token-demo-bom-import` | — | skipped | Maven BOM |
| `sa-token-demo-dubbo/*` | — | skipped | 无 Dubbo 适配 |
| `sa-token-demo-grpc`（注释） | — | skipped | 无 gRPC 适配 |
| `sa-token-demo-solon` | — | skipped | Solon |
| `sa-token-demo-solon-redisson` | — | skipped | Solon |
| `sa-token-demo-loveqq-boot` | — | skipped | Loveqq |
| `sa-token-demo-ssm` | — | skipped | SSM |

## 端口一览

| 场景 | axum | actix |
|------|------|-------|
| 主示例 | 8081 | 8082 |
| jwt | 8083 | 8084 |
| redis | 8085 | 8086 |
| websocket | 8097 | 8098 |
| apikey | 8091 | 8092 |
| oauth2-server | 8093 | 8094 |
| sso-server | 8095 | 8096 |
| case | 8101 | 8102 |
| async | 8103 | 8104 |
| remember-me | 8105 | 8106 |
| device-lock | 8107 | 8108 |
| quick-login | 8109 | 8110 |
| sse | 8111 | 8112 |
| alone-redis | 8113 | 8114 |
| oauth2-client | 8115 | 8116 |
| sso-client | 8117 | 8118 |
| sign | 8119 | 8120 |

## 运行

```bash
cargo run -p sa-token-demo-axum-case
cargo run -p sa-token-demo-actix-case
# Redis 类需本机 Redis，可用 REDIS_URL 覆盖
cargo run -p sa-token-demo-axum-alone-redis
```
