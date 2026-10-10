use uuid::Uuid;

use crate::{
    features::{
        accounts::{dto::UpdateProfileRequest, models::Profile, repository},
        auth::service as auth_service,
    },
    middleware::authentication::AuthUser,
    shared::{
        errors::{AppError, AppResult, ValidationError},
        validation::Validator,
    },
    state::AppState,
};

pub async fn get_profile(state: &AppState, caller: AuthUser) -> AppResult<Profile> {
    let user = auth_service::current_user(state, caller.id).await?;
    Ok(Profile {
        id: user.id,
        email: user.email,
        display_name: user.display_name,
        created_at: user.created_at,
    })
}

pub async fn update_profile(
    state: &AppState,
    caller: AuthUser,
    request: UpdateProfileRequest,
) -> AppResult<Profile> {
    Validator::new()
        .require("display_name", &request.display_name, 2, 60)
        .into_result()?;

    repository::update_display_name(&state.pool, caller.id, request.display_name.trim()).await?;
    get_profile(state, caller).await
}

/// Password rules live in the auth feature; account deletion lives here because
/// it is an account-lifecycle operation, not a login one.
pub async fn delete_account(
    state: &AppState,
    caller: AuthUser,
    confirm_display_name: &str,
) -> AppResult<crate::features::accounts::dto::DeletionSummary> {
    let user = auth_service::current_user(state, caller.id).await?;

    if confirm_display_name.trim() != user.display_name {
        return Err(ValidationError::field(
            "confirm_display_name",
            "does not match the account name",
        )
        .into());
    }

    let (summary, paths) = repository::delete_account(&state.pool, caller.id).await?;

    for path in paths {
        if let Err(error) = state.storage.delete(&path).await {
            tracing::warn!(%error, "attachment left on disk after account deletion");
        }
    }

    Ok(summary)
}

pub async fn require_account(state: &AppState, user_id: Uuid) -> AppResult<()> {
    auth_service::current_user(state, user_id)
        .await
        .map(|_| ())
        .map_err(|_| AppError::NotFound("Account not found.".into()))
}
