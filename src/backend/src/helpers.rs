use crate::{
    db::DbPool,
    models::{User, UserTribe},
};
use axum::http::StatusCode;

/// Result type for helper functions that can fail with HTTP errors
pub type ApiResult<T> = Result<T, (StatusCode, &'static str)>;

/// Fetch a user by their internal UUID
pub async fn get_user_by_id(db: &DbPool, id: i64) -> Result<Option<User>, sqlx::Error> {
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

/// Fetch all tribes for a user
pub async fn get_user_tribes(db: &DbPool, user_id: i64) -> Result<Vec<String>, sqlx::Error> {
    let tribes = sqlx::query_as::<_, UserTribe>("SELECT * FROM user_tribes WHERE user_id = ?")
        .bind(user_id)
        .fetch_all(db)
        .await?;

    Ok(tribes.into_iter().map(|ut| ut.tribe).collect())
}

/// Require that a user exists, is an admin, and is in a tribe.
/// If tribe parameter is provided, verifies user belongs to that specific tribe.
/// If tribe is None and user has exactly one tribe, uses that tribe.
/// If tribe is None and user has multiple tribes, returns error.
/// Returns (User, selected_tribe, all_tribes) on success, or an HTTP error tuple on failure.
pub async fn require_admin_in_tribe(
    db: &DbPool,
    user_id: i64,
    tribe: Option<&str>,
) -> ApiResult<(User, String, Vec<String>)> {
    let user = get_user_by_id(db, user_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?
        .ok_or((StatusCode::UNAUTHORIZED, "User not found"))?;

    if !user.is_admin {
        return Err((StatusCode::FORBIDDEN, "Access denied: Admins only"));
    }

    let user_tribes = get_user_tribes(db, user_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?;

    if user_tribes.is_empty() {
        return Err((
            StatusCode::FORBIDDEN,
            "Access denied: You are not in any tribe",
        ));
    }

    let selected_tribe = match tribe {
        Some(t) => {
            // Verify user belongs to the specified tribe
            if !user_tribes.contains(&t.to_string()) {
                return Err((
                    StatusCode::FORBIDDEN,
                    "Access denied: You are not in the specified tribe",
                ));
            }
            t.to_string()
        }
        None => {
            // If user has exactly one tribe, use it
            if user_tribes.len() == 1 {
                user_tribes[0].clone()
            } else {
                // User has multiple tribes but didn't specify which one
                return Err((
                    StatusCode::BAD_REQUEST,
                    "Please specify a tribe - you belong to multiple tribes",
                ));
            }
        }
    };

    Ok((user, selected_tribe, user_tribes))
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
        let result = get_user_by_id(&db, 99999).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_user_by_id_found() {
        let db = setup_db().await;

        sqlx::query("INSERT INTO users (id, discord_id, username, discriminator, is_admin) VALUES (?, ?, ?, ?, ?)")
            .bind(101_i64)
            .bind("123456")
            .bind("TestUser")
            .bind("0000")
            .bind(false)
            .execute(&db)
            .await
            .unwrap();

        let result = get_user_by_id(&db, 101).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().username, "TestUser");
    }

    #[tokio::test]
    async fn test_require_admin_in_tribe_not_admin() {
        let db = setup_db().await;

        sqlx::query("INSERT INTO users (id, discord_id, username, discriminator, is_admin) VALUES (?, ?, ?, ?, ?)")
            .bind(202_i64)
            .bind("123456")
            .bind("RegularUser")
            .bind("0000")
            .bind(false)
            .execute(&db)
            .await
            .unwrap();

        // Add user to a tribe
        sqlx::query("INSERT INTO user_tribes (user_id, tribe) VALUES (?, ?)")
            .bind(202_i64)
            .bind("Fire")
            .execute(&db)
            .await
            .unwrap();

        let result = require_admin_in_tribe(&db, 202, None).await;
        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_require_admin_in_tribe_success() {
        let db = setup_db().await;

        sqlx::query("INSERT INTO users (id, discord_id, username, discriminator, is_admin) VALUES (?, ?, ?, ?, ?)")
            .bind(303_i64)
            .bind("789")
            .bind("AdminUser")
            .bind("0000")
            .bind(true)
            .execute(&db)
            .await
            .unwrap();

        // Add user to a tribe
        sqlx::query("INSERT INTO user_tribes (user_id, tribe) VALUES (?, ?)")
            .bind(303_i64)
            .bind("Fire")
            .execute(&db)
            .await
            .unwrap();

        let result = require_admin_in_tribe(&db, 303, None).await;
        assert!(result.is_ok());
        let (user, tribe, all_tribes) = result.unwrap();
        assert_eq!(user.username, "AdminUser");
        assert_eq!(tribe, "Fire");
        assert_eq!(all_tribes, vec!["Fire"]);
    }
}
