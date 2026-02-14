use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Router,
};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::Deserialize;
use sqlx::SqlitePool;
use std::{env, net::SocketAddr};
use tower_http::cors::CorsLayer;
use uuid::Uuid;
use void_eid_backend::{auth::Claims, db::init_db, state::AppState, wallet};

#[derive(Deserialize)]
struct StubLoginParams {
    user_id: i64,
}

async fn stub_login(
    Query(params): Query<StubLoginParams>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET missing");
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    // We can fetch the user to get real username if we want, but for stub we might just trust the ID
    // or fetch it. Let's fetch it to be correct and populate claims correctly.
    // Actually, to avoid async db call complexity here if unnecessary, we can hardcode claims based on ID
    // or just fetch it. Fetching is better.

    // For now, let's assume we seeded the DB and the ID exists.
    // Getting the user details would require importing User model and doing a query.
    // Let's do it properly.

    let user =
        sqlx::query_as::<_, void_eid_backend::models::User>("SELECT * FROM users WHERE id = ?")
            .bind(params.user_id)
            .fetch_one(&_state.db)
            .await
            .expect("User not found in stub DB");

    let claims = Claims {
        id: user.id.to_string(), // Serialize i64 ID to string for JWT
        discord_id: user.discord_id,
        username: user.username,
        exp: expiration,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .expect("Token generation failed");

    // Generate auth code and store JWT temporarily (same as real auth flow)
    let auth_code = Uuid::new_v4().to_string();
    _state
        .auth_codes
        .lock()
        .unwrap()
        .insert(auth_code.clone(), (token, Utc::now()));

    let frontend_url =
        env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());

    // Redirect to the same callback as real auth
    Redirect::to(&format!(
        "{}/auth/callback?code={}",
        frontend_url, auth_code
    ))
}

async fn seed_db(pool: &SqlitePool) {
    let now = Utc::now();

    // 1. Admin User
    let admin_id = 1001_i64;
    sqlx::query("INSERT INTO users (id, discord_id, username, discriminator, avatar, is_admin, last_login_at) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(admin_id)
        .bind("admin-discord-id")
        .bind("AdminUser")
        .bind("0001")
        .bind("avatar_url")
        .bind(true)
        .bind(now)
        .execute(pool)
        .await
        .expect("Failed to insert admin");

    // 2. Regular User
    let user_id = 1002_i64;
    sqlx::query("INSERT INTO users (id, discord_id, username, discriminator, avatar, is_admin, last_login_at) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(user_id)
        .bind("regular-discord-id")
        .bind("RegularUser")
        .bind("0002")
        .bind("avatar_url")
        .bind(false)
        .bind(now)
        .execute(pool)
        .await
        .expect("Failed to insert user");

    // 3. Regular User Wallet
    let regular_wallet_id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO wallets (id, user_id, address, verified_at) VALUES (?, ?, ?, ?)")
        .bind(&regular_wallet_id)
        .bind(user_id)
        .bind("0xregularwallet987654321")
        .bind(now)
        .execute(pool)
        .await
        .expect("Failed to insert regular wallet");

    // 4. Tribe for Regular User Wallet
    sqlx::query(
        "INSERT INTO user_tribes (user_id, wallet_id, tribe, is_admin) VALUES (?, ?, ?, ?)",
    )
    .bind(user_id)
    .bind(&regular_wallet_id)
    .bind("void_tribe")
    .bind(false)
    .execute(pool)
    .await
    .expect("Failed to insert regular user tribe");

    // 5. Admin Wallet
    let wallet_id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO wallets (id, user_id, address, verified_at) VALUES (?, ?, ?, ?)")
        .bind(&wallet_id)
        .bind(admin_id)
        .bind("0xadminwallet123456789")
        .bind(now)
        .execute(pool)
        .await
        .expect("Failed to insert wallet");

    // 6. Tribe for Admin Wallet
    sqlx::query(
        "INSERT INTO user_tribes (user_id, wallet_id, tribe, is_admin) VALUES (?, ?, ?, ?)",
    )
    .bind(admin_id)
    .bind(&wallet_id)
    .bind("void_tribe")
    .bind(true)
    .execute(pool)
    .await
    .expect("Failed to insert tribe");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    // Force in-memory DB for stub
    env::set_var("DATABASE_URL", "sqlite::memory:");

    // Ensure critical env vars are set if not present (for default behavior)
    if env::var("JWT_SECRET").is_err() {
        env::set_var("JWT_SECRET", "stub-jwt-secret");
    }
    if env::var("FRONTEND_URL").is_err() {
        env::set_var("FRONTEND_URL", "http://localhost:5173");
    }

    let db_pool = init_db().await?;
    seed_db(&db_pool).await;

    let state = AppState::new(db_pool);

    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any) // Be permissive for stub
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let app = Router::new()
        // Stub Auth Route
        .route("/api/auth/stub-login", get(stub_login))
        .route(
            "/api/auth/exchange",
            post(void_eid_backend::auth::exchange_code),
        )
        // Wallet routes (not rate-limited in stub)
        .route("/api/wallets/link-nonce", post(wallet::link_nonce))
        .route("/api/wallets/link-verify", post(wallet::link_verify))
        // Mock the original login route to redirect to stub login?
        // Or just let the frontend call stub-login directly if in test mode.
        // Let's redirect /api/auth/discord/login to a page that auto-logs in as admin for convenience?
        // Or better, let the manual usage via Playwright hit the stub-login endpoint.
        .route(
            "/api/auth/discord/login",
            get(|| async { "Use /api/auth/stub-login?user_id=1001 for testing" }),
        )
        // Add /docs endpoint for ApiGuard health check
        .route("/docs", get(|| async { "Stub API OK" }))
        .merge(void_eid_backend::get_common_router())
        .layer(cors)
        .with_state(state);

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "5038".to_string())
        .parse::<u16>()?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!("Stub API Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
