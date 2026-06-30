// application entry point — wiring the router, middleware, state, and launching the Axum server

mod auth;
mod config;
mod db;
mod error;
mod llm;
mod pantry;
mod recipes;
mod state;

use std::net::SocketAddr;

use axum::{
    middleware,
    routing::get,
    Router,
};
use tower_http::{services::ServeDir, trace::TraceLayer};

use crate::{
    auth::middleware::require_login,
    config::Config,
    db::connect,
    state::AppState,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "recipe_creator=info,tower_http=info".into()),
        )
        .init();

    let config = Config::from_env().map_err(|e| format!("config error: {e}"))?;
    let pool = connect(&config.database_url).await?;
    let state = AppState::new(pool, config.clone());

    let public = Router::new()
        .merge(auth::routes::router())
        .nest_service("/static", ServeDir::new("static"));

    let protected = Router::new()
        .merge(pantry::routes::router())
        .merge(recipes::routes::router())
        .route("/health", get(|| async { "ok" }))
        .layer(middleware::from_fn_with_state(state.clone(), require_login));

    let app = Router::new()
        .merge(public)
        .merge(protected)
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("recipe creator listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
