use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::{handlers::admin_users, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users", get(admin_users::list_users).post(admin_users::create_user))
        .route(
            "/users/:id",
            delete(admin_users::deactivate_user).put(admin_users::update_user),
        )
}
