# Sa-Token Java ↔ Sa-Token-Rs 目录 Diff

> 本文档对比 Java Sa-Token `sa-token-core` 与 Rust Sa-Token-Rs `sa-token-core` 的目录结构差异。
> 基线：Java `dev` 分支 `89e47c12`，Rust 参照 [IMPLEMENTATION_PLAN.md](../IMPLEMENTATION_PLAN.md)。

---

## 完整 Diff

```diff
 sa-token-core/src/
 ├── lib.rs                              (Java 无对应，Rust 入口)
 ├── manager.rs                          ← SaManager.java
 │
 ├── annotation/
 │   └── mod.rs                          ← annotation/SaMode.java（仅保留枚举）
-│   ├── SaCheckDisable.java             → 移至 sa-token-derive crate
-│   ├── SaCheckHttpBasic.java           → 移至 sa-token-derive crate
-│   ├── SaCheckHttpDigest.java          → 移至 sa-token-derive crate
-│   ├── SaCheckLogin.java               → 移至 sa-token-derive crate
-│   ├── SaCheckOr.java                  → 移至 sa-token-derive crate
-│   ├── SaCheckPermission.java          → 移至 sa-token-derive crate
-│   ├── SaCheckRole.java                → 移至 sa-token-derive crate
-│   ├── SaCheckSafe.java                → 移至 sa-token-derive crate
-│   ├── SaIgnore.java                   → 移至 sa-token-derive crate
-│   └── handler/
-│       ├── SaAnnotationHandlerInterface.java  → 移至 sa-token-derive crate
-│       ├── SaCheckDisableHandler.java         → 移至 sa-token-derive crate
-│       ├── ...（10 个 Handler）                → 移至 sa-token-derive crate
 │
 ├── application/
 │   ├── mod.rs
 │   ├── application_info.rs            ← ApplicationInfo.java
 │   ├── sa_application.rs              ← SaApplication.java
 │   ├── sa_get_value_interface.rs      ← SaGetValueInterface.java
 │   └── sa_set_value_interface.rs      ← SaSetValueInterface.java
 │
 ├── config/
 │   ├── mod.rs
 │   ├── sa_token_config.rs             ← SaTokenConfig.java
 │   ├── sa_cookie_config.rs            ← SaCookieConfig.java
 │   └── sa_token_config_factory.rs     ← SaTokenConfigFactory.java
 │
 ├── context/
 │   ├── mod.rs
 │   ├── sa_holder.rs                   ← SaHolder.java
 │   ├── sa_token_context.rs            ← SaTokenContext.java
 │   ├── sa_token_context_default_impl.rs  ← SaTokenContextDefaultImpl.java
 │   ├── sa_token_context_for_read_only.rs ← SaTokenContextForReadOnly.java
 │   ├── sa_token_context_for_thread_local.rs ← SaTokenContextForThreadLocal.java
-│   ├── SaTokenContextForThreadLocalStaff.java → 合并到 for_thread_local.rs
 │   └── model/
 │       ├── mod.rs
 │       ├── sa_cookie.rs               ← SaCookie.java
 │       ├── sa_request.rs              ← SaRequest.java
 │       ├── sa_response.rs             ← SaResponse.java
 │       ├── sa_storage.rs              ← SaStorage.java
 │       ├── sa_http_method.rs          ← router/SaHttpMethod.java（位置变更）
 │       └── sa_token_context_model_box.rs ← SaTokenContextModelBox.java
-│   └── mock/                           → 移至独立 sa-token-context-mock crate
 │
 ├── dao/
 │   ├── mod.rs
 │   ├── sa_token_dao.rs                ← SaTokenDao.java
 │   ├── sa_token_dao_default_impl.rs   ← SaTokenDaoDefaultImpl.java
 │   ├── auto/
-│   │   └── 3 个辅助类                  → 合并到 default_impl 或删除
 │   └── timed_cache/
-│       └── 3 个缓存类                  → 合并到 default_impl 或删除
 │
 ├── error/
 │   └── mod.rs                          ← SaErrorCode.java
 │
 ├── exception/
 │   └── mod.rs                          ← SaTokenException.java（折叠为单一 enum）
-│   ├── 20 个独立异常类                   → 全部折叠为 SaTokenException enum variant
 │
 ├── filter/
 │   ├── mod.rs
 │   ├── sa_filter.rs                   ← SaFilter.java
 │   ├── sa_filter_auth_strategy.rs     ← SaFilterAuthStrategy.java
 │   └── sa_filter_error_strategy.rs    ← SaFilterErrorStrategy.java
 │
 ├── fun/
 │   ├── mod.rs
 │   ├── strategy/                      ← fun/strategy/（15 个函数式接口）
 │   └── hooks/                         ← fun/hooks/
 │
 ├── http/
 │   ├── mod.rs
 │   ├── sa_http_template.rs            ← SaHttpTemplate.java
 │   ├── sa_http_template_default_impl.rs ← SaHttpTemplateDefaultImpl.java
 │   └── sa_http_util.rs               ← SaHttpUtil.java
 │
 ├── httpauth/
 │   ├── basic/
 │   │   ├── sa_http_basic_account.rs   ← SaHttpBasicAccount.java
 │   │   ├── sa_http_basic_template.rs  ← SaHttpBasicTemplate.java
 │   │   └── sa_http_basic_util.rs      ← SaHttpBasicUtil.java
 │   └── digest/
 │       ├── sa_http_digest_model.rs    ← SaHttpDigestModel.java
 │       ├── sa_http_digest_template.rs ← SaHttpDigestTemplate.java
 │       └── sa_http_digest_util.rs     ← SaHttpDigestUtil.java
 │
 ├── json/
 │   ├── mod.rs
 │   ├── sa_json_template.rs            ← SaJsonTemplate.java
 │   └── sa_json_template_default_impl.rs ← SaJsonTemplateDefaultImpl.java
 │
 ├── listener/
 │   ├── mod.rs
 │   ├── sa_token_listener.rs           ← SaTokenListener.java
 │   ├── sa_token_event_center.rs       ← SaTokenEventCenter.java
 │   ├── sa_token_listener_for_log.rs   ← SaTokenListenerForLog.java
 │   └── sa_token_listener_for_simple.rs ← SaTokenListenerForSimple.java
 │
 ├── log/
 │   ├── mod.rs
 │   ├── sa_log.rs                      ← SaLog.java
 │   └── sa_log_for_console.rs          ← SaLogForConsole.java
 │
 ├── model/
 │   └── wrapper_info/
 │       └── sa_disable_wrapper_info.rs ← SaDisableWrapperInfo.java
 │
 ├── plugin/
 │   ├── mod.rs
 │   ├── sa_token_plugin.rs             ← SaTokenPlugin.java
 │   ├── sa_token_plugin_holder.rs      ← SaTokenPluginHolder.java
 │   └── sa_token_plugin_hook_model.rs  ← SaTokenPluginHookModel.java
 │
 ├── router/
 │   ├── mod.rs
 │   ├── sa_router.rs                   ← SaRouter.java
 │   └── sa_router_staff.rs             ← SaRouterStaff.java
-│   └── SaHttpMethod.java              → 移至 context/model/sa_http_method.rs
 │
 ├── same/
 │   ├── mod.rs
 │   ├── sa_same_template.rs            ← SaSameTemplate.java
 │   └── sa_same_util.rs               ← SaSameUtil.java
 │
 ├── secure/
 │   ├── mod.rs
 │   ├── b_crypt.rs                     ← BCrypt.java
 │   ├── sa_base32_util.rs             ← SaBase32Util.java
 │   ├── sa_base64_util.rs             ← SaBase64Util.java
 │   ├── sa_secure_util.rs             ← SaSecureUtil.java
 │   └── totp/
 │       ├── sa_totp_template.rs        ← SaTotpTemplate.java
 │       └── sa_totp_util.rs            ← SaTotpUtil.java
 │
 ├── serializer/
 │   ├── mod.rs
 │   ├── sa_serializer_template.rs      ← SaSerializerTemplate.java
 │   └── impl/
-│       ├── 5 个 JDK 序列化实现          → 简化为 1~2 个（serde_json 为主）
 │       ├── sa_serializer_template_for_json.rs ← SaSerializerTemplateForJson.java
 │       └── *.rs
 │
 ├── session/
 │   ├── mod.rs
 │   ├── sa_session.rs                  ← SaSession.java
 │   ├── sa_session_custom_util.rs      ← SaSessionCustomUtil.java
 │   ├── sa_terminal_info.rs            ← SaTerminalInfo.java
 │   └── raw/
 │       ├── sa_raw_session_delegator.rs ← SaRawSessionDelegator.java
 │       └── sa_raw_session_util.rs     ← SaRawSessionUtil.java
 │
 ├── stp/
 │   ├── mod.rs
 │   ├── stp_logic.rs                   ← StpLogic.java
 │   ├── stp_util.rs                    ← StpUtil.java
 │   ├── stp_interface.rs               ← StpInterface.java
 │   ├── stp_interface_default_impl.rs  ← StpInterfaceDefaultImpl.java
 │   ├── sa_token_info.rs               ← SaTokenInfo.java
 │   ├── sa_login_config.rs             ← SaLoginConfig.java
 │   ├── sa_login_model.rs              ← SaLoginModel.java
 │   └── parameter/
 │       ├── mod.rs
 │       ├── sa_login_parameter.rs      ← SaLoginParameter.java
 │       ├── sa_logout_parameter.rs     ← SaLogoutParameter.java
 │       └── enums/
 │           ├── sa_logout_mode.rs      ← SaLogoutMode.java
 │           ├── sa_logout_range.rs     ← SaLogoutRange.java
 │           ├── sa_replaced_login_exit_mode.rs ← SaReplacedLoginExitMode.java
 │           └── sa_replaced_range.rs   ← SaReplacedRange.java
 │
 ├── strategy/
 │   ├── mod.rs
 │   ├── sa_strategy.rs                 ← SaStrategy.java
 │   ├── sa_annotation_strategy.rs      ← SaAnnotationStrategy.java
 │   ├── sa_firewall_strategy.rs        ← SaFirewallStrategy.java
 │   └── hooks/                         ← strategy/hooks/（9 个 Hook）
 │
 ├── temp/
 │   ├── mod.rs
 │   ├── sa_temp_template.rs            ← SaTempTemplate.java
 │   └── sa_temp_util.rs               ← SaTempUtil.java
 │
 └── util/
     ├── mod.rs
     ├── sa_fox_util.rs                 ← SaFoxUtil.java
     ├── sa_hex_util.rs                 ← SaHexUtil.java
     ├── sa_result.rs                   ← SaResult.java
     ├── sa_sugar.rs                    ← SaSugar.java
     ├── sa_token_consts.rs             ← SaTokenConsts.java
     ├── sa_ttl_methods.rs              ← SaTtlMethods.java
     ├── sa_value2_box.rs               ← SaValue2Box.java
     └── str_formatter.rs               ← StrFormatter.java
```

---

## 变更摘要

| 变更类型 | 数量 | 说明 |
|---|---|---|
| **位置不变** | ~130 | Java 包 → Rust 模块，1:1 映射 |
| **重命名** | ~10 | Java 大驼峰 → Rust 蛇形命名 |
| **移出至 sa-token-derive** | 20 | 注解类 + Handler → proc-macro crate |
| **移出至 sa-token-context-mock** | 5 | Mock 类 → 独立测试 crate |
| **合并/简化** | ~15 | dao/auto、dao/timedcache、serializer/impl、exception 子类 |
| **位置变更** | 1 | SaHttpMethod.java → context/model/ |
| **Rust 新增** | 1 | lib.rs（入口文件） |
| **Rust 新增** | 1 | exception/mod.rs（单一 enum 替代 20 个类） |

---

## 与 Java 包结构对应关系

| Java 包 | Rust 模块 | 说明 |
|---|---|---|
| `cn.dev33.satoken` | `sa_token_core::` | 根模块 |
| `cn.dev33.satoken.annotation` | `sa_token_core::annotation` + `sa_token_derive::` | 注解放 core，宏放 derive |
| `cn.dev33.satoken.application` | `sa_token_core::application` | 1:1 |
| `cn.dev33.satoken.config` | `sa_token_core::config` | 1:1 |
| `cn.dev33.satoken.context` | `sa_token_core::context` | 1:1 |
| `cn.dev33.satoken.context.mock` | `sa_token_context_mock::` | 移出为独立 crate |
| `cn.dev33.satoken.context.model` | `sa_token_core::context::model` | 1:1 |
| `cn.dev33.satoken.dao` | `sa_token_core::dao` + `sa_token_dao_memory::` | 移出 Memory 实现 |
| `cn.dev33.satoken.error` | `sa_token_core::error` | 1:1 |
| `cn.dev33.satoken.exception` | `sa_token_core::exception` | 折叠为单一 enum |
| `cn.dev33.satoken.filter` | `sa_token_core::filter` | 1:1 |
| `cn.dev33.satoken.fun` | `sa_token_core::fun` | 1:1 |
| `cn.dev33.satoken.http` | `sa_token_core::http` | 1:1 |
| `cn.dev33.satoken.httpauth.basic` | `sa_token_core::httpauth::basic` | 1:1 |
| `cn.dev33.satoken.httpauth.digest` | `sa_token_core::httpauth::digest` | 1:1 |
| `cn.dev33.satoken.json` | `sa_token_core::json` | 1:1 |
| `cn.dev33.satoken.listener` | `sa_token_core::listener` | 1:1 |
| `cn.dev33.satoken.log` | `sa_token_core::log` | 1:1 |
| `cn.dev33.satoken.model` | `sa_token_core::model` | 1:1 |
| `cn.dev33.satoken.plugin` | `sa_token_core::plugin` | 1:1 |
| `cn.dev33.satoken.router` | `sa_token_core::router` + `context::model` | SaHttpMethod 移入 context |
| `cn.dev33.satoken.same` | `sa_token_core::same` | 1:1 |
| `cn.dev33.satoken.secure` | `sa_token_core::secure` | 1:1 |
| `cn.dev33.satoken.serializer` | `sa_token_core::serializer` | 1:1 |
| `cn.dev33.satoken.session` | `sa_token_core::session` | 1:1 |
| `cn.dev33.satoken.stp` | `sa_token_core::stp` | 1:1 |
| `cn.dev33.satoken.stp.parameter` | `sa_token_core::stp::parameter` | 1:1 |
| `cn.dev33.satoken.strategy` | `sa_token_core::strategy` | 1:1 |
| `cn.dev33.satoken.temp` | `sa_token_core::temp` | 1:1 |
| `cn.dev33.satoken.util` | `sa_token_core::util` | 1:1 |

---

## 参考

- [java-tree-full.md](./java-tree-full.md) - Java 完整目录树
- [rust-tree-full.md](./rust-tree-full.md) - Rust 完整目录树
