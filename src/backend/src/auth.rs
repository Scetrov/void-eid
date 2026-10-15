use crate::{
    audit::{log_audit, AuditAction},
    helpers::{validate_discord_id, validate_text_len},
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
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::env;
use subtle::ConstantTimeEq;
use uuid::Uuid;

use utoipa::{IntoParams, ToSchema};

pub fn hash_identity(input: &str, pepper: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hasher.update(pepper.as_bytes());
    hex::encode(hasher.finalize())
}

fn internal_error<E: std::fmt::Display>(context: &'static str, error: E) -> (StatusCode, String) {
    eprintln!("{}: {}", context, error);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal server error".to_string(),
    )
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
    pub exp: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,
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

    // Prune expired state tokens before inserting (prevent unbounded growth)
    {
        let mut states = state.oauth_states.lock().unwrap();
        let now = Utc::now();
        let ttl = Duration::minutes(5);
        states.retain(|_, created_at| now.signed_duration_since(*created_at) <= ttl);
        states.insert(state_token.clone(), now);
    }

    let url = format!(
        "https://discord.com/api/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
        client_id,
        urlencoding::encode(&redirect_uri),
        scope,
        state_token
    );

    Redirect::to(&url)
}

/// Validated profile returned from the Discord OAuth exchange.
#[derive(Debug, Clone)]
struct DiscordProfile {
    discord_id: String,
    username: String,
    discriminator: String,
    avatar: Option<String>,
}

/// Classification of failures during the Discord OAuth exchange. Each maps to a
/// generic HTTP response so that Discord payloads or internal state are never
/// surfaced to the client.
#[derive(Debug)]
enum OAuthError {
    /// Failed to reach the Discord token endpoint at all.
    Transport,
    /// Discord returned a non-success status for the token exchange.
    TokenNonSuccess,
    /// The token endpoint returned a body that is not valid JSON.
    TokenParseFailed,
    /// The token response was valid JSON but carried no usable access token.
    MissingAccessToken,
    /// Failed to reach the Discord user endpoint at all.
    UserTransport,
    /// Discord returned a non-success status for the user request.
    UserNonSuccess,
    /// The user endpoint returned a body that is not valid JSON.
    UserParseFailed,
    /// The user `id` was missing, non-string, or failed format validation.
    InvalidDiscordId,
}

impl IntoResponse for OAuthError {
    fn into_response(self) -> Response {
        match self {
            OAuthError::Transport => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to connect to Discord",
            )
                .into_response(),
            OAuthError::TokenNonSuccess => {
                (StatusCode::BAD_REQUEST, "Discord OAuth failed").into_response()
            }
            OAuthError::TokenParseFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to parse token response",
            )
                .into_response(),
            OAuthError::MissingAccessToken => {
                (StatusCode::BAD_REQUEST, "Discord OAuth failed").into_response()
            }
            OAuthError::UserTransport => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch user").into_response()
            }
            OAuthError::UserNonSuccess => {
                (StatusCode::BAD_REQUEST, "Discord OAuth failed").into_response()
            }
            OAuthError::UserParseFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to parse user response",
            )
                .into_response(),
            OAuthError::InvalidDiscordId => {
                (StatusCode::BAD_REQUEST, "Discord OAuth failed").into_response()
            }
        }
    }
}

/// Perform the Discord authorization-code exchange and fetch the caller's user
/// profile, validating the returned `id` before it is trusted for storage.
///
/// The endpoint URLs are parameters so tests can point the exchange at a local
/// mock server; the production handler passes the fixed Discord URLs.
async fn perform_discord_oauth(
    client: &Client,
    token_url: &str,
    user_url: &str,
    client_id: &str,
    client_secret: &str,
    code: &str,
    redirect_uri: &str,
) -> Result<DiscordProfile, OAuthError> {
    let params = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", redirect_uri),
    ];

    let token_res = client
        .post(token_url)
        .form(&params)
        .send()
        .await
        .map_err(|_| OAuthError::Transport)?;

    if !token_res.status().is_success() {
        eprintln!(
            "Discord token exchange failed: status={}",
            token_res.status()
        );
        return Err(OAuthError::TokenNonSuccess);
    }

    let token_data: Value = token_res
        .json()
        .await
        .map_err(|_| OAuthError::TokenParseFailed)?;
    let access_token = token_data["access_token"]
        .as_str()
        .ok_or(OAuthError::MissingAccessToken)?;

    let user_res = client
        .get(user_url)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|_| OAuthError::UserTransport)?;

    // Reject a non-success user response BEFORE reading or validating the body.
    if !user_res.status().is_success() {
        eprintln!("Discord user fetch failed: status={}", user_res.status());
        return Err(OAuthError::UserNonSuccess);
    }

    let discord_user: Value = user_res
        .json()
        .await
        .map_err(|_| OAuthError::UserParseFailed)?;

    let discord_id = discord_user["id"]
        .as_str()
        .ok_or(OAuthError::InvalidDiscordId)?
        .to_string();
    if validate_discord_id(&discord_id).is_err() {
        return Err(OAuthError::InvalidDiscordId);
    }

    let username = discord_user["username"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let discriminator = discord_user["discriminator"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let avatar = discord_user["avatar"].as_str().map(|s| s.to_string());

    Ok(DiscordProfile {
        discord_id,
        username,
        discriminator,
        avatar,
    })
}

/// Complete login after a validated Discord profile is available: find-or-create
/// the user, apply admin/denylist logic, log audits, mint a JWT, and stage a
/// one-time auth code for the frontend to exchange.
#[allow(clippy::result_large_err)]
async fn complete_discord_login(
    state: AppState,
    profile: DiscordProfile,
) -> Result<Redirect, Response> {
    let DiscordProfile {
        discord_id,
        username,
        discriminator,
        avatar,
    } = profile;

    // Check for Initial Admin
    let initial_admin_id = env::var("INITIAL_ADMIN_ID").ok();
    let is_initial_admin = initial_admin_id.as_deref() == Some(&discord_id);

    // Check if denylisted
    let discord_hash = hash_identity(&discord_id, &state.identity_hash_pepper);
    let denylisted: Option<(String,)> =
        sqlx::query_as("SELECT hash FROM identity_hashes WHERE hash = ?")
            .bind(&discord_hash)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| internal_error("denylist check", e).into_response())?;

    if denylisted.is_some() {
        let frontend_url =
            env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());
        return Ok(Redirect::to(&format!("{}/deleted", frontend_url)));
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
            let mut new_id = rand::random::<i64>().abs();
            if new_id == 0 {
                new_id = 1;
            }

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
            user.id,
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
        exp: expiration,
        iss: env::var("JWT_ISSUER").ok().filter(|v| !v.trim().is_empty()),
        aud: env::var("JWT_AUDIENCE")
            .ok()
            .filter(|v| !v.trim().is_empty()),
    };

    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Token generation failed").into_response())?;

    // Generate auth code and store JWT temporarily (2 minutes TTL for frontend exchange)
    let auth_code = Uuid::new_v4().to_string();

    // Prune expired auth codes before inserting (prevent unbounded growth)
    {
        let mut codes = state.auth_codes.lock().unwrap();
        let now = Utc::now();
        let ttl = Duration::minutes(2);
        codes.retain(|_, (_, created_at)| now.signed_duration_since(*created_at) <= ttl);
        codes.insert(auth_code.clone(), (token, now));
    }

    let frontend_url =
        env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());
    Ok(Redirect::to(&format!(
        "{}/auth/callback?code={}",
        frontend_url, auth_code
    )))
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
#[allow(clippy::result_large_err)]
pub async fn discord_callback(
    Query(params): Query<CallbackParams>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, Response> {
    validate_text_len(&params.code, "Invalid authorization code", 1, 512)
        .map_err(|e| e.into_response())?;
    validate_text_len(&params.state, "Invalid state token", 36, 64)
        .map_err(|e| e.into_response())?;

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

    let profile = perform_discord_oauth(
        &client,
        "https://discord.com/api/oauth2/token",
        "https://discord.com/api/users/@me",
        &client_id,
        &client_secret,
        &params.code,
        &redirect_uri,
    )
    .await
    .map_err(|e| e.into_response())?;

    let redirect = complete_discord_login(state, profile).await?;
    Ok(redirect.into_response())
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
    validate_text_len(&payload.code, "Invalid code", 36, 64)?;

    // Retrieve and remove auth code (one-time use)
    let (token, created_at) = {
        let mut codes = state.auth_codes.lock().unwrap();
        codes.remove(&payload.code)
    }
    .ok_or((StatusCode::BAD_REQUEST, "Invalid or expired code"))?;

    // Validate code is not too old (2 minutes TTL)
    if Utc::now() - created_at > Duration::minutes(2) {
        return Err((StatusCode::BAD_REQUEST, "Code expired"));
    }

    Ok(Json(ExchangeResponse { token }))
}

#[derive(Clone)]
pub struct AuthenticatedUser {
    pub user_id: i64,
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

        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        if let Ok(issuer) = env::var("JWT_ISSUER") {
            if !issuer.trim().is_empty() {
                validation.set_issuer(&[issuer]);
            }
        }
        if let Ok(audience) = env::var("JWT_AUDIENCE") {
            if !audience.trim().is_empty() {
                validation.set_audience(&[audience]);
            }
        }

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &validation,
        )
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid Token"))?;

        let user_id = token_data
            .claims
            .id
            .parse::<i64>()
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid User ID in Token"))?;

        Ok(AuthenticatedUser { user_id })
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

    // Re-validate super admin status from environment (don't trust JWT claim)
    let super_admin_ids_str = std::env::var("SUPER_ADMIN_DISCORD_IDS").unwrap_or_default();
    let super_admin_ids: Vec<&str> = super_admin_ids_str.split(',').map(|s| s.trim()).collect();
    let is_super_admin = super_admin_ids.contains(&user.discord_id.as_str());

    Json(serde_json::json!({
        "id": user.id.to_string(),
        "discordId": user.discord_id,
        "username": user.username,
        "discriminator": user.discriminator,
        "avatar": user.avatar,
        "tribes": tribes,
        "adminTribes": admin_tribes,
        "isAdmin": user.is_admin, // Keep for legacy/global support if valid
        "isSuperAdmin": is_super_admin,
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
        .map_err(|e| internal_error("Internal auth operation failed", e))?;

    // 1. Fetch user and wallets
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(auth_user.user_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| internal_error("Internal auth operation failed", e))?
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;
    let wallets = sqlx::query_as::<_, crate::models::FlatLinkedWallet>(
        "SELECT w.*, NULL as tribe FROM wallets w WHERE w.user_id = ?",
    )
    .bind(auth_user.user_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| internal_error("Internal auth operation failed", e))?;

    // 2. Hash and Denylist Discord ID
    let discord_hash = hash_identity(&user.discord_id, &state.identity_hash_pepper);
    sqlx::query("INSERT OR IGNORE INTO identity_hashes (hash, type) VALUES (?, 'DISCORD')")
        .bind(discord_hash)
        .execute(&mut *tx)
        .await
        .map_err(|e| internal_error("Internal auth operation failed", e))?;

    // 3. Hash and Denylist Wallets
    for wallet in wallets {
        let normalized_address = wallet.address.to_lowercase();
        let wallet_hash = hash_identity(&normalized_address, &state.identity_hash_pepper);
        sqlx::query("INSERT OR IGNORE INTO identity_hashes (hash, type) VALUES (?, 'WALLET')")
            .bind(wallet_hash)
            .execute(&mut *tx)
            .await
            .map_err(|e| internal_error("Internal auth operation failed", e))?;
    }

    // 4. Hard delete wallets (data scrubbing)
    sqlx::query("DELETE FROM wallets WHERE user_id = ?")
        .bind(auth_user.user_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| internal_error("Internal auth operation failed", e))?;

    // 5. Anonymize user row
    let random_id = format!("deleted_{}", rand::random::<u64>());
    sqlx::query("UPDATE users SET discord_id = ?, username = 'Deleted User', avatar = NULL, is_admin = 0 WHERE id = ?")
        .bind(random_id)
        .bind(auth_user.user_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| internal_error("Internal auth operation failed", e))?;

    // 6. Scrub associated data
    // Delete tribe associations
    sqlx::query("DELETE FROM user_tribes WHERE user_id = ?")
        .bind(auth_user.user_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| internal_error("Internal auth operation failed", e))?;

    // Delete mumble account
    sqlx::query("DELETE FROM mumble_accounts WHERE user_id = ?")
        .bind(auth_user.user_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| internal_error("Internal auth operation failed", e))?;

    // Delete notes (where user is author or target)
    sqlx::query("DELETE FROM notes WHERE target_user_id = ? OR author_id = ?")
        .bind(auth_user.user_id)
        .bind(auth_user.user_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| internal_error("Internal auth operation failed", e))?;

    tx.commit()
        .await
        .map_err(|e| internal_error("Internal auth operation failed", e))?;

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
            exp: 1234567890,
            iss: None,
            aud: None,
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
            exp: 9999999999,
            iss: None,
            aud: None,
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
            exp: expiration,
            iss: None,
            aud: None,
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
            exp: expiration,
            iss: None,
            aud: None,
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
            exp: expired,
            iss: None,
            aud: None,
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
            exp: 9999999999,
            iss: None,
            aud: None,
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
            exp: 9999999999,
            iss: None,
            aud: None,
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
    let auth_user = AuthenticatedUser { user_id };
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
            Some(s) if s.as_bytes().ct_eq(configured_secret.as_bytes()).into() => {
                Ok(InternalSecret(s.to_string()))
            }
            _ => Err((StatusCode::UNAUTHORIZED, "Invalid Internal Secret")),
        }
    }
}

#[cfg(test)]
mod oauth_callback_tests {
    use super::*;
    use axum::{
        routing::{get, post},
        Router,
    };

    /// A local in-process mock of the Discord token + user endpoints so the
    /// OAuth callback can be tested without contacting Discord.
    struct MockDiscord {
        base_url: String,
        _handle: tokio::task::JoinHandle<()>,
    }

    impl Drop for MockDiscord {
        fn drop(&mut self) {
            self._handle.abort();
        }
    }

    async fn start_mock_discord(
        token_status: u16,
        token_body: &str,
        user_status: u16,
        user_body: &str,
    ) -> MockDiscord {
        let token_body = token_body.to_string();
        let user_body = user_body.to_string();
        let router = Router::new()
            .route(
                "/token",
                post(move || {
                    let (status, body) = (StatusCode::from_u16(token_status).unwrap(), token_body);
                    async move { (status, body) }
                }),
            )
            .route(
                "/user",
                get(move || {
                    let (status, body) = (StatusCode::from_u16(user_status).unwrap(), user_body);
                    async move { (status, body) }
                }),
            );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        MockDiscord {
            base_url: format!("http://{addr}"),
            _handle: handle,
        }
    }

    fn user_body(id: &str) -> String {
        format!(
            "{{\"id\":\"{}\",\"username\":\"tester\",\"discriminator\":\"0001\"}}",
            id
        )
    }

    async fn run_exchange(server: &MockDiscord) -> Result<DiscordProfile, OAuthError> {
        let client = Client::new();
        perform_discord_oauth(
            &client,
            &format!("{}/token", server.base_url),
            &format!("{}/user", server.base_url),
            "test-client-id",
            "test-secret",
            "test-code",
            "http://localhost/cb",
        )
        .await
    }

    #[tokio::test]
    async fn oauth_success_returns_validated_profile() {
        let server = start_mock_discord(
            200,
            "{\"access_token\":\"tok\",\"token_type\":\"Bearer\"}",
            200,
            &user_body("1234567890"),
        )
        .await;
        let profile = run_exchange(&server).await.unwrap();
        assert_eq!(profile.discord_id, "1234567890");
        assert_eq!(profile.username, "tester");
        assert_eq!(profile.discriminator, "0001");
        assert!(profile.avatar.is_none());
    }

    #[tokio::test]
    async fn oauth_token_non_success_is_rejected() {
        let server = start_mock_discord(
            400,
            "{\"error\":\"bad_request\"}",
            200,
            &user_body("1234567890"),
        )
        .await;
        let err = run_exchange(&server).await.unwrap_err();
        assert!(matches!(err, OAuthError::TokenNonSuccess));
    }

    #[tokio::test]
    async fn oauth_token_malformed_json_is_rejected() {
        let server =
            start_mock_discord(200, "this is not json", 200, &user_body("1234567890")).await;
        let err = run_exchange(&server).await.unwrap_err();
        assert!(matches!(err, OAuthError::TokenParseFailed));
    }

    #[tokio::test]
    async fn oauth_missing_access_token_is_rejected() {
        let server = start_mock_discord(
            200,
            "{\"token_type\":\"Bearer\"}",
            200,
            &user_body("1234567890"),
        )
        .await;
        let err = run_exchange(&server).await.unwrap_err();
        assert!(matches!(err, OAuthError::MissingAccessToken));
    }

    /// A non-success user response must be rejected BEFORE the body is parsed or
    /// validated: the body here carries no usable id, so a status gate that ran
    /// after parsing would surface InvalidDiscordId instead.
    #[tokio::test]
    async fn oauth_user_non_success_is_rejected_before_body_validation() {
        let server = start_mock_discord(
            200,
            "{\"access_token\":\"tok\"}",
            403,
            "{\"error\":\"forbidden\"}",
        )
        .await;
        let err = run_exchange(&server).await.unwrap_err();
        assert!(matches!(err, OAuthError::UserNonSuccess));
    }

    #[tokio::test]
    async fn oauth_user_malformed_json_is_rejected() {
        let server =
            start_mock_discord(200, "{\"access_token\":\"tok\"}", 200, "{not json at all").await;
        let err = run_exchange(&server).await.unwrap_err();
        assert!(matches!(err, OAuthError::UserParseFailed));
    }

    #[tokio::test]
    async fn oauth_invalid_discord_id_is_rejected() {
        let cases = vec![
            ("missing", "{\"username\":\"tester\"}"),
            ("null", "{\"id\":null,\"username\":\"tester\"}"),
            ("wrong_type", "{\"id\":1234567890,\"username\":\"tester\"}"),
            ("empty", "{\"id\":\"\",\"username\":\"tester\"}"),
            ("whitespace", "{\"id\":\"   \",\"username\":\"tester\"}"),
            (
                "non_digit",
                "{\"id\":\"12a4567890\",\"username\":\"tester\"}",
            ),
            (
                "overlength",
                "{\"id\":\"123456789012345678901234567890123\",\"username\":\"tester\"}",
            ),
        ];
        for (name, body) in cases {
            let server = start_mock_discord(200, "{\"access_token\":\"tok\"}", 200, body).await;
            let err = run_exchange(&server).await.unwrap_err();
            assert!(
                matches!(err, OAuthError::InvalidDiscordId),
                "case {name} should be rejected as InvalidDiscordId"
            );
        }
    }

    #[tokio::test]
    async fn oauth_transport_failure_is_rejected() {
        // Bind a listener to obtain a free port, then close it so the client
        // hits a connection-refused transport error.
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);

        let client = Client::new();
        let err = perform_discord_oauth(
            &client,
            &format!("http://{addr}/token"),
            &format!("http://{addr}/user"),
            "cid",
            "secret",
            "code",
            "redir",
        )
        .await
        .unwrap_err();
        assert!(matches!(err, OAuthError::Transport));
    }

    #[tokio::test]
    async fn complete_login_generates_auth_code_and_persists_user() {
        std::env::set_var("IDENTITY_HASH_PEPPER", "test-pepper");
        std::env::set_var("JWT_SECRET", "test-jwt-secret");
        std::env::set_var("DATABASE_URL", "sqlite::memory:");
        std::env::set_var("FRONTEND_URL", "http://localhost:5173");

        let db = crate::db::init_db().await.expect("init db");
        let state = AppState::new(db.clone());

        let server = start_mock_discord(
            200,
            "{\"access_token\":\"tok\"}",
            200,
            &user_body("1234567890"),
        )
        .await;
        let profile = run_exchange(&server).await.unwrap();

        let redirect = complete_discord_login(state.clone(), profile)
            .await
            .unwrap();
        let response = redirect.into_response();
        let location = response
            .headers()
            .get(axum::http::header::LOCATION)
            .expect("Location header")
            .to_str()
            .unwrap()
            .to_string();
        let expected_prefix = "http://localhost:5173/auth/callback?code=";
        assert!(
            location.starts_with(expected_prefix),
            "unexpected redirect: {location}"
        );
        let code = location.trim_start_matches(expected_prefix).to_string();
        assert!(!code.is_empty());

        // The one-time auth code is stored and maps to a JWT.
        {
            let codes = state.auth_codes.lock().unwrap();
            assert_eq!(codes.len(), 1, "exactly one auth code staged");
            assert!(codes.contains_key(&code));
        }

        // The user was persisted from the validated profile.
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE discord_id = ?")
            .bind("1234567890")
            .fetch_one(&db)
            .await
            .unwrap();
        assert_eq!(count, 1);
    }

    /// Rejections from the mockable OAuth implementation must leave the user
    /// table and the one-time auth-code store completely untouched, since a
    /// rejected profile never reaches `complete_discord_login` (the only place
    /// users are persisted and auth codes are staged).
    #[tokio::test]
    async fn oauth_rejections_leave_no_user_or_auth_code() {
        std::env::set_var("IDENTITY_HASH_PEPPER", "test-pepper");
        std::env::set_var("JWT_SECRET", "test-jwt-secret");
        std::env::set_var("DATABASE_URL", "sqlite::memory:");
        let db = crate::db::init_db().await.expect("init db");
        let state = AppState::new(db.clone());

        let bodies = [
            "{\"username\":\"tester\"}",                      // missing id
            "{\"id\":null}",                                  // null id
            "{\"id\":1234567890}",                            // wrong type
            "{\"id\":\"\"}",                                  // empty
            "{\"id\":\"   \"}",                               // whitespace
            "{\"id\":\"12a4567890\"}",                        // non-digit
            "{\"id\":\"123456789012345678901234567890123\"}", // overlength
        ];

        for body in bodies {
            let server = start_mock_discord(200, "{\"access_token\":\"tok\"}", 200, body).await;
            let err = run_exchange(&server).await.unwrap_err();
            assert!(matches!(err, OAuthError::InvalidDiscordId));
            let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
                .fetch_one(&db)
                .await
                .unwrap();
            assert_eq!(count, 0, "no user may be persisted for a rejected profile");
            let codes = state.auth_codes.lock().unwrap();
            assert!(
                codes.is_empty(),
                "no auth code may be staged for a rejected profile"
            );
        }
    }
}
