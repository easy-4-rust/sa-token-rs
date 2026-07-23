use salvo::prelude::Depot;

/// Depot key containing the authenticated login id.
pub const LOGIN_ID_KEY: &str = "sa_token.login_id";
/// Depot key containing the authenticated token.
pub const TOKEN_KEY: &str = "sa_token.token";

/// Reads the authenticated login id from the request depot.
pub fn login_id(depot: &Depot) -> Option<&str> {
    depot.get::<String>(LOGIN_ID_KEY).ok().map(String::as_str)
}
