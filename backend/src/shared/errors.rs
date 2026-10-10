use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

pub use crate::shared::validation::ValidationError;

/// Application errors are expected outcomes and carry a stable machine-readable code.
/// Anything else becomes an opaque 500.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Validation(#[from] ValidationError),
    #[error("{0}")]
    Unauthorized(String),
    #[error("{0}")]
    Forbidden(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    PayloadTooLarge(String),
    #[error("database error: {0}")]
    Database(sqlx::Error),
    #[error("storage error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("{0}")]
    Internal(String),
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Validation(_) => "validation_error",
            Self::Unauthorized(_) => "unauthorized",
            Self::Forbidden(_) => "forbidden",
            Self::NotFound(_) => "not_found",
            Self::Conflict(_) => "conflict",
            Self::PayloadTooLarge(_) => "payload_too_large",
            Self::Database(_) | Self::Io(_) | Self::Serde(_) | Self::Internal(_) => {
                "internal_error"
            }
        }
    }

    fn status(&self) -> StatusCode {
        match self {
            Self::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::PayloadTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
            Self::Database(_) | Self::Io(_) | Self::Serde(_) | Self::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        // Infrastructure failures are logged in full but never leaked to the client.
        if self.status().is_server_error() {
            tracing::error!(error = %self, "request failed");
        }

        let message = if self.status().is_server_error() {
            "Something went wrong.".to_owned()
        } else {
            self.to_string()
        };

        (
            self.status(),
            Json(ErrorBody {
                code: self.code(),
                message,
            }),
        )
            .into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        match &error {
            sqlx::Error::RowNotFound => Self::NotFound("Resource not found.".into()),
            sqlx::Error::Database(db) if db.is_unique_violation() => {
                Self::Conflict("That value is already taken.".into())
            }
            sqlx::Error::Database(db) if db.is_foreign_key_violation() => {
                Self::Validation(ValidationError::new("referenced record does not exist"))
            }
            _ => Self::Database(error),
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn error_type_crosses_threads() {
        assert_send_sync::<AppError>();
    }
}
