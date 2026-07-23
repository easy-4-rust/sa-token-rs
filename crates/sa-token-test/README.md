# sa-token-test（对齐 Java `sa-token-test`）

> Java 父模块为 `packaging=pom` 的测试合集；Rust 侧为 **目录分组**（无独立 library crate），
> 子 crate 一一对应 Java `<modules>`。

## 模块对照

| Java 模块 | Rust crate | 说明 |
|-----------|------------|------|
| `sa-token-easy-test` | `sa-token-easy-test` | 无 Web 框架的核心契约 + Java golden |
| `sa-token-springboot-test` | `sa-token-springboot-test` | **Spring Boot → axum**（`sa-token-web-axum`） |
| `sa-token-jwt-test` | `sa-token-jwt-test` | JWT Simple/Mixin/Stateless |
| `sa-token-temp-jwt-test` | `sa-token-temp-jwt-test` | 临时 Token；`sa-token-temp-jwt` 插件未迁完时用 core `SaTempUtil` |
| `sa-token-json-test` | `sa-token-json-test` | JSON 序列化契约（Jackson → serde） |
| `sa-token-jackson3-test` | `sa-token-jackson3-test` | Jackson3 → `serde_json` |
| `sa-token-serializer-test` | `sa-token-serializer-test` | serializer-features |

## 运行

```bash
cargo test -p sa-token-easy-test
cargo test -p sa-token-springboot-test
cargo test -p sa-token-jwt-test
cargo test -p sa-token-temp-jwt-test
cargo test -p sa-token-json-test
cargo test -p sa-token-jackson3-test
cargo test -p sa-token-serializer-test
```

## 约定

- 目录 / 文件：`snake_case`
- 类型：`PascalCase`
- 测试文件顶部注释标注对应 Java 类名
- Jackson / Jackson3 → **serde** / **serde_json**
- Spring Boot → **axum**；第二栈 Quarkus → actix（可另增 `sa-token-quarkus-test` 映射 `sa-token-web-actix`，当前与 Java 模块表保持 7 项对齐）
