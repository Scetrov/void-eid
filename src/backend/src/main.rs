use axum::{
    extract::ConnectInfo,
    http::StatusCode,
    routing::{delete, get, patch, post},
    Router,
};
use std::{net::SocketAddr, sync::Arc};
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let db_pool = init_db().await?;
    let state = AppState::new(db_pool);

    // CORS Configuration - Restrict to allowed origins
    let frontend_url =
        std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());
    let production_url = "https://voideid.scetrov.live".to_string();

    let allowed_origins: Vec<_> = [frontend_url, production_url]
        .iter()
        .map(|url| url.trim_end_matches('/').to_string())
        .filter_map(|url| url.parse::<axum::http::HeaderValue>().ok())
        .collect();

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

    // Internal routes (NO rate limiting - protected by INTERNAL_SECRET instead)
    let internal_routes =
        Router::new().route("/api/internal/mumble/verify", post(mumble::verify_login));

    let app = Router::new()
        .route("/ping", get(ping))
        .merge(auth_routes)
        .merge(wallet_routes)
        .merge(internal_routes)
        // Admin Routes
        .route("/api/admin/users", get(admin::list_users))
        .route("/api/admin/users/{id}", patch(admin::update_user))
        .route(
            "/api/admin/tribes",
            get(admin::list_tribes).post(admin::create_tribe),
        )
        .route("/api/admin/tribes/{id}", patch(admin::update_tribe))
        .route(
            "/api/admin/tribes/{id}/users",
            post(admin::add_user_to_tribe),
        )
        .route("/api/admin/wallets/{id}", delete(admin::delete_wallet))
        // Mumble routes
        .route("/api/mumble/account", post(mumble::create_account))
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
