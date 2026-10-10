use axum::{
    extract::{Extension, State},
    http::{header, HeaderMap, HeaderValue},
    response::{IntoResponse, Response},
    Json,
};

use crate::{
    features::auth::{dto::*, models::IssuedSession, service},
    infrastructure::session,
    middleware::authentication::AuthUser,
    shared::errors::AppResult,
    state::AppState,
};

pub async fn issue_csrf(State(state): State<AppState>) -> Response {
    let token = session::generate_csrf_token();
    let max_age = state.config.session_ttl_hours * 3600;
    let cookie = session::build_csrf_cookie(
        session::CookieOptions::new(&state.config.csrf_cookie_name, &state.config),
        &token,
        max_age,
    );

    ([(header::SET_COOKIE, cookie)], Json(CsrfResponse { token })).into_response()
}

pub async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<RegisterRequest>,
) -> AppResult<Response> {
    let issued = service::register(&state, request, user_agent(&headers)).await?;
    Ok(with_session_cookie(&state, issued))
}

pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<LoginRequest>,
) -> AppResult<Response> {
    let issued = service::login(&state, request, user_agent(&headers)).await?;
    Ok(with_session_cookie(&state, issued))
}

pub async fn logout(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Response> {
    service::logout(&state, user).await?;

    let cleared = session::clear_cookie(session::CookieOptions::new(
        &state.config.session_cookie_name,
        &state.config,
    ));
    Ok((
        [(header::SET_COOKIE, cleared)],
        Json(MessageResponse::new("Signed out.")),
    )
        .into_response())
}

pub async fn change_password(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(request): Json<ChangePasswordRequest>,
) -> AppResult<Response> {
    service::change_password(&state, user, request).await?;
    Ok(Json(MessageResponse::new("Password updated.")).into_response())
}

pub async fn current_session(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<SessionResponse>> {
    let stored = service::current_user(&state, user.id).await?;

    Ok(Json(SessionResponse {
        user: UserProfile::from(&stored),
        expires_at: session::current_expiry(&state.config),
    }))
}

fn with_session_cookie(state: &AppState, issued: IssuedSession) -> Response {
    let max_age = state.config.session_ttl_hours * 3600;
    let cookie = session::build_cookie(
        session::CookieOptions::new(&state.config.session_cookie_name, &state.config),
        &issued.token,
        max_age,
    );
    let body = SessionResponse {
        user: UserProfile::from(&issued.user),
        expires_at: issued.expires_at,
    };

    let mut response = Json(body).into_response();
    if let Ok(value) = HeaderValue::from_str(&cookie) {
        response.headers_mut().append(header::SET_COOKIE, value);
    }
    response
}

fn user_agent(headers: &HeaderMap) -> Option<&str> {
    headers.get(header::USER_AGENT)?.to_str().ok()
}

#[derive(serde::Serialize)]
struct CsrfResponse {
    token: String,
}
