use axum::{
    routing::{delete, get, post},
    Router,
};
use state::AppState;

pub mod audit;
pub mod auth;
pub mod db;
pub mod helpers;
pub mod models;
pub mod roster;
pub mod state;
pub mod wallet;

pub fn get_common_router() -> Router<AppState> {
    Router::new()
        .route("/api/me", get(auth::get_me))
        .route("/api/wallets/link-nonce", post(wallet::link_nonce))
        .route("/api/wallets/link-verify", post(wallet::link_verify))
        .route("/api/wallets/{id}", delete(wallet::unlink_wallet))
        .route("/api/roster", get(roster::get_roster))
        .route("/api/roster/{discord_id}", get(roster::get_roster_member))
}
