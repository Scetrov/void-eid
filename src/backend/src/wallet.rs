use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::auth::AuthenticatedUser;
use crate::{
    audit::{log_audit, AuditAction},
    models::FlatLinkedWallet,
    state::AppState,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::Utc;
use shared_crypto::intent::{Intent, IntentMessage};
use sui_sdk::types::base_types::SuiAddress;
use sui_sdk::types::crypto::{Signature, SuiSignature, ToFromBytes};
use uuid::Uuid;

use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct NonceRequest {
    address: String,
}

#[derive(Serialize, ToSchema)]
pub struct NonceResponse {
    nonce: String,
}

#[derive(Deserialize, ToSchema)]
pub struct VerifyRequest {
    address: String,
    signature: String,
}

#[utoipa::path(
    post,
    path = "/api/wallets/link-nonce",
    request_body = NonceRequest,
    responses(
        (status = 200, description = "Nonce generated", body = NonceResponse)
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn link_nonce(
    State(state): State<AppState>,
    _: AuthenticatedUser, // Require login
    Json(payload): Json<NonceRequest>,
) -> impl IntoResponse {
    let nonce = Uuid::new_v4().to_string();
    let mut nonces = state.wallet_nonces.lock().unwrap();
    nonces.insert(payload.address.to_lowercase(), nonce.clone());

    Json(NonceResponse { nonce })
}

#[derive(Serialize)]
struct PersonalMessage<'a> {
    message: &'a [u8],
}

#[utoipa::path(
    post,
    path = "/api/wallets/link-verify",
    request_body = VerifyRequest,
    responses(
        (status = 200, description = "Wallet linked successfully"),
        (status = 400, description = "Invalid signature or wallet already linked")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn link_verify(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<VerifyRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let address_str = payload.address.to_lowercase();

    // Check Nonce
    let stored_nonce = {
        let mut nonces = state.wallet_nonces.lock().unwrap();
        nonces
            .remove(&address_str)
            .ok_or((StatusCode::BAD_REQUEST, "Nonce invalid or expired".into()))?
    };

    // Verify Signature
    let sig_bytes = STANDARD
        .decode(&payload.signature)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid base64: {}", e)))?;

    let sig = Signature::from_bytes(&sig_bytes).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Invalid signature format: {}", e),
        )
    })?;

    let sui_address = SuiAddress::from_str(&address_str).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Invalid address format: {}", e),
        )
    })?;

    let message_bytes = stored_nonce.as_bytes();

    let msg_struct = PersonalMessage {
        message: message_bytes,
    };

    let intent = Intent::personal_message();
    let intent_msg = IntentMessage::new(intent, msg_struct);

    let result = sig.verify_secure(&intent_msg, sui_address, sig.scheme());

    // If verify_secure expects the Struct, we might need a wrapper.
    // Assuming verify_secure<T>(value: &T, intent, author)

    if result.is_err() {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Signature verification failed: {:?}", result.err()),
        ));
    }

    // Check availability (including soft-deleted)
    let existing: Option<FlatLinkedWallet> =
        sqlx::query_as("SELECT *, NULL as tribe FROM wallets WHERE address = ?")
            .bind(&address_str)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(w) = existing {
        if w.deleted_at.is_none() {
            return Err((StatusCode::BAD_REQUEST, "Wallet already linked".into()));
        }

        // Re-link: Update user_id and clear deleted_at
        sqlx::query(
            "UPDATE wallets SET user_id = ?, verified_at = ?, deleted_at = NULL WHERE id = ?",
        )
        .bind(auth_user.user_id)
        .bind(Utc::now())
        .bind(&w.id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        // Audit log for re-linking
        let _ = log_audit(
            &state.db,
            AuditAction::LinkWallet,
            auth_user.user_id,
            None,
            &format!("Re-linked wallet {}", address_str),
        )
        .await;

        return Ok(Json(
            serde_json::json!({ "message": "Wallet re-linked successfully" }),
        ));
    }

    // Link new wallet
    let _ =
        sqlx::query("INSERT INTO wallets (id, user_id, address, verified_at) VALUES (?, ?, ?, ?)")
            .bind(Uuid::new_v4().to_string())
            .bind(auth_user.user_id)
            .bind(&address_str)
            .bind(Utc::now())
            .execute(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Audit log
    let _ = log_audit(
        &state.db,
        AuditAction::LinkWallet,
        auth_user.user_id,
        None,
        &format!("Linked wallet {}", address_str),
    )
    .await;

    Ok(Json(
        serde_json::json!({ "message": "Wallet linked successfully" }),
    ))
}

#[utoipa::path(
    delete,
    path = "/api/wallets/{id}",
    params(
        ("id" = String, Path, description = "Wallet ID")
    ),
    responses(
        (status = 200, description = "Wallet unlinked"),
        (status = 404, description = "Wallet not found")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn unlink_wallet(
    Path(wallet_id): Path<String>,
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> impl IntoResponse {
    // First fetch the wallet to get the address for the audit log
    // Only search in active wallets for regular users
    let wallet = sqlx::query_as::<_, FlatLinkedWallet>(
        "SELECT *, NULL as tribe FROM wallets WHERE id = ? AND user_id = ? AND deleted_at IS NULL",
    )
    .bind(&wallet_id)
    .bind(auth_user.user_id)
    .fetch_optional(&state.db)
    .await;

    let wallet_address = match &wallet {
        Ok(Some(w)) => w.address.clone(),
        _ => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Wallet not found or not owned by user" })),
            )
                .into_response();
        }
    };

    // Also remove from user_tribes where verified by this wallet
    let _ = sqlx::query("UPDATE user_tribes SET wallet_id = NULL WHERE wallet_id = ?")
        .bind(&wallet_id)
        .execute(&state.db)
        .await;

    let result = sqlx::query("UPDATE wallets SET deleted_at = CURRENT_TIMESTAMP WHERE id = ? AND user_id = ? AND deleted_at IS NULL")
        .bind(&wallet_id)
        .bind(auth_user.user_id)
        .execute(&state.db)
        .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            // Audit log
            let _ = log_audit(
                &state.db,
                AuditAction::UnlinkWallet,
                auth_user.user_id,
                None,
                &format!("Unlinked wallet {}", wallet_address),
            )
            .await;
            Json(serde_json::json!({ "message": "Unlinked" })).into_response()
        }
        Ok(_) => Json(serde_json::json!({ "error": "Wallet not found or not owned by user" }))
            .into_response(),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })).into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    use std::collections::HashMap;
    use std::sync::Mutex;

    async fn setup_db() -> crate::db::DbPool {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create memory pool");

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Migrations failed");

        // Insert user
        sqlx::query("INSERT INTO users (id, discord_id, username, discriminator, is_admin) VALUES (?, ?, ?, ?, ?)")
            .bind(1001_i64)
            .bind("test-discord-id")
            .bind("TestUser")
            .bind("0001")
            .bind(false)
            .execute(&pool)
            .await
            .unwrap();

        pool
    }

    #[test]
    fn test_nonce_request_deserialization() {
        let json = r#"{"address":"0x1234567890abcdef1234567890abcdef12345678"}"#;
        let request: NonceRequest = serde_json::from_str(json).expect("Deserialize failed");
        assert_eq!(
            request.address,
            "0x1234567890abcdef1234567890abcdef12345678"
        );
    }

    #[test]
    fn test_nonce_response_serialization() {
        let response = NonceResponse {
            nonce: "test-nonce-uuid".to_string(),
        };

        let json = serde_json::to_string(&response).expect("Serialize failed");
        assert!(json.contains("\"nonce\":\"test-nonce-uuid\""));
    }

    #[test]
    fn test_verify_request_deserialization() {
        let json = r#"{"address":"0xabcdef","signature":"base64signature=="}"#;
        let request: VerifyRequest = serde_json::from_str(json).expect("Deserialize failed");
        assert_eq!(request.address, "0xabcdef");
        assert_eq!(request.signature, "base64signature==");
    }

    #[test]
    fn test_nonce_generation_uniqueness() {
        let nonce1 = Uuid::new_v4().to_string();
        let nonce2 = Uuid::new_v4().to_string();
        assert_ne!(nonce1, nonce2);
    }

    #[test]
    fn test_address_lowercase_normalization() {
        let address = "0xABCDEF1234567890ABCDEF1234567890ABCDEF12";
        let normalized = address.to_lowercase();
        assert_eq!(normalized, "0xabcdef1234567890abcdef1234567890abcdef12");
    }

    #[test]
    fn test_nonce_storage_and_retrieval() {
        let nonces: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
        let address = "0xtest".to_lowercase();
        let nonce = Uuid::new_v4().to_string();

        // Store
        {
            let mut map = nonces.lock().unwrap();
            map.insert(address.clone(), nonce.clone());
        }

        // Retrieve and remove
        {
            let mut map = nonces.lock().unwrap();
            let stored = map.remove(&address);
            assert!(stored.is_some());
            assert_eq!(stored.unwrap(), nonce);
        }

        // Verify it's gone
        {
            let map = nonces.lock().unwrap();
            assert!(map.get(&address).is_none());
        }
    }

    #[tokio::test]
    async fn test_wallet_insert_and_query() {
        let db = setup_db().await;
        let wallet_id = Uuid::new_v4().to_string();
        let address = "0xtest1234567890abcdef1234567890abcdef1234";
        let now = Utc::now();

        // Insert wallet
        sqlx::query("INSERT INTO wallets (id, user_id, address, verified_at) VALUES (?, ?, ?, ?)")
            .bind(&wallet_id)
            .bind(1001_i64)
            .bind(address)
            .bind(now)
            .execute(&db)
            .await
            .unwrap();

        // Query wallet
        let wallet: Option<crate::models::FlatLinkedWallet> =
            sqlx::query_as("SELECT *, NULL as tribe FROM wallets WHERE address = ?")
                .bind(address)
                .fetch_optional(&db)
                .await
                .unwrap();

        assert!(wallet.is_some());
        let wallet = wallet.unwrap();
        assert_eq!(wallet.address, address);
        assert_eq!(wallet.user_id, 1001);
    }

    #[tokio::test]
    async fn test_wallet_duplicate_address_rejected() {
        let db = setup_db().await;
        let address = "0xduplicate12345678901234567890123456789012";
        let now = Utc::now();

        // Insert first wallet
        sqlx::query("INSERT INTO wallets (id, user_id, address, verified_at) VALUES (?, ?, ?, ?)")
            .bind(Uuid::new_v4().to_string())
            .bind(1001_i64)
            .bind(address)
            .bind(now)
            .execute(&db)
            .await
            .unwrap();

        // Try to insert duplicate - should fail due to UNIQUE constraint
        let result = sqlx::query(
            "INSERT INTO wallets (id, user_id, address, verified_at) VALUES (?, ?, ?, ?)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(1001_i64)
        .bind(address)
        .bind(now)
        .execute(&db)
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_wallet_ownership_check() {
        let db = setup_db().await;
        let wallet_id = Uuid::new_v4().to_string();

        sqlx::query("INSERT INTO wallets (id, user_id, address, verified_at) VALUES (?, ?, ?, ?)")
            .bind(&wallet_id)
            .bind(1001_i64)
            .bind("0xownedwallet")
            .bind(Utc::now())
            .execute(&db)
            .await
            .unwrap();

        // Check ownership with correct user
        let wallet: Option<crate::models::FlatLinkedWallet> =
            sqlx::query_as("SELECT *, NULL as tribe FROM wallets WHERE id = ? AND user_id = ?")
                .bind(&wallet_id)
                .bind(1001_i64)
                .fetch_optional(&db)
                .await
                .unwrap();

        assert!(wallet.is_some());

        // Check with wrong user
        let wallet_wrong_user: Option<crate::models::FlatLinkedWallet> =
            sqlx::query_as("SELECT *, NULL as tribe FROM wallets WHERE id = ? AND user_id = ?")
                .bind(&wallet_id)
                .bind(9999_i64)
                .fetch_optional(&db)
                .await
                .unwrap();

        assert!(wallet_wrong_user.is_none());
    }

    #[tokio::test]
    async fn test_wallet_deletion() {
        let db = setup_db().await;
        let wallet_id = Uuid::new_v4().to_string();

        sqlx::query("INSERT INTO wallets (id, user_id, address, verified_at) VALUES (?, ?, ?, ?)")
            .bind(&wallet_id)
            .bind(1001_i64)
            .bind("0xtobedeleted")
            .bind(Utc::now())
            .execute(&db)
            .await
            .unwrap();

        // Delete
        let result = sqlx::query("DELETE FROM wallets WHERE id = ? AND user_id = ?")
            .bind(&wallet_id)
            .bind(1001_i64)
            .execute(&db)
            .await
            .unwrap();

        assert_eq!(result.rows_affected(), 1);

        // Verify deleted
        let wallet: Option<crate::models::FlatLinkedWallet> =
            sqlx::query_as("SELECT *, NULL as tribe FROM wallets WHERE id = ?")
                .bind(&wallet_id)
                .fetch_optional(&db)
                .await
                .unwrap();

        assert!(wallet.is_none());
    }
}
