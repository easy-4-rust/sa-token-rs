# sa-token-rs 项目约束

本仓库是 Java Sa-Token 的 Rust 迁移实现。Java 基线固定为
`dev@902886c2149261ccb53a9c982068b7ccd0990237`。

## 命名与模块布局

- Cargo package/crate 目录使用 `kebab-case`。
- Rust 模块目录、模块文件、函数和变量使用 `snake_case`。
- struct、enum、trait 和类型别名使用 `PascalCase`，常量使用
  `SCREAMING_SNAKE_CASE`。
- 使用现代 `foo.rs + foo/` 布局，禁止新增 `mod.rs`。
- 禁止通过 `XInner + pub use XInner as X` 规避模块/类型同名问题。
- `lib.rs` 只声明顶层模块并定向导出稳定 API，禁止 crate 级 glob 导出。

## 迁移与质量

- `docs/migration/file-map.csv` 是唯一迁移账本。不能用空文件或仅声明类型的
  占位实现将条目标记为 `complete`。
- 只有目标文件存在、包含真实实现且 `test_evidence` 非空时才能标记
  `complete`。
- 当前工作树中的修改均视为用户资产；不得 reset、覆盖或丢弃。
- library 生产路径不得新增 `unwrap`、`expect`、静默吞错或阻塞网络 I/O。
- 新依赖优先在根 `[workspace.dependencies]` 统一声明，并保持 Rust 1.88 MSRV。
- 提交前至少执行迁移审计、fmt、workspace check、Clippy 和 workspace test。

## 关键命令

```bash
cargo run -p xtask -- migration-audit
cargo run -p xtask -- migration-audit-strict
cargo fmt --all --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
```
