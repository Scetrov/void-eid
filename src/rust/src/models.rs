use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct User {
    pub id: String,
    pub discord_id: String,
    pub username: String,
    pub discriminator: String,
    pub avatar: Option<String>,
    #[serde(default)]
    pub is_admin: bool,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct UserTribe {
    pub user_id: String,
    pub tribe: String,
    pub wallet_id: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub is_admin: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LinkedWallet {
    pub id: String,
    pub user_id: String,
    pub address: String,
    pub verified_at: DateTime<Utc>,
    pub tribes: Vec<String>,
}

// Internal helper for SQL mapping before grouping
#[derive(Debug, FromRow)]
pub struct FlatLinkedWallet {
    pub id: String,
    pub user_id: String,
    pub address: String,
    pub verified_at: DateTime<Utc>,
    pub tribe: Option<String>,
}
