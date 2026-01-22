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
    pub tribe: Option<String>,
    #[serde(default)]
    pub is_admin: bool,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LinkedWallet {
    pub id: String,
    pub user_id: String,
    pub address: String,
    pub verified_at: DateTime<Utc>,
}
