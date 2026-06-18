use axum::{
    extract::ConnectInfo,
    http::{HeaderValue, StatusCode, Uri},
    routing::{delete, get, patch, post, put},
    Router,
};
use std::{env, net::SocketAddr, sync::Arc};
use tower_governor::{
    errors::GovernorError,
    governor::GovernorConfigBuilder,
    key_extractor::{KeyExtractor, SmartIpKeyExtractor},
    GovernorLayer,
};
use tower_http::cors::CorsLayer;
use void_eid_backend::db::init_db;
use void_eid_backend::state::AppState;

use void_eid_backend::{admin, auth, models, mumble, notes, roster, wallet};

use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

/// Custom key extractor that falls back to a default value if IP extraction fails
#[derive(Clone)]
struct FallbackIpKeyExtractor;

impl KeyExtractor for FallbackIpKeyExtractor {
    type Key = String;

    fn extract<T>(&self, req: &axum::http::Request<T>) -> Result<Self::Key, GovernorError> {
        // Try SmartIpKeyExtractor first
        let smart_extractor = SmartIpKeyExtractor;
        if let Ok(ip) = smart_extractor.extract(req) {
            return Ok(ip.to_string());
        }

        // Fallback 1: Try to get ConnectInfo
        if let Some(ConnectInfo(addr)) = req.extensions().get::<ConnectInfo<SocketAddr>>() {
            return Ok(addr.ip().to_string());
        }

        // Fallback 2: Use a default key for internal/unknown sources
        // This ensures rate limiting still works but groups unknown requesters
        Ok("fallback-internal".to_string())
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        auth::discord_login,
        auth::discord_callback,
        auth::exchange_code,
        auth::get_me,
        auth::delete_me,
        wallet::link_nonce,
        wallet::link_verify,
        wallet::unlink_wallet,

        admin::list_users,
        admin::update_user,
        admin::list_tribes,
        admin::create_tribe,
        admin::update_tribe,
        admin::add_user_to_tribe,
        admin::delete_wallet,

        roster::get_roster,
        roster::get_roster_member,
        roster::grant_admin,

        notes::get_notes,
        notes::create_note,
        notes::edit_note
    ),
    components(
        schemas(
            models::User,
            models::LinkedWallet,
            wallet::NonceRequest,
            wallet::NonceResponse,
            wallet::VerifyRequest,
            auth::CallbackParams,
            auth::Claims,
            auth::ExchangeRequest,
            auth::ExchangeResponse,
            admin::UserResponse,
            admin::UpdateUserRequest,
            admin::CreateTribeRequest,
            admin::AddUserToTribeRequest,
            roster::RosterMember,
            roster::GrantAdminRequest,
            notes::Note,
            notes::NoteWithAuthor,
            notes::CreateNoteRequest,
            notes::EditNoteRequest
        )
    ),
    tags(
        (name = "auth", description = "Authentication Endpoints"),
        (name = "wallet", description = "Wallet Management Endpoints"),
        (name = "roster", description = "Roster Management Endpoints"),
        (name = "notes", description = "Notes Management Endpoints")
    ),
    security(
        ("jwt" = [])
    ),
    modifiers(&SecurityAddon)
)]
struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "jwt",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            )
        }
    }
}

/// Health check endpoint for frontend connection verification
async fn ping() -> (StatusCode, &'static str) {
    (StatusCode::OK, "pong")
}

fn require_env(name: &str) -> anyhow::Result<String> {
    let value = env::var(name).map_err(|_| anyhow::anyhow!("{name} must be set"))?;
    if value.trim().is_empty() {
        anyhow::bail!("{name} must not be empty");
    }
    Ok(value)
}

fn require_strong_secret(name: &str, min_len: usize) -> anyhow::Result<String> {
    let value = require_env(name)?;
    if value.len() < min_len {
        anyhow::bail!("{name} must be at least {min_len} characters");
    }
    if value.chars().all(|c| c.is_ascii_alphanumeric()) {
        anyhow::bail!("{name} must include non-alphanumeric entropy");
    }
    Ok(value)
}

fn validate_origin(name: &str, raw: &str) -> anyhow::Result<HeaderValue> {
    let trimmed = raw.trim().trim_end_matches('/');
    let uri: Uri = trimmed
        .parse()
        .map_err(|_| anyhow::anyhow!("{name} must be a valid absolute URL"))?;
    match uri.scheme_str() {
        Some("http") | Some("https") => {}
        _ => anyhow::bail!("{name} must use http or https"),
    }
    if uri.host().is_none() {
        anyhow::bail!("{name} must include a host");
    }
    if uri.path() != "/" || uri.query().is_some() {
        anyhow::bail!("{name} must be an origin without path or query");
    }
    HeaderValue::from_str(trimmed)
        .map_err(|_| anyhow::anyhow!("{name} is not a valid header value"))
}

fn validate_security_config() -> anyhow::Result<Vec<HeaderValue>> {
    require_env("DISCORD_CLIENT_ID")?;
    require_strong_secret("DISCORD_CLIENT_SECRET", 32)?;
    require_env("DISCORD_REDIRECT_URI")?;
    require_strong_secret("JWT_SECRET", 32)?;
    require_strong_secret("INTERNAL_SECRET", 32)?;
    require_strong_secret("IDENTITY_HASH_PEPPER", 32)?;

    let frontend_url =
        env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());
    let production_url = "https://voideid.scetrov.live";

    Ok(vec![
        validate_origin("FRONTEND_URL", &frontend_url)?,
        validate_origin("PRODUCTION_FRONTEND_URL", production_url)?,
    ])
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let allowed_origins = validate_security_config()?;

    let db_pool = init_db().await?;
    let state = AppState::new(db_pool);

    // CORS Configuration - Restrict to validated allowed origins
    let cors = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::PATCH,
            axum::http::Method::DELETE,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
        ]);

    // Rate limiting configuration for sensitive endpoints
    // Use FallbackIpKeyExtractor to handle Docker networking gracefully
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(2)
            .burst_size(5)
            .key_extractor(FallbackIpKeyExtractor)
            .finish()
            .expect("Failed to create rate limit config"),
    );
    let rate_limit_layer = GovernorLayer::new(governor_conf);

    // Rate-limited authentication routes
    let auth_routes = Router::new()
        .route("/api/auth/discord/login", get(auth::discord_login))
        .route("/api/auth/discord/callback", get(auth::discord_callback))
        .route("/api/auth/exchange", post(auth::exchange_code))
        .layer(rate_limit_layer.clone());

    // Rate-limited wallet routes
    let wallet_routes = Router::new()
        .route("/api/wallets/link-nonce", post(wallet::link_nonce))
        .route("/api/wallets/link-verify", post(wallet::link_verify))
        .layer(rate_limit_layer.clone());

    // Rate-limited internal verification routes (also protected by INTERNAL_SECRET)
    let internal_routes = Router::new()
        .route("/api/internal/mumble/verify", post(mumble::verify_login))
        .layer(rate_limit_layer.clone());

    // Rate-limited write-oriented routes
    let sensitive_routes = Router::new()
        .route("/api/admin/users/{id}", patch(admin::update_user))
        .route("/api/admin/tribes", post(admin::create_tribe))
        .route("/api/admin/tribes/{id}", patch(admin::update_tribe))
        .route(
            "/api/admin/tribes/{id}/users",
            post(admin::add_user_to_tribe),
        )
        .route("/api/admin/wallets/{id}", delete(admin::delete_wallet))
        .route("/api/wallets/{id}", delete(wallet::unlink_wallet))
        .route(
            "/api/roster/{discord_id}/grant-admin",
            post(roster::grant_admin),
        )
        .route("/api/roster/{discord_id}/notes", post(notes::create_note))
        .route("/api/notes/{note_id}", put(notes::edit_note))
        .route("/api/mumble/account", post(mumble::create_account))
        .layer(rate_limit_layer.clone());

    let app = Router::new()
        .route("/ping", get(ping))
        .merge(auth_routes)
        .merge(wallet_routes)
        .merge(internal_routes)
        .merge(sensitive_routes)
        // Admin read routes
        .route("/api/admin/users", get(admin::list_users))
        .route("/api/admin/tribes", get(admin::list_tribes))
        // Mumble read routes
        .route("/api/mumble/status", get(mumble::get_status))
        .merge(void_eid_backend::get_common_router())
        .merge(Scalar::with_url("/docs", ApiDoc::openapi()))
        .layer(cors)
        .with_state(state);

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "5038".to_string())
        .parse::<u16>()?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
