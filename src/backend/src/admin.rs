use crate::{
    audit::{alert_admin_action, AuditAction},
    middleware::admin::RequireSuperAdmin,
    models::User,
    state::AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

async fn get_admin_id(db: &crate::db::DbPool, discord_id: &str) -> i64 {
    sqlx::query_scalar("SELECT id FROM users WHERE discord_id = ?")
        .bind(discord_id)
        .fetch_one(db)
        .await
        .unwrap_or(0)
}

// --- Users ---
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub id: String,
    pub discord_id: String,
    pub username: String,
    pub discriminator: String,
    pub avatar: Option<String>,
    pub tribes: Vec<String>,
    pub admin_tribes: Vec<String>,
    pub is_admin: bool,
    pub is_super_admin: bool,
    pub last_login_at: Option<chrono::DateTime<chrono::Utc>>,
    pub wallets: Vec<crate::models::LinkedWallet>,
}

pub async fn list_users(
    State(state): State<AppState>,
    _admin: RequireSuperAdmin,
) -> impl IntoResponse {
    let users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY username ASC LIMIT 100")
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

    let mut response = Vec::new();

    for user in users {
        // Fetch wallets
        let flat_wallets = sqlx::query_as::<_, crate::models::FlatLinkedWallet>(
            "SELECT w.*, ut.tribe FROM wallets w LEFT JOIN user_tribes ut ON w.id = ut.wallet_id WHERE w.user_id = ?"
        )
            .bind(user.id)
            .fetch_all(&state.db)
            .await
            .unwrap_or_default();

        let mut wallet_map: std::collections::BTreeMap<String, crate::models::LinkedWallet> =
            std::collections::BTreeMap::new();
        for flat in flat_wallets {
            let entry =
                wallet_map
                    .entry(flat.id.clone())
                    .or_insert_with(|| crate::models::LinkedWallet {
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
        let wallets: Vec<crate::models::LinkedWallet> = wallet_map.into_values().collect();

        // Fetch tribes
        let user_tribes_rows = sqlx::query_as::<_, crate::models::UserTribe>(
            "SELECT * FROM user_tribes WHERE user_id = ?",
        )
        .bind(user.id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

        let tribes: Vec<String> = user_tribes_rows.iter().map(|ut| ut.tribe.clone()).collect();
        let admin_tribes: Vec<String> = user_tribes_rows
            .iter()
            .filter(|ut| ut.is_admin || user.is_admin)
            .map(|ut| ut.tribe.clone())
            .collect();

        // We don't have is_super_admin stored in users table (it's env var based or calculated).
        // For list view, we can just say false or check env var if we really want, but frontend might not care for other users.
        // Let's check `auth.rs` logic.
        // `is_super_admin` in `User` struct used in `get_me` comes from JWT claims.
        // Here we are listing OTHER users. They are likely NOT super admins unless their ID is in the env var.
        // Let's implement the same check if possible or just default to false since `is_admin` covers the main "admin" flag storage.
        // Actually, let's reuse the logic:
        let super_admin_ids_str = std::env::var("SUPER_ADMIN_DISCORD_IDS").unwrap_or_default();
        let super_admin_ids: Vec<&str> = super_admin_ids_str.split(',').map(|s| s.trim()).collect();
        let is_super_admin = super_admin_ids.contains(&user.discord_id.as_str());

        response.push(UserResponse {
            id: user.id.to_string(),
            discord_id: user.discord_id,
            username: user.username,
            discriminator: user.discriminator,
            avatar: user.avatar,
            tribes,
            admin_tribes,
            is_admin: user.is_admin,
            is_super_admin,
            last_login_at: user.last_login_at,
            wallets,
        });
    }

    Json(response).into_response()
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub is_admin: bool,
    pub username: String,
    pub discriminator: String,
}

pub async fn update_user(
    State(state): State<AppState>,
    admin: RequireSuperAdmin,
    Path(user_id): Path<i64>,
    Json(payload): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            eprintln!("Failed to start transaction for update_user: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let admin_id = get_admin_id(&state.db, &admin.discord_id).await;

    // calculate diff for audit
    let old_user = match sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            let _ = tx.rollback().await;
            return StatusCode::NOT_FOUND.into_response();
        }
        Err(e) => {
            eprintln!("Failed to fetch old user for update_user: {}", e);
            let _ = tx.rollback().await;
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let update_res =
        sqlx::query("UPDATE users SET is_admin = ?, username = ?, discriminator = ? WHERE id = ?")
            .bind(payload.is_admin)
            .bind(&payload.username)
            .bind(&payload.discriminator)
            .bind(user_id)
            .execute(&mut *tx)
            .await;

    if let Err(e) = update_res {
        eprintln!("Failed to update user in update_user: {}", e);
        let _ = tx.rollback().await;
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let changes = format!(
        "is_admin: {}->{}, username: {}->{}, discriminator: {}->{}",
        old_user.is_admin,
        payload.is_admin,
        old_user.username,
        payload.username,
        old_user.discriminator,
        payload.discriminator
    );

    // Manual audit insert with transaction
    let audit_id = uuid::Uuid::new_v4().to_string();
    let audit_res = sqlx::query(
        "INSERT INTO audit_logs (id, action, actor_id, target_id, details, created_at) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(audit_id)
    .bind(AuditAction::SuperAdminUpdateUser.as_str())
    .bind(admin_id)
    .bind(user_id)
    .bind(&changes)
    .bind(chrono::Utc::now())
    .execute(&mut *tx)
    .await;

    if let Err(e) = audit_res {
        eprintln!("Audit log insert failed for update_user: {}", e);
        let _ = tx.rollback().await;
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    if let Err(e) = tx.commit().await {
        eprintln!("Transaction commit failed for update_user: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    alert_admin_action(
        format!("SuperAdmin ({})", admin.discord_id),
        AuditAction::SuperAdminUpdateUser,
        format!("Updated User {}: {}", user_id, changes),
    );

    StatusCode::OK.into_response()
}

// --- Tribes ---

pub async fn list_tribes(
    State(state): State<AppState>,
    _admin: RequireSuperAdmin,
) -> impl IntoResponse {
    // List unique tribes from user_tribes... or is there a tribes table?
    // Looking at `helpers.rs` or `models.rs` might help.
    // The requirement says "List all tribes". In this DB schema `user_tribes` links users to tribes.
    // There might not be a `tribes` table if it's just strings.
    // Checking `models.rs`... `UserTribe` struct exists.
    // We can SELECT DISTINCT tribe FROM user_tribes.

    let tribes: Vec<String> = sqlx::query_scalar("SELECT name FROM tribes ORDER BY name ASC")
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

    Json(tribes).into_response()
}

#[derive(Deserialize)]
pub struct CreateTribeRequest {
    pub name: String,
}

pub async fn create_tribe(
    State(state): State<AppState>,
    admin: RequireSuperAdmin,
    Json(payload): Json<CreateTribeRequest>,
) -> impl IntoResponse {
    if payload.name.trim().is_empty() {
        return StatusCode::BAD_REQUEST.into_response();
    }

    // Insert into tribes table
    let res = sqlx::query("INSERT INTO tribes (name) VALUES (?)")
        .bind(&payload.name)
        .execute(&state.db)
        .await;

    if res.is_err() {
        return StatusCode::CONFLICT.into_response(); // Assuming unique constraint violation
    }

    alert_admin_action(
        format!("SuperAdmin ({})", admin.discord_id),
        AuditAction::SuperAdminCreateTribe,
        format!("Created Tribe '{}'", payload.name),
    );

    StatusCode::CREATED.into_response()
}

pub async fn update_tribe(
    State(state): State<AppState>,
    admin: RequireSuperAdmin,
    Path(tribe_name): Path<String>,
    Json(payload): Json<CreateTribeRequest>, // reusing struct for name update
) -> impl IntoResponse {
    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            eprintln!("Failed to start transaction for update_tribe: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let admin_id = get_admin_id(&state.db, &admin.discord_id).await;

    // Update the tribe name in the tribes table
    let update_tribes_res = sqlx::query("UPDATE tribes SET name = ? WHERE name = ?")
        .bind(&payload.name)
        .bind(&tribe_name)
        .execute(&mut *tx)
        .await;

    if let Err(e) = update_tribes_res {
        eprintln!("Failed to update tribe name in update_tribe: {}", e);
        let _ = tx.rollback().await;
        return StatusCode::CONFLICT.into_response(); // Potential duplicate name
    }

    // Update all instances of this tribe name in user_tribes
    let _ = sqlx::query("UPDATE user_tribes SET tribe = ? WHERE tribe = ?")
        .bind(&payload.name)
        .bind(&tribe_name)
        .execute(&mut *tx)
        .await;

    // Audit
    let audit_res = sqlx::query(
        "INSERT INTO audit_logs (id, action, actor_id, target_id, details, created_at) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(AuditAction::SuperAdminUpdateTribe.as_str())
    .bind(admin_id)
    .bind(None::<i64>)
    .bind(format!("Renamed Tribe '{}' to '{}'", tribe_name, payload.name))
    .bind(chrono::Utc::now())
    .execute(&mut *tx)
    .await;

    if let Err(e) = audit_res {
        eprintln!("Audit log insert failed for update_tribe: {}", e);
        let _ = tx.rollback().await;
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    if let Err(e) = tx.commit().await {
        eprintln!("Transaction commit failed for update_tribe: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    alert_admin_action(
        format!("SuperAdmin ({})", admin.discord_id),
        AuditAction::SuperAdminUpdateTribe,
        format!("Renamed Tribe '{}' to '{}'", tribe_name, payload.name),
    );

    StatusCode::OK.into_response()
}

#[derive(Deserialize)]
pub struct AddUserToTribeRequest {
    pub username: String,
}

pub async fn add_user_to_tribe(
    State(state): State<AppState>,
    admin: RequireSuperAdmin,
    Path(tribe_name): Path<String>,
    Json(payload): Json<AddUserToTribeRequest>,
) -> impl IntoResponse {
    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            eprintln!("Failed to start transaction for add_user_to_tribe: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let admin_id = get_admin_id(&state.db, &admin.discord_id).await;

    // 1. Find user by username (case-insensitive)
    let user =
        match sqlx::query_as::<_, User>("SELECT * FROM users WHERE LOWER(username) = LOWER(?)")
            .bind(&payload.username)
            .fetch_optional(&mut *tx)
            .await
        {
            Ok(Some(u)) => u,
            Ok(None) => {
                let _ = tx.rollback().await;
                return (StatusCode::NOT_FOUND, "User not found").into_response();
            }
            Err(e) => {
                eprintln!("Failed to find user in add_user_to_tribe: {}", e);
                let _ = tx.rollback().await;
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };

    // 2. Check if already in tribe
    let exists: bool = match sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM user_tribes WHERE user_id = ? AND tribe = ?)",
    )
    .bind(user.id)
    .bind(&tribe_name)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(e) => e,
        Err(e) => {
            eprintln!(
                "Failed to check tribe membership in add_user_to_tribe: {}",
                e
            );
            let _ = tx.rollback().await;
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if exists {
        let _ = tx.rollback().await;
        return (StatusCode::CONFLICT, "User already in tribe").into_response();
    }

    // 3. Add to user_tribes
    let res = sqlx::query(
        "INSERT INTO user_tribes (user_id, tribe, is_admin, created_at) VALUES (?, ?, ?, ?)",
    )
    .bind(user.id)
    .bind(&tribe_name)
    .bind(false)
    .bind(chrono::Utc::now())
    .execute(&mut *tx)
    .await;

    if let Err(e) = res {
        eprintln!("Failed to insert user_tribe in add_user_to_tribe: {}", e);
        let _ = tx.rollback().await;
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    // Audit
    let audit_id = uuid::Uuid::new_v4().to_string();
    let audit_res = sqlx::query(
        "INSERT INTO audit_logs (id, action, actor_id, target_id, details, created_at) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(audit_id)
    .bind(AuditAction::SuperAdminUpdateTribe.as_str()) // Reusing action or create new
    .bind(admin_id)
    .bind(user.id)
    .bind(format!("Added User '{}' to Tribe '{}'", user.username, tribe_name))
    .bind(chrono::Utc::now())
    .execute(&mut *tx)
    .await;

    if let Err(e) = audit_res {
        eprintln!("Audit log insert failed for add_user_to_tribe: {}", e);
        let _ = tx.rollback().await;
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    if let Err(e) = tx.commit().await {
        eprintln!("Transaction commit failed for add_user_to_tribe: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    alert_admin_action(
        format!("SuperAdmin ({})", admin.discord_id),
        AuditAction::SuperAdminUpdateTribe,
        format!("Added User '{}' to Tribe '{}'", user.username, tribe_name),
    );

    StatusCode::OK.into_response()
}

// --- Wallets ---

pub async fn delete_wallet(
    State(state): State<AppState>,
    admin: RequireSuperAdmin,
    Path(wallet_id): Path<String>,
) -> impl IntoResponse {
    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            eprintln!("Failed to start transaction for delete_wallet: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let admin_id = get_admin_id(&state.db, &admin.discord_id).await;

    let audit_res = sqlx::query(
        "INSERT INTO audit_logs (id, action, actor_id, target_id, details, created_at) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(AuditAction::SuperAdminDeleteWallet.as_str())
    .bind(admin_id)
    .bind(None::<i64>)
    .bind(format!("Forced soft delete wallet {}", wallet_id))
    .bind(chrono::Utc::now())
    .execute(&mut *tx)
    .await;

    if let Err(e) = audit_res {
        eprintln!("Audit log insert failed for delete_wallet: {}", e);
        let _ = tx.rollback().await;
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    // Also remove from user_tribes where verified by this wallet
    let _ = sqlx::query("UPDATE user_tribes SET wallet_id = NULL WHERE wallet_id = ?")
        .bind(&wallet_id)
        .execute(&mut *tx)
        .await;

    let del_res = sqlx::query(
        "UPDATE wallets SET deleted_at = CURRENT_TIMESTAMP WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(&wallet_id)
    .execute(&mut *tx)
    .await;

    if let Err(e) = del_res {
        eprintln!("Soft delete from wallets failed: {}", e);
        let _ = tx.rollback().await;
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    // Check if wallet was actually updated
    if del_res.unwrap().rows_affected() == 0 {
        let _ = tx.rollback().await;
        return (StatusCode::NOT_FOUND, "Wallet not found or already deleted").into_response();
    }

    if let Err(e) = tx.commit().await {
        eprintln!("Transaction commit failed for delete_wallet: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    alert_admin_action(
        format!("SuperAdmin ({})", admin.discord_id),
        AuditAction::SuperAdminDeleteWallet,
        format!("Deleted Wallet {}", wallet_id),
    );

    StatusCode::OK.into_response()
}
