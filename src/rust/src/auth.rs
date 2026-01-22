use crate::{
    audit::{log_audit, AuditAction},
    models::{LinkedWallet, User},
    state::AppState,
};
use axum::{
    async_trait,
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
use uuid::Uuid;

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
        return (StatusCode::BAD_REQUEST, "Discord token exchange failed").into_response();
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
    let user = match sqlx::query_as::<_, User>("SELECT * FROM users WHERE discord_id = ?")
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
                .bind(&u.id)
                .execute(&state.db)
                .await;
            u.last_login_at = Some(now);
            u
        }
        Ok(None) => {
            let now = Utc::now();
            let new_user = User {
                id: Uuid::new_v4().to_string(),
                discord_id: discord_id.clone(),
                username: username.clone(),
                discriminator: discriminator.clone(),
                avatar: avatar.clone(),
                tribe: None,
                is_admin: is_initial_admin,
                last_login_at: Some(now),
            };

            let _ = sqlx::query("INSERT INTO users (id, discord_id, username, discriminator, avatar, is_admin, last_login_at) VALUES (?, ?, ?, ?, ?, ?, ?)")
                .bind(&new_user.id)
                .bind(&new_user.discord_id)
                .bind(&new_user.username)
                .bind(&new_user.discriminator)
                .bind(&new_user.avatar)
                .bind(new_user.is_admin)
                .bind(now)
                .execute(&state.db)
                .await;

            new_user
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
        &user.id,
        None,
        &format!("User {} logged in via Discord", user.username),
    )
    .await;

    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET missing");
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        id: user.id.clone(),
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
    pub user_id: String,
}

#[async_trait]
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

        Ok(AuthenticatedUser {
            user_id: token_data.claims.id,
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
        .bind(&auth_user.user_id)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(u)) => u,
        _ => return (StatusCode::UNAUTHORIZED, "User not found").into_response(),
    };

    let wallets = sqlx::query_as::<_, LinkedWallet>("SELECT * FROM wallets WHERE user_id = ?")
        .bind(&auth_user.user_id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

    Json(serde_json::json!({
        "id": user.id,
        "discordId": user.discord_id,
        "username": user.username,
        "discriminator": user.discriminator,
        "avatar": user.avatar,
        "tribe": user.tribe,
        "isAdmin": user.is_admin,
        "lastLoginAt": user.last_login_at,
        "wallets": wallets
    }))
    .into_response()
}
