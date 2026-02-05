use crate::db::DbPool;
use chrono::Utc;
use uuid::Uuid;

/// Actions that can be recorded in the audit log
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // Some variants are defined for future use
pub enum AuditAction {
    Login,
    LinkWallet,
    UnlinkWallet,
    ViewRoster,
    ViewMember,
    AdminGrant,
    AdminRevoke,
    TribeJoin,
    TribeLeave,
    NoteCreate,
    NoteEdit,
    MumbleCreateAccount,
    MumbleLogin,
}

impl AuditAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditAction::Login => "LOGIN",
            AuditAction::LinkWallet => "LINK_WALLET",
            AuditAction::UnlinkWallet => "UNLINK_WALLET",
            AuditAction::ViewRoster => "VIEW_ROSTER",
            AuditAction::ViewMember => "VIEW_MEMBER",
            AuditAction::AdminGrant => "ADMIN_GRANT",
            AuditAction::AdminRevoke => "ADMIN_REVOKE",
            AuditAction::TribeJoin => "TRIBE_JOIN",
            AuditAction::TribeLeave => "TRIBE_LEAVE",
            AuditAction::NoteCreate => "NOTE_CREATE",
            AuditAction::NoteEdit => "NOTE_EDIT",
            AuditAction::MumbleCreateAccount => "MUMBLE_CREATE_ACCOUNT",
            AuditAction::MumbleLogin => "MUMBLE_LOGIN",
        }
    }
}

/// Log an action to the audit_logs table
pub async fn log_audit(
    db: &DbPool,
    action: AuditAction,
    actor_id: i64,
    target_id: Option<i64>,
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
            .bind(1001_i64)
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

        let result = log_audit(&db, AuditAction::ViewRoster, 1001, None, "Viewed roster").await;
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
            .bind(2002_i64)
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
            1001,
            Some(2002),
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
        assert_eq!(AuditAction::AdminGrant.as_str(), "ADMIN_GRANT");
        assert_eq!(AuditAction::AdminRevoke.as_str(), "ADMIN_REVOKE");
        assert_eq!(AuditAction::TribeJoin.as_str(), "TRIBE_JOIN");
        assert_eq!(AuditAction::TribeLeave.as_str(), "TRIBE_LEAVE");
        assert_eq!(AuditAction::NoteCreate.as_str(), "NOTE_CREATE");
        assert_eq!(AuditAction::NoteEdit.as_str(), "NOTE_EDIT");
    }
}
