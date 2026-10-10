//! Vercel Function entrypoint.
//!
//! Serves the same router as `main.rs`, but through Vercel's function bridge
//! instead of a TCP listener. Every feature module is reused unchanged; only
//! the process entry differs.
//!
//! Used when the repository is deployed as a Vercel project with services,
//! where the API is internal and reached through the `/api` rewrite.

use dear_my_diary::{
    app,
    config::Config,
    infrastructure::{database, storage, telemetry},
    state::AppState,
};
use tower::Layer;
use vercel_runtime::{axum::VercelLayer, run, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    telemetry::init();

    let state = build_state().await?;
    let service = VercelLayer::new().layer(app::build_router(state));

    run(service).await
}

async fn build_state() -> Result<AppState, Error> {
    let config = Config::from_env().map_err(Error::from)?;
    let pool = database::connect(&config.database_url).await?;
    let storage = storage::init(config.storage_root.clone()).map_err(Error::from)?;

    tracing::info!("dear my diary api ready as a vercel function");

    Ok(AppState::new(pool, config, storage))
}
