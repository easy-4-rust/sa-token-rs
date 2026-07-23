use salvo::http::StatusCode;
use salvo::prelude::{FlowCtrl, Json, Response};

/// Writes a JSON error response and stops the remaining hoop chain.
pub fn reject(response: &mut Response, ctrl: &mut FlowCtrl, status: StatusCode, message: &'static str) {
    response.status_code(status);
    response.render(Json(serde_json::json!({
        "code": status.as_u16(),
        "message": message,
    })));
    ctrl.skip_rest();
}
