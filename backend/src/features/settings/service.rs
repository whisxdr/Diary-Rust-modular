use crate::{
    features::settings::{
        models::{Preferences, UpdatePreferencesRequest},
        repository,
    },
    middleware::authentication::AuthUser,
    shared::{errors::AppResult, validation::Validator},
    state::AppState,
};

pub async fn get(state: &AppState, caller: AuthUser) -> AppResult<Preferences> {
    repository::load(&state.pool, caller.id).await
}

pub async fn update(
    state: &AppState,
    caller: AuthUser,
    request: UpdatePreferencesRequest,
) -> AppResult<Preferences> {
    if let Some(per_page) = request.entries_per_page {
        Validator::new()
            .range("entries_per_page", per_page, 5, 100)
            .into_result()?;
    }

    let mut preferences = repository::load(&state.pool, caller.id).await?;
    request.apply(&mut preferences);
    repository::save(&state.pool, caller.id, &preferences).await?;

    Ok(preferences)
}
