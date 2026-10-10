use axum::{
    body::Body,
    extract::{Extension, State},
    http::{header, StatusCode},
    response::Response,
};

use crate::{
    features::exports::service, middleware::authentication::AuthUser, shared::errors::AppResult,
    state::AppState,
};

pub async fn download(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthUser>,
) -> AppResult<Response> {
    let bundle = service::build_archive(&state, caller).await?;
    let bytes = service::serialize(&bundle.archive)?;

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::CONTENT_LENGTH, bytes.len())
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", bundle.file_name),
        )
        .body(Body::from(bytes))
        .map_err(|e| {
            crate::shared::errors::AppError::Internal(format!("cannot build export response: {e}"))
        })
}
