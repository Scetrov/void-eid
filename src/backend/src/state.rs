use crate::db::DbPool;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    // Address -> Nonce
    pub wallet_nonces: Arc<Mutex<HashMap<String, String>>>,
    pub mumble_required_tribe: String,
}

impl AppState {
    pub fn new(db: DbPool) -> Self {
        let mumble_required_tribe =
            std::env::var("MUMBLE_REQUIRED_TRIBE").unwrap_or_else(|_| "Fire".to_string());
        Self {
            db,
            wallet_nonces: Arc::new(Mutex::new(HashMap::new())),
            mumble_required_tribe,
        }
    }
}
