use uuid::Uuid;

use crate::{
    features::{
        entries::{
            dto::{
                AutosaveRequest, CreateEntryRequest, EntryQuery, StatusRequest, UpdateEntryRequest,
            },
            models::{excerpt, word_count, Entry, EntryRow, EntryStatus, EntrySummary},
            repository::{self, EntryFilter},
            tag_link, validation,
        },
        moods, tags,
    },
    middleware::authentication::AuthUser,
    shared::{
        errors::{AppError, AppResult, ValidationError},
        pagination::Page,
        time::parse_date,
    },
    state::AppState,
};

pub async fn create(
    state: &AppState,
    caller: AuthUser,
    request: CreateEntryRequest,
) -> AppResult<Entry> {
    validation::validate_title(&request.title)?;
    validation::validate_content(&request.content)?;
    validation::validate_tags(&request.tags)?;

    let date = parse_date(&request.date).map_err(ValidationError::new)?;

    let new_entry = repository::NewEntry {
        date,
        title: request.title.trim().to_owned(),
        content: request.content.clone(),
        word_count: word_count(&request.content),
        mood_id: request.mood_id,
        status: EntryStatus::Published.as_str().to_owned(),
    };

    let row = repository::insert(&state.pool, caller.id, &new_entry).await?;

    let tag_ids = tags::service::resolve_by_names(&state.pool, caller.id, &request.tags).await?;
    tag_link::replace_for_entry(&state.pool, caller.id, row.id, &tag_ids).await?;

    hydrate(state, caller.id, row).await
}

pub async fn get(state: &AppState, caller: AuthUser, id: Uuid) -> AppResult<Entry> {
    let row = repository::find_by_id(&state.pool, caller.id, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Entry not found.".into()))?;

    hydrate(state, caller.id, row).await
}

/// The diary opens on a date, so reading by date is a first-class operation.
pub async fn get_by_date(
    state: &AppState,
    caller: AuthUser,
    date: &str,
) -> AppResult<Option<Entry>> {
    let date = parse_date(date).map_err(crate::shared::errors::ValidationError::new)?;
    let Some(row) = repository::find_by_date(&state.pool, caller.id, date).await? else {
        return Ok(None);
    };
    hydrate(state, caller.id, row).await.map(Some)
}

pub async fn list(
    state: &AppState,
    caller: AuthUser,
    query: EntryQuery,
) -> AppResult<Page<EntrySummary>> {
    let (limit, offset) = query.pagination.normalized()?;

    let filter = EntryFilter {
        from: query
            .from_date()
            .map_err(crate::shared::errors::ValidationError::new)?,
        to: query
            .to_date()
            .map_err(crate::shared::errors::ValidationError::new)?,
        mood_id: query.mood_id,
        tag_id: match &query.tag {
            Some(name) => tags::service::find_id(&state.pool, caller.id, name).await?,
            None => None,
        },
        status: query.status.clone(),
    };

    let (rows, total) = repository::list(&state.pool, caller.id, &filter, limit, offset).await?;
    let ids = rows.iter().map(|row| row.id).collect::<Vec<_>>();
    let bodies = repository::contents_for(&state.pool, &ids).await?;

    let items = rows
        .into_iter()
        .map(|row| {
            let mut summary = EntrySummary::from(row);
            if let Some((_, content)) = bodies.iter().find(|(id, _)| *id == summary.id) {
                summary.excerpt = excerpt(content);
            }
            summary
        })
        .collect();

    Ok(Page::new(items, total, limit, offset))
}

pub async fn update(
    state: &AppState,
    caller: AuthUser,
    id: Uuid,
    request: UpdateEntryRequest,
) -> AppResult<Entry> {
    if let Some(title) = &request.title {
        validation::validate_title(title)?;
    }
    if let Some(content) = &request.content {
        validation::validate_content(content)?;
    }
    if let Some(tags) = &request.tags {
        validation::validate_tags(tags)?;
    }

    let next_word_count = request.content.as_deref().map(word_count);
    let row = repository::update_content(
        &state.pool,
        caller.id,
        id,
        request.title.as_deref().map(str::trim),
        request.content.as_deref(),
        next_word_count,
        request.mood_id,
    )
    .await?
    .ok_or_else(|| AppError::NotFound("Entry not found.".into()))?;

    if let Some(tags) = &request.tags {
        let tag_ids = tags::service::resolve_by_names(&state.pool, caller.id, tags).await?;
        tag_link::replace_for_entry(&state.pool, caller.id, id, &tag_ids).await?;
    }

    hydrate(state, caller.id, row).await
}

/// Called by the editor's autosave loop; behaves like `update` without tag work.
pub async fn autosave(
    state: &AppState,
    caller: AuthUser,
    id: Uuid,
    request: AutosaveRequest,
) -> AppResult<Entry> {
    update(
        state,
        caller,
        id,
        UpdateEntryRequest {
            title: request.title,
            content: request.content,
            mood_id: None,
            tags: None,
        },
    )
    .await
}

pub async fn set_status(
    state: &AppState,
    caller: AuthUser,
    id: Uuid,
    request: StatusRequest,
) -> AppResult<Entry> {
    let status = EntryStatus::parse(&request.status).ok_or_else(|| {
        crate::shared::errors::ValidationError::field("status", "must be draft or published")
    })?;

    repository::update_status(&state.pool, caller.id, id, status.as_str()).await?;
    get(state, caller, id).await
}

pub async fn delete(state: &AppState, caller: AuthUser, id: Uuid) -> AppResult<()> {
    if repository::delete(&state.pool, caller.id, id).await? {
        return Ok(());
    }
    Err(AppError::NotFound("Entry not found.".into()))
}

/// Attaches mood and tag details to a stored row. Ownership was already proven
/// by the query that produced the row.
async fn hydrate(state: &AppState, user_id: Uuid, row: EntryRow) -> AppResult<Entry> {
    let tag_rows = tag_link::for_entries(&state.pool, user_id, &[row.id]).await?;
    let mood = match row.mood_id {
        Some(id) => moods::service::summary(&state.pool, id).await?,
        None => None,
    };

    Ok(Entry {
        id: row.id,
        date: row.entry_date,
        title: row.title,
        content: row.content,
        word_count: row.word_count,
        mood,
        status: EntryStatus::parse(&row.status).unwrap_or(EntryStatus::Published),
        tags: tag_rows.into_iter().map(|(_, tag)| tag).collect(),
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}
