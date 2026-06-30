use std::sync::Arc;

use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use reqwest::Client;
use sqlx::SqlitePool;

use crate::{auth::session::cookie_key, config::Config};

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub config: Arc<Config>,
    pub cookie_key: Key,
    pub http: Client,
}

impl FromRef<AppState> for SqlitePool {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.cookie_key.clone()
    }
}

impl FromRef<AppState> for Arc<Config> {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}

impl FromRef<AppState> for Client {
    fn from_ref(state: &AppState) -> Self {
        state.http.clone()
    }
}

impl AppState {
    pub fn new(db: SqlitePool, config: Config) -> Self {
        let cookie_key = cookie_key(&config.app_secret);
        Self {
            db,
            config: Arc::new(config),
            cookie_key,
            http: Client::new(),
        }
    }
}
