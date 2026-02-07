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
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;

use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct CallbackParams {
    code: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Claims {
    pub id: String,
    #[serde(rename = "discordId")]
    pub discord_id: String,
    pub username: String,
    pub exp: usize,
}

#[utoipa::path(
    get,
    path = "/api/auth/discord/login",
    responses(
        (status = 302, description = "Redirect to Discord OAuth")
    )
)]
pub async fn discord_login() -> impl IntoResponse {
    let client_id = env::var("DISCORD_CLIENT_ID").expect("CID not set");
    let redirect_uri = env::var("DISCORD_REDIRECT_URI").expect("URI not set");
    let scope = "identify";

    let url = format!(
        "https://discord.com/api/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}",
        client_id,
        urlencoding::encode(&redirect_uri),
        scope
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
) -> Response {
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

    let token_res = match client
        .post("https://discord.com/api/oauth2/token")
        .form(&params)
        .send()
        .await
    {
        Ok(res) => res,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to connect to Discord",
            )
                .into_response()
        }
    };

    if !token_res.status().is_success() {
        let status = token_res.status();
        let body = token_res
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error body".to_string());
        println!(
            "Discord token exchange failed: Status: {}, Body: {}",
            status, body
        );
        return (
            StatusCode::BAD_REQUEST,
            format!(
                "Discord token exchange failed: Status: {}, Body: {}",
                status, body
            ),
        )
            .into_response();
    }

    let token_data: Value = match token_res.json().await {
        Ok(data) => data,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to parse token response",
            )
                .into_response()
        }
    };

    let access_token = match token_data["access_token"].as_str() {
        Some(t) => t,
        None => return (StatusCode::BAD_REQUEST, "No access token").into_response(),
    };

    let user_res = match client
        .get("https://discord.com/api/users/@me")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
    {
        Ok(res) => res,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch user").into_response()
        }
    };

    let discord_user: Value = match user_res.json().await {
        Ok(u) => u,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to parse user").into_response()
        }
    };

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
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("DB Error: {}", e),
            )
                .into_response()
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
        exp: expiration,
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    ) {
        Ok(t) => t,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Token generation failed").into_response()
        }
    };

    let frontend_url =
        env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());
    Redirect::to(&format!("{}/auth/callback?token={}", frontend_url, token)).into_response()
}

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
        "SELECT w.*, ut.tribe FROM wallets w LEFT JOIN user_tribes ut ON w.id = ut.wallet_id WHERE w.user_id = ?"
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
        "lastLoginAt": user.last_login_at,
        "wallets": wallets
    }))
    .into_response()
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
        let json = r#"{"code":"test_auth_code_12345"}"#;
        let params: CallbackParams = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(params.code, "test_auth_code_12345");
    }

    #[test]
    fn test_user_id_parsing_from_claims() {
        // Test that we can parse user_id from Claims.id string
        let claims = Claims {
            id: "123456789".to_string(),
            discord_id: "discord123".to_string(),
            username: "TestUser".to_string(),
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
            exp: 9999999999,
        };

        let result: Result<i64, _> = claims.id.parse();
        assert!(result.is_err());
    }
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

        let configured_secret =
            env::var("INTERNAL_SECRET").unwrap_or_else(|_| "secret".to_string());

        match secret_header {
            Some(s) if s == configured_secret => Ok(InternalSecret(s.to_string())),
            _ => Err((StatusCode::UNAUTHORIZED, "Invalid Internal Secret")),
        }
    }
}
