use sqlx::PgPool;

use crate::{config::Config, infrastructure::storage::Storage};

/// Shared, read-only runtime dependencies. Feature-specific state stays inside
/// the feature that owns it.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Config,
    pub storage: Storage,
}

impl AppState {
    pub fn new(pool: PgPool, config: Config, storage: Storage) -> Self {
        Self {
            pool,
            config,
            storage,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send_sync<T: Send + Sync + Clone>() {}

    #[test]
    fn app_state_is_shareable() {
        assert_send_sync::<AppState>();
    }
}
