use crate::db::DbPool;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    // Address -> Nonce
    pub wallet_nonces: Arc<Mutex<HashMap<String, String>>>,
}

impl AppState {
    pub fn new(db: DbPool) -> Self {
        Self {
            db,
            wallet_nonces: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
