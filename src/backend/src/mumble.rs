use crate::auth::{self, InternalSecret};
use crate::state::AppState;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use rand::{distr::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;

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
    auth::AuthenticatedUser { user_id, .. }: auth::AuthenticatedUser,
) -> impl IntoResponse {
    // user_id is already i64 from AuthenticatedUser extractor

    // 1. Check if user exists and is in the required tribe
    let user_valid = sqlx::query("SELECT 1 FROM user_tribes WHERE user_id = ? AND tribe = ?")
        .bind(user_id)
        .bind(&state.mumble_required_tribe)
        .fetch_optional(&state.db)
        .await;

    match user_valid {
        Ok(Some(_)) => {}
        Ok(None) => {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({"error": "User not in required tribe"})),
            )
                .into_response()
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Database error"})),
            )
                .into_response()
        }
    };

    // ... (previous code)

    // 2. Get username (based on rider name/wallet from tribe) - simplified: using database username for now or custom logic
    // Requirement: "create a username based upon their rider name (the wallet that is in the Fire tribe)"
    // We need to fetch the wallet/rider name associated with the tribe.

    // Update: If no wallet_id is found (manual assignment), fall back to the user's website username.

    let rider_name_query = sqlx::query(
        "SELECT ut.wallet_id, u.username
         FROM user_tribes ut
         JOIN users u ON ut.user_id = u.id
         WHERE ut.user_id = ? AND ut.tribe = ?",
    )
    .bind(user_id)
    .bind(&state.mumble_required_tribe)
    .fetch_one(&state.db)
    .await;

    let username = match rider_name_query {
        Ok(row) => {
            let wallet_id: Option<String> = row.get("wallet_id");
            let user_username: String = row.get("username");
            resolve_mumble_username(wallet_id, user_username)
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch rider name"})),
            )
                .into_response()
        }
    };

    // Sanitize username for Mumble (alphanumeric only ideally, but Murmur is flexible)
    // Replacing spaces with underscores
    let mumble_username = sanitize_username(&username);

    // 3. Generate Password
    let password: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();

    // 4. Hash Password
    let hashed = match hash(&password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to hash password"})),
            )
                .into_response()
        }
    };

    // 5. Store in DB (Upsert)
    let result = sqlx::query(
        "INSERT INTO mumble_accounts (user_id, username, password_hash, updated_at)
         VALUES (?, ?, ?, CURRENT_TIMESTAMP)
         ON CONFLICT(user_id) DO UPDATE SET
            username=excluded.username,
            password_hash=excluded.password_hash,
            updated_at=excluded.updated_at",
    )
    .bind(user_id)
    .bind(&mumble_username)
    .bind(&hashed)
    .execute(&state.db)
    .await;

    if let Err(e) = result {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response();
    }

    // 6. Audit Log
    use crate::audit::{log_audit, AuditAction};
    if let Err(e) = log_audit(
        &state.db,
        AuditAction::MumbleCreateAccount,
        user_id,
        Some(user_id),
        &format!("Created mumble account: {}", mumble_username),
    )
    .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response();
    }

    (
        StatusCode::OK,
        Json(CreateAccountResponse {
            username: mumble_username,
            password,
        }),
    )
        .into_response()
}

/// Resolves the username to use for Mumble.
/// Prioritizes the wallet_id (rider name/address) if present.
/// Falls back to the user's username if wallet_id is None.
fn resolve_mumble_username(wallet_id: Option<String>, username: String) -> String {
    match wallet_id {
        Some(w) if !w.trim().is_empty() => w,
        _ => username,
    }
}

/// Sanitizes the username for Mumble (e.g. replacing spaces).
fn sanitize_username(name: &str) -> String {
    name.replace(" ", "_")
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

    if let Ok(Some(record)) = row {
        let hash_str: String = record.get("password_hash");
        let user_id: i64 = record.get("user_id");

        if verify(&payload.password, &hash_str).unwrap_or(false) {
            // Log Audit
            use crate::audit::{log_audit, AuditAction};
            // We ignore audit errors here to not block login, but log them to stderr
            if let Err(e) = log_audit(
                &state.db,
                AuditAction::MumbleLogin,
                user_id,
                None,
                "Mumble authentication successful",
            )
            .await
            {
                eprintln!("Failed to log mumble login audit: {}", e);
            }

            return (
                StatusCode::OK,
                Json(VerifyLoginResponse {
                    user_id,
                    username: payload.username,
                }),
            )
                .into_response();
        }
    }

    (
        StatusCode::UNAUTHORIZED,
        Json(json!({"error": "Invalid credentials"})),
    )
        .into_response()
}

// Reset password technically re-uses create_account logic but might merit a separate endpoint if logic diverges.
// For now, allow create_account to handle "reset" via upsert as per requirement 4 "unlimited number of times".

#[derive(Debug, Serialize)]
pub struct MumbleStatusResponse {
    pub username: Option<String>,
    pub required_tribe: String,
}

pub async fn get_status(
    State(state): State<AppState>,
    auth::AuthenticatedUser { user_id, .. }: auth::AuthenticatedUser,
) -> impl IntoResponse {
    // user_id is already i64 from AuthenticatedUser extractor

    let row = sqlx::query("SELECT username FROM mumble_accounts WHERE user_id = ?")
        .bind(user_id)
        .fetch_optional(&state.db)
        .await;

    match row {
        Ok(Some(record)) => {
            let username: String = record.get("username");
            (
                StatusCode::OK,
                Json(MumbleStatusResponse {
                    username: Some(username),
                    required_tribe: state.mumble_required_tribe.clone(),
                }),
            )
                .into_response()
        }
        Ok(None) => (
            StatusCode::OK,
            Json(MumbleStatusResponse {
                username: None,
                required_tribe: state.mumble_required_tribe.clone(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_mumble_username_with_wallet() {
        let wallet = Some("Friendly Rider".to_string());
        let username = "VoidUser".to_string();
        let result = resolve_mumble_username(wallet, username);
        assert_eq!(result, "Friendly Rider");
    }

    #[test]
    fn test_resolve_mumble_username_fallback() {
        let wallet = None;
        let username = "VoidUser".to_string();
        let result = resolve_mumble_username(wallet, username);
        assert_eq!(result, "VoidUser");
    }

    #[test]
    fn test_resolve_mumble_username_empty_wallet() {
        let wallet = Some("   ".to_string());
        let username = "VoidUser".to_string();
        let result = resolve_mumble_username(wallet, username);
        assert_eq!(result, "VoidUser");
    }

    #[test]
    fn test_sanitize_username() {
        let raw = "My Cool Name";
        let sanitized = sanitize_username(raw);
        assert_eq!(sanitized, "My_Cool_Name");
    }
}
