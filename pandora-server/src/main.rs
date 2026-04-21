use axum::{routing::get, Json, Router};
use pandora_server::{
    config, db,
    middleware::{auth::auth_middleware, tenant::tenant_middleware},
    routes,
    state::AppState,
};
use serde_json::json;
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    init_tracing();

    let config = config::Config::from_env().unwrap_or_else(|e| {
        eprintln!("Config error: {e}");
        std::process::exit(1);
    });

    let db = db::connect(&config.database_url).await.unwrap_or_else(|e| {
        eprintln!("DB connection failed: {e}");
        std::process::exit(1);
    });

    let state = AppState::new(db, config.clone());

    let app = build_router(state);

    let addr: SocketAddr = format!("{}:{}", config.server_host, config.server_port)
        .parse()
        .expect("invalid server address");

    info!("Pandora running on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn build_router(state: AppState) -> Router {
    let public = Router::new().route("/health", get(health));

    // /auth/* — tenant context only (no JWT yet)
    let auth_routes = routes::auth::router().layer(axum::middleware::from_fn_with_state(
        state.clone(),
        tenant_middleware,
    ));

    // /users/* — tenant context + JWT auth (tenant runs first, outermost layer)
    let user_routes = routes::user::router()
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            tenant_middleware,
        ));

    Router::new()
        .merge(public)
        .merge(auth_routes)
        .merge(user_routes)
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok", "version": env!("CARGO_PKG_VERSION") }))
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "pandora_server=info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();
}
