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

    // Check availability
    let existing: Option<FlatLinkedWallet> =
        sqlx::query_as("SELECT * FROM wallets WHERE address = ?")
            .bind(&address_str)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if existing.is_some() {
        return Err((StatusCode::BAD_REQUEST, "Wallet already linked".into()));
    }

    // Link
    let _ =
        sqlx::query("INSERT INTO wallets (id, user_id, address, verified_at) VALUES (?, ?, ?, ?)")
            .bind(Uuid::new_v4().to_string())
            .bind(&auth_user.user_id)
            .bind(&address_str)
            .bind(Utc::now())
            .execute(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Audit log
    let _ = log_audit(
        &state.db,
        AuditAction::LinkWallet,
        &auth_user.user_id,
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
    let wallet =
        sqlx::query_as::<_, FlatLinkedWallet>("SELECT * FROM wallets WHERE id = ? AND user_id = ?")
            .bind(&wallet_id)
            .bind(&auth_user.user_id)
            .fetch_optional(&state.db)
            .await;

    let wallet_address = match &wallet {
        Ok(Some(w)) => w.address.clone(),
        _ => "unknown".to_string(),
    };

    let result = sqlx::query("DELETE FROM wallets WHERE id = ? AND user_id = ?")
        .bind(&wallet_id)
        .bind(&auth_user.user_id)
        .execute(&state.db)
        .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            // Audit log
            let _ = log_audit(
                &state.db,
                AuditAction::UnlinkWallet,
                &auth_user.user_id,
                None,
                &format!("Unlinked wallet {}", wallet_address),
            )
            .await;
            Json(serde_json::json!({ "message": "Unlinked" }))
        }
        Ok(_) => Json(serde_json::json!({ "error": "Wallet not found or not owned by user" })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}
