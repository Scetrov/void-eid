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
use serde::{Deserialize, Serialize};

async fn get_admin_id(db: &crate::db::DbPool, discord_id: &str) -> i64 {
    sqlx::query_scalar("SELECT id FROM users WHERE discord_id = ?")
        .bind(discord_id)
        .fetch_one(db)
        .await
        .expect("failed to resolve admin user for audit logging")
}

// --- Users ---
#[derive(serde::Serialize, utoipa::ToSchema)]
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

#[utoipa::path(
    get,
    path = "/api/admin/users",
    tag = "Admin",
    responses(
        (status = 200, description = "List of all users", body = Vec<UserResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "User is not super admin"),
    )
)]
pub async fn list_users(
    State(state): State<AppState>,
    _admin: RequireSuperAdmin,
) -> impl IntoResponse {
    let users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY username ASC LIMIT 100")
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

    let user_ids: Vec<i64> = users.iter().map(|u| u.id).collect();

    // Batch-fetch all wallets for all users at once
    let flat_wallets = if !user_ids.is_empty() {
        sqlx::query_as::<_, crate::models::FlatLinkedWallet>(
            "SELECT w.*, ut.tribe FROM wallets w LEFT JOIN user_tribes ut ON w.id = ut.wallet_id WHERE w.user_id IN (SELECT id FROM users ORDER BY username ASC LIMIT 100)"
        )
            .fetch_all(&state.db)
            .await
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    // Batch-fetch all tribes for all users at once
    let all_user_tribes = if !user_ids.is_empty() {
        sqlx::query_as::<_, crate::models::UserTribe>(
            "SELECT * FROM user_tribes WHERE user_id IN (SELECT id FROM users ORDER BY username ASC LIMIT 100)"
        )
            .fetch_all(&state.db)
            .await
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    // Group wallets by user_id in memory
    let mut wallets_by_user: std::collections::HashMap<i64, Vec<crate::models::LinkedWallet>> =
        std::collections::HashMap::new();
    for flat in flat_wallets {
        let entry = wallets_by_user.entry(flat.user_id).or_default();

        // Check if we already have this wallet in the list
        if let Some(pos) = entry.iter().position(|w| w.id == flat.id) {
            if let Some(tribe) = &flat.tribe {
                if !entry[pos].tribes.contains(tribe) {
                    entry[pos].tribes.push(tribe.clone());
                }
            }
        } else {
            entry.push(crate::models::LinkedWallet {
                id: flat.id,
                user_id: flat.user_id,
                address: flat.address,
                verified_at: flat.verified_at,
                deleted_at: flat.deleted_at,
                tribes: flat.tribe.map(|t| vec![t]).unwrap_or_default(),
                network: flat.network,
            });
        }
    }

    // Group tribes by user_id in memory
    let mut tribes_by_user: std::collections::HashMap<i64, Vec<String>> =
        std::collections::HashMap::new();
    let mut admin_tribes_by_user: std::collections::HashMap<i64, Vec<String>> =
        std::collections::HashMap::new();

    for user_tribe in all_user_tribes {
        tribes_by_user
            .entry(user_tribe.user_id)
            .or_default()
            .push(user_tribe.tribe.clone());

        // Find the user to check is_admin flag
        if let Some(user) = users.iter().find(|u| u.id == user_tribe.user_id) {
            if user_tribe.is_admin || user.is_admin {
                admin_tribes_by_user
                    .entry(user_tribe.user_id)
                    .or_default()
                    .push(user_tribe.tribe);
            }
        }
    }

    let super_admin_ids_str = std::env::var("SUPER_ADMIN_DISCORD_IDS").unwrap_or_default();
    let super_admin_ids: Vec<&str> = super_admin_ids_str.split(',').map(|s| s.trim()).collect();

    let response: Vec<UserResponse> = users
        .into_iter()
        .map(|user| {
            let is_super_admin = super_admin_ids.contains(&user.discord_id.as_str());

            UserResponse {
                id: user.id.to_string(),
                discord_id: user.discord_id,
                username: user.username,
                discriminator: user.discriminator,
                avatar: user.avatar,
                tribes: tribes_by_user.get(&user.id).cloned().unwrap_or_default(),
                admin_tribes: admin_tribes_by_user
                    .get(&user.id)
                    .cloned()
                    .unwrap_or_default(),
                is_admin: user.is_admin,
                is_super_admin,
                last_login_at: user.last_login_at,
                wallets: wallets_by_user.get(&user.id).cloned().unwrap_or_default(),
            }
        })
        .collect();

    Json(response).into_response()
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct UpdateUserRequest {
    pub is_admin: bool,
    pub username: String,
    pub discriminator: String,
    pub admin_tribes: Vec<String>,
}

#[utoipa::path(
    patch,
    path = "/api/admin/users/{id}",
    tag = "Admin",
    params(
        ("id" = i64, Path, description = "User ID")
    ),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated successfully"),
        (status = 404, description = "User not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "User is not super admin"),
    )
)]
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

    // Update tribe admin status
    // 1. Reset all admin flags for this user in user_tribes
    let reset_res = sqlx::query("UPDATE user_tribes SET is_admin = FALSE WHERE user_id = ?")
        .bind(user_id)
        .execute(&mut *tx)
        .await;

    if let Err(e) = reset_res {
        eprintln!("Failed to reset tribe admin status: {}", e);
        let _ = tx.rollback().await;
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    // 2. Set admin=TRUE for the tribes in the payload
    for tribe in &payload.admin_tribes {
        let set_res =
            sqlx::query("UPDATE user_tribes SET is_admin = TRUE WHERE user_id = ? AND tribe = ?")
                .bind(user_id)
                .bind(tribe)
                .execute(&mut *tx)
                .await;

        if let Err(e) = set_res {
            eprintln!("Failed to set tribe admin status for {}: {}", tribe, e);
            let _ = tx.rollback().await;
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }

    let changes = format!(
        "is_admin: {}->{}, username: {}->{}, discriminator: {}->{}, admin_tribes: {:?}",
        old_user.is_admin,
        payload.is_admin,
        old_user.username,
        payload.username,
        old_user.discriminator,
        payload.discriminator,
        payload.admin_tribes
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

#[utoipa::path(
    get,
    path = "/api/admin/tribes",
    tag = "Admin",
    responses(
        (status = 200, description = "List of all tribe names", body = Vec<String>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "User is not super admin"),
    )
)]
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

#[derive(Deserialize, Serialize, utoipa::ToSchema)]
pub struct CreateTribeRequest {
    pub name: String,
}

#[utoipa::path(
    post,
    path = "/api/admin/tribes",
    tag = "Admin",
    request_body = CreateTribeRequest,
    responses(
        (status = 201, description = "Tribe created successfully"),
        (status = 400, description = "Invalid tribe name (empty or whitespace)"),
        (status = 409, description = "Tribe already exists"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "User is not super admin"),
    )
)]
pub async fn create_tribe(
    State(state): State<AppState>,
    admin: RequireSuperAdmin,
    Json(payload): Json<CreateTribeRequest>,
) -> impl IntoResponse {
    if payload.name.trim().is_empty() {
        return StatusCode::BAD_REQUEST.into_response();
    }

    if payload.name.len() > 100 {
        return (
            StatusCode::BAD_REQUEST,
            "Tribe name exceeds maximum length (100 characters)",
        )
            .into_response();
    }

    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            eprintln!("Failed to start transaction for create_tribe: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let admin_id = get_admin_id(&state.db, &admin.discord_id).await;

    let tribe_insert_res = sqlx::query("INSERT INTO tribes (name, created_at) VALUES (?, ?)")
        .bind(&payload.name)
        .bind(chrono::Utc::now())
        .execute(&mut *tx)
        .await;

    if let Err(e) = tribe_insert_res {
        eprintln!("Failed to insert tribe in create_tribe: {}", e);
        let _ = tx.rollback().await;
        return StatusCode::CONFLICT.into_response(); // Likely duplicate
    }

    // Audit log
    let audit_res = sqlx::query(
        "INSERT INTO audit_logs (id, action, actor_id, target_id, details, created_at) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(AuditAction::SuperAdminCreateTribe.as_str())
    .bind(admin_id)
    .bind(None::<i64>)
    .bind(format!("Created Tribe '{}'", payload.name))
    .bind(chrono::Utc::now())
    .execute(&mut *tx)
    .await;

    if let Err(e) = audit_res {
        eprintln!("Failed to insert audit log for create_tribe: {}", e);
        let _ = tx.rollback().await;
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    if let Err(e) = tx.commit().await {
        eprintln!("Transaction commit failed for create_tribe: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    alert_admin_action(
        format!("SuperAdmin ({})", admin.discord_id),
        AuditAction::SuperAdminCreateTribe,
        format!("Created Tribe '{}'", payload.name),
    );

    StatusCode::CREATED.into_response()
}

#[utoipa::path(
    patch,
    path = "/api/admin/tribes/{id}",
    tag = "Admin",
    params(
        ("id" = String, Path, description = "Tribe name")
    ),
    request_body = CreateTribeRequest,
    responses(
        (status = 200, description = "Tribe updated successfully"),
        (status = 400, description = "Invalid tribe name"),
        (status = 409, description = "Tribe name already exists"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "User is not super admin"),
    )
)]
pub async fn update_tribe(
    State(state): State<AppState>,
    admin: RequireSuperAdmin,
    Path(tribe_name): Path<String>,
    Json(payload): Json<CreateTribeRequest>, // reusing struct for name update
) -> impl IntoResponse {
    if payload.name.trim().is_empty() {
        return StatusCode::BAD_REQUEST.into_response();
    }

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
    let update_user_tribes_res = sqlx::query("UPDATE user_tribes SET tribe = ? WHERE tribe = ?")
        .bind(&payload.name)
        .bind(&tribe_name)
        .execute(&mut *tx)
        .await;

    if let Err(e) = update_user_tribes_res {
        eprintln!(
            "Failed to update tribe name in user_tribes in update_tribe: {}",
            e
        );
        let _ = tx.rollback().await;
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

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

#[derive(Deserialize, utoipa::ToSchema)]
pub struct AddUserToTribeRequest {
    pub username: String,
}

#[utoipa::path(
    post,
    path = "/api/admin/tribes/{id}/users",
    tag = "Admin",
    params(
        ("id" = String, Path, description = "Tribe name")
    ),
    request_body = AddUserToTribeRequest,
    responses(
        (status = 200, description = "User added to tribe successfully"),
        (status = 400, description = "Tribe does not exist or user already in tribe"),
        (status = 404, description = "User not found"),
        (status = 409, description = "User already in tribe"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "User is not super admin"),
    )
)]
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

    // Validate username length
    if payload.username.len() > 100 {
        let _ = tx.rollback().await;
        return (
            StatusCode::BAD_REQUEST,
            "Username exceeds maximum length (100 characters)",
        )
            .into_response();
    }

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

    // 3. Verify tribe exists in tribes table
    let tribe_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM tribes WHERE name = ?)")
            .bind(&tribe_name)
            .fetch_one(&mut *tx)
            .await
            .unwrap_or(false);

    if !tribe_exists {
        let _ = tx.rollback().await;
        return (StatusCode::BAD_REQUEST, "Tribe does not exist").into_response();
    }

    // 4. Add to user_tribes
    let res = sqlx::query(
        "INSERT INTO user_tribes (user_id, tribe, is_admin, created_at, source) VALUES (?, ?, ?, ?, 'MANUAL')",
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

#[utoipa::path(
    delete,
    path = "/api/admin/wallets/{id}",
    tag = "Admin",
    params(
        ("id" = String, Path, description = "Wallet ID")
    ),
    responses(
        (status = 200, description = "Wallet deleted successfully"),
        (status = 404, description = "Wallet not found or already deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "User is not super admin"),
    )
)]
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
    let cleanup_res = sqlx::query("UPDATE user_tribes SET wallet_id = NULL WHERE wallet_id = ?")
        .bind(&wallet_id)
        .execute(&mut *tx)
        .await;

    if let Err(e) = cleanup_res {
        eprintln!(
            "Failed to cleanup user_tribes wallet reference in delete_wallet: {}",
            e
        );
        let _ = tx.rollback().await;
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let del_res = match sqlx::query(
        "UPDATE wallets SET deleted_at = CURRENT_TIMESTAMP WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(&wallet_id)
    .execute(&mut *tx)
    .await
    {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Soft delete from wallets failed: {}", e);
            let _ = tx.rollback().await;
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Check if wallet was actually updated
    if del_res.rows_affected() == 0 {
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
