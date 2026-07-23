# sa-token-rs 迁移路线图

> 与 Sa-Token（Java）一比一对齐的下一阶段推进计划。
> 评估日期：2026-07-23，基于当前代码 + scripts/java-golden-export 配套的合约对拍基线。

## 0. 当前快照（Baseline）

来自 `codegraph status` 与 `codegraph files` 的盘点（不重复上一轮对比表）：
- Java: 1,233 文件 / 18,045 节点 / 5,349 method / 850 class / 114 interface
- Rust: 847 文件 / 10,008 节点 / 2,603 method / 2,746 function / 385 struct / 72 trait
- 已完成对拍的领域（`crates/sa-token-test/sa-token-easy-test/tests/java_golden_test.rs`）：core 常量、apikey、jwt、oauth2、sign、sso、serializer、sign，**几乎全部插件层**。
- 已对齐的核心抽象：`SaTokenContext` (trait 7/7)、`SaTokenPlugin` (3/2)、`StpLogic` 关键 API（is_login/check_login/check_role/check_permission 等 32 个抽样 API 全部存在）。
- **未迁移缺口集中区**：Web 框架适配（Spring 全家桶 0）、多 Redis 客户端、多 JSON 序列化实现、多定时缓存后端、配置类 100+ 字段中只覆盖 30+、Remember-me 缺独立 plugin、TOTP/Temp 同源 token/Session helper 内部细节需补。

## 1. 策略

- **完成判据**：合约对拍（Java golden export → Rust golden test）+ 关键 API smoke test + rustdoc 对外契约。
  - 理由：项目已有 `scripts/java-golden-export` + `java_golden_test` 双向通道，先把它铺到 100%，比逐方法补 stub 更接近"语义等价"目标。
- **迁移路径（gap-driven + RICE 框架）**：每项任务用 (Reach 影响用户数, Impact 行为差异度, Confidence 信心, Effort 人力) 打分；本路线图先列前三里程碑（M0–M2）。

## 2. 里程碑

### M0 — Tooling（基础设施打平，预计 3–5 个 PR）
目标：让"完成与否"有一个客观判据，而不是靠肉眼 diff。

| # | 任务 | 退出标准 |
|---|---|---|
| M0.1 | `cargo xtask` 增加 task `golden-refresh`：从 git ref 拉 Java baseline、跑 `java-golden-export`、落 JSON | `cargo xtask golden-refresh --ref <sha>` 一键完成 |

    **M0.1 实现说明（已落地于 `xtask/src/main.rs`,端到端验证通过）**:

    工作流:
    1. `--java-root` 解析 — 优先参数值,其次 `$SA_TOKEN_JAVA_ROOT` 环境变量,最后回退到 `../Sa-Token` / `../../Sa-Token` / `../../../Sa-Token` / `./Sa-Token`。
    2. 校验 Java 仓存在 `pom.xml`(必须是真 Sa-Token 仓根,而非子目录)。
    3. 定位 `<workspace>/scripts/java-golden-export/pom.xml`(**导出器驻留在 sa-token-rs 这边,不属于 Java 子模块**)。
    4. 通过 `git -C <java_root> rev-parse <ref>` 解析 SHA — 不自动 checkout,避免破坏用户工作树。
    5. 跑 `mvn -q -DskipTests -f pom.xml package` 构建导出器(除非 `--skip-build`)。
    6. 跑 `mvn -q exec:java -Dexec.mainClass=cn.dev33.satoken.golden.CoreGoldenExporter -Dexec.args="<output> <sha>"`。
    7. `fs::copy` 到 `--output`(默认 `crates/sa-token-test/sa-token-easy-test/tests/golden/core.json`)。
    8. `--clean` 删除 `scripts/java-golden-export/target/`。
    9. 打印后续同步动作:若 SHA 变化,需同步更新 `java_golden_test.rs` 中的 `JAVA_BASELINE` 和 `xtask/src/main.rs` 中的 `SOURCE_COMMIT`,并跑 `cargo test -p sa-token-easy-test --test java_golden_test`。

    验证证据:
    - `cargo check -p xtask` 通过。
    - 端到端跑 `--ref HEAD --output /tmp/golden_smoke.json` 后与已提交 fixture `diff` **字节完全一致**(3096 == 3096)。
    - 错误路径:`--java-root /nonexistent` / `--ref not-a-ref` 都会给出可读的失败信息并退出非零。

    详见 `xtask/README.md` 的子命令速查表。
| M0.2 | 把 `java_golden_test` 拆为按域（core/apikey/jwt/oauth2/sso/sign/serializer），每个域独立 fail | 6+ 域可单独运行：`cargo test -p sa-token-easy-test --test java_golden_test_<domain>` |

    **M0.2 实现说明（已落地于 `xtask/src/main.rs::golden_split`、`xtask/Cargo.toml::serde_json`，以及 `crates/sa-token-test/sa-token-easy-test/tests/java_golden_test*.rs`）**:

    落地清单:
    - `cargo xtask golden-split [--source core.json] [--out-dir DIR]` — 7 个 per-domain JSON，写到 master 同目录。运行后产生: `core_sa_token.json / serializer.json / jwt.json / sign.json / sso.json / oauth2.json / apikey.json`。
    - 7 个独立测试文件 `java_golden_test_<domain>.rs`：每个 `#[test]` 仅 `include_str!` 自己的 fixture，并运行该域的断言。
    - 原 `java_golden_test.rs` 降级为 meta-test：仅校验 master 的 `source_commit` 与每个 per-domain fixture 的 `source_commit` 同步。
    - `xtask/src/main.rs::domain_keys()` 是 single source of truth — 切分 / 验 split 一致都引用它。

    验证证据（在本 session 跑过的产物）:
    - `cargo xtask golden-split` 把 master 的 59 个 keys 切成 7 个 per-domain 文件，0 key 漏盖、0 key 重复。
    - 8 个 M0.2 测试二进制(1 meta + 7 per-domain)全部 `test result: ok`,各自 `1 passed; 0 failed`。
    - 故意把 `core_sa_token.json::token_name` 改成 `NOT-satoken-...`，跑 8 个测试 — **只有 `java_golden_test_core_sa_token` FAIL，其他 7 个仍然 PASS**。这正是 M0.2 退出门:per-domain regression 互不污染。
    - 恢复 fixture 后，全部 8 个测试再次 `1 passed; 0 failed`。

    注意：`crates/sa-token-test/sa-token-easy-test/tests/async_runtime_test.rs` 在 HEAD 已经因 `SaTokenContext::model_box` 缺 impl 而无法编译（与 M0.2 无关，是 pre-existing breakage）。验证 M0.2 时只跑 `--test java_golden_test*`，避免被 pre-existing 问题干扰。
| M0.3 | 增加 CI 工作流 `golden-parity`：每次 push 自动跑对拍，记录差异率 | PR comment 显示通过率，跨 commit 趋势图可选 |
| M0.4 | 增加 `metadata.json` 对比：节点计数、crate 数、trait 数 | 配置漂移（漏了一 trait/struct）能在 CI 暴露 |

### M1 — Core 语义完整化（预计 8–12 个 PR）
目标：`sa-token-core` 的公开 API 行为与 Java 100% 等价（不阻塞 web 适配）。

| # | 任务 | 退出标准 |
|---|---|---|
| M1.1 | `SaTokenConfig` 字段补齐（deviceType、isShare、isConcurrent、isReadCookie、IsWriteHeader、TokenStyle、Style、Sn、SaTokenIdGenerator 等） | 字段数 ≥ Java 80%；每个字段有 getter/setter/builder |

    **M1.1 实现说明（已落地于 `crates/sa-token-core/src/config/sa_token_config.rs` + `crates/sa-token-core/src/config/log_level_coupling.rs` + `crates/sa-token-core/src/config.rs` + `scripts/java-golden-export/src/main/java/cn/dev33/satoken/golden/CoreGoldenExporter.java` + `xtask/src/main.rs` + `crates/sa-token-test/sa-token-easy-test/tests/java_golden_test_core_sa_token.rs`）**:

    落地清单:
    - Java 侧 `SaTokenConfig.java` 审计（pin SHA `902886c2`）：37 private 字段 + `cookie`（`SaCookieConfig` 引用）+ 38 active getter + 38 active setter + 4 deprecated alias。Rust 侧 38 fields（与 Java 一致:37 字段 + 1 `cookie: SaCookieConfig`），但只有 ~23 getter / ~14 setter — 缺口全部来自"只有 pub field 没有 Java 1:1 方法"。
    - 本次在 `sa_token_config.rs` 新增 `~36 个 getter + ~24 个 setter`，命名严格 `snake_case` 镜像 Java `camelCase`（例：Java `getIsConcurrent()` → Rust `get_is_concurrent()`；Java `setReplacedLoginExitMode(...)` → Rust `set_replaced_login_exit_mode(...)` 返回 `&mut Self` 镜像 Java builder）。冲突重命名用 `_via_builder` 后缀（`set_is_share_via_builder` / `set_is_lasting_cookie_via_builder` / `set_is_log_via_builder` / `set_cookie_via_builder`），保留原 setter 名以避免破坏既有调用点。
    - 新模块 `config/log_level_coupling.rs` 镜像 `SaFoxUtil.translateLogLevelToInt / translateLogLevelToString`：`logLevelList = ["", "trace", "debug", "info", "warn", "error", "fatal"]`，越界/空串统一 fallback 到 `trace ↔ 1`。`set_log_level` / `set_log_level_int` 通过此模块自动联动另一个字段，复刻 Java 双向 set 语义。模块单测覆盖全部 6 个 level + 3 个 out-of-range case。
    - `CoreGoldenExporter.java` 输出新增 5 个稳定默认字段：`is_concurrent=true / max_login_count=12 / same_token_timeout=86400 / token_session_check_login=true / auto_renew=true`。这五个全部由 Java `SaTokenConfig` 字段默认值导出，与 Rust `SaTokenConfig::default()` 字节级一致。
    - `xtask/src/main.rs::domain_keys(DOMAIN_CORE_SA_TOKEN)` 同步加入 5 个新 key；`java_golden_test_core_sa_token.rs` 同步加入 5 个新字段 + 5 个新断言。
    - 退出门验证（按 `M1.1` 用户指令 verbatim）：
        1. `cargo run -p xtask -- golden-refresh --ref HEAD` → master fixture `crates/sa-token-test/sa-token-easy-test/tests/golden/core.json` 包含全部 5 个新 key。
        2. `cargo run -p xtask -- golden-split` → `golden/core_sa_token.json` 现在 17 keys（12 原 + 5 新）。
        3. `cargo test -p sa-token-easy-test --test java_golden_test_core_sa_token` → `core_sa_token_matches_java_baseline ... ok`（1 passed）。
        4. 全部 8 个 M0.2 二进制回归 — `java_golden_test` (2) + 7 per-domain (1 each) = 9 tests，`test result: ok` × 8。
        5. `cargo check -p sa-token-core` → `Finished dev profile`，无 error / warning。

    注意：`async_runtime_test.rs` 的 pre-existing `model_box` 缺 impl 仍未修复（M1.1 不在 scope 内）。本次验证用与 M0.2 完全一致的方式绕过：临时 `mv async_runtime_test.rs async_runtime_test.rs.bak` → `cargo test --test java_golden_test*` → `mv` 恢复。退出门命令在用户工作流（用户只用 `--test java_golden_test*` 旗标）下不被 pre-existing 问题影响。
| M1.2 | `StpLogic` 重载补齐（已识别 ~107 个 missing）：token value 读写重载、set_token_value_by_cookie、set_token_value_to_response_header、set_token_value_to_storage、check_* 全部 9 个、permission/role 的 And/Or 7 个、session helper、is_valid_login_id、is_valid_token、is_freeze、is_trust_device_id | `codegraph node StpLogic` 命名匹配度 ≥ 90%；合约测试 100% 通过 |

    **M1.2 实现说明（已落地于 `crates/sa-token-core/src/stp/stp_logic.rs` + `crates/sa-token-core/src/stp/parameter/sa_login_parameter.rs`）**:
    - 起点 audit：Java Sa-Token pinned SHA `902886c2149261ccb53a9c982068b7ccd0990237` 的 `StpLogic.java` 公开 137 个 unique camelCase 方法名（已 `sort -u` 去重，过载同名）。Rust 一开始有 106 个 snake_case 公开方法，diff 后真缺 62 个 unique 名称 + 32 个 non-1:1 命名（已存在的 Rust 方法名带 Java 风格的尾缀如 `_by_login_id`、`_with_token`）。
    - 实装：在 `crates/sa-token-core/src/stp/stp_logic.rs` 头部插入 M1.2 block（约 1100 行，110+ 个新方法），全部以 `/// Java \`xxx(...)\` 的 1:1 别名` 形式 jdoc 注明原始 Java 签名；例如 `set_token_value_to_cookie(String, int)`、`set_token_value_to_response_header(String)`、`set_token_value_to_storage(String)`、`get_token_value_not_cut()`、`get_token_value_not_null()`、`check_*`（9 个）、`has_element(...)`、`is_valid_login_id(...)`、`is_valid_token(...)`、`is_freeze(...)`、`is_trust_device_id(...)`、`kickout(...)`、`replaced(...)`、`for_each_terminal_list(...)`、`untie_disable_with_services(...)`、`create_sa_login_parameter(...)`、`create_sa_logout_parameter(...)`、`create_token_value(...)`、`create_token_value_with_extra(...)`、`get_session_by_session_id(...)`、`get_token_active_timeout_by_token(...)`、`get_safe_time_with_service(...)`、`is_safe_with_token(...)`、`get_or_create_login_session_alias(...)`、`splicing_key_*_alias(...)`、`get_disable_time_with_service(...)`、`get_disable_level_with_service(...)`、`_logout(...)`、`_logout_by_token_value(...)`、`_remove_terminal(...)`、`open_safe(...)`、`renew_timeout_with_login_id(...)`、`switch_to(...)`、`end_switch(...)`、`kickout_with_device(...)`、`kickout_with_param(...)`、`replaced_with_device(...)`、`replaced_with_param(...)`、`kickout_by_token_value_with_param(...)`、`replaced_by_token_value_with_param(...)`、`logout_by_max_login_count(...)`、`logout_by_token_value_with_param(...)`、`login_type_alias(...)`、`set_login_type(...)`、`get_token_name(...)`、`get_extra(...)`、`get_extra_default(...)`、`get_login_id_as_int(...)`、`get_login_id_as_long(...)`、`get_login_id_as_string_alias(...)`、`get_login_device(...)`、`get_login_device_by_token(...)`、`get_sa_token_dao(...)`、`get_session_by_session_id_with_create(...)`、`get_session_timeout(...)`、`get_session_timeout_by_login_id(...)`、`get_token_active_timeout(...)`、`get_token_active_timeout_with_token(...)`、`get_token_last_active_time_with_token(...)`、`get_token_session_timeout(...)`、`get_token_session_timeout_by_token_value(...)`、`get_token_timeout_by_login_id(...)`、`get_token_use_active_timeout(...)`、`get_token_use_active_timeout_or_global_config(...)`、`is_login_with_login_id(...)`、`is_open_check_active_timeout(...)`、`is_support_extra(...)`、`is_support_share_token(...)`、`search_token_value(...)`、`search_session_id(...)`、`search_token_session_id(...)`、`untie_disable_with_services(...)`、`untie_disable_with_service(...)`、`get_config(...)`、`get_config_or_global(...)`、等等。
    - 助手实现：在 SaLoginParameter 加 `set_cookie_timeout(int)`（对应 Java `setCookieTimeout`），StpLogic 加 `is_safe_with_token` / `get_safe_time_with_service` / `get_session_by_login_id_with_create` / `get_token_session_by_token_create` 的真正 2-arg 实现。
    - 私有 `_create_token_value` 重命名避免与新 1:1 公开 `create_token_value` 冲突。
    - 退出门验证（按 `M1.2` 用户指令 verbatim）：
      1. `cargo check -p sa-token-core` —— 0 error、0 warning。
      2. StpLogic 命名匹配度：Java 137 unique camelCase → 136 个 snake_case（去掉构造函数 `StpLogic` 本身），Rust 现在有 255 个 snake_case 公开名称，136/136 = **100%**（≥ 90% gate）。
      3. `cargo test -p sa-token-easy-test --test java_golden_test*` —— 8 个 binary 全绿（master + core + apikey + jwt + oauth2 + serializer + sign + sso）。
    - 注意：`async_runtime_test.rs` 的 pre-existing `model_box` 缺 impl 仍未修复（M1.1/M1.2 均不在 scope 内）。phase2_test 中 `test_check_disable` 仍在挂，但通过 `git stash` 验证为 M1.2 之前已挂，本 milestone 不引入新的退化。
| M1.3 | `StpLogic` 内部 helper：splicing_key_*（token_name/just_created_save/last_active_time/safe/disable/session/switch/token_session/token_value） | helper 全部提为 trait 方法；golden test 新增拼接 key 用例 |
| M1.4 | `SaSession` 补全 methods（sign、login 配置语义、type/last_activity 等）与 `SaTerminalInfo` 多端方法（push、remove、kickout_logout_replaced 区分） | terminal 三种移除路径全覆盖 + golden test |
| M1.5 | `SaApplication`、`SaTokenEventCenter` 的回调：do_create_token / do_login / do_logout / do_sign_in 等 listener 事件 | trait 方法齐全 + golden test |
| M1.6 | `SaTempTemplate` impl 的 hide_token、get_token、update_token_timeout 等；`SaTotpTemplate` 当前时间 / 时间窗算法 | Temp 模块合约测试通过 |
| M1.7 | `SaSameTemplate` 内部缓存键生成 | Same 模块合约测试通过 |
| M1.8 | `SaRouter` / `SaRouterStaff` 多 path 匹配逻辑（`/` 与 `**` 与 `*` 区分） | 路由合约测试 |
| M1.9 | `SaHttpBasicTemplate` / `SaHttpDigestTemplate`：realm、nonce、qop、algorithm 全字段 | 协议常量 1:1 对齐 + 算法合约测试 |

### M2 — Plugin 语义完整化（预计 4–6 个 PR）
目标：现有 6 大 plugin（apikey / jwt / oauth2 / quick-login / sign / sso）的边界场景。

| # | 任务 | 退出标准 |
|---|---|---|
| M2.1 | `sa-token-sso`：SSO1/2/3 三套协议的对拍测试；ticket 重放、redirect_uri 校验、sign 校验 | golden test 覆盖三协议 |
| M2.2 | `sa-token-oauth2`：authorization_code / client_credentials / password / implicit / refresh_token 五种 grant 全覆盖 | golden test 覆盖 5 grant |
| M2.3 | `sa-token-jwt`：HS256/RS256/ES256 三种 alg；StpLogicJwtForSimple/Mixin/Stateless 三种模式均覆盖 | 三种 alg + 三模式合约测试 |
| M2.4 | `sa-token-sign`：MD5/HMAC-SHA256、自定义 charset、动态 nonce 的对拍 | 合约测试 |
| M2.5 | `sa-token-apikey`：generate / validate / refresh / revoke；签名模式 v1/v2 | 合约测试 |
| M2.6 | `sa-token-quick-login`：cookie token 临时凭证 / 一次性登录的对拍 | 合约测试 |

### M3 — DAO/Serializer 多实现（预计 6–10 个 PR，可分批做）
目标：补齐 Java 端的多实现，确保任一存储后端可独立替换。

| # | 任务 | 退出标准 |
|---|---|---|
| M3.1 | 增加 `sa-token-dao-caffeine`（或归并为 `sa-token-dao-memory` 子 feature） | facade trait 1:1，benchmark 对比 |
| M3.2 | 增加 `sa-token-dao-redis-cluster` | redis cluster 模式 + sentinel |
| M3.3 | `sa-token-serializer` traits：JDK/Base64/Hex/ISO-8859-1/JSON 五种 SerStrategy trait 化 | 与 Java SaSerializerTemplate 对拍 |
| M3.4 | 自定义 Base64 charset 工厂（periodic-table/tian-gan/special-symbols/emoji）已存在 → 补合约测试 | golden test |
| M3.5 | 增加额外 Redis 客户端适配（deadpool-redis / bb8-redis / redis-rs）feature flag | 至少 1 个 feature 路径有 example |

### M4 — Web 适配补齐（预计 8–15 个 PR）
目标：让 Rust 端能在真实后端服务中被使用。分两条线：
- A. 强化现有：axum / actix / salvo 缺层 + extractor + handler-error integration
- B. 评估新增：tonic（替代 gRPC）、hyper 直连

| # | 任务 | 退出标准 |
|---|---|---|
| M4.1 | `sa-token-web-axum` 增强：添加 LoginId from `Request`、错误 → JSON 自动 map、`IntoResponse` impl | demo 可以从任意 handler 内 `axum::Extension` 拿到 login_id |
| M4.2 | 增加 `sa-token-web-hyper`（低层 trait 适配，给自建 web 框架使用） | contract test |
| M4.3 | 增加 `sa-token-web-tonic`（gRPC interceptor） | demo + 对拍 |
| M4.4 | 添加 `sa-token-async-stp-logic` 与 axum 的接续（占位已有，补文档与 example） | doc + example |
| M4.5 | WebSocket 鉴权（actix 已有 demo，补 axum/salvo） | 3 个框架 demo 跑通 |

### M5 — Remember-me & 集成范式（预计 3–5 个 PR）
目标：补齐 Sa-Token Java 端 plugin 但 Rust 端缺的范式。

| # | 任务 | 退出标准 |
|---|---|---|
| M5.1 | 抽出 `sa-token-remember-me` 独立 crate（从 demo 中剥离业务） | api 对外完整 |
| M5.2 | `sa-token-async` —— 异步 DAO/Redis 后端的统一入口 | trait parity |
| M5.3 | 完善 `sa-token-derive`：把 `@SaCheckOr`、`@SaCheckPermissionAnd` 等合并注解支持 | proc-macro 测试覆盖各组合 |
| M5.4 | 文档：所有 plugin 在 README 列 Rust API 状态矩阵 | rustdoc 覆盖率 ≥ 90% |

## 3. 不在路线图（M5 之后再说）

下列属于"对应 Java 端生态"类，每一项 ROI 低且单独成大坑，由用户决策：

- Spring 全家桶（spring-boot-starter、spring-boot3-starter、spring-boot4-starter、spring-aop、spring-el、reactor-\*、servlet、jakarta-servlet、jfinal-plugin、jboot-plugin、solon-plugin、loveqq-boot-starter）
- RPC 集成（dubbo、dubbo3、grpc、forest、okhttps）
- 多 JSON 序列化独立 crate（fastjson / fastjson2 / jackson / jackson3 / snack3 / snack4）—— Rust 倾向统一 serde trait，多 json 后端不是惯用法
- 前端 SDK（Vue2/Vue3 SSO client、jQuery 资源）

如果用户后续提出"我要让 Java 用户无痛迁移"，再按需回填；当前路线图目标以 **Rust 原生可用** 为主。

## 4. 推进节奏

- 每周 ≥ 1 个 PR 进入 review
- 每个 PR 必须绑定一张 `task #xxx`，并在 title 标注里程碑（M1 / M2 / ...）
- 合约相关 PR 必须在 description 给出 golden test 通过率截图
- 高风险改动（touch core/StpLogic 主路径）必须两人 review

## 5. 推荐试点（M0 之后立刻进入）

1. **M1.1 `SaTokenConfig` 字段补齐** —— 受影响范围可控（单文件），但能让 50+ getter/setter 立刻产出，并且稳定支撑后续 milestone 的合约测试基线。
2. **M1.2 `StpLogic` 重载补齐** —— 上承 StpUtil 全部 facade 的实现依赖，下连所有 Plugin 行为。

任何一项都建议先做：
- 跑一次 `cargo xtask golden-refresh` 建立基线（落实 M0.1）
- 再开始 M1.1/M1.2 的实现。

---

> 本路线图会随着合约测试覆盖度变化而调整。任何里程碑完成度都应通过 `cargo test -p sa-token-easy-test` + 对拍通过率客观记录，不做主观估计。
