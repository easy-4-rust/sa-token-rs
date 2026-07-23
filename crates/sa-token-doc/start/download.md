# 其它环境引入 Sa-Token-Rs 的示例

目前已实现的对接框架综合。

如需一次性获取官方仓库内全部可运行示例，请见：[Sa-Token-Rs 集成示例大全](/more/download-demos) 。

------

## Cargo 依赖

根据不同基础框架引入不同的 Sa-Token-Rs 依赖：

<!------------------------------ tabs:start ------------------------------>

<!------------- tab:axum（对应 SpringBoot / ServletAPI）  ------------->
如果你使用 **axum**（对应 Java SpringMVC / SpringBoot），请引入：

``` toml
[dependencies]
sa-token = "0.1"
sa-token-web-axum = "0.1"
axum = "0.8"
tokio = { version = "1", features = ["full"] }
```

| Java | Rust |
|---|---|
| `sa-token-spring-boot-starter` | `sa-token` + `sa-token-web-axum` |
| `sa-token-spring-boot3-starter` | 同上 |
| `sa-token-spring-boot4-starter` | 同上 |

<!------------- tab:axum 异步（对应 WebFlux / Reactor）  ------------->
注：异步场景仍推荐 **axum + tokio**；也可用 `AsyncStpUtil` / `AsyncSaTokenRuntime`。

``` toml
[dependencies]
sa-token = "0.1"
sa-token-web-axum = "0.1"
```

| Java | Rust |
|---|---|
| `sa-token-reactor-spring-boot-starter` | `sa-token-web-axum`（tokio） |

<!------------- tab:actix-web（对应 Solon / Quarkus）  ------------->
``` toml
[dependencies]
sa-token = "0.1"
sa-token-web-actix = "0.1"
actix-web = "4"
```

| Java | Rust |
|---|---|
| `sa-token-solon-plugin` | `sa-token-web-actix` |
| `quarkus-satoken-*` | `sa-token-web-actix` |

<!------------- tab:Salvo  ------------->
``` toml
[dependencies]
sa-token = "0.1"
sa-token-web-salvo = "0.1"
```

<!------------- tab:仅 core（自定义 Context）  ------------->
注：若项目不使用上述 Web 框架，可只引入 core，并自行实现 / 注入 `SaTokenContext`：

``` toml
[dependencies]
sa-token-core = "0.1"
# 测试可用：
# sa-token-context-mock = "0.1"
```

参考：[自定义 SaTokenContext 指南](/fun/sa-token-context)

<!---------------------------- tabs:end ------------------------------>


## 版本要求

注：Rust **MSRV 1.88+**，Edition **2024**；当前 workspace 版本 `0.1.0`。


## 测试版

更多内测版本了解：[Sa-Token-Rs 最新版本](/start/new-version.md)

Cargo 依赖一直无法加载成功？[参考解决方案](/start/maven-pull)


## crate / 源码获取

如果你想深入了解 Sa-Token-Rs，请从本 monorepo 获取源码：

- 仓库根目录：`sa-token-rs`
- 文档：`crates/sa-token-doc`
- Demo：`crates/sa-token-demo/*`
- 源码目录介绍：[仓库目录](/arch/dir-intro)

Java 原版仓库仍可参考：
- **Gitee**：[https://gitee.com/dromara/sa-token](https://gitee.com/dromara/sa-token)
- **GitHub**：[https://github.com/dromara/sa-token](https://github.com/dromara/sa-token)


## 运行示例

- 1、克隆本仓库（学习测试用主分支）。
- 2、从根目录打开 workspace。
- 3、选择相应的 demo 运行，例如：

```bash
cargo run -p sa-token-demo-axum
cargo run -p sa-token-demo-actix-web
```

<img src="/big-file/doc/start/import-demo-run.png" alt="运行示例" title="s-w-sh">
