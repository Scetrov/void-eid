use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use rand::{distr::Alphanumeric, Rng};
use bcrypt::{hash, verify, DEFAULT_COST};
use crate::auth::{self, InternalSecret};
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAccountRequest {
    pub user_id: i64,
}

#[derive(Debug, Serialize)]
pub struct CreateAccountResponse {
    pub username: String,
    pub password: String, // Only shown once
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyLoginRequest {
    pub username: String,
    pub password: String,
    pub extra: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct VerifyLoginResponse {
    pub user_id: i64,
    pub username: String,
}

pub async fn create_account(
    State(state): State<AppState>,
    auth::AuthenticatedUser { user_id }: auth::AuthenticatedUser,
) -> impl IntoResponse {
    // user_id is already i64 from AuthenticatedUser extractor

    // 1. Check if user exists and is in the required tribe
    let user_valid = sqlx::query(
        "SELECT 1 FROM user_tribes WHERE user_id = ? AND tribe = ?"
    )
    .bind(user_id)
    .bind(&state.mumble_required_tribe)
    .fetch_optional(&state.db)
    .await;

    match user_valid {
        Ok(Some(_)) => {},
        Ok(None) => return (StatusCode::FORBIDDEN, Json(json!({"error": "User not in required tribe"}))).into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Database error"}))).into_response(),
    };

    // 2. Get username (based on rider name/wallet from tribe) - simplified: using database username for now or custom logic
    // Requirement: "create a username based upon their rider name (the wallet that is in the Fire tribe)"
    // We need to fetch the wallet/rider name associated with the tribe.
    
    // Assuming `user_tribes` has `wallet_id` which might be the rider name or linked to wallets table.
    // Let's check `user_tribes` table: `wallet_id` TEXT.
    // If wallet_id IS the rider name (which seems implied by "rider name (the wallet...)"), we use that.
    
    let rider_name_query = sqlx::query(
        "SELECT wallet_id FROM user_tribes WHERE user_id = ? AND tribe = ?"
    )
    .bind(user_id)
    .bind(&state.mumble_required_tribe)
    .fetch_one(&state.db)
    .await;

    let username = match rider_name_query {
        Ok(row) => {
             let w: Option<String> = row.get("wallet_id");
             match w {
                 Some(name) => name,
                 None => return (StatusCode::BAD_REQUEST, Json(json!({"error": "No rider name/wallet found for tribe membership"}))).into_response(),
             }
        },
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to fetch rider name"}))).into_response(),
    };

    // Sanitize username for Mumble (alphanumeric only ideally, but Murmur is flexible)
    // Replacing spaces with underscores
    let mumble_username = username.replace(" ", "_");
    
    // 3. Generate Password
    let password: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();

    // 4. Hash Password
    let hashed = match hash(&password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to hash password"}))).into_response(),
    };

    // 5. Store in DB (Upsert)
    let result = sqlx::query(
        "INSERT INTO mumble_accounts (user_id, username, password_hash, updated_at) 
         VALUES (?, ?, ?, CURRENT_TIMESTAMP)
         ON CONFLICT(user_id) DO UPDATE SET 
            username=excluded.username, 
            password_hash=excluded.password_hash,
            updated_at=excluded.updated_at"
    )
    .bind(user_id)
    .bind(&mumble_username)
    .bind(&hashed)
    .execute(&state.db)
    .await;

    if let Err(e) = result {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response();
    }

    // 6. Audit Log
    use crate::audit::{log_audit, AuditAction};
    if let Err(e) = log_audit(
        &state.db,
        AuditAction::MumbleCreateAccount,
        user_id,
        Some(user_id),
        &format!("Created mumble account: {}", mumble_username),
    ).await {
         return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response();
    }

    (StatusCode::OK, Json(CreateAccountResponse {
        username: mumble_username,
        password,
    })).into_response()
}

pub async fn verify_login(
    State(state): State<AppState>,
    InternalSecret(_secret): InternalSecret, // Ensures this is only called by trusted Authenticator
    Json(payload): Json<VerifyLoginRequest>,
) -> impl IntoResponse {
    let row = sqlx::query("SELECT user_id, password_hash FROM mumble_accounts WHERE username = ?")
        .bind(&payload.username)
        .fetch_optional(&state.db)
        .await;

    match row {
        Ok(Some(record)) => {
            let hash_str: String = record.get("password_hash");
            let user_id: i64 = record.get("user_id");

            if verify(&payload.password, &hash_str).unwrap_or(false) {
                return (StatusCode::OK, Json(VerifyLoginResponse {
                    user_id,
                    username: payload.username,
                })).into_response();
            }
        },
        _ => {}
    }

    (StatusCode::UNAUTHORIZED, Json(json!({"error": "Invalid credentials"}))).into_response()
}

// Reset password technically re-uses create_account logic but might merit a separate endpoint if logic diverges.
// For now, allow create_account to handle "reset" via upsert as per requirement 4 "unlimited number of times".

#[derive(Debug, Serialize)]
pub struct MumbleStatusResponse {
    pub username: Option<String>,
}

pub async fn get_status(
    State(state): State<AppState>,
    auth::AuthenticatedUser { user_id }: auth::AuthenticatedUser,
) -> impl IntoResponse {
    // user_id is already i64 from AuthenticatedUser extractor

    let row = sqlx::query("SELECT username FROM mumble_accounts WHERE user_id = ?")
        .bind(user_id)
        .fetch_optional(&state.db)
        .await;

    match row {
        Ok(Some(record)) => {
            let username: String = record.get("username");
            (StatusCode::OK, Json(MumbleStatusResponse { username: Some(username) })).into_response()
        },
        Ok(None) => (StatusCode::OK, Json(MumbleStatusResponse { username: None })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}
