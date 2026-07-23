# Sa-Token-Rs 迁移质量报告

审计日期：2026-07-23。Rust 工作树基线：`main@61924d2` 加当前未提交迁移输入。
Java 源基线：`dev@902886c2149261ccb53a9c982068b7ccd0990237`。

## 结果摘要

| 检查项 | 结果 |
| --- | --- |
| 迁移矩阵普通审计 | 通过：895 行、881 映射、14 排除、0 重复 |
| 迁移矩阵严格审计 | 未通过（预期）：`complete=173/881` |
| Rust 源文件 | 461（包含基础设施） |
| 命名 | 0 个 `mod.rs`，0 个非 snake_case 源文件 |
| workspace check | Rust 1.88.0 通过；stable 1.97.1 通过 |
| rustfmt | `cargo fmt --all --check` 通过 |
| Clippy | all targets/all features，`-D warnings` 通过 |
| 测试 | 172 passed / 0 failed；此前 130 项连续两轮通过；含严格审计参数回归、Java golden、JWT、Sign、SSO、OAuth2 foundation、serializer、API Key、真实 Redis、Web adapter、async runtime |
| cargo-deny | 通过（重复版本有显式兼容清单） |

## 已完成的结构与运行时整改

- 模块目录和文件全部改为 snake_case，类型保持 PascalCase；使用
  `foo.rs + foo/`，清除 `mod.rs`。
- 修复 `migration-audit --strict` 尾随参数被静默忽略的问题；严格模式现在在
  121/881 时按预期失败，并有参数解析回归测试防止 CI 假阳性。
- 删除实际类型中的 `XInner + pub use XInner as X` 模式，更新 facade 与测试路径。
- `SaManager` 的组件槽从不可替换的 `OnceLock<Arc<_>>` 改为可替换的
  `OnceLock<RwLock<Arc<_>>>`，修复 setter 静默失效；全局 facade 测试显式串行。
- 新增 `SaTokenRuntime`、`AsyncSaTokenRuntime` 和对象安全 `AsyncSaTokenDao`。
- 同步 `SaTokenDao` 统一为 `SaResult<Option<T>>` / `SaResult<()>`，core、memory、
  SSO、OAuth2、Axum 调用链不再将基础设施故障折叠为缺失值或默认值。
- 新增显式 runtime 驱动的 `AsyncStpLogic` / `AsyncStpUtil`；两个独立 runtime 的
  登录、权限、查询与注销契约测试证明状态互不串扰；扩展契约覆盖顶人、设备、
  终端、TTL、角色、封禁、二级认证、请求级身份切换、共享/预定 Token、非并发
  替换、终端上限、Token-Session 和毫秒级活跃超时。
- application 四个 Java 文件已逐项对齐：`ApplicationInfo` 只裁剪已配置的非根
  route prefix，读写 trait 支持空值、类型转换、lazy 与 set-by-null 语义，
  `SaApplication` 支持 TTL、keys 和 clear。契约测试发现内存 DAO `search_data`
  返回 value 而非 key，现已修复并以排序 key 回归测试锁定。
- config 三个 Java 文件已逐项对齐：Cookie 恢复 nullable domain/path/sameSite、
  `httpOnly=false` 和额外属性；`SaTokenConfig` 补齐动态活跃、替换/溢出/注销策略、
  DAO 清理、日志、HTTP 认证与域名字段，并修复 share、最大登录数、header、日志等
  默认值偏差；properties 工厂对缺失文件返回默认值，对 I/O/赋值错误保留
  10021/10022。
- 删除错误放在 `model/` 根下且包含 loginId 的重复 `SaDisableWrapperInfo` 草稿，
  唯一实现保留 Java 的 isDisable/disableTime/disableLevel、三类构造与 camelCase
  序列化。
- Same-Token template/util 已对齐当前/past token、TTL、64 字符生成、请求头与 10301
  错误；新增显式 DAO/config 构造和 manager 可替换默认模板，隔离测试验证两次轮换
  后当前与上一 token 同时有效。
- Router 三个映射项已对齐：HTTP 方法完整覆盖 `CONNECT/ALL`，无效方法返回 10321
  而非静默放宽为通配；链式匹配保持惰性短路，`free` 仅局部捕获 `StopMatch`，
  顶层及链内 `stop/back` 均通过显式 `Result` 交给 adapter 处理。
- `fun` 根包八个函数端口和插件 hook 已逐项纠正：`IsRunFunction` 不再误建为
  trait，所有回调参数数量与返回类型恢复 Java 契约，泛型返回不再强制降级为 JSON，
  路由回调保留 request/response/handler，Rust 闭包可直接实现对应 trait。
- Redis 适配器改为 Redis 1.4 异步连接管理；连接/序列化错误不再折叠为
  `None`；更新通过 Lua 原子保持 TTL；搜索使用 `SCAN`，不使用 `KEYS`。
- Redis 8.4 真实进程测试覆盖 TTL 保持、SCAN 分页、并发读写、坏 JSON 和不可达
  服务；由测试发现并修复了初始重连缺少整体超时的问题。
- 新增 Actix Web 4.14 middleware/extractor 和 Salvo 0.85 hoop/depot adapter；两者
  均通过异步 DAO 查询，不执行 `block_on` 或同步网络 I/O。
- 新增可重复运行的 Java golden exporter；首份固定提交数据发现并修复了 Rust
  session key 多拼接一次 `login` 且未读取自定义 `token_name` 的 parity 缺陷；
  第二批 key 数据继续发现并修复 disable/safe 字段顺序及 switch 前缀差异。
- 迁移完整 `SaErrorCode`、`SaTokenException` 基础编码语义和 5 个 serializer；清除
  未接入模块树的重复草稿，所有转换错误均显式返回。第三批 Java golden 发现并
  修复 `SaHexUtil` 应输出大写十六进制的兼容差异。
- `SaSerializerTemplate` 端口已与五个实现一起完成验收：文本/字节、typed decode、
  null 透传、非法输入显式失败及 Java golden 均有证据。
- core plugin interface/holder/hook model 已恢复同类型去重、未安装销毁拒绝、
  before/override/after hook 消费，以及安装后追加 after hook 立即执行语义；Rust
  workspace 采用编译期静态插件注册，不复制 Java classpath SPI 反射。
- 新增独立 `sa-token-apikey` crate，完成 13 个 Java 业务文件的一一映射；API Key
  生成、DAO 持久化与缓存、account 索引、scope AND/OR 校验、login-id 归属校验、
  请求提取和插件生命周期均由异步契约测试覆盖，配置默认值、保存 key 与错误码由
  固定 Java 提交生成的 golden 数据校验。
- 新增独立 `sa-token-serializer-features` crate，完成 6 个 Java 业务文件的一一
  映射；五类字符表的字节往返、非法配置与非法输入显式报错、对象序列化端口和插件
  生命周期均有契约测试，四个固定字符表的输出与 Java golden 完全一致。
- 重构 `sa-token-jwt` 为 7 个 Java 业务文件的一一映射，删除旧有 `sub/exp` 秒级
  claims 与嵌套 extra 偏差；改用 Java 的 `loginType/loginId/deviceType/eff/rnStr`
  顶层载荷、毫秒 `eff`、原始 secret HS256 和 30201–30206 错误分类。Simple、Mixin、
  Stateless 的存储边界由模式契约覆盖，Rust 成功验证 Java golden Token 的签名。
- 重构 `sa-token-sign` 为 11 个 Java 业务文件的一一映射；恢复 `k=v&...&key=secret`
  canonical 字符串、字典序、MD5/SHA 系列、毫秒时间窗口和 DAO nonce 防重放，管理器
  改为显式隔离实例且不输出 full string/secret。默认配置和 MD5 结果通过 Java golden。
- SSO 已完成 37/37 项精确映射：配置、模型、名称、常量、错误码、六个函数端口、
  消息/handler、隔离 manager、client/server template、util 与框架无关 processor
  均由契约覆盖；旧顶层重复模块已删除。固定 Java golden 覆盖 auth URL、嵌套 back
  编码、ticket key 与 ticket-index key，并据此修复了 redirect 被整体编码的偏差。
  消息缺失类型、缺失字段和缺失 handler 分别保留 30022/30024/30021，配置 Debug
  输出对 secret 做脱敏。
- OAuth2 已完成 64/64 项精确映射：三种校验注解及 handler、OIDC/服务端配置、
  客户端模型、Access/Client/Refresh Token、Code、请求/ID Token 模型、数据转换
  端口与默认实现、授权类型、协议常量、29 个错误码和九类结构化异常均有契约测试。
  默认转换器保持 Java scope 分隔符、Bearer/grant type、TTL 和 extra-data 传递语义，
  并通过显式 token 生成端口传播故障。异步 DAO 保持 Java 存储 key、CRUD、三类
  FIFO/TTL 索引、scope/state/nonce 行为，反序列化错误不会降级为缺失值；DAO key
  已由固定 Java golden 校验。
  默认 loader 从隔离配置读取 client，openid/unionid MD5 已通过 Java golden；异步
  generate 链覆盖 code 替换与单次消费、access/refresh/client token 持久化及索引、
  scope hook、state 防重放和 query/fragment redirect。
  framework-neutral resolver 保持请求参数优先级、Basic client credentials、Bearer
  access/client token、RequestAuthModel 构建及 `hideStatusField` /
  `mode4ReturnAccessToken` 响应语义。
  `SaOAuth2Manager` 通过显式 `SaOAuth2Runtime` 聚合所有协议组件，组件依赖由构造器
  注入，测试实例互不共享可变全局状态。
  四类 token value 函数端口委托显式 generator 并保留错误传播；grant 认证端口使用
  `SaOAuth2Request`，两类 scope 加工端口直接修改目标模型，不依赖 Web 框架全局状态。
  授权确认、未登录视图和登录处理函数均为可注入、线程安全端口，并以 JSON value
  承载 Java `Object` 返回值。
  CommonScope 及 openid、unionid、userid handler 已恢复；handler 从显式 loader
  读取数据，缺失 client 保留 30105，access-token extra-data 不依赖预先初始化。
  OIDC handler 将 request/session/nonce 上下文与 JWT signer 作为端口注入，保持
  `iss/sub/aud/exp/iat/auth_time/nonce/azp` claims 和 refresh 重建语义。三类 grant
  handler 均为 async：授权码校验显式注入，password 不提供不安全生产默认认证器，
  refresh-token 保留 30111/30122，password 登录失败保留 30161。
  `SaOAuth2Strategy` 使用实例级 scope/grant handler registry，并显式注入 client
  校验和服务端回调；异步 `SaOAuth2Template` 统一 client/redirect/scope/token
  校验与撤销，DAO 错误全程传播；`SaOAuth2ServerProcessor` 以框架无关请求/响应模型
  实现 authorize、token、refresh、revoke、login、confirm 和 client-token 流程。
  `SaTokenPluginForOAuth2` 将三类注解处理器注册到 runtime 自有 registry，安装与销毁
  幂等。旧顶层 OAuth2 重复模块已删除，避免两套公开模型继续漂移。
- 移除未使用的 `tower-util`，缩减 Tokio feature，依赖统一到 workspace。

## 主要直接依赖

| 依赖 | 锁定版本 |
| --- | --- |
| axum | 0.8.9 |
| axum-extra | 0.12.6 |
| tower-http | 0.7.0 |
| redis | 1.4.1 |
| rand | 0.10.2 |
| jsonwebtoken | 10.4.0 |
| bcrypt | 0.19.2 |
| actix-web | 4.14.0（MSRV 1.88） |
| salvo | 0.85.0（0.95.0 要求 Rust 1.94） |
| openssl | 0.10.81（core RSA OAEP-SHA256） |
| aws-lc-rs | 1.17.3（jsonwebtoken backend） |

## 已知安全边界

已移除 `rsa 0.9.10` 及 `RUSTSEC-2023-0071` 临时豁免。core 的 RSA
OAEP-SHA256 改由 OpenSSL 0.10.81 实现，jsonwebtoken 改用 `aws_lc_rs` backend；
`cargo tree -i rsa` 已确认依赖图中不存在 rust-rsa。cargo-deny 的 advisory、
license、source 和 ban 检查通过。Actix Web 与 Salvo 因 HTTP/cookie/rand 生态版本线
产生的重复依赖已逐项写入 `deny.toml` 允许清单。

## 尚未完成

- 881 个业务映射目前为 145 `in_progress`、563 `planned`、173 `complete`；现有文件
  未逐项完成 Java parity 及证据登记，不能宣称全量迁移完成。
- `AsyncStpLogic` 的高频状态契约已覆盖，但动态 active-timeout、不同设备范围替换、
  溢出下线模式和全部 AND/OR 权限组合仍需与同步 API 做全量 parity。
- Java golden 已建立 core、serializer、API Key、JWT 与 Sign 基线，但全量 parity、
  全部 starter/demo 仍待后续阶段；OAuth2、
  密码与 RSA 还需完成面向 Java golden 数据的独立安全 parity 审计。

## 可复现命令

```bash
cargo run -p xtask -- migration-audit
cargo fmt --all --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo +stable check --workspace --all-targets --all-features --locked
cargo deny check
```
