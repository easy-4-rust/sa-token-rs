//! Web integration mapping for Java `SaFirewallCheckFilterForServlet`.
//! Responsibility is implemented by the `axum` adapter instead of Spring/Servlet crates.

/// Firewall hooks are evaluated in core strategy; adapters call this before auth.
pub fn check_firewall() -> Result<(), sa_token_core::exception::SaTokenException> {
    // Firewall hooks are wired through core strategy in application setup.
    Ok(())
}
