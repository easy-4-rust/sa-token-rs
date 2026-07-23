# 会话查询

---

> Sa-Token → Sa-Token-Rs。单账号终端列表用 `get_terminal_list_by_login_id`；全局检索底层走 DAO `search_data`（若 `StpUtil` 尚未封装 `search_*` 门面，可直接用 DAO / logic）。

| Java | Rust |
|---|---|
| `StpUtil.getTerminalListByLoginId` | `StpUtil::get_terminal_list_by_login_id` |
| `StpUtil.searchTokenValue` | DAO `search_data` / 待封装的 `search_token_value` |
| `SaTerminalInfo` | `SaTerminalInfo`（字段 snake_case） |

### 1、单账号会话查询

使用 `StpUtil::get_terminal_list_by_login_id(login_id)` 可获取指定账号已登录终端列表信息，例如：

``` rust
use sa_token::prelude::*;

fn main() -> SaResult<()> {
    println!("账号 10001 登录设备信息：");
    let terminal_list = StpUtil::get_terminal_list_by_login_id("10001")?;
    for ter in terminal_list {
        println!(
            "登录index={}, 设备type={}, token={}, 登录time={}",
            ter.index, ter.device_type, ter.token_value, ter.create_time
        );
    }
    Ok(())
}
```

控制台打印结果：

``` txt
账号 10001 登录设备信息：
登录index=1, 设备type=PC, token=a8fbb46f-e043-459a-a875-0a2874911be8, 登录time=1742354951192
登录index=2, 设备type=APP, token=882b6c9c-bdf9-4e8f-a42b-6e17d2fe0e34, 登录time=1742354960950
登录index=3, 设备type=WEB, token=dacac78c-0983-4819-ab8b-07e7603597fc, 登录time=1742354962848
```

一个 `SaTerminalInfo` 对象代表一个终端信息，其有如下字段：

``` rust
ter.index;        // 登录会话索引值 (该账号第几个登录的设备)
ter.device_type;  // 所属设备类型，例如：PC、WEB、HD、MOBILE、APP
ter.token_value;  // 此次登录的token值
ter.create_time;  // 登录时间
ter.device_id;    // 设备id, 设备唯一标识
ter.get_extra("key");  // 此次登录的额外自定义参数
```

`Extra` 自定义参数可以在登录时通过如下方式指定:
``` rust
use serde_json::json;
use sa_token::prelude::*;

let param = SaLoginParameter::create()
    .set_terminal_extra_data(json!({ "key": "value" }));
StpUtil::login_with_param("10001", &param)?;
```



### 2、全部会话检索

``` rust
// 查询所有已登录的 Token / Session —— Java:
// StpUtil.searchTokenValue(keyword, start, size, sortType);
// StpUtil.searchSessionId(...);
// StpUtil.searchTokenSessionId(...);
//
// Rust：底层为 SaTokenDao::search_data(prefix, keyword, start, size, ascending)
let dao = SaManager::sa_token_dao();
let token_keys = dao.search_data("satoken:login:token:", "1000", 0, 10, true)?;
```


#### 参数详解：
- `keyword`: 查询关键字，只有包括这个字符串的 token 值才会被查询出来。
- `start`: 数据开始处索引。
- `size`: 要获取的数据条数 （值为-1代表一直获取到末尾）。
- `sortType` / `ascending`: 排序方式（true=正序：先登录的在前，false=反序：后登录的在前）。

简单样例：
``` rust
// 查询 value 包括 1000 的所有 token，结果集从第 0 条开始，返回 10 条
let token_list = SaManager::sa_token_dao()
    .search_data("satoken:login:token:", "1000", 0, 10, true)?;
for token in token_list {
    println!("{token}");
}
```

#### 深入：`searchTokenValue` 和 `searchSessionId` 的区别？

- `searchTokenValue` 查询的是登录产生的所有 Token。
- `searchSessionId` 查询的是所有已登录账号会话id。

举个例子，项目配置如下：
``` rust
use std::sync::Arc;
use sa_token::prelude::*;

SaManager::set_config(Arc::new(SaTokenConfig {
    // 允许同一账号在多个设备一起登录
    is_concurrent: true,
    // 同一账号每次登录产生不同的token
    is_share: false,
    ..Default::default()
}));
```

假设此时账号A在 电脑、手机、平板 依次登录（共3次登录），账号B在 电脑、手机 依次登录（共2次登录），那么：

- `searchTokenValue` 将返回一共 5 个Token。
- `searchSessionId` 将返回一共 2 个 SessionId。

综上，若要遍历系统所有已登录的会话，代码将大致如下：
``` rust
// 获取所有已登录的会话id（prefix 以实现为准）
let session_id_list = SaManager::sa_token_dao()
    .search_data("satoken:login:session:", "", 0, -1, false)?;

for session_id in session_id_list {
    // 根据会话id，查询对应的 SaSession 对象
    let session = SaManager::sa_token_dao()
        .get_session(&session_id)?
        .expect("session");

    // 查询这个账号都在哪些设备登录了
    let terminal_list = session.terminal_list();
    println!(
        "会话id：{}，共在 {} 设备登录",
        session_id,
        terminal_list.len()
    );
}
```



<br/>

#### 注意事项：
由于会话查询底层采用了遍历方式获取数据，当数据量过大时此操作将会比较耗时，有多耗时呢？这里提供一份参考数据：
- 单机模式下：百万会话取出10条 Token 平均耗时 `0.255s`。
- Redis模式下：百万会话取出10条 Token 平均耗时 `3.322s`。

请根据业务实际水平合理调用API。


> [!WARNING| label:注意]
> 基于活跃 Token 的统计方式会比实际情况略有延迟，如果需要精确统计实时在线用户信息需要采用 WebSocket。


---

<a class="case-btn" href="https://github.com/sa-token-rust/sa-token-rs/tree/main/crates/sa-token-demo"
	target="_blank">
	本章代码示例：Sa-Token-Rs 会话查询 —— 参考 demo / DAO search_data
</a>
