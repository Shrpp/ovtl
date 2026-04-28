use axum::{
    routing::{get, put},
    Router,
};

use crate::{handlers::admin_identity_providers, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/identity-providers",
            get(admin_identity_providers::list_idps).post(admin_identity_providers::create_idp),
        )
        .route(
            "/identity-providers/:id",
            put(admin_identity_providers::update_idp).delete(admin_identity_providers::delete_idp),
        )
}
