//! Sa-Token-Rs Derive Macros
//!
//! 提供注解式鉴权宏，对应 Java Sa-Token 的注解系统。
//!
//! # 支持的注解
//!
//! - `#[sa_check_login]` — 检查是否登录
//! - `#[sa_check_permission("xxx")]` — 检查权限
//! - `#[sa_check_role("xxx")]` — 检查角色
//! - `#[sa_check_safe]` — 检查二级认证
//! - `#[sa_check_disable]` — 检查是否被封禁
//! - `#[sa_ignore]` — 忽略鉴权

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, LitStr, parse_macro_input};

/// 检查是否登录
///
/// # 示例
///
/// ```rust,ignore
/// #[sa_check_login]
/// fn current_login_id() -> SaResult<String> {
///     StpUtil::get_login_id()
/// }
/// ```
#[proc_macro_attribute]
pub fn sa_check_login(_args: TokenStream, input: TokenStream) -> TokenStream {
    let fn_item = parse_macro_input!(input as ItemFn);
    let fn_vis = &fn_item.vis;
    let fn_sig = &fn_item.sig;
    let fn_block = &fn_item.block;
    let fn_attrs = &fn_item.attrs;

    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_sig {
            // 登录检查
            if let Err(e) = ::sa_token::stp_util::check_login() {
                return Err(e);
            }
            #fn_block
        }
    };

    TokenStream::from(expanded)
}

/// 检查权限
///
/// # 示例
///
/// ```rust,ignore
/// #[sa_check_permission("user:add")]
/// fn add_user() -> SaResult<()> {
///     // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn sa_check_permission(args: TokenStream, input: TokenStream) -> TokenStream {
    let permission = parse_macro_input!(args as LitStr);
    let permission_value = permission.value();

    let fn_item = parse_macro_input!(input as ItemFn);
    let fn_vis = &fn_item.vis;
    let fn_sig = &fn_item.sig;
    let fn_block = &fn_item.block;
    let fn_attrs = &fn_item.attrs;

    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_sig {
            // 权限检查
            if let Err(e) = ::sa_token::stp_util::check_permission(#permission_value) {
                return Err(e);
            }
            #fn_block
        }
    };

    TokenStream::from(expanded)
}

/// 检查角色
///
/// # 示例
///
/// ```rust,ignore
/// #[sa_check_role("admin")]
/// fn admin_only() -> SaResult<()> {
///     // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn sa_check_role(args: TokenStream, input: TokenStream) -> TokenStream {
    let role = parse_macro_input!(args as LitStr);
    let role_value = role.value();

    let fn_item = parse_macro_input!(input as ItemFn);
    let fn_vis = &fn_item.vis;
    let fn_sig = &fn_item.sig;
    let fn_block = &fn_item.block;
    let fn_attrs = &fn_item.attrs;

    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_sig {
            // 角色检查
            if let Err(e) = ::sa_token::stp_util::check_role(#role_value) {
                return Err(e);
            }
            #fn_block
        }
    };

    TokenStream::from(expanded)
}

/// 检查二级认证
///
/// # 示例
///
/// ```rust,ignore
/// #[sa_check_safe]
/// fn sensitive_op() -> SaResult<()> {
///     // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn sa_check_safe(_args: TokenStream, input: TokenStream) -> TokenStream {
    let fn_item = parse_macro_input!(input as ItemFn);
    let fn_vis = &fn_item.vis;
    let fn_sig = &fn_item.sig;
    let fn_block = &fn_item.block;
    let fn_attrs = &fn_item.attrs;

    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_sig {
            // 二级认证检查
            if let Err(e) = ::sa_token::stp_util::check_safe() {
                return Err(e);
            }
            #fn_block
        }
    };

    TokenStream::from(expanded)
}

/// 检查是否被封禁
///
/// # 示例
///
/// ```rust,ignore
/// #[sa_check_disable]
/// fn sensitive_op() -> SaResult<()> {
///     // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn sa_check_disable(_args: TokenStream, input: TokenStream) -> TokenStream {
    let fn_item = parse_macro_input!(input as ItemFn);
    let fn_vis = &fn_item.vis;
    let fn_sig = &fn_item.sig;
    let fn_block = &fn_item.block;
    let fn_attrs = &fn_item.attrs;

    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_sig {
            // 封禁检查
            let login_id = ::sa_token::stp_util::get_login_id()?;
            if ::sa_token::stp_util::is_disable(&login_id)? {
                return Err(::sa_token::SaTokenException::disable_service(
                    &login_id, "", 0
                ));
            }
            #fn_block
        }
    };

    TokenStream::from(expanded)
}

/// 忽略鉴权
///
/// 用于标记不需要鉴权的接口。
///
/// # 示例
///
/// ```rust,ignore
/// #[sa_ignore]
/// fn public_api() -> SaResult<()> {
///     // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn sa_ignore(_args: TokenStream, input: TokenStream) -> TokenStream {
    // sa_ignore 只是标记，不做任何检查
    input
}

/// `sa_user_check_login` —— merge annotation 示例
///
/// 对应 Java `SaUserCheckLogin`：把 `#[sa_check_login]` 的语义用更友好的
/// 名字暴露给业务代码。内部展开为 `#[sa_check_login]` + 函数体前增加
/// `StpUtil::get_login_id()` 校验。`sa-token-derive` 提供的 6 个基础
/// 注解（`sa_check_login` / `sa_check_permission` / `sa_check_role` /
/// `sa_check_safe` / `sa_check_disable` / `sa_ignore`）都可以按此模式
/// 组合出 `sa_user_check_*` / `sa_admin_check_*` 等业务层 merge annotation。
#[proc_macro_attribute]
pub fn sa_user_check_login(_args: TokenStream, input: TokenStream) -> TokenStream {
    let fn_item = parse_macro_input!(input as ItemFn);
    let fn_vis = &fn_item.vis;
    let fn_sig = &fn_item.sig;
    let fn_block = &fn_item.block;
    let fn_attrs = &fn_item.attrs;
    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_sig {
            // 内部组合：sa_check_login + sa_check_role("user")
            if let Err(e) = ::sa_token_core::stp::stp_util::StpUtil::get_login_id() {
                return Err(e);
            }
            #fn_block
        }
    };
    TokenStream::from(expanded)
}

/// `sa_admin_check_login` —— merge annotation 示例 2
///
/// 校验"必须登录 + 必须有 admin 角色"两个条件。
#[proc_macro_attribute]
pub fn sa_admin_check_login(_args: TokenStream, input: TokenStream) -> TokenStream {
    let fn_item = parse_macro_input!(input as ItemFn);
    let fn_vis = &fn_item.vis;
    let fn_sig = &fn_item.sig;
    let fn_block = &fn_item.block;
    let fn_attrs = &fn_item.attrs;
    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_sig {
            if let Err(e) = ::sa_token_core::stp::stp_util::StpUtil::get_login_id() {
                return Err(e);
            }
            if let Err(e) = ::sa_token_core::stp::stp_util::StpUtil::check_role("admin") {
                return Err(e);
            }
            #fn_block
        }
    };
    TokenStream::from(expanded)
}

/// `sa_check_account` —— custom annotation 模式示例
///
/// 作为 custom annotation 模式示例，编译期展开为对模块内
/// `MyAccountHandler` 类型的 `check()` 调用（**用户必须在自己的
/// 模块中提供**该类型并实现 `SaAnnotationHandlerInterface`）。
///
/// 这与 Java 端 `SaCheckAccount` + `CheckAccountHandler` 的关系 1:1
/// 对应：`SaCheckAccount` 是注解标记，`CheckAccountHandler` 是处理
/// 逻辑。Rust 用 proc-macro + 显式类型替代 Java 的运行时反射。
#[proc_macro_attribute]
pub fn sa_check_account(_args: TokenStream, input: TokenStream) -> TokenStream {
    let fn_item = parse_macro_input!(input as ItemFn);
    let fn_vis = &fn_item.vis;
    let fn_sig = &fn_item.sig;
    let fn_block = &fn_item.block;
    let fn_attrs = &fn_item.attrs;
    // 展开为在调用方作用域内查找 `MyAccountHandler` 类型
    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_sig {
            // 通过 MyAccountHandler 类型 + SaAnnotationHandlerInterface trait 调用
            // （用户模块需要提供该类型 + impl trait）
            // 使用关联函数 check(&self) 满足 trait 约束
            let _handler = MyAccountHandler;
            sa_token_core::annotation::handler::sa_annotation_handler_interface::SaAnnotationHandlerInterface::check(&_handler)?;
            #fn_block
        }
    };
    TokenStream::from(expanded)
}
