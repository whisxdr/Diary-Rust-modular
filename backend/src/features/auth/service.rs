use chrono::{Duration, Utc};

use crate::{
    features::auth::{
        dto::{ChangePasswordRequest, LoginRequest, RegisterRequest},
        models::IssuedSession,
        password, repository,
        validation::validate_login,
    },
    infrastructure::session,
    middleware::authentication::AuthUser,
    shared::errors::{AppError, AppResult, ValidationError},
    state::AppState,
};

/// Creates the account and opens the first session.
pub async fn register(
    state: &AppState,
    request: RegisterRequest,
    user_agent: Option<&str>,
) -> AppResult<IssuedSession> {
    request.validate()?;

    let email = request.normalized_email();
    if repository::email_exists(&state.pool, &email).await? {
        return Err(ValidationError::field("email", "is already registered").into());
    }

    let password_hash = password::hash(&request.password)?;
    let Some(user) = repository::insert_user(
        &state.pool,
        &email,
        request.display_name.trim(),
        &password_hash,
    )
    .await?
    else {
        return Err(ValidationError::field("email", "is already registered").into());
    };

    issue_session(state, user, user_agent).await
}

pub async fn login(
    state: &AppState,
    request: LoginRequest,
    user_agent: Option<&str>,
) -> AppResult<IssuedSession> {
    validate_login(&request.email, &request.password)?;

    let email = request.email.trim().to_lowercase();
    // Unknown email and wrong password return the same error on purpose.
    let Some(user) = repository::find_by_email(&state.pool, &email).await? else {
        return Err(password::invalid_credentials());
    };

    password::verify(&request.password, &user.password_hash)?;
    issue_session(state, user, user_agent).await
}

pub async fn logout(state: &AppState, caller: AuthUser) -> AppResult<()> {
    repository::delete_session(&state.pool, caller.session_id).await
}

/// Loads the stored account for an authenticated caller.
pub async fn current_user(
    state: &AppState,
    user_id: uuid::Uuid,
) -> AppResult<crate::features::auth::models::User> {
    repository::find_by_id(&state.pool, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Account not found.".into()))
}

/// Resolves a cookie token into the caller identity. Used by the auth middleware.
pub async fn resolve_session(state: &AppState, token: &str) -> AppResult<AuthUser> {
    let token_hash = session::hash_token(token);
    let Some(session) = repository::find_active_session(&state.pool, &token_hash).await? else {
        return Err(AppError::Unauthorized("Your session has expired.".into()));
    };

    Ok(AuthUser::new(session.user_id, session.id))
}

pub async fn change_password(
    state: &AppState,
    caller: AuthUser,
    request: ChangePasswordRequest,
) -> AppResult<()> {
    request.validate()?;

    let user = repository::find_by_id(&state.pool, caller.id)
        .await?
        .ok_or_else(|| AppError::NotFound("Account not found.".into()))?;

    password::verify(&request.current_password, &user.password_hash)?;

    let hash = password::hash(&request.new_password)?;
    repository::update_password_hash(&state.pool, caller.id, &hash).await?;
    repository::delete_sessions_except(&state.pool, caller.id, caller.session_id).await?;

    Ok(())
}

async fn issue_session(
    state: &AppState,
    user: crate::features::auth::models::User,
    user_agent: Option<&str>,
) -> AppResult<IssuedSession> {
    let token = session::generate_token();
    let token_hash = session::hash_token(&token);
    let expires_at = Utc::now() + Duration::hours(state.config.session_ttl_hours);

    let session =
        repository::insert_session(&state.pool, user.id, &token_hash, user_agent, expires_at)
            .await?;

    Ok(IssuedSession {
        session_id: session.id,
        token,
        expires_at,
        user,
    })
}
