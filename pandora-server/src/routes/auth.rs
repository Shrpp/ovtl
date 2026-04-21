use axum::{routing::post, Router};

use crate::{
    handlers::{login::login, logout::logout, refresh::refresh, register::register, revoke::revoke},
    state::AppState,
};

/// Routes that need only tenant context (no JWT required).
pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/refresh", post(refresh))
}

/// Routes that need both tenant context + valid JWT.
pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/auth/logout", post(logout))
        .route("/auth/revoke", post(revoke))
}
