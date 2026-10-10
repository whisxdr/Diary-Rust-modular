use axum::{
    extract::{Extension, Multipart, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use uuid::Uuid;

use crate::{
    features::imports::{dto::ImportRequest, parser, service},
    middleware::authentication::AuthUser,
    shared::errors::{AppError, AppResult, ValidationError},
    state::AppState,
};

/// Upload and validate an archive. The response is the preview; nothing has been
/// written to the diary yet.
pub async fn upload(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthUser>,
    mut multipart: Multipart,
) -> AppResult<Response> {
    let mut filename = "backup.json".to_owned();
    let mut bytes: Vec<u8> = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(ValidationError::new(e.to_string())))?
    {
        if field.name() != Some("file") {
            continue;
        }

        filename = field.file_name().unwrap_or("backup.json").to_owned();
        let data = field
            .bytes()
            .await
            .map_err(|e| AppError::Validation(ValidationError::new(e.to_string())))?;
        bytes = data.to_vec();
        break;
    }

    if bytes.is_empty() {
        return Err(AppError::Validation(ValidationError::field(
            "file",
            "is required",
        )));
    }
    if bytes.len() > parser::MAX_ARCHIVE_BYTES {
        return Err(AppError::PayloadTooLarge(format!(
            "archive exceeds the {} byte limit",
            parser::MAX_ARCHIVE_BYTES
        )));
    }

    let preview = service::stage(&state, caller, &filename, &bytes).await?;
    Ok((StatusCode::OK, Json(serde_json::to_value(preview)?)).into_response())
}

pub async fn preview(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthUser>,
    Path(job_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::to_value(
        service::preview(&state, caller, job_id).await?,
    )?))
}

pub async fn commit(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthUser>,
    Json(request): Json<ImportRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let summary =
        service::commit(&state, caller, request.job_id, request.conflict_strategy).await?;
    Ok(Json(serde_json::to_value(summary)?))
}

pub async fn history(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthUser>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::to_value(
        service::history(&state, caller).await?,
    )?))
}
