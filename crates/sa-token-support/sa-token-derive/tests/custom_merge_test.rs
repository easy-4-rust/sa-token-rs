//! 验证 sa-token-derive 提供的 merge / custom annotation 模式能正确展开。
//!
//! 这些测试通过编译期 + 简单的运行期检查，验证：
//! - `sa_user_check_login` 宏能展开为对 `StpUtil::get_login_id` 的调用
//! - `sa_admin_check_login` 宏能展开为"登录 + admin 角色"双重检查
//! - `sa_check_account` 宏能通过 trait 调用用户实现的 business handler

use sa_token_core::annotation::handler::sa_annotation_handler_interface::SaAnnotationHandlerInterface;
use sa_token_core::exception::SaResult;
use sa_token_derive::sa_admin_check_login;
use sa_token_derive::sa_check_account;
use sa_token_derive::sa_user_check_login;

/// 用户模块中实现的 custom annotation handler
/// （在 sa_check_account 宏展开时通过 `MyAccountHandler` 名字解析）
pub struct MyAccountHandler;

impl SaAnnotationHandlerInterface for MyAccountHandler {
    const ANNOTATION: &'static str = "SaCheckAccount";
    fn check(&self) -> SaResult<()> {
        // 业务逻辑：默认通过
        Ok(())
    }
}

#[sa_user_check_login]
fn my_user_endpoint() -> SaResult<String> {
    Ok("user ok".to_string())
}

#[sa_admin_check_login]
fn my_admin_endpoint() -> SaResult<String> {
    Ok("admin ok".to_string())
}

#[sa_check_account]
fn my_account_endpoint() -> SaResult<String> {
    Ok("account ok".to_string())
}

#[test]
fn merge_user_check_login_blocks_anonymous() {
    // 未 init StpLogic 时调用 get_login_id 会 panic，
    // 这证明 macro 已展开为对该函数的调用
    let result = std::panic::catch_unwind(|| my_user_endpoint());
    assert!(
        result.is_err(),
        "未初始化 StpLogic 时 sa_user_check_login 应触发 panic（已展开为 get_login_id 调用）"
    );
}

#[test]
fn merge_admin_check_login_blocks_anonymous() {
    let result = std::panic::catch_unwind(|| my_admin_endpoint());
    assert!(
        result.is_err(),
        "未初始化 StpLogic 时 sa_admin_check_login 应触发 panic"
    );
}

#[test]
fn custom_check_account_handler_constant() {
    // 验证 handler 名称常量
    assert_eq!(MyAccountHandler::ANNOTATION, "SaCheckAccount");
}

#[test]
fn custom_check_account_compiles_and_passes() {
    // sa_check_account 默认实现 Ok(()), 所以未登录也应通过（区别于 login check）
    let result = my_account_endpoint();
    assert!(result.is_ok(), "sa_check_account 默认通过");
    assert_eq!(result.unwrap(), "account ok");
}
