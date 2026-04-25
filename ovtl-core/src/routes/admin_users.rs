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
        .route("/users/:id/verification-code", get(admin_users::get_verification_code))
        .route("/users/:id/password-reset-token", get(admin_users::get_password_reset_token))
}
