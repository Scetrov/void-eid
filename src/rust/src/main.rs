use crate::db::init_db;
use crate::state::AppState;
use axum::{
    routing::{delete, get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

mod audit;
mod auth;
mod db;
mod helpers;
mod models;
mod roster;
mod state;
mod wallet;

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

        roster::get_roster,
        roster::get_roster_member
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
            roster::RosterMember
        )
    ),
    tags(
        (name = "auth", description = "Authentication Endpoints"),
        (name = "wallet", description = "Wallet Management Endpoints")
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
        .filter_map(|url| url.parse::<axum::http::HeaderValue>().ok())
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::DELETE,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
        ]);

    let app = Router::new()
        .route("/api/auth/discord/login", get(auth::discord_login))
        .route("/api/auth/discord/callback", get(auth::discord_callback))
        .route("/api/me", get(auth::get_me))
        .route("/api/wallets/link-nonce", post(wallet::link_nonce))
        .route("/api/wallets/link-verify", post(wallet::link_verify))
        .route("/api/wallets/{id}", delete(wallet::unlink_wallet))
        .route("/api/roster", get(roster::get_roster))
        .route("/api/roster/{discord_id}", get(roster::get_roster_member))
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
