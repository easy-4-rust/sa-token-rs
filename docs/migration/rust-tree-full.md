# Sa-Token-Rs 完整目录树

> 目标结构，参照 [IMPLEMENTATION_PLAN.md](../IMPLEMENTATION_PLAN.md) 的设计。
> 标注 ⬜ 表示待实现，✅ 表示已完成。

---

## Workspace 根

```text
sa-token-rs/
├── Cargo.toml                         ⬜ [workspace] 根清单
├── README.md                          ⬜
├── CHANGELOG.md                       ⬜
├── .gitignore                         ⬜
├── .github/workflows/ci.yml           ⬜
├── scripts/
│   ├── coverage.sh                    ⬜
│   ├── gap-check.sh                   ⬜
│   └── java-golden-export/            ⬜
├── docs/
│   ├── IMPLEMENTATION_PLAN.md         ✅
│   ├── ARCHITECTURE.md                ✅
│   ├── GUIDE.md                       ✅
│   ├── compatibility.md               ✅
│   ├── ecosystem-roadmap.md           ✅
│   └── migration/                     ✅
│       ├── MIGRATION_STATUS.md        ✅
│       ├── object-method-matrix.md    ✅
│       ├── CODEGRAPH_METHOD_MAP.md    ✅
│       ├── java-tree-full.md          ✅
│       ├── rust-tree-full.md          ✅（本文档）
│       ├── project-tree-diff.md       ✅
│       └── TEST_AUDIT_REPORT.md       ✅
│
└── crates/
    ├── sa-token/                      ⬜ facade
    ├── sa-token-core/                 ⬜ 核心库
    ├── sa-token-web/
    │   ├── sa-token-web-axum/         ⬜ axum 适配
    │   ├── sa-token-web-actix/        ⬜ actix 适配
    │   └── sa-token-web-salvo/        ⬜ salvo 适配
    ├── sa-token-dao/
    │   ├── sa-token-dao-memory/       ⬜ Memory DAO
    │   ├── sa-token-dao-redis/        ⬜ Redis DAO
    │   └── sa-token-dao-moka/         ⬜ moka DAO（待）
    ├── sa-token-support/
    │   ├── sa-token-derive/           ⬜ proc-macro
    │   └── sa-token-context-mock/     ⬜ Mock 上下文
    ├── sa-token-plugin/
    │   ├── sa-token-jwt/              ⬜ JWT 插件
    │   ├── sa-token-sign/             ⬜ Sign 插件
    │   ├── sa-token-oauth2/           ⬜ OAuth2 插件
    │   ├── sa-token-sso/              ⬜ SSO 插件
    │   └── sa-token-apikey/           ⬜ ApiKey 插件
    ├── sa-token-test/                 # 对齐 Java sa-token-test（pom 合集）
    │   ├── sa-token-easy-test/
    │   ├── sa-token-springboot-test/  # Spring Boot → axum
    │   ├── sa-token-jwt-test/
    │   ├── sa-token-temp-jwt-test/
    │   ├── sa-token-json-test/
    │   ├── sa-token-jackson3-test/    # Jackson3 → serde
    │   └── sa-token-serializer-test/
    └── sa-token-demo/
        ├── sa-token-demo-axum/        ⬜ axum 示例
        ├── sa-token-demo-actix-web/   ⬜ actix 示例
        └── sa-token-demo-salvo/       ⬜ salvo 示例
```

---

## sa-token-core/src/ 详细目录

```text
sa-token-core/src/
├── lib.rs                             ⬜ 对外 re-export
├── manager.rs                         ⬜ ← SaManager.java
│
├── annotation/
│   └── mod.rs                         ⬜ SaMode 枚举
│
├── application/
│   ├── mod.rs                         ⬜
│   ├── application_info.rs            ⬜ ← ApplicationInfo.java
│   ├── sa_application.rs              ⬜ ← SaApplication.java
│   ├── sa_get_value_interface.rs      ⬜ ← SaGetValueInterface.java
│   └── sa_set_value_interface.rs      ⬜ ← SaSetValueInterface.java
│
├── config/
│   ├── mod.rs                         ⬜
│   ├── sa_token_config.rs             ⬜ ← SaTokenConfig.java
│   ├── sa_cookie_config.rs            ⬜ ← SaCookieConfig.java
│   └── sa_token_config_factory.rs     ⬜ ← SaTokenConfigFactory.java
│
├── context/
│   ├── mod.rs                         ⬜
│   ├── sa_holder.rs                   ⬜ ← SaHolder.java
│   ├── sa_token_context.rs            ⬜ ← SaTokenContext.java
│   ├── sa_token_context_default_impl.rs ⬜ ← SaTokenContextDefaultImpl.java
│   ├── sa_token_context_for_read_only.rs ⬜ ← SaTokenContextForReadOnly.java
│   ├── sa_token_context_for_thread_local.rs ⬜ ← SaTokenContextForThreadLocal.java
│   └── model/
│       ├── mod.rs                     ⬜
│       ├── sa_cookie.rs               ⬜ ← SaCookie.java
│       ├── sa_request.rs              ⬜ ← SaRequest.java
│       ├── sa_response.rs             ⬜ ← SaResponse.java
│       ├── sa_storage.rs              ⬜ ← SaStorage.java
│       ├── sa_http_method.rs          ⬜ ← SaHttpMethod.java
│       └── sa_token_context_model_box.rs ⬜ ← SaTokenContextModelBox.java
│
├── dao/
│   ├── mod.rs                         ⬜
│   ├── sa_token_dao.rs                ⬜ ← SaTokenDao.java（trait）
│   └── sa_token_dao_default_impl.rs   ⬜ ← SaTokenDaoDefaultImpl.java
│
├── error/
│   └── mod.rs                         ⬜ ← SaErrorCode.java
│
├── exception/
│   └── mod.rs                         ⬜ ← SaTokenException（单一 enum）
│
├── filter/
│   ├── mod.rs                         ⬜
│   ├── sa_filter.rs                   ⬜ ← SaFilter.java
│   ├── sa_filter_auth_strategy.rs     ⬜
│   └── sa_filter_error_strategy.rs    ⬜
│
├── fun/
│   ├── mod.rs                         ⬜
│   ├── strategy/                      ⬜
│   └── hooks/                         ⬜
│
├── http/
│   ├── mod.rs                         ⬜
│   ├── sa_http_template.rs            ⬜
│   ├── sa_http_template_default_impl.rs ⬜
│   └── sa_http_util.rs               ⬜
│
├── httpauth/
│   ├── basic/                         ⬜
│   └── digest/                        ⬜
│
├── json/
│   ├── mod.rs                         ⬜
│   ├── sa_json_template.rs            ⬜
│   └── sa_json_template_default_impl.rs ⬜
│
├── listener/
│   ├── mod.rs                         ⬜
│   ├── sa_token_listener.rs           ⬜
│   ├── sa_token_event_center.rs       ⬜
│   ├── sa_token_listener_for_log.rs   ⬜
│   └── sa_token_listener_for_simple.rs ⬜
│
├── log/
│   ├── mod.rs                         ⬜
│   ├── sa_log.rs                      ⬜
│   └── sa_log_for_console.rs          ⬜
│
├── model/
│   └── wrapper_info/
│       └── sa_disable_wrapper_info.rs ⬜
│
├── plugin/
│   ├── mod.rs                         ⬜
│   ├── sa_token_plugin.rs             ⬜
│   ├── sa_token_plugin_holder.rs      ⬜
│   └── sa_token_plugin_hook_model.rs  ⬜
│
├── router/
│   ├── mod.rs                         ⬜
│   ├── sa_router.rs                   ⬜
│   └── sa_router_staff.rs             ⬜
│
├── same/
│   ├── mod.rs                         ⬜
│   ├── sa_same_template.rs            ⬜
│   └── sa_same_util.rs               ⬜
│
├── secure/
│   ├── mod.rs                         ⬜
│   ├── b_crypt.rs                     ⬜
│   ├── sa_base32_util.rs             ⬜
│   ├── sa_base64_util.rs             ⬜
│   ├── sa_secure_util.rs             ⬜
│   └── totp/                          ⬜
│
├── serializer/
│   ├── mod.rs                         ⬜
│   ├── sa_serializer_template.rs      ⬜
│   └── impl/                          ⬜
│
├── session/
│   ├── mod.rs                         ⬜
│   ├── sa_session.rs                  ⬜
│   ├── sa_session_custom_util.rs      ⬜
│   ├── sa_terminal_info.rs            ⬜
│   └── raw/                           ⬜
│
├── stp/
│   ├── mod.rs                         ⬜
│   ├── stp_logic.rs                   ⬜ ← StpLogic.java ★
│   ├── stp_util.rs                    ⬜ ← StpUtil.java ★
│   ├── stp_interface.rs               ⬜ ← StpInterface.java
│   ├── stp_interface_default_impl.rs  ⬜
│   ├── sa_token_info.rs               ⬜
│   ├── sa_login_config.rs             ⬜
│   ├── sa_login_model.rs              ⬜
│   └── parameter/
│       ├── mod.rs                     ⬜
│       ├── sa_login_parameter.rs      ⬜
│       ├── sa_logout_parameter.rs     ⬜
│       └── enums/
│           ├── sa_logout_mode.rs      ⬜
│           ├── sa_logout_range.rs     ⬜
│           ├── sa_replaced_login_exit_mode.rs ⬜
│           └── sa_replaced_range.rs   ⬜
│
├── strategy/
│   ├── mod.rs                         ⬜
│   ├── sa_strategy.rs                 ⬜
│   ├── sa_annotation_strategy.rs      ⬜
│   ├── sa_firewall_strategy.rs        ⬜
│   └── hooks/                         ⬜
│
├── temp/
│   ├── mod.rs                         ⬜
│   ├── sa_temp_template.rs            ⬜
│   └── sa_temp_util.rs               ⬜
│
└── util/
    ├── mod.rs                         ⬜
    ├── sa_fox_util.rs                 ⬜
    ├── sa_hex_util.rs                 ⬜
    ├── sa_result.rs                   ⬜
    ├── sa_sugar.rs                    ⬜
    ├── sa_token_consts.rs             ⬜
    ├── sa_ttl_methods.rs              ⬜
    ├── sa_value2_box.rs               ⬜
    └── str_formatter.rs               ⬜
```

---

## 参考

- [java-tree-full.md](./java-tree-full.md) - Java 完整目录树
- [project-tree-diff.md](./project-tree-diff.md) - 两侧目录 Diff
- [MIGRATION_STATUS.md](./MIGRATION_STATUS.md) - 迁移进度
