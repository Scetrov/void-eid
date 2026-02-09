use crate::auth::Claims;
use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts, StatusCode},
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::env;

pub struct RequireSuperAdmin {
    pub discord_id: String,
}

impl<S> FromRequestParts<S> for RequireSuperAdmin
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 1. Extract Bearer Token
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "Missing Auth Header"))?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or((StatusCode::UNAUTHORIZED, "Invalid Auth Header"))?;

        // 2. Decode Token
        let secret = env::var("JWT_SECRET")
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "JWT Config Error"))?;

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid Token"))?;

        let discord_id = token_data.claims.discord_id;

        // 3. Strict Check against Environment Variable
        let super_admin_ids_str = env::var("SUPER_ADMIN_DISCORD_IDS").unwrap_or_default();
        let super_admin_ids: Vec<&str> = super_admin_ids_str.split(',').map(|s| s.trim()).collect();

        if super_admin_ids.contains(&discord_id.as_str()) {
            Ok(RequireSuperAdmin { discord_id })
        } else {
            Err((StatusCode::FORBIDDEN, "Not a Super Admin"))
        }
    }
}
