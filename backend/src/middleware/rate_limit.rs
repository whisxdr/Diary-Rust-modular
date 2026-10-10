use std::{net::SocketAddr, sync::Arc, time::Instant};

use axum::{
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use tokio::sync::Mutex;

const LIMIT: u32 = 300;
const WINDOW_SECONDS: u64 = 60;

/// Fixed-window request limiter held in process memory.
///
/// ponytail: one process, in-memory buckets. Move to a shared store when the API
/// runs behind more than one instance or when limits must survive a restart.
#[derive(Clone, Default)]
pub struct RateLimiter {
    windows: Arc<Mutex<Vec<(SocketAddr, Instant, u32)>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn allow(&self, client: SocketAddr) -> bool {
        let now = Instant::now();
        let mut windows = self.windows.lock().await;
        windows.retain(|(_, started, _)| now.duration_since(*started).as_secs() < WINDOW_SECONDS);

        match windows.iter_mut().find(|(known, _, _)| *known == client) {
            Some((_, _, count)) => {
                *count += 1;
                *count <= LIMIT
            }
            None => {
                windows.push((client, now, 1));
                true
            }
        }
    }
}

pub async fn enforce(
    State(limiter): State<RateLimiter>,
    ConnectInfo(client): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if limiter.allow(client).await {
        Ok(next.run(request).await)
    } else {
        tracing::warn!(%client, "rate limit exceeded");
        Err(StatusCode::TOO_MANY_REQUESTS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn allows_bursts_until_the_limit_is_reached() {
        let limiter = RateLimiter::new();
        let client: SocketAddr = "127.0.0.1:5000".parse().unwrap();

        for _ in 0..LIMIT {
            assert!(limiter.allow(client).await);
        }
        assert!(!limiter.allow(client).await);
    }
}
