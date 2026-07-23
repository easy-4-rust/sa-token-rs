//! SSO extension function ports.

pub mod check_ticket_append_data_function;
pub mod do_login_handle_function;
pub mod not_login_view_function;
pub mod sa_sso_message_handle_function;
pub mod send_request_function;
pub mod ticket_result_handle_function;

pub use check_ticket_append_data_function::CheckTicketAppendDataFunction;
pub use do_login_handle_function::DoLoginHandleFunction;
pub use not_login_view_function::NotLoginViewFunction;
pub use sa_sso_message_handle_function::SaSsoMessageHandleFunction;
pub use send_request_function::SendRequestFunction;
pub use ticket_result_handle_function::TicketResultHandleFunction;
