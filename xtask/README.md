# xtask — sa-token-rs 仓库维护命令

`cargo xtask` 是 workspace 内的二进制(`xtask/Cargo.toml`),集中放跨 crate 的日常工具脚本,
按子命令分发。

## 调用方式

```bash
cargo xtask <SUBCOMMAND> [ARGS]
cargo xtask --help       # 等同 `cargo xtask help`
cargo xtask help         # 列出所有子命令与 flag
```

## 子命令速查

### `migration-generate [JAVA_ROOT]`

扫描 Java 仓里所有 `src/main/java/**.java`,生成 `docs/migration/file-map.csv`。
- `JAVA_ROOT` 缺省为 `../Sa-Token`。

### `migration-audit [--strict]`

把 `docs/migration/file-map.csv` 与 Rust 工作区实际存在的源文件对账,打印
missing/excess 行数。非 `--strict` 模式仅扫描;`--strict` 模式下 missing 即非零退出。

### `migration-audit-strict`

`--strict` 别名,直接以非零退出码驱动 CI。

### `golden-refresh`

**对应 ROADMAP M0.1。** 重新生成 Java baseline 黄金值(JSON),用于
`crates/sa-token-test/sa-token-easy-test/tests/java_golden_test.rs` 的合约对拍。

| Flag | 含义 |
| --- | --- |
| `--java-root <PATH>` | Sa-Token(Java)仓根。优先级:参数 → `$SA_TOKEN_JAVA_ROOT` → `../Sa-Token` → `../../Sa-Token` → `../../../Sa-Token` → `./Sa-Token`。 |
| `--ref <REF>` | 在 Java 仓上 `git rev-parse <REF>`,解析出 40 字符 SHA;不自动 checkout,避免动你的工作树。 |
| `--output <FILE>` | JSON 落地路径。默认 `crates/sa-token-test/sa-token-easy-test/tests/golden/core.json`。 |
| `--skip-build` | 跳过 `mvn package`,复用 `scripts/java-golden-export/target/` 下已有的 classpath。 |
| `--clean` | 跑完后删除 `scripts/java-golden-export/target/`。 |
| `--mvn <PATH>` | 指定 `mvn` 可执行文件。默认从 `$MVN` 取,否则用 `mvn`。 |

#### 典型用法

```bash
# 默认:用 ../Sa-Token 的 HEAD,落回默认 fixture 路径
cargo xtask golden-refresh

# 显式锁 baseline 到某个 commit,导出到临时 JSON 验证
cargo xtask golden-refresh --ref v1.45.0 --output /tmp/golden_check.json

# Java 仓路径不在常见回退位置时手动指定
cargo xtask golden-refresh \
  --java-root /opt/work/Sa-Token \
  --ref 902886c2149261ccb53a9c982068b7ccd0990237 \
  --clean
```

#### 跑完之后

如果 Java 仓的 SHA 与上次不同时,需要手动同步两处常量:

1. `crates/sa-token-test/sa-token-easy-test/tests/java_golden_test.rs` 顶部的
   `const JAVA_BASELINE: &str = "<sha>";`
2. `xtask/src/main.rs` 顶部的
   `const SOURCE_COMMIT: &str = "<sha>";`

再跑一次 `cargo xtask golden-split` 把刷新过的 master 切成各 domain 子 fixture,然后:

```bash
cargo test -p sa-token-easy-test --test java_golden_test
cargo test -p sa-token-easy-test --test java_golden_test_<domain>
```

确认新 fixture 与 Rust 实现仍合约对拍。

### `golden-split`

**对应 ROADMAP M0.2。** 把 `golden/core.json`(M0.1 产物)切成 7 个 per-domain
子 fixture,供 7 个独立的 `java_golden_test_<domain>.rs` 单独 fail 用。

| Flag | 含义 |
| --- | --- |
| `--source <FILE>` | Master JSON 输入。默认 `crates/sa-token-test/sa-token-easy-test/tests/golden/core.json`。 |
| `--out-dir <DIR>` | 7 个 per-domain 文件输出目录。默认 `<source 的父目录>`(与 master 同目录)。 |

Domain → 文件对应关系(也写死在 `xtask/src/main.rs::ALL_DOMAINS` 与
`domain_keys()`):

| Domain | Fixture 文件 | Per-test |
| --- | --- | --- |
| `core_sa_token` | `core_sa_token.json` | `java_golden_test_core_sa_token` |
| `serializer` | `serializer.json` | `java_golden_test_serializer` |
| `jwt` | `jwt.json` | `java_golden_test_jwt` |
| `sign` | `sign.json` | `java_golden_test_sign` |
| `sso` | `sso.json` | `java_golden_test_sso` |
| `oauth2` | `oauth2.json` | `java_golden_test_oauth2` |
| `apikey` | `apikey.json` | `java_golden_test_apikey` |

每个 per-domain 文件包含其全部原 keys + 同样的 `source_commit` 字段。meta 测试
`java_golden_test` 同时校验 `master`、`source_commit`、与 per-domain 子文件
之间的一致性。

#### 典型用法

```bash
# 默认:从 master JSON 切到默认 out dir
cargo xtask golden-split

# 输出到任意目录(例如发布工件):
cargo xtask golden-split --out-dir dist/golden-fixtures
```

#### 新增一个 domain 时

1. 在 `xtask/src/main.rs` 的 `ALL_DOMAINS` 加一个新的 `&str` 常量,
   在 `domain_keys()` 加一组 keys。
2. 把新 keys 加到 `scripts/java-golden-export/.../CoreGoldenExporter.java` 的输出 JSON。
3. 跑 `cargo xtask golden-refresh --ref HEAD` 重建 master,然后 `cargo xtask golden-split`。
4. 在 `crates/sa-token-test/sa-token-easy-test/tests/` 新建
   `java_golden_test_<domain>.rs`,参照已有 7 个文件之一。
5. 把新 domain 加到 `java_golden_test.rs::DOMAIN_FIXTURES` 以保持 meta-test
   的 catalog 同步。

## 实现要点(供修改时参考)

- **导出器位置**:`scripts/java-golden-export/` 在 sa-token-rs 这边,不是 Java 仓里的子模块。
  pom 自带 `cn.dev33.*` 的 sa-token 依赖解析,所以构建期间才会从 Maven Central 拉取真正的 sa-token 类路径 —
  这意味着导出器是独立可单跑的 Maven 模块,Java 仓里用不到修改。
- **`-Dexec.args` 是一个值,不是两个 argv**:`exec:java` 把 `args` 当字符串交给主程序,所以
  xtask 用单 `-Dexec.args="<output> <sha>"` 把参数整体传出。
- **写入失败的可恢复策略**:`golden-refresh` 默认不修改任何版本化字段;是否更新 `JAVA_BASELINE` /
  `SOURCE_COMMIT` 留给调用者根据打印出来的 SHA 决定(避免一次性改动让评审变复杂)。
- **per-domain fixture 通过 `include_str!` 嵌入**:每个 `java_golden_test_<domain>.rs`
  都在编译期把对应的 JSON 字节吃进二进制,跑测试时不依赖 cwd。运行时切 cwd 不会让 per-domain 测试静默退化。
- **domain 切分是数据,不是逻辑**:`xtask/src/main.rs::domain_keys()` 是 source of truth,
  `golden_split()` 据此切 master,而 `java_golden_test.rs::DOMAIN_FIXTURES` 据此
  校验所有 per-domain 文件都已存在 — 同步增删一个 domain 仅需改两个地方
  (`ALL_DOMAINS` + `domain_keys`),其余由 golden-refresh + golden-split 自动串联。
