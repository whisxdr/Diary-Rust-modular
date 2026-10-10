//! Applies the committed migrations. Schema changes never run implicitly during
//! application startup; this binary is the single, explicit path.
//!
//! Usage: `cargo run --bin migrate`

use dear_my_diary::{
    config::Config,
    infrastructure::{database, telemetry},
};

#[tokio::main]
async fn main() {
    telemetry::init();

    if let Err(error) = run().await {
        tracing::error!(%error, "migration failed");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env().map_err(std::io::Error::other)?;
    let pool = database::connect(&config.database_url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("migrations are up to date");

    Ok(())
}
