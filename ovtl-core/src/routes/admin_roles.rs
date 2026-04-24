use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::{handlers::admin_roles, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/roles", get(admin_roles::list_roles).post(admin_roles::create_role))
        .route("/roles/:id", put(admin_roles::update_role).delete(admin_roles::delete_role))
        .route("/users/:id/roles", get(admin_roles::list_user_roles).post(admin_roles::assign_user_role))
        .route("/users/:user_id/roles/:role_id", delete(admin_roles::revoke_user_role))
}
