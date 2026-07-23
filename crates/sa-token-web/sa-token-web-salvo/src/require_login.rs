use std::sync::Arc;

use async_trait::async_trait;
use sa_token_core::stp::AsyncStpUtil;
use salvo::http::StatusCode;
use salvo::prelude::{Depot, FlowCtrl, Handler, Request, Response};

use crate::login_id::{LOGIN_ID_KEY, TOKEN_KEY};
use crate::reject::reject;
use crate::token::extract_token;

/// Salvo hoop that authenticates a request through an async Sa-Token runtime.
pub struct RequireLogin {
    util: Arc<AsyncStpUtil>,
}

impl RequireLogin {
    /// Creates a login hoop bound to an isolated runtime facade.
    pub fn new(util: Arc<AsyncStpUtil>) -> Self {
        Self { util }
    }
}

#[async_trait]
impl Handler for RequireLogin {
    async fn handle(
        &self,
        request: &mut Request,
        depot: &mut Depot,
        response: &mut Response,
        ctrl: &mut FlowCtrl,
    ) {
        let Some(token) = extract_token(request, &self.util) else {
            reject(response, ctrl, StatusCode::UNAUTHORIZED, "authentication token is missing");
            return;
        };
        match self.util.get_login_id_by_token(&token).await {
            Ok(Some(login_id)) => {
                depot.insert(LOGIN_ID_KEY, login_id);
                depot.insert(TOKEN_KEY, token);
            }
            Ok(None) => reject(response, ctrl, StatusCode::UNAUTHORIZED, "authentication token is invalid"),
            Err(error) => {
                tracing::error!(error = %error, "Sa-Token DAO failure in Salvo adapter");
                reject(response, ctrl, StatusCode::INTERNAL_SERVER_ERROR, "authentication service is unavailable");
            }
        }
    }
}
