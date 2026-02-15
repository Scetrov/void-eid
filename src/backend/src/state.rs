use crate::db::DbPool;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// (User ID, Tribe) -> Last Viewed At
pub type RosterViews = Arc<Mutex<HashMap<(i64, String), chrono::DateTime<chrono::Utc>>>>;

// State token -> Created At (for OAuth2 CSRF protection)
pub type OAuthStates = Arc<Mutex<HashMap<String, chrono::DateTime<chrono::Utc>>>>;

// Auth code -> (JWT token, Created At) (for secure token exchange)
pub type AuthCodes = Arc<Mutex<HashMap<String, (String, chrono::DateTime<chrono::Utc>)>>>;

// Wallet address -> (Nonce, Created At) (for signature verification)
pub type WalletNonces = Arc<Mutex<HashMap<String, (String, chrono::DateTime<chrono::Utc>)>>>;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    // Address -> (Nonce, Created At)
    pub wallet_nonces: WalletNonces,
    pub mumble_required_tribe: String,
    pub roster_views: RosterViews,
    pub identity_hash_pepper: String,
    pub oauth_states: OAuthStates,
    pub auth_codes: AuthCodes,
}

impl AppState {
    pub fn new(db: DbPool) -> Self {
        let mumble_required_tribe =
            std::env::var("MUMBLE_REQUIRED_TRIBE").unwrap_or_else(|_| "Fire".to_string());
        let identity_hash_pepper = std::env::var("IDENTITY_HASH_PEPPER")
            .expect("IDENTITY_HASH_PEPPER must be set for security and deterministic hashing");
        Self {
            db,
            wallet_nonces: Arc::new(Mutex::new(HashMap::new())),
            mumble_required_tribe,
            roster_views: Arc::new(Mutex::new(HashMap::new())),
            identity_hash_pepper,
            oauth_states: Arc::new(Mutex::new(HashMap::new())),
            auth_codes: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
