use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use utoipa::ToSchema;

pub mod i64_as_string {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &i64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&value.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<i64, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct User {
    #[serde(with = "i64_as_string")]
    pub id: i64,
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
    #[serde(with = "i64_as_string")]
    pub user_id: i64,
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
    #[serde(with = "i64_as_string")]
    pub user_id: i64,
    pub address: String,
    pub verified_at: DateTime<Utc>,
    pub tribes: Vec<String>,
}

// Internal helper for SQL mapping before grouping
#[derive(Debug, FromRow)]
pub struct FlatLinkedWallet {
    pub id: String,
    pub user_id: i64,
    pub address: String,
    pub verified_at: DateTime<Utc>,
    pub tribe: Option<String>,
}
