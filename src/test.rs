use super::*;
use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, header};
use tower::ServiceExt;

fn required_env(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| {
        panic!(
            "missing required env var {key}; set TESTNET_PAYMENT_SIGNATURE_DESIGN, X402_PAY_TO, and X402_ASSET for live testnet x402 tests"
        )
    })
}

fn optional_env(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn test_app() -> (Router, SharedState) {
    let state = SharedState {
        inner: Arc::new(RwLock::new(AppState::new())),
    };
    (build_app(state.clone()), state)
}

async fn post_json(
    app: &Router,
    uri: &str,
    body: serde_json::Value,
    payment_signature: Option<&str>,
) -> axum::response::Response {
    let mut builder = Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");

    if let Some(signature) = payment_signature {
        builder = builder.header(PAYMENT_SIGNATURE_HEADER, signature);
    }

    app.clone()
        .oneshot(
            builder
                .body(Body::from(body.to_string()))
                .expect("request should build"),
        )
        .await
        .expect("router should handle request")
}

async fn read_json(response: axum::response::Response) -> serde_json::Value {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    serde_json::from_slice::<serde_json::Value>(&bytes).expect("response should be valid json")
}

async fn configure_local_x402(state: &SharedState) {
    let mut locked = state.inner.write().await;
    locked.config.x402_facilitator_url = "https://x402.org/facilitator".to_string();
    locked.config.x402_verify_path = "/verify".to_string();
    locked.config.x402_settle_path = "/settle".to_string();
    locked.config.x402_network = "base-sepolia".to_string();
    locked.config.x402_pay_to = Some("0x1111111111111111111111111111111111111111".to_string());
    locked.config.x402_asset = Some("0x2222222222222222222222222222222222222222".to_string());
    locked.config.public_base_url = "http://localhost:3000".to_string();
}

async fn configure_live_x402_from_env(state: &SharedState) {
    let mut locked = state.inner.write().await;
    locked.config.x402_facilitator_url =
        optional_env("X402_FACILITATOR_URL", "https://x402.org/facilitator");
    locked.config.x402_verify_path = optional_env("X402_VERIFY_PATH", "/verify");
    locked.config.x402_settle_path = optional_env("X402_SETTLE_PATH", "/settle");
    locked.config.x402_network = optional_env("X402_NETWORK", "base-sepolia");
    locked.config.x402_pay_to = Some(required_env("X402_PAY_TO"));
    locked.config.x402_asset = Some(required_env("X402_ASSET"));
    locked.config.public_base_url = optional_env("PUBLIC_BASE_URL", "http://localhost:3000");
    locked.config.x402_facilitator_bearer_token =
        std::env::var("X402_FACILITATOR_BEARER_TOKEN").ok();
}

#[tokio::test]
async fn testnet_tool_requires_payment_signature_challenge() {
    let (app, state) = test_app();
    configure_local_x402(&state).await;

    let response = post_json(
        &app,
        "/tool/design/run",
        serde_json::json!({
            "user_id": Uuid::new_v4(),
            "input": "test payload"
        }),
        None,
    )
    .await;

    assert_eq!(response.status(), StatusCode::PAYMENT_REQUIRED);
    assert!(response.headers().contains_key(PAYMENT_REQUIRED_HEADER));
    assert!(response.headers().contains_key(X402_VERSION_HEADER));

    let json = read_json(response).await;
    assert_eq!(json["accepted_header"], PAYMENT_SIGNATURE_HEADER);
    assert_eq!(json["service"], "design");
}

#[tokio::test]
async fn testnet_invalid_payment_signature_rejected() {
    let (app, state) = test_app();
    configure_local_x402(&state).await;

    let response = post_json(
        &app,
        "/tool/design/run",
        serde_json::json!({
            "user_id": Uuid::new_v4(),
            "input": "test payload"
        }),
        Some("not-base64"),
    )
    .await;

    assert_eq!(response.status(), StatusCode::PAYMENT_REQUIRED);
    assert!(response.headers().contains_key(PAYMENT_REQUIRED_HEADER));
    let json = read_json(response).await;
    assert!(
        json["message"]
            .as_str()
            .unwrap_or_default()
            .contains("payment rejected")
    );
}

#[tokio::test]
async fn testnet_payment_signature_unlocks_tool() {
    let (app, state) = test_app();
    configure_live_x402_from_env(&state).await;
    let signature = required_env("TESTNET_PAYMENT_SIGNATURE_DESIGN");

    let response = post_json(
        &app,
        "/tool/design/run",
        serde_json::json!({
            "user_id": Uuid::new_v4(),
            "input": "testnet paid run"
        }),
        Some(signature.as_str()),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().contains_key(PAYMENT_RESPONSE_HEADER));
    let json = read_json(response).await;
    assert_eq!(json["payment_mode"], "user_direct");
    assert_eq!(json["service"], "design");
}

#[test]
fn migration_0007_consents_has_expected_schema() {
    let sql = include_str!("../migrations/0007_consents.sql");
    let lower = sql.to_lowercase();

    // Table creation
    assert!(lower.contains("create table if not exists consents"), "should create consents table");

    // Required columns
    assert!(lower.contains("id uuid primary key"), "should have id uuid primary key");
    assert!(lower.contains("user_id uuid not null references users(id) on delete cascade"), "should have user_id FK to users");
    assert!(lower.contains("campaign_id uuid not null references campaigns(id) on delete cascade"), "should have campaign_id FK to campaigns");
    assert!(lower.contains("consent_type text not null"), "should have consent_type column");
    assert!(lower.contains("granted boolean not null"), "should have granted column");
    assert!(lower.contains("purpose text"), "should have purpose column");
    assert!(lower.contains("retention_days integer"), "should have retention_days column");
    assert!(lower.contains("created_at timestamptz not null default now()"), "should have created_at column");

    // CHECK constraint on consent_type
    assert!(lower.contains("data_sharing"), "consent_type should include data_sharing");
    assert!(lower.contains("contact"), "consent_type should include contact");
    assert!(lower.contains("retention"), "consent_type should include retention");

    // Indexes
    assert!(lower.contains("consents_user_campaign_idx"), "should have user_campaign composite index");
    assert!(lower.contains("on consents(user_id, campaign_id)"), "composite index should be on (user_id, campaign_id)");
    assert!(lower.contains("consents_user_id_idx"), "should have user_id index");
}

#[test]
fn migration_0008_add_user_source_has_expected_schema() {
    let sql = include_str!("../migrations/0008_add_user_source.sql");
    let lower = sql.to_lowercase();

    // ALTER TABLE to add source column
    assert!(lower.contains("alter table"), "should use ALTER TABLE");
    assert!(lower.contains("users"), "should target users table");
    assert!(lower.contains("add column"), "should add a column");
    assert!(lower.contains("source"), "should add source column");
    assert!(lower.contains("text"), "source should be TEXT type");
    assert!(lower.contains("default 'web'"), "source should default to 'web'");

    // Safety: IF NOT EXISTS for idempotent migration
    assert!(lower.contains("if not exists"), "should use IF NOT EXISTS for safety");
}

#[test]
fn migration_0009_gpt_sessions_has_expected_schema() {
    let sql = include_str!("../migrations/0009_gpt_sessions.sql");
    let lower = sql.to_lowercase();

    // Table creation
    assert!(lower.contains("create table if not exists gpt_sessions"), "should create gpt_sessions table");

    // Required columns
    assert!(lower.contains("token uuid primary key"), "should have token uuid primary key");
    assert!(lower.contains("gen_random_uuid()"), "token should default to gen_random_uuid()");
    assert!(lower.contains("user_id uuid not null references users(id) on delete cascade"), "should have user_id FK to users");
    assert!(lower.contains("created_at timestamptz not null default now()"), "should have created_at column");
    assert!(lower.contains("expires_at timestamptz not null"), "should have expires_at column");
    assert!(lower.contains("interval '30 days'"), "expires_at should default to NOW() + 30 days");

    // Indexes
    assert!(lower.contains("gpt_sessions_user_id_idx"), "should have user_id index");
    assert!(lower.contains("on gpt_sessions(user_id)"), "user_id index should target correct column");
    assert!(lower.contains("gpt_sessions_expires_at_idx"), "should have expires_at index");
    assert!(lower.contains("on gpt_sessions(expires_at)"), "expires_at index should target correct column");
}

#[tokio::test]
async fn testnet_payment_signature_service_mismatch_is_rejected() {
    let (app, state) = test_app();
    configure_live_x402_from_env(&state).await;
    let signature = required_env("TESTNET_PAYMENT_SIGNATURE_DESIGN");

    let response = post_json(
        &app,
        "/tool/storage/run",
        serde_json::json!({
            "user_id": Uuid::new_v4(),
            "input": "testnet paid run"
        }),
        Some(signature.as_str()),
    )
    .await;

    assert_eq!(response.status(), StatusCode::PAYMENT_REQUIRED);
    let json = read_json(response).await;
    assert!(
        json["message"]
            .as_str()
            .unwrap_or_default()
            .contains("payment rejected")
    );
}
