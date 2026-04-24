use sea_orm::DatabaseConnection;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Instant,
};
use uuid::Uuid;

use crate::{config::Config, services::jwk_service::JwkService};

pub type RateLimiterStore = Arc<Mutex<HashMap<String, Vec<Instant>>>>;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub config: Config,
    pub jwk: Arc<JwkService>,
    pub rate_limiter: RateLimiterStore,
    /// UUID of the master tenant, resolved at startup. Used to verify admin Bearer tokens.
    pub master_tenant_id: Option<Uuid>,
}

impl AppState {
    pub fn new(
        db: DatabaseConnection,
        config: Config,
        jwk: JwkService,
        master_tenant_id: Option<Uuid>,
    ) -> Self {
        Self {
            db,
            config,
            jwk: Arc::new(jwk),
            rate_limiter: Arc::new(Mutex::new(HashMap::new())),
            master_tenant_id,
        }
    }
}
