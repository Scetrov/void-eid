use crate::db::DbPool;
use chrono::Utc;
use uuid::Uuid;

/// Actions that can be recorded in the audit log
#[derive(Debug, Clone, Copy)]
pub enum AuditAction {
    Login,
    LinkWallet,
    UnlinkWallet,
    ViewRoster,
    ViewMember,
}

impl AuditAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditAction::Login => "LOGIN",
            AuditAction::LinkWallet => "LINK_WALLET",
            AuditAction::UnlinkWallet => "UNLINK_WALLET",
            AuditAction::ViewRoster => "VIEW_ROSTER",
            AuditAction::ViewMember => "VIEW_MEMBER",
        }
    }
}

/// Log an action to the audit_logs table
pub async fn log_audit(
    db: &DbPool,
    action: AuditAction,
    actor_id: &str,
    target_id: Option<&str>,
    details: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO audit_logs (id, action, actor_id, target_id, details, created_at) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(Uuid::new_v4().to_string())
    .bind(action.as_str())
    .bind(actor_id)
    .bind(target_id)
    .bind(details)
    .bind(Utc::now())
    .execute(db)
    .await?;

    Ok(())
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

        // Insert a user for foreign key constraint
        sqlx::query("INSERT INTO users (id, discord_id, username, discriminator, is_admin) VALUES (?, ?, ?, ?, ?)")
            .bind("actor-id")
            .bind("123456")
            .bind("TestActor")
            .bind("0000")
            .bind(false)
            .execute(&pool)
            .await
            .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_log_audit_without_target() {
        let db = setup_db().await;

        let result = log_audit(
            &db,
            AuditAction::ViewRoster,
            "actor-id",
            None,
            "Viewed roster",
        )
        .await;
        assert!(result.is_ok());

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM audit_logs")
            .fetch_one(&db)
            .await
            .unwrap();
        assert_eq!(count.0, 1);
    }

    #[tokio::test]
    async fn test_log_audit_with_target() {
        let db = setup_db().await;

        // Insert target user
        sqlx::query("INSERT INTO users (id, discord_id, username, discriminator, is_admin) VALUES (?, ?, ?, ?, ?)")
            .bind("target-id")
            .bind("789")
            .bind("TargetUser")
            .bind("0000")
            .bind(false)
            .execute(&db)
            .await
            .unwrap();

        let result = log_audit(
            &db,
            AuditAction::ViewMember,
            "actor-id",
            Some("target-id"),
            "Viewed member details",
        )
        .await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_audit_action_as_str() {
        assert_eq!(AuditAction::Login.as_str(), "LOGIN");
        assert_eq!(AuditAction::LinkWallet.as_str(), "LINK_WALLET");
        assert_eq!(AuditAction::UnlinkWallet.as_str(), "UNLINK_WALLET");
        assert_eq!(AuditAction::ViewRoster.as_str(), "VIEW_ROSTER");
        assert_eq!(AuditAction::ViewMember.as_str(), "VIEW_MEMBER");
    }
}
