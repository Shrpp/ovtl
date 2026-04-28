use axum::{routing::get, Router};

use crate::{handlers::audit_log::list_audit_log, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new().route("/audit-log", get(list_audit_log))
}
