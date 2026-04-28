use axum::{
    routing::{delete, get},
    Router,
};

use crate::{handlers::admin_sessions, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/sessions", get(admin_sessions::list_sessions))
        .route("/sessions/:id", delete(admin_sessions::delete_session))
}
