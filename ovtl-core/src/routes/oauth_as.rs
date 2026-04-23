use axum::{
    routing::{get, post},
    Router,
};

use crate::{handlers::oauth_as, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/oauth/authorize", get(oauth_as::authorize))
        .route("/oauth/token", post(oauth_as::token))
        .route("/oauth/introspect", post(oauth_as::introspect))
}
