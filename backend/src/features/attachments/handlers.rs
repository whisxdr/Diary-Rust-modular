use axum::{
    body::Body,
    extract::{Extension, Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use uuid::Uuid;

use crate::{
    features::attachments::{dto::UploadQuery, models::UploadedFile, service},
    middleware::authentication::AuthUser,
    shared::errors::{AppError, AppResult},
    state::AppState,
};

pub async fn upload(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthUser>,
    Query(query): Query<UploadQuery>,
    mut multipart: Multipart,
) -> AppResult<Response> {
    let mut file: Option<UploadedFile> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::Validation(crate::shared::errors::ValidationError::new(e.to_string()))
    })? {
        if field.name() != Some("file") {
            continue;
        }

        let file_name = field
            .file_name()
            .map(str::to_owned)
            .ok_or_else(|| AppError::Validation(missing_file_name()))?;
        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_owned();

        let bytes = field.bytes().await.map_err(|e| {
            AppError::Validation(crate::shared::errors::ValidationError::new(e.to_string()))
        })?;

        if bytes.len() > state.config.max_upload_bytes {
            return Err(AppError::PayloadTooLarge(format!(
                "file exceeds the {} byte limit",
                state.config.max_upload_bytes
            )));
        }

        file = Some(UploadedFile {
            file_name,
            content_type,
            bytes: bytes.to_vec(),
        });
        break;
    }

    let file = file.ok_or_else(|| AppError::Validation(missing_file_name()))?;
    let attachment = service::upload(&state, caller, query.entry_id, file).await?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(attachment)?)).into_response())
}

pub async fn list_for_entry(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthUser>,
    Path(entry_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let attachments = service::list_for_entry(&state, caller, entry_id).await?;
    Ok(Json(serde_json::to_value(attachments)?))
}

pub async fn download(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Response> {
    let (row, bytes) = service::download(&state, caller, id).await?;

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, row.content_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("inline; filename=\"{}\"", sanitize(&row.original_name)),
        )
        .header(header::CONTENT_LENGTH, bytes.len())
        .body(Body::from(bytes))
        .map_err(|e| AppError::Internal(format!("cannot build download response: {e}")))
}

/// Removes characters that would break a Content-Disposition header.
fn sanitize(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .collect()
}

pub async fn delete(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Response> {
    service::delete(&state, caller, id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

fn missing_file_name() -> crate::shared::errors::ValidationError {
    crate::shared::errors::ValidationError::field("file", "is required")
}
