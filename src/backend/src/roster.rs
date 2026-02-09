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
    pub actor_id: i64,
    pub target_id: Option<i64>,
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
#[serde(rename_all = "camelCase")]
pub struct RosterMember {
    pub discord_id: String,
    pub username: String,
    pub avatar: Option<String>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub wallets: Vec<LinkedWallet>,
    pub audits: Option<PaginatedAudits>,
}

#[derive(Deserialize, IntoParams)]
pub struct MemberQuery {
    pub tribe: Option<String>,
    pub audit_page: Option<i64>,
    pub audit_per_page: Option<i64>,
}

#[derive(Deserialize, IntoParams)]
pub struct RosterQuery {
    pub tribe: Option<String>,
    pub sort: Option<String>,  // "username", "wallet_count", "last_login"
    pub order: Option<String>, // "asc", "desc"
    pub search: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct GrantAdminRequest {
    pub wallet_id: String,
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
    let (current_user, tribe, _all_tribes) =
        match require_admin_in_tribe(&state.db, auth_user.user_id, query.tribe.as_deref()).await {
            Ok(result) => result,
            Err((status, msg)) => return (status, msg).into_response(),
        };

    // 2. Fetch Target Member
    let target_member = match get_user_by_discord_id(&state.db, &discord_id).await {
        Ok(Some(u)) => u,
        Ok(None) => return (StatusCode::NOT_FOUND, "Member not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
    };

    // 3. Verify Tribe Match - check if target member is in the specified tribe
    let target_tribes = match crate::helpers::get_user_tribes(&state.db, target_member.id).await {
        Ok(tribes) => tribes,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
    };

    if !target_tribes.contains(&tribe) {
        return (
            StatusCode::FORBIDDEN,
            "Access denied: Member is not in the specified tribe",
        )
            .into_response();
    }

    // 4. Fetch Wallets with Tribe Info
    let flat_wallets = sqlx::query_as::<_, crate::models::FlatLinkedWallet>(
        "SELECT w.*, ut.tribe FROM wallets w LEFT JOIN user_tribes ut ON w.id = ut.wallet_id WHERE w.user_id = ?"
    )
    .bind(target_member.id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    // Group flat results into LinkedWallet with tribes Vec
    let mut wallet_map: std::collections::BTreeMap<String, LinkedWallet> =
        std::collections::BTreeMap::new();
    for flat in flat_wallets {
        let entry = wallet_map
            .entry(flat.id.clone())
            .or_insert_with(|| LinkedWallet {
                id: flat.id,
                user_id: flat.user_id,
                address: flat.address,
                verified_at: flat.verified_at,
                deleted_at: flat.deleted_at,
                tribes: Vec::new(),
            });
        if let Some(t) = flat.tribe {
            if !entry.tribes.contains(&t) {
                entry.tribes.push(t);
            }
        }
    }
    let wallets: Vec<LinkedWallet> = wallet_map.into_values().collect();

    // 5. Fetch Audit Logs with Actor Info (paginated)
    // Include both: actions targeting this user AND self-actions (login, wallet link/unlink)
    let page = query.audit_page.unwrap_or(1).max(1);
    let per_page = query.audit_per_page.unwrap_or(10).clamp(1, 100);
    let offset = (page - 1) * per_page;

    // Get total count
    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM audit_logs WHERE target_id = ? OR (actor_id = ? AND target_id IS NULL)"
    )
    .bind(target_member.id)
    .bind(target_member.id)
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
    .bind(target_member.id)
    .bind(target_member.id)
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
            current_user.id,
            Some(target_member.id),
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
        last_login_at: target_member.last_login_at,
        wallets,
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
    let (current_user, tribe, _all_tribes) =
        match require_admin_in_tribe(&state.db, auth_user.user_id, query.tribe.as_deref()).await {
            Ok(result) => result,
            Err((status, msg)) => {
                // Special case: if no tribe, return empty roster instead of error
                if msg == "Access denied: You are not in any tribe" {
                    return Json(Vec::<RosterMember>::new()).into_response();
                }
                return (status, msg).into_response();
            }
        };

    // 2. Build Query - get all users in the specified tribe from user_tribes table
    let mut sql =
        "SELECT u.* FROM users u INNER JOIN user_tribes ut ON u.id = ut.user_id WHERE ut.tribe = ?"
            .to_string();

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
        Some("last_login") => {
            sql.push_str(" ORDER BY last_login_at");
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
    let member_ids: Vec<i64> = members.iter().map(|m| m.id).collect();

    let all_wallets: Vec<crate::models::FlatLinkedWallet> = if !member_ids.is_empty() {
        // Build parameterized IN clause
        let placeholders: String = member_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!("SELECT w.*, ut.tribe FROM wallets w LEFT JOIN user_tribes ut ON w.id = ut.wallet_id WHERE w.user_id IN ({})", placeholders);

        let mut query_builder = sqlx::query_as::<_, crate::models::FlatLinkedWallet>(&sql);
        for id in &member_ids {
            query_builder = query_builder.bind(*id);
        }
        query_builder.fetch_all(&state.db).await.unwrap_or_default()
    } else {
        Vec::new()
    };

    // 5. Group wallets by user_id, and internally group tribes by wallet ID
    use std::collections::HashMap;
    let mut wallets_by_user: HashMap<i64, Vec<LinkedWallet>> = HashMap::new();

    // Intermediate map to group tribes by wallet ID first
    let mut wallet_map: HashMap<String, LinkedWallet> = HashMap::new();
    for flat in all_wallets {
        let entry = wallet_map
            .entry(flat.id.clone())
            .or_insert_with(|| LinkedWallet {
                id: flat.id,
                user_id: flat.user_id,
                address: flat.address,
                verified_at: flat.verified_at,
                deleted_at: flat.deleted_at,
                tribes: Vec::new(),
            });
        if let Some(t) = flat.tribe {
            if !entry.tribes.contains(&t) {
                entry.tribes.push(t);
            }
        }
    }

    // Now group the unique wallets by user_id
    for (_, wallet) in wallet_map {
        wallets_by_user
            .entry(wallet.user_id)
            .or_default()
            .push(wallet);
    }

    // 6. Final Roster Response
    let mut roster: Vec<RosterMember> = members
        .into_iter()
        .map(|m| {
            let user_wallets = wallets_by_user.get(&m.id).cloned().unwrap_or_default();
            RosterMember {
                discord_id: m.discord_id,
                username: m.username,
                avatar: m.avatar,
                last_login_at: m.last_login_at,
                wallets: user_wallets,
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
        current_user.id,
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
            tribe: None,
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
            id: 111_i64,
            discord_id: "123".to_string(),
            username: "admin".to_string(),
            discriminator: "0000".to_string(),
            avatar: None,
            is_admin: true,
            last_login_at: None,
        };
        sqlx::query("INSERT INTO users (id, discord_id, username, discriminator, avatar, is_admin) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(admin.id).bind(&admin.discord_id).bind(&admin.username).bind(&admin.discriminator).bind(&admin.avatar).bind(admin.is_admin)
            .execute(&db).await.unwrap();

        // Add admin to Fire tribe
        sqlx::query("INSERT INTO user_tribes (user_id, tribe) VALUES (?, ?)")
            .bind(admin.id)
            .bind("Fire")
            .execute(&db)
            .await
            .unwrap();

        // 2. Create Normal User in same tribe
        let member = User {
            id: 222_i64,
            discord_id: "456".to_string(),
            username: "member".to_string(),
            discriminator: "1111".to_string(),
            avatar: None,
            is_admin: false,
            last_login_at: None,
        };
        sqlx::query("INSERT INTO users (id, discord_id, username, discriminator, avatar, is_admin) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(member.id).bind(&member.discord_id).bind(&member.username).bind(&member.discriminator).bind(&member.avatar).bind(member.is_admin)
            .execute(&db).await.unwrap();

        // Add member to Fire tribe
        sqlx::query("INSERT INTO user_tribes (user_id, tribe) VALUES (?, ?)")
            .bind(member.id)
            .bind("Fire")
            .execute(&db)
            .await
            .unwrap();

        // 3. Create User in different tribe
        let alien = User {
            id: 333_i64,
            discord_id: "789".to_string(),
            username: "alien".to_string(),
            discriminator: "2222".to_string(),
            avatar: None,
            is_admin: false,
            last_login_at: None,
        };
        sqlx::query("INSERT INTO users (id, discord_id, username, discriminator, avatar, is_admin) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(alien.id).bind(&alien.discord_id).bind(&alien.username).bind(&alien.discriminator).bind(&alien.avatar).bind(alien.is_admin)
            .execute(&db).await.unwrap();

        // Add alien to Water tribe
        sqlx::query("INSERT INTO user_tribes (user_id, tribe) VALUES (?, ?)")
            .bind(alien.id)
            .bind("Water")
            .execute(&db)
            .await
            .unwrap();

        // 4. Test as Admin
        let auth_user = AuthenticatedUser {
            user_id: admin.id,
            is_super_admin: false,
        };
        let query = RosterQuery {
            tribe: None,
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
            id: 444_i64,
            discord_id: "123".to_string(),
            username: "user".to_string(),
            discriminator: "0000".to_string(),
            avatar: None,
            is_admin: false,
            last_login_at: None,
        };
        sqlx::query("INSERT INTO users (id, discord_id, username, discriminator, avatar, is_admin) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(user.id).bind(&user.discord_id).bind(&user.username).bind(&user.discriminator).bind(&user.avatar).bind(user.is_admin)
            .execute(&db).await.unwrap();

        // Add user to Fire tribe
        sqlx::query("INSERT INTO user_tribes (user_id, tribe) VALUES (?, ?)")
            .bind(user.id)
            .bind("Fire")
            .execute(&db)
            .await
            .unwrap();

        let auth_user = AuthenticatedUser {
            user_id: user.id,
            is_super_admin: false,
        };
        let query = RosterQuery {
            tribe: None,
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

#[utoipa::path(
    post,
    path = "/api/roster/{discord_id}/grant-admin",
    params(
        ("discord_id" = String, Path, description = "Discord ID of the member"),
        MemberQuery
    ),
    request_body = GrantAdminRequest,
    responses(
        (status = 200, description = "Admin granted successfully"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "User or wallet not found")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn grant_admin(
    Path(discord_id): Path<String>,
    Query(query): Query<MemberQuery>,
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<GrantAdminRequest>,
) -> impl IntoResponse {
    // Verify admin in tribe
    let (current_user, tribe, _all_tribes) =
        match require_admin_in_tribe(&state.db, auth_user.user_id, query.tribe.as_deref()).await {
            Ok(result) => result,
            Err(e) => return e.into_response(),
        };

    // Get target user
    let target_user = match get_user_by_discord_id(&state.db, &discord_id).await {
        Ok(Some(u)) => u,
        Ok(None) => return (StatusCode::NOT_FOUND, "User not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
    };

    // Verify wallet exists, belongs to target user, and is active
    let wallet: Option<crate::models::FlatLinkedWallet> = sqlx::query_as(
        "SELECT *, NULL as tribe FROM wallets WHERE id = ? AND user_id = ? AND deleted_at IS NULL",
    )
    .bind(&payload.wallet_id)
    .bind(target_user.id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    if wallet.is_none() {
        return (
            StatusCode::NOT_FOUND,
            "Wallet not found or doesn't belong to user",
        )
            .into_response();
    }

    // Check if user_tribe entry already exists
    let existing: Option<(i64,)> =
        sqlx::query_as("SELECT user_id FROM user_tribes WHERE user_id = ? AND tribe = ?")
            .bind(target_user.id)
            .bind(&tribe)
            .fetch_optional(&state.db)
            .await
            .unwrap_or(None);

    if existing.is_some() {
        // Update existing entry
        let result = sqlx::query(
            "UPDATE user_tribes SET wallet_id = ?, is_admin = TRUE, source = 'MANUAL' WHERE user_id = ? AND tribe = ?",
        )
        .bind(&payload.wallet_id)
        .bind(target_user.id)
        .bind(&tribe)
        .execute(&state.db)
        .await;

        if result.is_err() {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to grant admin").into_response();
        }
    } else {
        // Insert new entry
        let result = sqlx::query(
            "INSERT INTO user_tribes (user_id, tribe, wallet_id, is_admin, source) VALUES (?, ?, ?, TRUE, 'MANUAL')",
        )
        .bind(target_user.id)
        .bind(&tribe)
        .bind(&payload.wallet_id)
        .execute(&state.db)
        .await;

        if result.is_err() {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to grant admin").into_response();
        }
    }

    // Audit log
    let _ = log_audit(
        &state.db,
        AuditAction::AdminGrant,
        current_user.id,
        Some(target_user.id),
        &format!(
            "Granted admin to {} in tribe {} via wallet {}",
            target_user.username, tribe, payload.wallet_id
        ),
    )
    .await;

    Json(serde_json::json!({ "message": "Admin granted successfully" })).into_response()
}
