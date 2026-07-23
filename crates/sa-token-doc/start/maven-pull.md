# Cargo 依赖一直无法拉取成功？

> Java 原文标题：`Maven 依赖一直无法拉取成功？`  
> 工具映射：**Maven → Cargo**

---
方法1、先重启一下试试（IDE / 终端 / `cargo` 相关进程）。

---
方法2、可能依赖还没有下载完毕，请看一下终端是否有正在 `Downloading` / `Compiling` 的进度输出。

---
方法3、可能是网络不太稳定，导致本地下载了一些残碎文件，先把这些残碎缓存删了，再重新构建项目试试。

一般本地的 Cargo 缓存目录：
- macOS / Linux：`~/.cargo/registry/`
- 也可清理指定 crate：删除 `~/.cargo/registry/src/` 与 `~/.cargo/registry/cache/` 下相关目录后重试。

然后执行：

```bash
cargo clean
cargo fetch
cargo check -p sa-token
```

---
方法4、可能你给 Cargo 配置了镜像，而部分 crate 无法通过该镜像加载成功。

打开你的 `~/.cargo/config.toml`，看看有没有类似配置：

``` toml
[source.crates-io]
replace-with = 'rsproxy'

[source.rsproxy]
registry = "https://rsproxy.cn/crates.io-index"
```

如果有的话，可以：
1. 先注释掉镜像，直连 crates.io；或
2. 换成其它镜像（如 USTC、rsproxy 等）后再试。

然后重新执行：

```bash
cargo fetch
```

--- ---
方法5、如果使用的是 Cargo workspace 父子项目，在根 `Cargo.toml` 声明依赖后，成员无法识别：

需要确认：
1. 成员 `Cargo.toml` 中使用 `xxx.workspace = true` 或显式 `path` / `version`；
2. 根目录 `[workspace.dependencies]` 已正确声明；
3. 执行 `cargo check -p <member>` 重新解析。

若还是不行，可以新建一个最小二进制 crate 单独引入 `sa-token` 验证网络与版本，再回到原 workspace。

--- ---
再不行的话，就加群反馈吧（见 [加入讨论群](/more/join-group)）。

### Java ↔ Rust 对照

| Java / Maven | Rust / Cargo |
|---|---|
| `~/.m2/repository` | `~/.cargo/registry` |
| `mvn clean install` | `cargo clean && cargo build` |
| `settings.xml` mirror | `~/.cargo/config.toml` source replace |
| 父子 POM | Cargo workspace |
