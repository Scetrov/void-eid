use axum::{routing::get, Router};
use state::AppState;

pub mod admin;
pub mod audit;
pub mod auth;
pub mod db;
pub mod helpers;
pub mod middleware;
pub mod models;
pub mod mumble;
pub mod notes;
pub mod roster;
pub mod state;

pub mod sui_verify;
pub mod wallet;

pub fn get_common_router() -> Router<AppState> {
    Router::new()
        .route("/api/me", get(auth::get_me).delete(auth::delete_me))
        .route("/api/roster", get(roster::get_roster))
        .route("/api/roster/{discord_id}", get(roster::get_roster_member))
        .route("/api/roster/{discord_id}/notes", get(notes::get_notes))
}
