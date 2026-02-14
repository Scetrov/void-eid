use crate::{
    audit::{log_audit, AuditAction},
    models::{LinkedWallet, User},
    state::AppState,
};
use axum::{
    extract::{FromRequestParts, Query, State},
    http::{header, request::Parts, StatusCode},
    response::{IntoResponse, Redirect, Response},
    Json,
};
use chrono::{Duration, Utc};
use hex;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::env;
use uuid::Uuid;

use utoipa::{IntoParams, ToSchema};

pub fn hash_identity(input: &str, pepper: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hasher.update(pepper.as_bytes());
    hex::encode(hasher.finalize())
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct CallbackParams {
    code: String,
    state: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Claims {
    pub id: String,
    #[serde(rename = "discordId")]
    pub discord_id: String,
    pub username: String,
    #[serde(default)]
    pub is_super_admin: bool,
    pub exp: usize,
}

#[utoipa::path(
    get,
    path = "/api/auth/discord/login",
    responses(
        (status = 302, description = "Redirect to Discord OAuth")
    )
)]
pub async fn discord_login(State(state): State<AppState>) -> impl IntoResponse {
    let client_id = env::var("DISCORD_CLIENT_ID").expect("CID not set");
    let redirect_uri = env::var("DISCORD_REDIRECT_URI").expect("URI not set");
    let scope = "identify";

    // Generate CSRF token
    let state_token = Uuid::new_v4().to_string();
    state
        .oauth_states
        .lock()
        .unwrap()
        .insert(state_token.clone(), Utc::now());

    let url = format!(
        "https://discord.com/api/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
        client_id,
        urlencoding::encode(&redirect_uri),
        scope,
        state_token
    );

    Redirect::to(&url)
}

#[utoipa::path(
    get,
    path = "/api/auth/discord/callback",
    params(
        CallbackParams
    ),
    responses(
        (status = 302, description = "Redirect to frontend with token")
    )
)]
pub async fn discord_callback(
    Query(params): Query<CallbackParams>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, Response> {
    // Validate state token (CSRF protection)
    let state_created_at = {
        let mut states = state.oauth_states.lock().unwrap();
        states.remove(&params.state)
    };

    let state_created_at = state_created_at.ok_or_else(|| {
        (StatusCode::BAD_REQUEST, "Invalid or expired state token").into_response()
    })?;

    // Check if state token is too old (5 minute TTL)
    if Utc::now() - state_created_at > Duration::minutes(5) {
        return Err((StatusCode::BAD_REQUEST, "State token expired").into_response());
    }

    let client_id = env::var("DISCORD_CLIENT_ID").expect("CID missing");
    let client_secret = env::var("DISCORD_CLIENT_SECRET").expect("Secret missing");
    let redirect_uri = env::var("DISCORD_REDIRECT_URI").expect("URI missing");

    let client = Client::new();

    let params = [
        ("client_id", client_id.as_str()),
        ("client_secret", client_secret.as_str()),
        ("grant_type", "authorization_code"),
        ("code", params.code.as_str()),
        ("redirect_uri", redirect_uri.as_str()),
    ];

    let token_res = client
        .post("https://discord.com/api/oauth2/token")
        .form(&params)
        .send()
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to connect to Discord",
            )
                .into_response()
        })?;

    if !token_res.status().is_success() {
        let status = token_res.status();
        let body = token_res
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error body".to_string());
        eprintln!(
            "Discord token exchange failed: Status: {}, Body: {}",
            status, body
        );
        return Err((StatusCode::BAD_REQUEST, "Discord OAuth failed").into_response());
    }

    let token_data: Value = token_res.json().await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to parse token response",
        )
            .into_response()
    })?;

    let access_token = match token_data["access_token"].as_str() {
        Some(t) => t,
        None => return Err((StatusCode::BAD_REQUEST, "No access token").into_response()),
    };

    let user_res = client
        .get("https://discord.com/api/users/@me")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch user").into_response())?;

    let discord_user: Value = user_res
        .json()
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to parse user").into_response())?;

    let discord_id = discord_user["id"].as_str().unwrap_or_default().to_string();
    let username = discord_user["username"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let discriminator = discord_user["discriminator"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let avatar = discord_user["avatar"].as_str().map(|s| s.to_string());

    // Check for Initial Admin
    let initial_admin_id = env::var("INITIAL_ADMIN_ID").ok();
    let is_initial_admin = initial_admin_id.as_deref() == Some(&discord_id);

    // Check for Super Admin
    let super_admin_ids_str = env::var("SUPER_ADMIN_DISCORD_IDS").unwrap_or_default();
    let super_admin_ids: Vec<&str> = super_admin_ids_str.split(',').map(|s| s.trim()).collect();
    let is_super_admin = super_admin_ids.contains(&discord_id.as_str());

    // Check if denylisted
    let discord_hash = hash_identity(&discord_id, &state.identity_hash_pepper);
    let denylisted: Option<(String,)> =
        sqlx::query_as("SELECT hash FROM identity_hashes WHERE hash = ?")
            .bind(&discord_hash)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| {
                eprintln!("Database error checking denylist: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
            })?;

    if denylisted.is_some() {
        let frontend_url =
            env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());
        return Ok(Redirect::to(&format!("{}/deleted", frontend_url)).into_response());
    }

    // Find or Create User
    let (user, admin_granted) = match sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE discord_id = ?",
    )
    .bind(&discord_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(mut u)) => {
            // If they are strictly the initial admin, force upgrade appropriately?
            // Or just if they match, ensure they are admin.
            // Let's only upgrade to admin if they are the initial admin.
            // We probably shouldn't downgrade them automatically if env var changes,
            // but for "initial" setup, ensuring they get admin is key.

            let should_be_admin = u.is_admin || is_initial_admin;
            let was_promoted = is_initial_admin && !u.is_admin;

            if is_initial_admin && !u.is_admin {
                u.is_admin = true;
            }

            let now = Utc::now();
            let _ = sqlx::query("UPDATE users SET username = ?, discriminator = ?, avatar = ?, is_admin = ?, last_login_at = ? WHERE id = ?")
                .bind(&username)
                .bind(&discriminator)
                .bind(&avatar)
                .bind(should_be_admin)
                .bind(now)
                .bind(u.id)
                .execute(&state.db)
                .await;
            u.last_login_at = Some(now);
            (u, was_promoted)
        }
        Ok(None) => {
            let now = Utc::now();
            // Generate random 64-bit integer (using i64 positive range for SQLite compatibility)
            // We loop to ensure uniqueness, though collision is extremely unlikely.
            let mut new_id = rand::random::<i64>().abs();
            // Ensure strictly positive and not 0 (though 0 is valid int, usually implementation detail)
            if new_id == 0 {
                new_id = 1;
            }

            // Simple collision check (optional but good practice)
            // In a real high-concurrency scenario, DB constraint unique error would handle this,
            // but here we can just retry if strict. For now, trust entropy.

            let new_user = User {
                id: new_id,
                discord_id: discord_id.clone(),
                username: username.clone(),
                discriminator: discriminator.clone(),
                avatar: avatar.clone(),
                is_admin: is_initial_admin,
                last_login_at: Some(now),
            };

            let _ = sqlx::query("INSERT INTO users (id, discord_id, username, discriminator, avatar, is_admin, last_login_at) VALUES (?, ?, ?, ?, ?, ?, ?)")
                .bind(new_user.id)
                .bind(&new_user.discord_id)
                .bind(&new_user.username)
                .bind(&new_user.discriminator)
                .bind(&new_user.avatar)
                .bind(new_user.is_admin)
                .bind(now)
                .execute(&state.db)
                .await;

            // New user created as admin
            (new_user, is_initial_admin)
        }
        Err(e) => {
            eprintln!("Database error in discord_callback: {}", e);
            return Err(
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
            );
        }
    };

    // Audit log for login
    let _ = log_audit(
        &state.db,
        AuditAction::Login,
        user.id,
        None,
        &format!("User {} logged in via Discord", user.username),
    )
    .await;

    // Audit log for admin grant (if applicable)
    if admin_granted {
        let _ = log_audit(
            &state.db,
            AuditAction::AdminGrant,
            user.id, // Actor is the system (represented by the user themselves for initial admin)
            Some(user.id),
            &format!("User {} granted admin via INITIAL_ADMIN_ID", user.username),
        )
        .await;
    }

    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET missing");
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        id: user.id.to_string(), // JWT ID as string
        discord_id: user.discord_id,
        username: user.username,
        is_super_admin,
        exp: expiration,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Token generation failed").into_response())?;

    // Generate auth code and store JWT temporarily (30s TTL)
    let auth_code = Uuid::new_v4().to_string();
    state
        .auth_codes
        .lock()
        .unwrap()
        .insert(auth_code.clone(), (token, Utc::now()));

    let frontend_url =
        env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());
    Ok(Redirect::to(&format!(
        "{}/auth/callback?code={}",
        frontend_url, auth_code
    ))
    .into_response())
}

#[derive(Deserialize, ToSchema)]
pub struct ExchangeRequest {
    pub code: String,
}

#[derive(Serialize, ToSchema)]
pub struct ExchangeResponse {
    pub token: String,
}

#[utoipa::path(
    post,
    path = "/api/auth/exchange",
    request_body = ExchangeRequest,
    responses(
        (status = 200, description = "JWT token", body = ExchangeResponse),
        (status = 400, description = "Invalid or expired code")
    )
)]
pub async fn exchange_code(
    State(state): State<AppState>,
    Json(payload): Json<ExchangeRequest>,
) -> Result<Json<ExchangeResponse>, (StatusCode, &'static str)> {
    // Retrieve and remove auth code (one-time use)
    let (token, created_at) = {
        let mut codes = state.auth_codes.lock().unwrap();
        codes.remove(&payload.code)
    }
    .ok_or((StatusCode::BAD_REQUEST, "Invalid or expired code"))?;

    // Validate code is not too old (30 second TTL)
    if Utc::now() - created_at > Duration::seconds(30) {
        return Err((StatusCode::BAD_REQUEST, "Code expired"));
    }

    Ok(Json(ExchangeResponse { token }))
}

#[derive(Clone)]
pub struct AuthenticatedUser {
    pub user_id: i64,
    pub is_super_admin: bool,
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "Missing Auth Header"))?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or((StatusCode::UNAUTHORIZED, "Invalid Auth Header"))?;

        let secret = env::var("JWT_SECRET")
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "JWT Config Error"))?;

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid Token"))?;

        let user_id = token_data
            .claims
            .id
            .parse::<i64>()
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid User ID in Token"))?;

        Ok(AuthenticatedUser {
            user_id,
            is_super_admin: token_data.claims.is_super_admin,
        })
    }
}

#[utoipa::path(
    get,
    path = "/api/me",
    responses(
        (status = 200, description = "Get current user info", body = User)
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn get_me(
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let user = match sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(auth_user.user_id)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(u)) => u,
        _ => return (StatusCode::UNAUTHORIZED, "User not found").into_response(),
    };

    let flat_wallets = sqlx::query_as::<_, crate::models::FlatLinkedWallet>(
        "SELECT w.*, ut.tribe FROM wallets w LEFT JOIN user_tribes ut ON w.id = ut.wallet_id WHERE w.user_id = ? AND w.deleted_at IS NULL"
    )
        .bind(auth_user.user_id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

    // Group flat results into LinkedWallet with tribes Vec
    let mut wallet_map: std::collections::BTreeMap<String, LinkedWallet> =
        std::collections::BTreeMap::new();
    for flat in flat_wallets {
        let entry = wallet_map
            .entry(flat.id.clone())
            .or_insert_with(|| LinkedWallet {
                id: flat.id,
                user_id: flat.user_id,
                address: flat.address,
                verified_at: flat.verified_at,
                deleted_at: flat.deleted_at,
                network: flat.network,
                tribes: Vec::new(),
            });
        if let Some(t) = flat.tribe {
            if !entry.tribes.contains(&t) {
                entry.tribes.push(t);
            }
        }
    }
    let wallets: Vec<LinkedWallet> = wallet_map.into_values().collect();

    // Fetch all tribes for the user, distinguishing admin ones
    let user_tribes = sqlx::query_as::<_, crate::models::UserTribe>(
        "SELECT * FROM user_tribes WHERE user_id = ?",
    )
    .bind(auth_user.user_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let tribes: Vec<String> = user_tribes.iter().map(|ut| ut.tribe.clone()).collect();
    let admin_tribes: Vec<String> = user_tribes
        .iter()
        .filter(|ut| ut.is_admin || user.is_admin) // Include global admin for now or deprecated? Let's obey strict logic: only if is_admin on tribe. But wait, we might have global override.
        // The user said "associated with a wallet", so let's stick to the tribe record.
        // However, if we migrated global admins to have is_admin=true on all tribes, this simplistic check is fine.
        // Let's assume global admin implies admin everywhere for backward compat/initial admin?
        // User said "admin in more than two tribes... associated with a wallet".
        // Let's assume user.is_admin is STRICTLY global super-admin (or deprecated).
        // For safety, let's include user.is_admin as a valid check for "is admin of this tribe" OR check the tribe record.
        .filter(|ut| ut.is_admin || user.is_admin)
        .map(|ut| ut.tribe.clone())
        .collect();

    Json(serde_json::json!({
        "id": user.id.to_string(),
        "discordId": user.discord_id,
        "username": user.username,
        "discriminator": user.discriminator,
        "avatar": user.avatar,
        "tribes": tribes,
        "adminTribes": admin_tribes,
        "isAdmin": user.is_admin, // Keep for legacy/global support if valid
        "isSuperAdmin": auth_user.is_super_admin,
        "lastLoginAt": user.last_login_at,
        "wallets": wallets
    }))
    .into_response()
}

#[utoipa::path(
    delete,
    path = "/api/me",
    responses(
        (status = 200, description = "Account deleted and anonymized"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn delete_me(
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 1. Fetch user and wallets
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(auth_user.user_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;
    let wallets = sqlx::query_as::<_, crate::models::FlatLinkedWallet>(
        "SELECT w.*, NULL as tribe FROM wallets w WHERE w.user_id = ?",
    )
    .bind(auth_user.user_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 2. Hash and Denylist Discord ID
    let discord_hash = hash_identity(&user.discord_id, &state.identity_hash_pepper);
    sqlx::query("INSERT OR IGNORE INTO identity_hashes (hash, type) VALUES (?, 'DISCORD')")
        .bind(discord_hash)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 3. Hash and Denylist Wallets
    for wallet in wallets {
        let normalized_address = wallet.address.to_lowercase();
        let wallet_hash = hash_identity(&normalized_address, &state.identity_hash_pepper);
        sqlx::query("INSERT OR IGNORE INTO identity_hashes (hash, type) VALUES (?, 'WALLET')")
            .bind(wallet_hash)
            .execute(&mut *tx)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    // 4. Hard delete wallets (data scrubbing)
    sqlx::query("DELETE FROM wallets WHERE user_id = ?")
        .bind(auth_user.user_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 5. Anonymize user row
    let random_id = format!("deleted_{}", rand::random::<u64>());
    sqlx::query("UPDATE users SET discord_id = ?, username = 'Deleted User', avatar = NULL, is_admin = 0 WHERE id = ?")
        .bind(random_id)
        .bind(auth_user.user_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 6. Scrub associated data
    // Delete tribe associations
    sqlx::query("DELETE FROM user_tribes WHERE user_id = ?")
        .bind(auth_user.user_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Delete mumble account
    sqlx::query("DELETE FROM mumble_accounts WHERE user_id = ?")
        .bind(auth_user.user_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Delete notes (where user is author or target)
    sqlx::query("DELETE FROM notes WHERE target_user_id = ? OR author_id = ?")
        .bind(auth_user.user_id)
        .bind(auth_user.user_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 7. Audit Log (After commit to avoid SQLite deadlock)
    let _ = log_audit(
        &state.db,
        AuditAction::DeleteUser,
        auth_user.user_id,
        None,
        "User deleted their own account (GDPR)",
    )
    .await;

    Ok(Json(
        serde_json::json!({ "message": "Account deleted successfully" }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_claims_serialization() {
        let claims = Claims {
            id: "12345".to_string(),
            discord_id: "discord123".to_string(),
            username: "TestUser".to_string(),
            is_super_admin: false,
            exp: 1234567890,
        };

        let json = serde_json::to_string(&claims).expect("Failed to serialize");
        assert!(json.contains("\"id\":\"12345\""));
        assert!(json.contains("\"discordId\":\"discord123\"")); // camelCase from serde rename
        assert!(json.contains("\"username\":\"TestUser\""));
        assert!(json.contains("\"exp\":1234567890"));
    }

    #[test]
    fn test_claims_deserialization() {
        let json =
            r#"{"id":"12345","discordId":"discord123","username":"TestUser","exp":1234567890}"#;
        let claims: Claims = serde_json::from_str(json).expect("Failed to deserialize");

        assert_eq!(claims.id, "12345");
        assert_eq!(claims.discord_id, "discord123");
        assert_eq!(claims.username, "TestUser");
        assert_eq!(claims.exp, 1234567890);
    }

    #[test]
    fn test_claims_roundtrip() {
        let original = Claims {
            id: "98765".to_string(),
            discord_id: "987654321".to_string(),
            username: "RoundtripUser".to_string(),
            is_super_admin: true,
            exp: 9999999999,
        };

        let json = serde_json::to_string(&original).expect("Serialize failed");
        let restored: Claims = serde_json::from_str(&json).expect("Deserialize failed");

        assert_eq!(original.id, restored.id);
        assert_eq!(original.discord_id, restored.discord_id);
        assert_eq!(original.username, restored.username);
        assert_eq!(original.exp, restored.exp);
    }

    #[test]
    fn test_jwt_encode_decode() {
        let secret = "test-secret-key";
        let expiration = (Utc::now().timestamp() + 3600) as usize; // 1 hour from now

        let claims = Claims {
            id: "1001".to_string(),
            discord_id: "discord_id_123".to_string(),
            username: "JwtTestUser".to_string(),
            is_super_admin: false,
            exp: expiration,
        };

        // Encode
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .expect("Encoding failed");

        assert!(!token.is_empty());
        assert!(token.contains('.')); // JWT has 3 parts separated by dots

        // Decode
        let decoded = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .expect("Decoding failed");

        assert_eq!(decoded.claims.id, "1001");
        assert_eq!(decoded.claims.discord_id, "discord_id_123");
        assert_eq!(decoded.claims.username, "JwtTestUser");
    }

    #[test]
    fn test_jwt_invalid_secret_fails() {
        let secret = "correct-secret";
        let wrong_secret = "wrong-secret";
        let expiration = (Utc::now().timestamp() + 3600) as usize;

        let claims = Claims {
            id: "1001".to_string(),
            discord_id: "discord123".to_string(),
            username: "TestUser".to_string(),
            is_super_admin: false,
            exp: expiration,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .expect("Encoding failed");

        // Decoding with wrong secret should fail
        let result = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(wrong_secret.as_bytes()),
            &Validation::default(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_jwt_expired_token_fails() {
        let secret = "test-secret";
        let expired = (Utc::now().timestamp() - 3600) as usize; // 1 hour ago

        let claims = Claims {
            id: "1001".to_string(),
            discord_id: "discord123".to_string(),
            username: "ExpiredUser".to_string(),
            is_super_admin: false,
            exp: expired,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .expect("Encoding failed");

        // Decoding expired token should fail
        let result = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_callback_params_deserialization() {
        // Simulate query string parsing
        let json = r#"{"code":"test_auth_code_12345","state":"test-state-token"}"#;
        let params: CallbackParams = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(params.code, "test_auth_code_12345");
        assert_eq!(params.state, "test-state-token");
    }

    #[test]
    fn test_user_id_parsing_from_claims() {
        // Test that we can parse user_id from Claims.id string
        let claims = Claims {
            id: "123456789".to_string(),
            discord_id: "discord123".to_string(),
            username: "TestUser".to_string(),
            is_super_admin: false,
            exp: 9999999999,
        };

        let user_id: i64 = claims.id.parse().expect("Failed to parse user_id");
        assert_eq!(user_id, 123456789);
    }

    #[test]
    fn test_user_id_parsing_invalid() {
        let claims = Claims {
            id: "not-a-number".to_string(),
            discord_id: "discord123".to_string(),
            username: "TestUser".to_string(),
            is_super_admin: false,
            exp: 9999999999,
        };

        let result: Result<i64, _> = claims.id.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_hash_identity_deterministic() {
        let pepper1 = "test-pepper";
        let hash1 = hash_identity("test-input", pepper1);
        let hash2 = hash_identity("test-input", pepper1);
        assert_eq!(hash1, hash2);

        let hash3 = hash_identity("other-input", pepper1);
        assert_ne!(hash1, hash3);

        let pepper2 = "other-pepper";
        let hash4 = hash_identity("test-input", pepper2);
        assert_ne!(hash1, hash4);
    }
}

#[tokio::test]
async fn test_delete_account_full_flow() {
    use crate::audit::AuditAction;
    use crate::db::init_db;
    use crate::models::User;
    use uuid::Uuid;

    let pepper = "test-pepper";
    std::env::set_var("IDENTITY_HASH_PEPPER", pepper);
    std::env::set_var("DATABASE_URL", "sqlite::memory:");
    let db = init_db().await.expect("Failed to init DB");
    let state = AppState::new(db.clone());

    // 1. Create a user with wallets and associations
    let user_id = rand::random::<i64>().abs();
    let discord_id = format!("discord_{}", user_id);
    let wallet_address = "0xTestWalletAddress".to_lowercase();
    let wallet_id = Uuid::new_v4().to_string();

    sqlx::query("INSERT INTO users (id, discord_id, username, discriminator) VALUES (?, ?, 'TestUser', '0000')")
            .bind(user_id).bind(&discord_id).execute(&db).await.unwrap();

    sqlx::query("INSERT INTO wallets (id, user_id, address, verified_at) VALUES (?, ?, ?, CURRENT_TIMESTAMP)")
            .bind(&wallet_id).bind(user_id).bind(&wallet_address).execute(&db).await.unwrap();

    sqlx::query("INSERT INTO user_tribes (user_id, tribe, wallet_id, is_admin) VALUES (?, 'TestTribe', ?, 1)")
            .bind(user_id).bind(&wallet_id).execute(&db).await.unwrap();

    sqlx::query("INSERT INTO mumble_accounts (user_id, username, password_hash) VALUES (?, 'testmumble', 'hash')")
            .bind(user_id).execute(&db).await.unwrap();

    sqlx::query("INSERT INTO notes (id, tribe, author_id, target_user_id, content) VALUES (?, 'TestTribe', ?, ?, 'Test content')")
            .bind(Uuid::new_v4().to_string()).bind(user_id).bind(user_id).execute(&db).await.unwrap();

    // 2. Run delete_me
    let auth_user = AuthenticatedUser {
        user_id,
        is_super_admin: false,
    };
    delete_me(auth_user, State(state))
        .await
        .expect("delete_me failed");

    // 3. Verify Verifications
    // User anonymized
    let user: User = sqlx::query_as("SELECT * FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert!(user.discord_id.starts_with("deleted_"));
    assert_eq!(user.username, "Deleted User");
    assert!(!user.is_admin);

    // Denylist populated
    let discord_hash = hash_identity(&discord_id, pepper);
    let wallet_hash = hash_identity(&wallet_address, pepper);
    let _dh: (String,) =
        sqlx::query_as("SELECT hash FROM identity_hashes WHERE hash = ? AND type = 'DISCORD'")
            .bind(discord_hash)
            .fetch_one(&db)
            .await
            .unwrap();
    let _wh: (String,) =
        sqlx::query_as("SELECT hash FROM identity_hashes WHERE hash = ? AND type = 'WALLET'")
            .bind(wallet_hash)
            .fetch_one(&db)
            .await
            .unwrap();

    // Data scrubbed
    let wallet_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM wallets WHERE user_id = ?")
        .bind(user_id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(wallet_count.0, 0);

    let tribe_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM user_tribes WHERE user_id = ?")
        .bind(user_id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(tribe_count.0, 0);

    let mumble_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM mumble_accounts WHERE user_id = ?")
            .bind(user_id)
            .fetch_one(&db)
            .await
            .unwrap();
    assert_eq!(mumble_count.0, 0);

    let note_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM notes WHERE target_user_id = ? OR author_id = ?")
            .bind(user_id)
            .bind(user_id)
            .fetch_one(&db)
            .await
            .unwrap();
    assert_eq!(note_count.0, 0);

    // Audit log exists
    let audit: (String,) = sqlx::query_as(
        "SELECT action FROM audit_logs WHERE actor_id = ? ORDER BY created_at DESC LIMIT 1",
    )
    .bind(user_id)
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(audit.0, AuditAction::DeleteUser.as_str());
}

#[derive(Debug)]
pub struct InternalSecret(pub String);

impl<S> FromRequestParts<S> for InternalSecret
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let secret_header = parts
            .headers
            .get("X-Internal-Secret")
            .and_then(|h| h.to_str().ok());

        let configured_secret = env::var("INTERNAL_SECRET").expect("INTERNAL_SECRET must be set");

        match secret_header {
            Some(s) if s == configured_secret => Ok(InternalSecret(s.to_string())),
            _ => Err((StatusCode::UNAUTHORIZED, "Invalid Internal Secret")),
        }
    }
}
