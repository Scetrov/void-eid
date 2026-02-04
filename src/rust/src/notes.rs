use crate::{
    audit::{log_audit, AuditAction},
    auth::AuthenticatedUser,
    helpers::{get_user_by_discord_id, require_admin_in_tribe},
    state::AppState,
};
use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: String,
    #[serde(with = "crate::models::i64_as_string")]
    pub target_user_id: i64,
    #[serde(with = "crate::models::i64_as_string")]
    pub author_id: i64,
    pub tribe: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NoteWithAuthor {
    pub id: String,
    #[serde(with = "crate::models::i64_as_string")]
    pub target_user_id: i64,
    #[serde(with = "crate::models::i64_as_string")]
    pub author_id: i64,
    pub tribe: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub author_username: String,
    pub author_discriminator: String,
}

#[derive(Deserialize, IntoParams)]
pub struct NotesQuery {
    pub tribe: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateNoteRequest {
    pub content: String,
}

#[derive(Deserialize, ToSchema)]
pub struct EditNoteRequest {
    pub content: String,
}

#[utoipa::path(
    get,
    path = "/api/roster/{discord_id}/notes",
    params(
        ("discord_id" = String, Path, description = "Discord ID of the member"),
        NotesQuery
    ),
    responses(
        (status = 200, description = "Get notes for a member", body = Vec<NoteWithAuthor>),
        (status = 403, description = "Forbidden")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn get_notes(
    Path(discord_id): Path<String>,
    Query(query): Query<NotesQuery>,
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> impl IntoResponse {
    // Verify admin in tribe
    let (_current_user, tribe, _all_tribes) =
        match require_admin_in_tribe(&state.db, auth_user.user_id, query.tribe.as_deref()).await {
            Ok(result) => result,
            Err(e) => return e.into_response(),
        };

    // Get target user
    let target_user = match get_user_by_discord_id(&state.db, &discord_id).await {
        Ok(Some(u)) => u,
        Ok(None) => return (StatusCode::NOT_FOUND, "User not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
    };

    // Fetch notes with author info
    let notes = sqlx::query_as::<_, NoteWithAuthor>(
        r#"
        SELECT n.*, u.username as author_username, u.discriminator as author_discriminator
        FROM notes n
        JOIN users u ON n.author_id = u.id
        WHERE n.target_user_id = ? AND n.tribe = ?
        ORDER BY n.created_at DESC
        "#,
    )
    .bind(target_user.id)
    .bind(&tribe)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Json(notes).into_response()
}

#[utoipa::path(
    post,
    path = "/api/roster/{discord_id}/notes",
    params(
        ("discord_id" = String, Path, description = "Discord ID of the member"),
        NotesQuery
    ),
    request_body = CreateNoteRequest,
    responses(
        (status = 201, description = "Note created", body = Note),
        (status = 403, description = "Forbidden")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn create_note(
    Path(discord_id): Path<String>,
    Query(query): Query<NotesQuery>,
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<CreateNoteRequest>,
) -> impl IntoResponse {
    // Verify admin in tribe
    let (current_user, tribe, _all_tribes) =
        match require_admin_in_tribe(&state.db, auth_user.user_id, query.tribe.as_deref()).await {
            Ok(result) => result,
            Err(e) => return e.into_response(),
        };

    // Get target user
    let target_user = match get_user_by_discord_id(&state.db, &discord_id).await {
        Ok(Some(u)) => u,
        Ok(None) => return (StatusCode::NOT_FOUND, "User not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
    };

    if payload.content.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "Note content cannot be empty").into_response();
    }

    let note_id = Uuid::new_v4().to_string();
    let now = Utc::now();

    // Insert note
    let result = sqlx::query(
        "INSERT INTO notes (id, target_user_id, author_id, tribe, content, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&note_id)
    .bind(target_user.id)
    .bind(current_user.id)
    .bind(&tribe)
    .bind(&payload.content)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await;

    if result.is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create note").into_response();
    }

    // Audit log
    let _ = log_audit(
        &state.db,
        AuditAction::NoteCreate,
        current_user.id,
        Some(target_user.id),
        &format!(
            "Created note for {} in tribe {}: {}",
            target_user.username,
            tribe,
            if payload.content.len() > 50 {
                format!("{}...", &payload.content[..50])
            } else {
                payload.content.clone()
            }
        ),
    )
    .await;

    let note = Note {
        id: note_id,
        target_user_id: target_user.id,
        author_id: current_user.id,
        tribe,
        content: payload.content,
        created_at: now,
        updated_at: now,
    };

    (StatusCode::CREATED, Json(note)).into_response()
}

#[utoipa::path(
    put,
    path = "/api/notes/{note_id}",
    params(
        ("note_id" = String, Path, description = "Note ID")
    ),
    request_body = EditNoteRequest,
    responses(
        (status = 200, description = "Note updated", body = Note),
        (status = 403, description = "Forbidden: Not the author"),
        (status = 404, description = "Note not found")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn edit_note(
    Path(note_id): Path<String>,
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<EditNoteRequest>,
) -> impl IntoResponse {
    // Fetch the note
    let note: Option<Note> = sqlx::query_as("SELECT * FROM notes WHERE id = ?")
        .bind(&note_id)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);

    let note = match note {
        Some(n) => n,
        None => return (StatusCode::NOT_FOUND, "Note not found").into_response(),
    };

    // Verify authorship
    if note.author_id != auth_user.user_id {
        return (StatusCode::FORBIDDEN, "You can only edit your own notes").into_response();
    }

    if payload.content.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "Note content cannot be empty").into_response();
    }

    let now = Utc::now();

    // Update note
    let result = sqlx::query("UPDATE notes SET content = ?, updated_at = ? WHERE id = ?")
        .bind(&payload.content)
        .bind(now)
        .bind(&note_id)
        .execute(&state.db)
        .await;

    if result.is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update note").into_response();
    }

    // Audit log
    let _ = log_audit(
        &state.db,
        AuditAction::NoteEdit,
        auth_user.user_id,
        Some(note.target_user_id),
        &format!(
            "Edited note in tribe {}: {}",
            note.tribe,
            if payload.content.len() > 50 {
                format!("{}...", &payload.content[..50])
            } else {
                payload.content.clone()
            }
        ),
    )
    .await;

    let updated_note = Note {
        content: payload.content,
        updated_at: now,
        ..note
    };

    Json(updated_note).into_response()
}
