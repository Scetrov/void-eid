use crate::{
    audit::{log_audit, AuditAction},
    auth::AuthenticatedUser,
    helpers::{get_user_by_discord_id, require_admin_in_tribe},
    models::{LinkedWallet, User},
    state::AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Serialize, ToSchema, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogWithActor {
    pub id: String,
    pub action: String,
    pub actor_id: String,
    pub target_id: Option<String>,
    pub details: String,
    pub created_at: DateTime<Utc>,
    pub actor_username: String,
    pub actor_discriminator: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedAudits {
    pub items: Vec<AuditLogWithActor>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

#[derive(Serialize, ToSchema)]
pub struct RosterMember {
    pub discord_id: String,
    pub username: String,
    pub avatar: Option<String>,
    pub wallets: Vec<String>,
    pub audits: Option<PaginatedAudits>,
}

#[derive(Deserialize, IntoParams)]
pub struct MemberQuery {
    pub audit_page: Option<i64>,
    pub audit_per_page: Option<i64>,
}

#[derive(Deserialize, IntoParams)]
pub struct RosterQuery {
    pub sort: Option<String>,  // "username", "wallet_count"
    pub order: Option<String>, // "asc", "desc"
    pub search: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/roster/{discord_id}",
    params(
        ("discord_id" = String, Path, description = "Discord ID of the member to fetch"),
        ("audit_page" = Option<i64>, Query, description = "Page number for audit logs (1-indexed)"),
        ("audit_per_page" = Option<i64>, Query, description = "Items per page for audit logs (default 10)")
    ),
    responses(
        (status = 200, description = "Get roster member details", body = RosterMember),
        (status = 403, description = "Forbidden: Not an admin or different tribe"),
        (status = 404, description = "Member not found")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn get_roster_member(
    auth_user: AuthenticatedUser,
    Path(discord_id): Path<String>,
    Query(query): Query<MemberQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // 1. Verify admin in tribe
    let (current_user, tribe) = match require_admin_in_tribe(&state.db, &auth_user.user_id).await {
        Ok(result) => result,
        Err((status, msg)) => return (status, msg).into_response(),
    };

    // 2. Fetch Target Member
    let target_member = match get_user_by_discord_id(&state.db, &discord_id).await {
        Ok(Some(u)) => u,
        Ok(None) => return (StatusCode::NOT_FOUND, "Member not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
    };

    // 3. Verify Tribe Match
    match target_member.tribe {
        Some(ref t) if t == &tribe => {} // OK
        _ => {
            return (
                StatusCode::FORBIDDEN,
                "Access denied: Member is in a different tribe",
            )
                .into_response()
        }
    }

    // 4. Fetch Wallets
    let wallets = sqlx::query_as::<_, LinkedWallet>("SELECT * FROM wallets WHERE user_id = ?")
        .bind(&target_member.id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

    // 5. Fetch Audit Logs with Actor Info (paginated)
    // Include both: actions targeting this user AND self-actions (login, wallet link/unlink)
    let page = query.audit_page.unwrap_or(1).max(1);
    let per_page = query.audit_per_page.unwrap_or(10).clamp(1, 100);
    let offset = (page - 1) * per_page;

    // Get total count
    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM audit_logs WHERE target_id = ? OR (actor_id = ? AND target_id IS NULL)"
    )
    .bind(&target_member.id)
    .bind(&target_member.id)
    .fetch_one(&state.db)
    .await
    .unwrap_or((0,));

    let audits = sqlx::query_as::<_, AuditLogWithActor>(
        r#"
        SELECT a.id, a.action, a.actor_id, a.target_id, a.details, a.created_at, u.username as actor_username, u.discriminator as actor_discriminator
        FROM audit_logs a
        JOIN users u ON a.actor_id = u.id
        WHERE target_id = ? OR (actor_id = ? AND target_id IS NULL)
        ORDER BY created_at DESC
        LIMIT ? OFFSET ?
        "#
    )
    .bind(&target_member.id)
    .bind(&target_member.id)
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let total_pages = (total.0 as f64 / per_page as f64).ceil() as i64;

    // 6. Audit Log (Write) - Only log if viewing someone else (not self)
    if current_user.id != target_member.id {
        let _ = log_audit(
            &state.db,
            AuditAction::ViewMember,
            &current_user.id,
            Some(&target_member.id),
            &format!(
                "Viewed member {} ({})",
                target_member.username, target_member.discord_id
            ),
        )
        .await;
    }

    // 7. Return RosterMember
    Json(RosterMember {
        discord_id: target_member.discord_id,
        username: target_member.username,
        avatar: target_member.avatar,
        wallets: wallets.into_iter().map(|w| w.address).collect(),
        audits: Some(PaginatedAudits {
            items: audits,
            total: total.0,
            page,
            per_page,
            total_pages,
        }),
    })
    .into_response()
}

#[utoipa::path(
    get,
    path = "/api/roster",
    params(RosterQuery),
    responses(
        (status = 200, description = "Get tribe roster", body = Vec<RosterMember>),
        (status = 403, description = "Forbidden: Not an admin"),
        (status = 404, description = "User not found")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn get_roster(
    auth_user: AuthenticatedUser,
    Query(query): Query<RosterQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // 1. Verify admin in tribe
    let (current_user, tribe) = match require_admin_in_tribe(&state.db, &auth_user.user_id).await {
        Ok(result) => result,
        Err((status, msg)) => {
            // Special case: if no tribe, return empty roster instead of error
            if msg == "Access denied: You are not in a tribe" {
                return Json(Vec::<RosterMember>::new()).into_response();
            }
            return (status, msg).into_response();
        }
    };

    // 2. Build Query
    let mut sql = "SELECT * FROM users WHERE tribe = ?".to_string();

    if let Some(search) = &query.search {
        if !search.is_empty() {
            sql.push_str(" AND (username LIKE ? OR discord_id LIKE ?)");
        }
    }

    // Sorting (basic implementation)
    // Note: complex sorting often safer to do in code if not strictly pagination,
    // but SQL order by is fine. We need to be careful with SQL injection on column names though.
    // Bind parameters cannot be used for column names or ASC/DESC.
    match query.sort.as_deref() {
        Some("username") => {
            sql.push_str(" ORDER BY username");
        }
        _ => {
            // default sort
            sql.push_str(" ORDER BY username");
        }
    }

    if let Some("desc") = query.order.as_deref() {
        sql.push_str(" DESC");
    } else {
        sql.push_str(" ASC");
    }

    // 5. Execute Query
    let mut q = sqlx::query_as::<_, User>(&sql).bind(&tribe);

    if let Some(search) = &query.search {
        if !search.is_empty() {
            let pattern = format!("%{}%", search);
            q = q.bind(pattern.clone()).bind(pattern);
        }
    }

    let members = match q.fetch_all(&state.db).await {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("DB Error: {}", e),
            )
                .into_response()
        }
    };

    // 4. Batch fetch all wallets for these members (fixes N+1 query)
    let member_ids: Vec<&str> = members.iter().map(|m| m.id.as_str()).collect();

    let all_wallets: Vec<LinkedWallet> = if !member_ids.is_empty() {
        // Build parameterized IN clause
        let placeholders: String = member_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!("SELECT * FROM wallets WHERE user_id IN ({})", placeholders);

        let mut query_builder = sqlx::query_as::<_, LinkedWallet>(&sql);
        for id in &member_ids {
            query_builder = query_builder.bind(*id);
        }
        query_builder.fetch_all(&state.db).await.unwrap_or_default()
    } else {
        Vec::new()
    };

    // 5. Group wallets by user_id
    use std::collections::HashMap;
    let mut wallets_by_user: HashMap<String, Vec<String>> = HashMap::new();
    for wallet in all_wallets {
        wallets_by_user
            .entry(wallet.user_id.clone())
            .or_default()
            .push(wallet.address);
    }

    // 6. Build roster
    let mut roster: Vec<RosterMember> = members
        .into_iter()
        .map(|member| {
            let wallets = wallets_by_user.remove(&member.id).unwrap_or_default();
            RosterMember {
                discord_id: member.discord_id,
                username: member.username,
                avatar: member.avatar,
                wallets,
                audits: None,
            }
        })
        .collect();

    // 7. Post-processing sort if by wallet count
    if let Some("wallet_count") = query.sort.as_deref() {
        roster.sort_by(|a, b| a.wallets.len().cmp(&b.wallets.len()));
        if let Some("desc") = query.order.as_deref() {
            roster.reverse();
        }
    }

    // 8. Audit Log
    let _ = log_audit(
        &state.db,
        AuditAction::ViewRoster,
        &current_user.id,
        None,
        &format!("Viewed roster for tribe {}", tribe),
    )
    .await;

    Json(roster).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roster_query_struct() {
        let q = RosterQuery {
            sort: Some("username".to_string()),
            order: Some("asc".to_string()),
            search: None,
        };
        assert_eq!(q.sort.unwrap(), "username");
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    // use std::sync::Arc;
    use crate::models::User;

    // Helper to setup DB
    async fn setup_db() -> crate::db::DbPool {
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
    async fn test_roster_logic_admin_only() {
        let db = setup_db().await;
        let state = AppState::new(db.clone());

        // 1. Create Admin User
        let admin = User {
            id: "admin-id".to_string(),
            discord_id: "123".to_string(),
            username: "admin".to_string(),
            discriminator: "0000".to_string(),
            avatar: None,
            tribe: Some("Fire".to_string()),
            is_admin: true,
            last_login_at: None,
        };
        sqlx::query("INSERT INTO users (id, discord_id, username, discriminator, avatar, tribe, is_admin) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind(&admin.id).bind(&admin.discord_id).bind(&admin.username).bind(&admin.discriminator).bind(&admin.avatar).bind(&admin.tribe).bind(admin.is_admin)
            .execute(&db).await.unwrap();

        // 2. Create Normal User in same tribe
        let member = User {
            id: "member-id".to_string(),
            discord_id: "456".to_string(),
            username: "member".to_string(),
            discriminator: "1111".to_string(),
            avatar: None,
            tribe: Some("Fire".to_string()),
            is_admin: false,
            last_login_at: None,
        };
        sqlx::query("INSERT INTO users (id, discord_id, username, discriminator, avatar, tribe, is_admin) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind(&member.id).bind(&member.discord_id).bind(&member.username).bind(&member.discriminator).bind(&member.avatar).bind(&member.tribe).bind(member.is_admin)
            .execute(&db).await.unwrap();

        // 3. Create User in different tribe
        let alien = User {
            id: "alien-id".to_string(),
            discord_id: "789".to_string(),
            username: "alien".to_string(),
            discriminator: "2222".to_string(),
            avatar: None,
            tribe: Some("Water".to_string()),
            is_admin: false,
            last_login_at: None,
        };
        sqlx::query("INSERT INTO users (id, discord_id, username, discriminator, avatar, tribe, is_admin) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind(&alien.id).bind(&alien.discord_id).bind(&alien.username).bind(&alien.discriminator).bind(&alien.avatar).bind(&alien.tribe).bind(alien.is_admin)
            .execute(&db).await.unwrap();

        // 4. Test as Admin
        let auth_user = AuthenticatedUser {
            user_id: admin.id.clone(),
        };
        let query = RosterQuery {
            sort: None,
            order: None,
            search: None,
        };

        let response = get_roster(auth_user, Query(query), State(state.clone()))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);

        // We can't easily parse body here without axum::body::to_bytes, but integration status OK proves it didn't fail logic.
    }

    #[tokio::test]
    async fn test_roster_logic_forbidden() {
        let db = setup_db().await;
        let state = AppState::new(db.clone());

        let user = User {
            id: "user-id".to_string(),
            discord_id: "123".to_string(),
            username: "user".to_string(),
            discriminator: "0000".to_string(),
            avatar: None,
            tribe: Some("Fire".to_string()),
            is_admin: false,
            last_login_at: None,
        };
        sqlx::query("INSERT INTO users (id, discord_id, username, discriminator, avatar, tribe, is_admin) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind(&user.id).bind(&user.discord_id).bind(&user.username).bind(&user.discriminator).bind(&user.avatar).bind(&user.tribe).bind(user.is_admin)
            .execute(&db).await.unwrap();

        let auth_user = AuthenticatedUser {
            user_id: user.id.clone(),
        };
        let query = RosterQuery {
            sort: None,
            order: None,
            search: None,
        };

        let response = get_roster(auth_user, Query(query), State(state.clone()))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
