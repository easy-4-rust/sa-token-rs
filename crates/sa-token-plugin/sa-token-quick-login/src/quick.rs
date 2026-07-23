//! Quick login module root.

pub mod config;
pub mod function;
pub mod sa_quick_inject;
pub mod sa_quick_manager;
pub mod sa_quick_register;
pub mod web;

pub use config::sa_quick_config::SaQuickConfig;
pub use function::do_login_handle_function::DoLoginHandleFunction;
pub use sa_quick_inject::SaQuickInject;
pub use sa_quick_manager::SaQuickManager;
pub use sa_quick_register::SaQuickRegister;
pub use web::sa_quick_controller::SaQuickController;
