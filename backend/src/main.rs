use dear_my_diary::{
    app,
    config::Config,
    infrastructure::{database, storage, telemetry},
    state::AppState,
};

#[tokio::main]
async fn main() {
    telemetry::init();

    if let Err(error) = run().await {
        tracing::error!(%error, "server stopped");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env().map_err(std::io::Error::other)?;
    let pool = database::connect(&config.database_url).await?;
    let storage = storage::init(config.storage_root.clone())?;

    let state = AppState::new(pool, config.clone(), storage);
    let router = app::build_router(state);

    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
    tracing::info!(address = %config.bind_addr, "dear my diary api listening");

    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("shutdown requested");
}
