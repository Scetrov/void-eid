use crate::{db::DbPool, models::User};
use axum::http::StatusCode;

/// Result type for helper functions that can fail with HTTP errors
pub type ApiResult<T> = Result<T, (StatusCode, &'static str)>;

/// Fetch a user by their internal UUID
pub async fn get_user_by_id(db: &DbPool, id: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(id)
        .fetch_optional(db)
        .await
}

/// Fetch a user by their Discord ID
pub async fn get_user_by_discord_id(
    db: &DbPool,
    discord_id: &str,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE discord_id = ?")
        .bind(discord_id)
        .fetch_optional(db)
        .await
}

/// Require that a user exists, is an admin, and is in a tribe.
/// Returns (User, tribe_name) on success, or an HTTP error tuple on failure.
pub async fn require_admin_in_tribe(db: &DbPool, user_id: &str) -> ApiResult<(User, String)> {
    let user = get_user_by_id(db, user_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?
        .ok_or((StatusCode::UNAUTHORIZED, "User not found"))?;

    if !user.is_admin {
        return Err((StatusCode::FORBIDDEN, "Access denied: Admins only"));
    }

    let tribe = user.tribe.clone().ok_or((
        StatusCode::FORBIDDEN,
        "Access denied: You are not in a tribe",
    ))?;

    Ok((user, tribe))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_db() -> DbPool {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create memory pool");

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Migrations failed");

        pool
    }

    #[tokio::test]
    async fn test_get_user_by_id_not_found() {
        let db = setup_db().await;
        let result = get_user_by_id(&db, "nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_user_by_id_found() {
        let db = setup_db().await;

        sqlx::query("INSERT INTO users (id, discord_id, username, discriminator, is_admin) VALUES (?, ?, ?, ?, ?)")
            .bind("test-id")
            .bind("123456")
            .bind("TestUser")
            .bind("0000")
            .bind(false)
            .execute(&db)
            .await
            .unwrap();

        let result = get_user_by_id(&db, "test-id").await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().username, "TestUser");
    }

    #[tokio::test]
    async fn test_require_admin_in_tribe_not_admin() {
        let db = setup_db().await;

        sqlx::query("INSERT INTO users (id, discord_id, username, discriminator, tribe, is_admin) VALUES (?, ?, ?, ?, ?, ?)")
            .bind("user-id")
            .bind("123456")
            .bind("RegularUser")
            .bind("0000")
            .bind("Fire")
            .bind(false)
            .execute(&db)
            .await
            .unwrap();

        let result = require_admin_in_tribe(&db, "user-id").await;
        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_require_admin_in_tribe_success() {
        let db = setup_db().await;

        sqlx::query("INSERT INTO users (id, discord_id, username, discriminator, tribe, is_admin) VALUES (?, ?, ?, ?, ?, ?)")
            .bind("admin-id")
            .bind("789")
            .bind("AdminUser")
            .bind("0000")
            .bind("Fire")
            .bind(true)
            .execute(&db)
            .await
            .unwrap();

        let result = require_admin_in_tribe(&db, "admin-id").await;
        assert!(result.is_ok());
        let (user, tribe) = result.unwrap();
        assert_eq!(user.username, "AdminUser");
        assert_eq!(tribe, "Fire");
    }
}
