use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use void_eid_backend::db::init_db;
use void_eid_backend::state::AppState;

use void_eid_backend::{admin, auth, models, mumble, notes, roster, wallet};

use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

#[derive(OpenApi)]
#[openapi(
    paths(
        auth::discord_login,
        auth::discord_callback,
        auth::get_me,
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

    let app = Router::new()
        .route("/api/auth/discord/login", get(auth::discord_login))
        .route("/api/auth/discord/callback", get(auth::discord_callback))
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
        .route("/api/internal/mumble/verify", post(mumble::verify_login))
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
    axum::serve(listener, app).await?;

    Ok(())
}
