use axum::{routing::post, Router};

use crate::{
    handlers::{login::login, register::register},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
}
