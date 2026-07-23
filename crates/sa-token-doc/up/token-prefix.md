# Token 提交前缀

> Sa-Token → Sa-Token-Rs。

| Java | Rust |
|---|---|
| `sa-token.token-prefix: Bearer` | `SaTokenConfig.token_prefix = "Bearer".into()` |
| `cookie-auto-fill-prefix` | `SaTokenConfig.cookie_auto_fill_prefix = true` |

### 需求场景

在某些系统中，前端提交token时会在前面加个固定的前缀，例如：

``` js
{
	"satoken": "Bearer xxxx-xxxx-xxxx-xxxx"
}
```

此时后端如果不做任何特殊处理，框架将会把`Bearer `视为token的一部分，无法正常读取token信息，导致鉴权失败。

为此，我们需要在配置中添加：

<!---------------------------- tabs:start ---------------------------->
<!------------- tab:Rust 配置  ------------->
``` rust
use std::sync::Arc;
use sa_token::prelude::*;

SaManager::set_config(Arc::new(SaTokenConfig {
    // 指定 token 提交时的前缀
    token_prefix: "Bearer".into(),
    ..Default::default()
}));
```
<!------------- tab:Java yaml 对照  ------------->
``` yaml
sa-token:
	# 指定 token 提交时的前缀
	token-prefix: Bearer
```
<!------------- tab:Java properties 对照  ------------->
``` properties
# token前缀
sa-token.token-prefix=Bearer
```
<!---------------------------- tabs:end ---------------------------->


此时 Sa-Token-Rs 便可在读取 Token 时裁剪掉 `Bearer`，成功获取`xxxx-xxxx-xxxx-xxxx`。

注：**Token前缀  与 Token值 之间必须有一个空格**


### Cookie 模式自动填充前缀

由于`Cookie`中无法存储空格字符，所以配置 Token 前缀后，Cookie 模式将会失效，无法成功提交带有前缀的 token。

如果需要在这种场景下仍然使用 Cookie 模式验证 token，可以使用 `cookie_auto_fill_prefix` 配置项打开 Cookie 模式自动填充前缀：

<!---------------------------- tabs:start ---------------------------->
<!------------- tab:Rust 配置  ------------->
``` rust
use std::sync::Arc;
use sa_token::prelude::*;

SaManager::set_config(Arc::new(SaTokenConfig {
    token_prefix: "Bearer".into(),
    // 指定 Cookie 模式下自动填充 token 提交前缀
    cookie_auto_fill_prefix: true,
    ..Default::default()
}));
```
<!------------- tab:Java yaml 对照  ------------->
``` yaml
sa-token:
	# 指定 Cookie 模式下自动填充 token 提交前缀
	cookie-auto-fill-prefix: true
```
<!------------- tab:Java properties 对照  ------------->
``` properties
# 指定 Cookie 模式下自动填充 token 提交前缀
sa-token.cookie-auto-fill-prefix=true
```
<!---------------------------- tabs:end ---------------------------->
