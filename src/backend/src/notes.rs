use crate::{
    audit::{log_audit, AuditAction},
    auth::AuthenticatedUser,
    helpers::{
        get_user_by_discord_id, get_user_tribes, require_admin_in_tribe, validate_discord_id,
        validate_token_text,
    },
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
    if let Err(e) = validate_discord_id(&discord_id) {
        return e.into_response();
    }
    if let Some(tribe) = query.tribe.as_deref() {
        if let Err(e) = validate_token_text(tribe, "Invalid tribe", 64) {
            return e.into_response();
        }
    }

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

    // Verify the target is a member of the selected tribe before reading.
    let target_tribes = match get_user_tribes(&state.db, target_user.id).await {
        Ok(t) => t,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
    };
    if !target_tribes.contains(&tribe) {
        return (StatusCode::FORBIDDEN, "Forbidden").into_response();
    }

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
    if let Err(e) = validate_discord_id(&discord_id) {
        return e.into_response();
    }
    if let Some(tribe) = query.tribe.as_deref() {
        if let Err(e) = validate_token_text(tribe, "Invalid tribe", 64) {
            return e.into_response();
        }
    }

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

    // Verify the target is a member of the selected tribe before creating.
    let target_tribes = match get_user_tribes(&state.db, target_user.id).await {
        Ok(t) => t,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
    };
    if !target_tribes.contains(&tribe) {
        return (StatusCode::FORBIDDEN, "Forbidden").into_response();
    }

    if payload.content.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "Note content cannot be empty").into_response();
    }

    if payload.content.len() > 10_000 {
        return (
            StatusCode::BAD_REQUEST,
            "Note content exceeds maximum length (10,000 characters)",
        )
            .into_response();
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
    if Uuid::parse_str(&note_id).is_err() {
        return (StatusCode::BAD_REQUEST, "Invalid note id").into_response();
    }

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

    if payload.content.len() > 10_000 {
        return (
            StatusCode::BAD_REQUEST,
            "Note content exceeds maximum length (10,000 characters)",
        )
            .into_response();
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

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_db() -> crate::db::DbPool {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create memory pool");

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Migrations failed");

        // Insert admin user
        sqlx::query("INSERT INTO users (id, discord_id, username, discriminator, is_admin) VALUES (?, ?, ?, ?, ?)")
            .bind(1001_i64)
            .bind("admin-discord-id")
            .bind("AdminUser")
            .bind("0001")
            .bind(true)
            .execute(&pool)
            .await
            .unwrap();

        // Insert target user
        sqlx::query("INSERT INTO users (id, discord_id, username, discriminator, is_admin) VALUES (?, ?, ?, ?, ?)")
            .bind(1002_i64)
            .bind("target-discord-id")
            .bind("TargetUser")
            .bind("0002")
            .bind(false)
            .execute(&pool)
            .await
            .unwrap();

        // Add admin to tribe
        sqlx::query("INSERT INTO user_tribes (user_id, tribe, is_admin) VALUES (?, ?, ?)")
            .bind(1001_i64)
            .bind("test_tribe")
            .bind(true)
            .execute(&pool)
            .await
            .unwrap();

        pool
    }

    #[test]
    fn test_note_serialization() {
        let note = Note {
            id: "note-123".to_string(),
            target_user_id: 1002,
            author_id: 1001,
            tribe: "test_tribe".to_string(),
            content: "Test note content".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string(&note).expect("Serialize failed");
        assert!(json.contains("\"id\":\"note-123\""));
        assert!(json.contains("\"targetUserId\":")); // camelCase
        assert!(json.contains("\"authorId\":"));
        assert!(json.contains("\"tribe\":\"test_tribe\""));
        assert!(json.contains("\"content\":\"Test note content\""));
    }

    #[test]
    fn test_note_with_author_serialization() {
        let note = NoteWithAuthor {
            id: "note-456".to_string(),
            target_user_id: 1002,
            author_id: 1001,
            tribe: "test_tribe".to_string(),
            content: "Note with author".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            author_username: "AuthorUser".to_string(),
            author_discriminator: "1234".to_string(),
        };

        let json = serde_json::to_string(&note).expect("Serialize failed");
        assert!(json.contains("\"authorUsername\":\"AuthorUser\""));
        assert!(json.contains("\"authorDiscriminator\":\"1234\""));
    }

    #[test]
    fn test_create_note_request_deserialization() {
        let json = r#"{"content":"This is a new note"}"#;
        let request: CreateNoteRequest = serde_json::from_str(json).expect("Deserialize failed");
        assert_eq!(request.content, "This is a new note");
    }

    #[test]
    fn test_edit_note_request_deserialization() {
        let json = r#"{"content":"Updated note content"}"#;
        let request: EditNoteRequest = serde_json::from_str(json).expect("Deserialize failed");
        assert_eq!(request.content, "Updated note content");
    }

    #[test]
    fn test_notes_query_with_tribe() {
        let json = r#"{"tribe":"fire_tribe"}"#;
        let query: NotesQuery = serde_json::from_str(json).expect("Deserialize failed");
        assert_eq!(query.tribe, Some("fire_tribe".to_string()));
    }

    #[test]
    fn test_notes_query_without_tribe() {
        let json = r#"{}"#;
        let query: NotesQuery = serde_json::from_str(json).expect("Deserialize failed");
        assert!(query.tribe.is_none());
    }

    #[test]
    fn test_content_validation_empty() {
        let content = "   ";
        assert!(content.trim().is_empty());
    }

    #[test]
    fn test_content_validation_valid() {
        let content = "This is valid content";
        assert!(!content.trim().is_empty());
    }

    #[tokio::test]
    async fn test_note_crud_in_database() {
        let db = setup_db().await;
        let note_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        // Create note
        let result = sqlx::query(
            "INSERT INTO notes (id, target_user_id, author_id, tribe, content, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&note_id)
        .bind(1002_i64)
        .bind(1001_i64)
        .bind("test_tribe")
        .bind("Test note content")
        .bind(now)
        .bind(now)
        .execute(&db)
        .await;

        assert!(result.is_ok());

        // Read note
        let note: Option<Note> = sqlx::query_as("SELECT * FROM notes WHERE id = ?")
            .bind(&note_id)
            .fetch_optional(&db)
            .await
            .unwrap();

        assert!(note.is_some());
        let note = note.unwrap();
        assert_eq!(note.content, "Test note content");
        assert_eq!(note.author_id, 1001);
        assert_eq!(note.target_user_id, 1002);

        // Update note
        let updated_content = "Updated content";
        sqlx::query("UPDATE notes SET content = ?, updated_at = ? WHERE id = ?")
            .bind(updated_content)
            .bind(Utc::now())
            .bind(&note_id)
            .execute(&db)
            .await
            .unwrap();

        let updated_note: Note = sqlx::query_as("SELECT * FROM notes WHERE id = ?")
            .bind(&note_id)
            .fetch_one(&db)
            .await
            .unwrap();

        assert_eq!(updated_note.content, "Updated content");
    }

    #[tokio::test]
    async fn test_fetch_notes_with_author() {
        let db = setup_db().await;
        let note_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        // Create note
        sqlx::query(
            "INSERT INTO notes (id, target_user_id, author_id, tribe, content, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&note_id)
        .bind(1002_i64)
        .bind(1001_i64)
        .bind("test_tribe")
        .bind("Note with author join")
        .bind(now)
        .bind(now)
        .execute(&db)
        .await
        .unwrap();

        // Fetch with author
        let notes: Vec<NoteWithAuthor> = sqlx::query_as(
            r#"
            SELECT n.*, u.username as author_username, u.discriminator as author_discriminator
            FROM notes n
            JOIN users u ON n.author_id = u.id
            WHERE n.target_user_id = ? AND n.tribe = ?
            ORDER BY n.created_at DESC
            "#,
        )
        .bind(1002_i64)
        .bind("test_tribe")
        .fetch_all(&db)
        .await
        .unwrap();

        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].author_username, "AdminUser");
        assert_eq!(notes[0].content, "Note with author join");
    }

    // --- Handler tests for target-membership enforcement (tasks 2.3 / 2.4) ---

    async fn insert_user(
        pool: &crate::db::DbPool,
        id: i64,
        discord_id: &str,
        username: &str,
        is_admin: bool,
    ) {
        sqlx::query(
            "INSERT INTO users (id, discord_id, username, discriminator, is_admin) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(discord_id)
        .bind(username)
        .bind("0000")
        .bind(is_admin)
        .execute(pool)
        .await
        .unwrap();
    }

    async fn add_tribe(pool: &crate::db::DbPool, user_id: i64, tribe: &str, is_admin: bool) {
        sqlx::query("INSERT INTO user_tribes (user_id, tribe, is_admin) VALUES (?, ?, ?)")
            .bind(user_id)
            .bind(tribe)
            .bind(is_admin)
            .execute(pool)
            .await
            .unwrap();
    }

    /// Fresh DB with numeric Discord IDs: a global admin who manages
    /// `test_tribe`, an in-tribe target (1002), and an out-of-tribe target (1003).
    async fn setup_notes_db() -> crate::db::DbPool {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("pool");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");

        insert_user(&pool, 1001, "1001", "AdminUser", true).await;
        add_tribe(&pool, 1001, "test_tribe", true).await;

        insert_user(&pool, 1002, "1002", "InTribMember", false).await;
        add_tribe(&pool, 1002, "test_tribe", false).await;

        insert_user(&pool, 1003, "1003", "OutTribMember", false).await;
        add_tribe(&pool, 1003, "other_tribe", false).await;

        pool
    }

    async fn call_create_note(
        state: &AppState,
        author: i64,
        discord_id: &str,
        tribe: Option<&str>,
        content: &str,
    ) -> axum::response::Response {
        create_note(
            Path(discord_id.to_string()),
            Query(NotesQuery {
                tribe: tribe.map(|s| s.to_string()),
            }),
            State(state.clone()),
            AuthenticatedUser { user_id: author },
            Json(CreateNoteRequest {
                content: content.to_string(),
            }),
        )
        .await
        .into_response()
    }

    async fn call_get_notes(
        state: &AppState,
        author: i64,
        discord_id: &str,
        tribe: Option<&str>,
    ) -> axum::response::Response {
        get_notes(
            Path(discord_id.to_string()),
            Query(NotesQuery {
                tribe: tribe.map(|s| s.to_string()),
            }),
            State(state.clone()),
            AuthenticatedUser { user_id: author },
        )
        .await
        .into_response()
    }

    #[tokio::test]
    async fn note_create_out_of_tribe_forbidden_no_insert() {
        std::env::set_var("IDENTITY_HASH_PEPPER", "test-pepper");
        let db = setup_notes_db().await;
        let state = AppState::new(db.clone());

        let resp = call_create_note(&state, 1001, "1003", Some("test_tribe"), "sneaky").await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM notes")
            .fetch_one(&db)
            .await
            .unwrap();
        assert_eq!(
            count, 0,
            "no note may be inserted for an out-of-tribe target"
        );
    }

    #[tokio::test]
    async fn note_create_in_tribe_allowed() {
        std::env::set_var("IDENTITY_HASH_PEPPER", "test-pepper");
        let db = setup_notes_db().await;
        let state = AppState::new(db.clone());

        let resp = call_create_note(&state, 1001, "1002", Some("test_tribe"), "legit").await;
        assert_eq!(resp.status(), StatusCode::CREATED);

        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM notes")
            .fetch_one(&db)
            .await
            .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn note_read_out_of_tribe_forbidden() {
        std::env::set_var("IDENTITY_HASH_PEPPER", "test-pepper");
        let db = setup_notes_db().await;
        let state = AppState::new(db.clone());

        let resp = call_get_notes(&state, 1001, "1003", Some("test_tribe")).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn note_read_in_tribe_allowed() {
        std::env::set_var("IDENTITY_HASH_PEPPER", "test-pepper");
        let db = setup_notes_db().await;
        let state = AppState::new(db.clone());

        let seed = call_create_note(&state, 1001, "1002", Some("test_tribe"), "hello").await;
        assert_eq!(seed.status(), StatusCode::CREATED);

        let resp = call_get_notes(&state, 1001, "1002", Some("test_tribe")).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let notes: Vec<NoteWithAuthor> = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), 1 << 20)
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].tribe, "test_tribe");
    }

    #[tokio::test]
    async fn note_read_unknown_target_404_preserved() {
        std::env::set_var("IDENTITY_HASH_PEPPER", "test-pepper");
        let db = setup_notes_db().await;
        let state = AppState::new(db.clone());

        let resp = call_get_notes(&state, 1001, "9999", Some("test_tribe")).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn note_read_multi_tribe_exposes_only_selected() {
        std::env::set_var("IDENTITY_HASH_PEPPER", "test-pepper");
        let db = setup_notes_db().await;
        // Target 1004 belongs to both tribes.
        insert_user(&db, 1004, "1004", "MultiTrib", false).await;
        add_tribe(&db, 1004, "test_tribe", false).await;
        add_tribe(&db, 1004, "other_tribe", false).await;
        let state = AppState::new(db.clone());

        // Legitimate note in the selected tribe.
        let c = call_create_note(&state, 1001, "1004", Some("test_tribe"), "in test").await;
        assert_eq!(c.status(), StatusCode::CREATED);

        // A note in a different tribe for the same target (direct insert).
        sqlx::query(
            "INSERT INTO notes (id, target_user_id, author_id, tribe, content, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(1004_i64)
        .bind(1001_i64)
        .bind("other_tribe")
        .bind("in other")
        .bind(Utc::now())
        .bind(Utc::now())
        .execute(&db)
        .await
        .unwrap();

        let resp = call_get_notes(&state, 1001, "1004", Some("test_tribe")).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let notes: Vec<NoteWithAuthor> = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), 1 << 20)
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            notes.len(),
            1,
            "only the selected tribe's notes may be exposed"
        );
        assert_eq!(notes[0].tribe, "test_tribe");
    }

    #[tokio::test]
    async fn note_read_membership_loss_and_restoration() {
        std::env::set_var("IDENTITY_HASH_PEPPER", "test-pepper");
        let db = setup_notes_db().await;
        let state = AppState::new(db.clone());

        // Legitimate historical note.
        let c = call_create_note(&state, 1001, "1002", Some("test_tribe"), "historical").await;
        assert_eq!(c.status(), StatusCode::CREATED);

        // Target leaves the tribe: reads become 403 but the note remains stored.
        sqlx::query("DELETE FROM user_tribes WHERE user_id = ? AND tribe = ?")
            .bind(1002_i64)
            .bind("test_tribe")
            .execute(&db)
            .await
            .unwrap();
        let blocked = call_get_notes(&state, 1001, "1002", Some("test_tribe")).await;
        assert_eq!(blocked.status(), StatusCode::FORBIDDEN);
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM notes")
            .fetch_one(&db)
            .await
            .unwrap();
        assert_eq!(count, 1, "historical notes must remain stored");

        // Membership restored: reads succeed again.
        add_tribe(&db, 1002, "test_tribe", false).await;
        let restored = call_get_notes(&state, 1001, "1002", Some("test_tribe")).await;
        assert_eq!(restored.status(), StatusCode::OK);
    }
}
