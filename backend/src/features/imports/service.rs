use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::{
    features::{
        entries::{
            models::{word_count, EntryStatus},
            repository as entries_repository, tag_link,
        },
        imports::{
            dto::{ConflictStrategy, DateRange, ImportPreview, ImportSummary, Problem},
            models::ImportJob,
            parser, repository,
        },
        moods,
    },
    middleware::authentication::AuthUser,
    shared::{
        errors::{AppError, AppResult},
        portable::schema::PortableArchive,
    },
    state::AppState,
};

/// Upload and validate. Nothing is written to the diary tables here: the parsed
/// archive is stored on the job so the commit can run as a separate request.
pub async fn stage(
    state: &AppState,
    caller: AuthUser,
    filename: &str,
    bytes: &[u8],
) -> AppResult<ImportPreview> {
    let archive = parser::parse(bytes)?;
    let job = repository::create_job(&state.pool, caller.id, filename, &archive).await?;

    let mut preview = build_preview(&state.pool, caller.id, &archive).await?;
    preview.job_id = Some(job.id);

    repository::finish_job(
        &state.pool,
        job.id,
        "pending",
        serde_json::to_value(&preview)?,
        None,
    )
    .await?;

    Ok(preview)
}

pub async fn preview(state: &AppState, caller: AuthUser, job_id: Uuid) -> AppResult<ImportPreview> {
    let job = load_job(state, caller, job_id).await?;
    let mut preview = build_preview(&state.pool, caller.id, &job.staged_archive()?).await?;
    preview.job_id = Some(job.id);
    Ok(preview)
}

/// Applies the staged archive inside one transaction: either every entry lands or
/// none does. Credentials are not part of the archive, so the destination account
/// keeps its own identity.
pub async fn commit(
    state: &AppState,
    caller: AuthUser,
    job_id: Uuid,
    strategy: ConflictStrategy,
) -> AppResult<ImportSummary> {
    let job = load_job(state, caller, job_id).await?;
    if job.status == "completed" {
        return Err(AppError::Conflict(
            "This import has already been applied.".into(),
        ));
    }

    let archive = job.staged_archive()?;
    let existing: HashSet<_> = repository::existing_dates(&state.pool, caller.id)
        .await?
        .into_iter()
        .collect();
    let mood_ids = repository::mood_ids(&state.pool, &archive).await?;

    let mut transaction = state.pool.begin().await?;
    let mut summary = ImportSummary {
        job_id,
        entries_created: 0,
        entries_updated: 0,
        entries_skipped: 0,
        tags_created: 0,
        status: "completed".into(),
    };
    let mut tag_ids: HashMap<String, Uuid> = HashMap::new();

    for entry in &archive.entries {
        let conflicts = existing.contains(&entry.date);
        if conflicts && strategy == ConflictStrategy::Skip {
            summary.entries_skipped += 1;
            continue;
        }

        for tag in &archive.tags {
            if !tag_ids.contains_key(&tag.key) {
                let id = repository::tag_id_for(&mut transaction, caller.id, &tag.name).await?;
                tag_ids.insert(tag.key.clone(), id);
            }
        }

        let mood_id = entry
            .mood
            .as_deref()
            .filter(|slug| !slug.trim().is_empty())
            .and_then(|slug| mood_ids.get(slug).copied());

        let imported = entries_repository::ImportedEntry {
            date: entry.date,
            title: entry.title.trim().to_owned(),
            content: entry.content.clone(),
            word_count: word_count(&entry.content),
            mood_id,
            status: normalize_status(&entry.status).to_owned(),
            created_at: entry.created_at,
            updated_at: entry.updated_at,
        };

        let entry_id =
            entries_repository::upsert_for_import(&mut transaction, caller.id, &imported).await?;

        let ids = entry
            .tags
            .iter()
            .filter_map(|key| tag_ids.get(key).copied())
            .collect::<Vec<_>>();
        tag_link::replace_for_entry_in(&mut transaction, caller.id, entry_id, &ids).await?;

        if conflicts {
            summary.entries_updated += 1;
        } else {
            summary.entries_created += 1;
        }
    }

    summary.tags_created = tag_ids.len();

    transaction.commit().await?;
    repository::finish_job(
        &state.pool,
        job_id,
        "completed",
        serde_json::to_value(&summary)?,
        None,
    )
    .await?;

    Ok(summary)
}

pub async fn history(state: &AppState, caller: AuthUser) -> AppResult<Vec<ImportJob>> {
    repository::list_jobs(&state.pool, caller.id).await
}

/// Describes what a commit would do, including entries that reference a mood
/// this installation does not have.
pub async fn build_preview(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    archive: &PortableArchive,
) -> AppResult<ImportPreview> {
    let known_moods: HashSet<String> = moods::repository::list(pool)
        .await?
        .into_iter()
        .map(|mood| mood.slug)
        .collect();

    let existing: HashSet<_> = repository::existing_dates(pool, user_id)
        .await?
        .into_iter()
        .collect();

    let mut first: Option<chrono::NaiveDate> = None;
    let mut last: Option<chrono::NaiveDate> = None;
    let mut entries_new = 0;
    let mut entries_updated = 0;

    for entry in &archive.entries {
        first = Some(first.map_or(entry.date, |d: chrono::NaiveDate| d.min(entry.date)));
        last = Some(last.map_or(entry.date, |d: chrono::NaiveDate| d.max(entry.date)));

        if existing.contains(&entry.date) {
            entries_updated += 1;
        } else {
            entries_new += 1;
        }
    }

    let problems = archive
        .entries
        .iter()
        .filter(|entry| {
            entry
                .mood
                .as_deref()
                .is_some_and(|slug| !known_moods.contains(slug))
        })
        .map(|entry| Problem {
            entry_key: entry.key.clone(),
            message: format!(
                "mood `{}` does not exist here and will be skipped",
                entry.mood.clone().unwrap_or_default()
            ),
        })
        .collect();

    Ok(ImportPreview {
        job_id: None,
        format_version: archive.manifest.format_version,
        exported_at: archive.manifest.exported_at,
        source: archive.manifest.source.clone(),
        entries_total: archive.entries.len(),
        entries_new,
        entries_updated,
        tags_new: archive.tags.len(),
        date_range: match (first, last) {
            (Some(first), Some(last)) => Some(DateRange { first, last }),
            _ => None,
        },
        problems,
    })
}

async fn load_job(state: &AppState, caller: AuthUser, job_id: Uuid) -> AppResult<ImportJob> {
    repository::find_job(&state.pool, caller.id, job_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Import not found.".into()))
}

fn normalize_status(status: &str) -> &'static str {
    EntryStatus::parse(status)
        .unwrap_or(EntryStatus::Published)
        .as_str()
}
