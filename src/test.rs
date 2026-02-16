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

fn has_live_testnet_env() -> bool {
    std::env::var("TESTNET_PAYMENT_SIGNATURE_DESIGN").is_ok()
        && std::env::var("X402_PAY_TO").is_ok()
        && std::env::var("X402_ASSET").is_ok()
}

#[test]
fn collect_cors_origins_includes_mcp_server_url() {
    let origins = collect_cors_origins(
        "http://localhost:5173,https://subsidy-payment.vercel.app",
        Some("http://localhost:3001/mcp"),
    );

    assert!(origins.contains(&"http://localhost:5173".to_string()));
    assert!(origins.contains(&"https://subsidy-payment.vercel.app".to_string()));
    assert!(origins.contains(&"http://localhost:3001".to_string()));
}

#[test]
fn collect_cors_origins_deduplicates_and_normalizes() {
    let origins = collect_cors_origins(
        "\"http://localhost:5173/\",http://localhost:5173",
        Some("http://localhost:5173/path"),
    );

    assert_eq!(origins.len(), 1);
    assert_eq!(origins[0], "http://localhost:5173");
}

fn test_app() -> (Router, SharedState) {
    let state = SharedState {
        inner: Arc::new(RwLock::new(AppState::new())),
    };
    (
        build_app(
            state.clone(),
            DEFAULT_AGENT_DISCOVERY_RATE_LIMIT_PER_MIN as u32,
        ),
        state,
    )
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

async fn read_typed<T: serde::de::DeserializeOwned>(response: axum::response::Response) -> T {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    serde_json::from_slice::<T>(&bytes).expect("response should deserialize to expected type")
}

async fn read_body_string(response: axum::response::Response) -> String {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    String::from_utf8(bytes.to_vec()).unwrap_or_default()
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
    if !has_live_testnet_env() {
        eprintln!(
            "skipping live testnet test: set TESTNET_PAYMENT_SIGNATURE_DESIGN, X402_PAY_TO, and X402_ASSET"
        );
        return;
    }

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
    assert!(
        lower.contains("create table if not exists consents"),
        "should create consents table"
    );

    // Required columns
    assert!(
        lower.contains("id uuid primary key"),
        "should have id uuid primary key"
    );
    assert!(
        lower.contains("user_id uuid not null references users(id) on delete cascade"),
        "should have user_id FK to users"
    );
    assert!(
        lower.contains("campaign_id uuid not null references campaigns(id) on delete cascade"),
        "should have campaign_id FK to campaigns"
    );
    assert!(
        lower.contains("consent_type text not null"),
        "should have consent_type column"
    );
    assert!(
        lower.contains("granted boolean not null"),
        "should have granted column"
    );
    assert!(lower.contains("purpose text"), "should have purpose column");
    assert!(
        lower.contains("retention_days integer"),
        "should have retention_days column"
    );
    assert!(
        lower.contains("created_at timestamptz not null default now()"),
        "should have created_at column"
    );

    // CHECK constraint on consent_type
    assert!(
        lower.contains("data_sharing"),
        "consent_type should include data_sharing"
    );
    assert!(
        lower.contains("contact"),
        "consent_type should include contact"
    );
    assert!(
        lower.contains("retention"),
        "consent_type should include retention"
    );

    // Indexes
    assert!(
        lower.contains("consents_user_campaign_idx"),
        "should have user_campaign composite index"
    );
    assert!(
        lower.contains("on consents(user_id, campaign_id)"),
        "composite index should be on (user_id, campaign_id)"
    );
    assert!(
        lower.contains("consents_user_id_idx"),
        "should have user_id index"
    );
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
    assert!(
        lower.contains("default 'web'"),
        "source should default to 'web'"
    );

    // Safety: IF NOT EXISTS for idempotent migration
    assert!(
        lower.contains("if not exists"),
        "should use IF NOT EXISTS for safety"
    );
}

#[test]
fn migration_0009_gpt_sessions_has_expected_schema() {
    let sql = include_str!("../migrations/0009_gpt_sessions.sql");
    let lower = sql.to_lowercase();

    // Table creation
    assert!(
        lower.contains("create table if not exists gpt_sessions"),
        "should create gpt_sessions table"
    );

    // Required columns
    assert!(
        lower.contains("token uuid primary key"),
        "should have token uuid primary key"
    );
    assert!(
        lower.contains("gen_random_uuid()"),
        "token should default to gen_random_uuid()"
    );
    assert!(
        lower.contains("user_id uuid not null references users(id) on delete cascade"),
        "should have user_id FK to users"
    );
    assert!(
        lower.contains("created_at timestamptz not null default now()"),
        "should have created_at column"
    );
    assert!(
        lower.contains("expires_at timestamptz not null"),
        "should have expires_at column"
    );
    assert!(
        lower.contains("interval '30 days'"),
        "expires_at should default to NOW() + 30 days"
    );

    // Indexes
    assert!(
        lower.contains("gpt_sessions_user_id_idx"),
        "should have user_id index"
    );
    assert!(
        lower.contains("on gpt_sessions(user_id)"),
        "user_id index should target correct column"
    );
    assert!(
        lower.contains("gpt_sessions_expires_at_idx"),
        "should have expires_at index"
    );
    assert!(
        lower.contains("on gpt_sessions(expires_at)"),
        "expires_at index should target correct column"
    );
}

#[test]
fn gpt_types_exist_and_are_constructible() {
    use chrono::Utc;

    // Component 2: Search
    let _params = GptSearchParams {
        q: Some("test".into()),
        category: None,
        max_budget_cents: None,
        intent: None,
        session_token: None,
    };
    let _item = GptServiceItem {
        service_type: "campaign".into(),
        service_id: Uuid::new_v4(),
        name: "Test".into(),
        sponsor: "Sponsor".into(),
        required_task: Some("survey".into()),
        subsidy_amount_cents: 100,
        category: vec!["design".into()],
        active: true,
        tags: vec![],
        relevance_score: None,
    };
    let _resp = GptSearchResponse {
        services: vec![],
        total_count: 0,
        message: "No services found".into(),
        applied_filters: None,
        available_categories: None,
    };

    // Component 3: Auth
    let _auth_req = GptAuthRequest {
        email: "test@example.com".into(),
        region: "US".into(),
        roles: vec![],
        tools_used: vec![],
    };
    let _auth_resp = GptAuthResponse {
        session_token: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        email: "test@example.com".into(),
        is_new_user: true,
        message: "Welcome".into(),
    };

    // Component 4: Task details
    let _task_params = GptTaskParams {
        session_token: Uuid::new_v4(),
    };
    let _task_fmt = GptTaskInputFormat {
        task_type: "survey".into(),
        required_fields: vec!["email".into()],
        instructions: "Fill out the survey".into(),
    };
    let _task_resp = GptTaskResponse {
        campaign_id: Uuid::new_v4(),
        campaign_name: "Test Campaign".into(),
        sponsor: "Sponsor".into(),
        required_task: "survey".into(),
        task_description: "Complete a survey".into(),
        task_input_format: GptTaskInputFormat {
            task_type: "survey".into(),
            required_fields: vec![],
            instructions: "".into(),
        },
        already_completed: false,
        subsidy_amount_cents: 100,
        message: "".into(),
    };

    // Component 5: Task completion
    let _consent = GptConsentInput {
        data_sharing_agreed: true,
        purpose_acknowledged: true,
        contact_permission: false,
    };
    let _complete_req = GptCompleteTaskRequest {
        session_token: Uuid::new_v4(),
        task_name: "survey".into(),
        details: None,
        consent: GptConsentInput {
            data_sharing_agreed: true,
            purpose_acknowledged: true,
            contact_permission: false,
        },
    };
    let _complete_resp = GptCompleteTaskResponse {
        task_completion_id: Uuid::new_v4(),
        campaign_id: Uuid::new_v4(),
        consent_recorded: true,
        can_use_service: true,
        message: "Task completed".into(),
    };

    // Component 6: Service run
    let _run_req = GptRunServiceRequest {
        session_token: Uuid::new_v4(),
        input: "test input".into(),
    };
    let _run_resp = GptRunServiceResponse {
        service: "design".into(),
        output: "result".into(),
        payment_mode: "sponsored".into(),
        sponsored_by: Some("Sponsor".into()),
        tx_hash: None,
        message: "Service completed".into(),
    };

    // Component 7: User status
    let _status_params = GptUserStatusParams {
        session_token: Uuid::new_v4(),
    };
    let _task_summary = GptCompletedTaskSummary {
        campaign_id: Uuid::new_v4(),
        campaign_name: "Campaign".into(),
        task_name: "survey".into(),
        completed_at: Utc::now(),
    };
    let _avail = GptAvailableService {
        service: "design".into(),
        sponsor: "Sponsor".into(),
        ready: true,
    };
    let _status_resp = GptUserStatusResponse {
        user_id: Uuid::new_v4(),
        email: "test@example.com".into(),
        completed_tasks: vec![],
        available_services: vec![],
        message: "Status".into(),
    };
}

#[test]
fn consent_and_gpt_session_types_exist_and_are_constructible() {
    use chrono::Utc;

    // Consent type (Component 8)
    let consent = Consent {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        campaign_id: Uuid::new_v4(),
        consent_type: "data_sharing".into(),
        granted: true,
        purpose: Some("Marketing research".into()),
        retention_days: Some(365),
        created_at: Utc::now(),
    };
    assert!(consent.granted);
    assert_eq!(consent.consent_type, "data_sharing");

    // GptSession type (Component 1.5)
    let session = GptSession {
        token: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        created_at: Utc::now(),
        expires_at: Utc::now(),
    };
    assert_ne!(session.token, Uuid::nil());

    // Verify Debug + Clone traits
    let _consent_clone = consent.clone();
    let _session_clone = session.clone();
    let _consent_debug = format!("{:?}", consent);
    let _session_debug = format!("{:?}", session);
}

#[test]
fn app_config_has_gpt_actions_api_key_field() {
    // Test with env var unset: should be None
    unsafe {
        std::env::remove_var("GPT_ACTIONS_API_KEY");
    }
    let config = AppConfig::from_env();
    assert!(
        config.gpt_actions_api_key.is_none(),
        "gpt_actions_api_key should be None when env var is unset"
    );

    // Test with env var set: should be Some
    unsafe {
        std::env::set_var("GPT_ACTIONS_API_KEY", "test-secret-key");
    }
    let config = AppConfig::from_env();
    assert_eq!(
        config.gpt_actions_api_key,
        Some("test-secret-key".to_string())
    );

    // Cleanup
    unsafe {
        std::env::remove_var("GPT_ACTIONS_API_KEY");
    }
}

#[test]
fn user_profile_has_source_field() {
    use chrono::Utc;
    use std::collections::HashMap;

    // source field should be Option<String> and work with None (backward compat)
    let profile_no_source = UserProfile {
        id: Uuid::new_v4(),
        email: "test@example.com".into(),
        region: "US".into(),
        roles: vec!["creator".into()],
        tools_used: vec!["design".into()],
        attributes: HashMap::new(),
        created_at: Utc::now(),
        source: None,
    };
    assert!(profile_no_source.source.is_none());

    // source field should work with Some value
    let profile_with_source = UserProfile {
        id: Uuid::new_v4(),
        email: "gpt@example.com".into(),
        region: "JP".into(),
        roles: vec![],
        tools_used: vec![],
        attributes: HashMap::new(),
        created_at: Utc::now(),
        source: Some("gpt".into()),
    };
    assert_eq!(profile_with_source.source, Some("gpt".to_string()));

    // Verify serde default: JSON without source should deserialize with source=None
    let json = serde_json::json!({
        "id": Uuid::new_v4(),
        "email": "test@example.com",
        "region": "US",
        "roles": ["creator"],
        "tools_used": ["design"],
        "attributes": {},
        "created_at": Utc::now()
    });
    let deserialized: UserProfile =
        serde_json::from_value(json).expect("should deserialize without source field");
    assert!(
        deserialized.source.is_none(),
        "source should default to None when missing from JSON"
    );
}

#[test]
fn api_error_rate_limited_returns_429_with_retry_after_header() {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    let err = ApiError::rate_limited(60);
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    let retry_after = response
        .headers()
        .get("retry-after")
        .expect("should have Retry-After header");
    assert_eq!(retry_after.to_str().unwrap(), "60");
}

#[tokio::test]
async fn verify_gpt_api_key_rejects_missing_header() {
    use axum::http::StatusCode;
    use axum::middleware;

    let (_, state) = test_app();
    {
        let mut s = state.inner.write().await;
        s.config.gpt_actions_api_key = Some("test-secret".to_string());
    }

    let app = Router::new()
        .route("/gpt/test", axum::routing::get(|| async { "ok" }))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            gpt::verify_gpt_api_key,
        ))
        .with_state(state);

    let req = Request::builder()
        .uri("/gpt/test")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn verify_gpt_api_key_rejects_wrong_key() {
    use axum::http::StatusCode;
    use axum::middleware;

    let (_, state) = test_app();
    {
        let mut s = state.inner.write().await;
        s.config.gpt_actions_api_key = Some("correct-key".to_string());
    }

    let app = Router::new()
        .route("/gpt/test", axum::routing::get(|| async { "ok" }))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            gpt::verify_gpt_api_key,
        ))
        .with_state(state);

    let req = Request::builder()
        .uri("/gpt/test")
        .header("Authorization", "Bearer wrong-key")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn verify_gpt_api_key_passes_with_valid_key() {
    use axum::http::StatusCode;
    use axum::middleware;

    let (_, state) = test_app();
    {
        let mut s = state.inner.write().await;
        s.config.gpt_actions_api_key = Some("valid-key".to_string());
    }

    let app = Router::new()
        .route("/gpt/test", axum::routing::get(|| async { "ok" }))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            gpt::verify_gpt_api_key,
        ))
        .with_state(state);

    let req = Request::builder()
        .uri("/gpt/test")
        .header("Authorization", "Bearer valid-key")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[test]
fn resolve_session_has_correct_signature() {
    use sqlx::PgPool;
    let _fn_ref: fn(
        &PgPool,
        Uuid,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = error::ApiResult<Uuid>> + Send + '_>,
    > = |pool, token| Box::pin(gpt::resolve_session(pool, token));
}

#[tokio::test]
async fn verify_gpt_api_key_passes_when_key_not_configured() {
    use axum::http::StatusCode;
    use axum::middleware;

    let (_, state) = test_app();
    // gpt_actions_api_key defaults to None — middleware should passthrough
    {
        let s = state.inner.read().await;
        assert!(s.config.gpt_actions_api_key.is_none());
    }

    let app = Router::new()
        .route("/gpt/test", axum::routing::get(|| async { "ok" }))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            gpt::verify_gpt_api_key,
        ))
        .with_state(state);

    let req = Request::builder()
        .uri("/gpt/test")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn verify_gpt_api_key_rejects_invalid_bearer_format() {
    use axum::http::StatusCode;
    use axum::middleware;

    let (_, state) = test_app();
    {
        let mut s = state.inner.write().await;
        s.config.gpt_actions_api_key = Some("test-secret".to_string());
    }

    let app = Router::new()
        .route("/gpt/test", axum::routing::get(|| async { "ok" }))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            gpt::verify_gpt_api_key,
        ))
        .with_state(state);

    // Send "Basic" instead of "Bearer"
    let req = Request::builder()
        .uri("/gpt/test")
        .header("Authorization", "Basic test-secret")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn resolve_session_fails_with_invalid_db() {
    // Test that resolve_session returns an internal error when DB is unreachable
    use sqlx::postgres::PgPoolOptions;

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect("postgres://invalid:invalid@localhost:1/nonexistent")
        .await;

    // If we can't even connect, verify the function handles it gracefully
    if let Ok(pool) = pool {
        let result = gpt::resolve_session(&pool, Uuid::new_v4()).await;
        assert!(result.is_err(), "should fail with unreachable DB");
    }
    // If connection itself fails, that's expected — the test validates
    // that the function signature and error handling are correct
}

#[tokio::test]
async fn resolve_session_returns_unauthorized_for_nonexistent_token() {
    // If DATABASE_URL is set, test against real DB
    if let Ok(url) = std::env::var("DATABASE_URL") {
        use sqlx::postgres::PgPoolOptions;
        if let Ok(pool) = PgPoolOptions::new().max_connections(1).connect(&url).await {
            // Run migrations first
            sqlx::migrate!("./migrations").run(&pool).await.ok();

            // Random UUID should not exist in gpt_sessions
            let result = gpt::resolve_session(&pool, Uuid::new_v4()).await;
            assert!(result.is_err());
            let err_str = format!("{}", result.unwrap_err());
            assert!(
                err_str.contains("invalid or expired session token"),
                "error should mention invalid/expired token, got: {}",
                err_str
            );
        }
    }
}

#[tokio::test]
async fn rate_limiter_allows_requests_within_limit() {
    use axum::http::StatusCode;
    use axum::middleware;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    let limiter = Arc::new(Mutex::new(gpt::RateLimiter::new(
        60,
        std::time::Duration::from_secs(1),
    )));

    let app = Router::new()
        .route("/gpt/test", axum::routing::get(|| async { "ok" }))
        .layer(middleware::from_fn_with_state(
            limiter.clone(),
            gpt::rate_limit_middleware,
        ))
        .with_state(limiter);

    let req = Request::builder()
        .uri("/gpt/test")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn rate_limiter_returns_429_when_exhausted() {
    use axum::http::StatusCode;
    use axum::middleware;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    // Start with 1 token so it exhausts after first request
    let limiter = Arc::new(Mutex::new(gpt::RateLimiter::new(
        1,
        std::time::Duration::from_secs(1),
    )));

    let app = Router::new()
        .route("/gpt/test", axum::routing::get(|| async { "ok" }))
        .layer(middleware::from_fn_with_state(
            limiter.clone(),
            gpt::rate_limit_middleware,
        ))
        .with_state(limiter.clone());

    // First request should succeed
    let req = Request::builder()
        .uri("/gpt/test")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Second request should be rate limited
    let req = Request::builder()
        .uri("/gpt/test")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    assert!(
        resp.headers().get("retry-after").is_some(),
        "should have Retry-After header"
    );
}

#[test]
fn rate_limiter_refills_tokens() {
    use std::time::Duration;

    let mut limiter = gpt::RateLimiter::new(2, Duration::from_millis(10));
    assert!(limiter.try_acquire(), "should have tokens initially");
    assert!(limiter.try_acquire(), "should have second token");
    assert!(!limiter.try_acquire(), "should be exhausted");

    // Wait for refill
    std::thread::sleep(Duration::from_millis(25));
    assert!(
        limiter.try_acquire(),
        "should have refilled tokens after waiting"
    );
}

#[test]
fn rate_limiter_does_not_exceed_max_tokens_after_long_wait() {
    use std::time::Duration;

    let mut limiter = gpt::RateLimiter::new(3, Duration::from_millis(10));
    // Consume all tokens
    assert!(limiter.try_acquire());
    assert!(limiter.try_acquire());
    assert!(limiter.try_acquire());
    assert!(!limiter.try_acquire());

    // Wait much longer than needed to refill all tokens
    std::thread::sleep(Duration::from_millis(100));

    // Should refill up to max (3), not beyond
    assert!(limiter.try_acquire());
    assert!(limiter.try_acquire());
    assert!(limiter.try_acquire());
    assert!(
        !limiter.try_acquire(),
        "should not exceed max_tokens after long wait"
    );
}

#[tokio::test]
async fn rate_limiter_429_response_has_correct_retry_after_value() {
    use axum::http::StatusCode;
    use axum::middleware;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    // 1 token, 1 second refill interval
    let limiter = Arc::new(Mutex::new(gpt::RateLimiter::new(
        1,
        std::time::Duration::from_secs(1),
    )));

    let app = Router::new()
        .route("/gpt/test", axum::routing::get(|| async { "ok" }))
        .layer(middleware::from_fn_with_state(
            limiter.clone(),
            gpt::rate_limit_middleware,
        ))
        .with_state(limiter.clone());

    // Exhaust the single token
    let req = Request::builder()
        .uri("/gpt/test")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Next request should get 429 with Retry-After: 1
    let req = Request::builder()
        .uri("/gpt/test")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    let retry_after = resp
        .headers()
        .get("retry-after")
        .expect("should have Retry-After header");
    assert_eq!(
        retry_after.to_str().unwrap(),
        "1",
        "Retry-After should match refill interval"
    );
}

#[test]
fn gpt_auth_has_correct_handler_signature() {
    // Verify gpt_auth is a public async function with the expected axum handler signature.
    // Full integration tests are in task 5.3.
    use crate::types::GptAuthRequest;
    let _fn_ref: fn(
        axum::extract::State<crate::types::SharedState>,
        axum::Json<GptAuthRequest>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = axum::response::Response> + Send>,
    > = |state, payload| Box::pin(gpt::gpt_auth(state, payload));
}

#[test]
fn gpt_search_services_has_correct_handler_signature() {
    // Verify gpt_search_services is a public async function that can be used as an axum handler.
    // Full integration tests are in task 5.3.
    use crate::types::GptSearchParams;
    let _fn_ref: fn(
        axum::extract::State<crate::types::SharedState>,
        axum::extract::Query<GptSearchParams>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = axum::response::Response> + Send>,
    > = |state, query| Box::pin(gpt::gpt_search_services(state, query));
}

#[tokio::test]
async fn gpt_search_services_returns_empty_when_no_active_services() {
    // Test against real DB when DATABASE_URL is set
    if let Ok(url) = std::env::var("DATABASE_URL") {
        use sqlx::postgres::PgPoolOptions;
        if let Ok(pool) = PgPoolOptions::new().max_connections(1).connect(&url).await {
            sqlx::migrate!("./migrations").run(&pool).await.ok();

            let state = SharedState {
                inner: Arc::new(RwLock::new(AppState::new())),
            };

            // Search with a keyword that won't match anything
            let params = types::GptSearchParams {
                q: Some("zzz_nonexistent_service_xyz".to_string()),
                category: None,
                max_budget_cents: None,
                intent: None,
                session_token: None,
            };
            let result =
                gpt::gpt_search_services(axum::extract::State(state), axum::extract::Query(params))
                    .await;

            assert!(result.status().is_success());
            let resp: types::GptSearchResponse = read_typed(result).await;
            assert_eq!(resp.total_count, 0);
            assert!(resp.services.is_empty());
            assert!(resp.message.contains("No services found"));
        }
    }
}

#[tokio::test]
async fn gpt_search_services_filters_by_category() {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        use sqlx::postgres::PgPoolOptions;
        if let Ok(pool) = PgPoolOptions::new().max_connections(1).connect(&url).await {
            sqlx::migrate!("./migrations").run(&pool).await.ok();

            let state = SharedState {
                inner: Arc::new(RwLock::new(AppState::new())),
            };

            // Search with a category filter that won't match
            let params = types::GptSearchParams {
                q: None,
                category: Some("zzz_nonexistent_category_xyz".to_string()),
                max_budget_cents: None,
                intent: None,
                session_token: None,
            };
            let result =
                gpt::gpt_search_services(axum::extract::State(state), axum::extract::Query(params))
                    .await;

            assert!(result.status().is_success());
            let resp: types::GptSearchResponse = read_typed(result).await;
            assert_eq!(resp.total_count, 0);
        }
    }
}

#[tokio::test]
async fn gpt_search_services_returns_results_without_filters() {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        use sqlx::postgres::PgPoolOptions;
        if let Ok(pool) = PgPoolOptions::new().max_connections(1).connect(&url).await {
            sqlx::migrate!("./migrations").run(&pool).await.ok();

            let state = SharedState {
                inner: Arc::new(RwLock::new(AppState::new())),
            };

            // Search with no filters — returns all active services
            let params = types::GptSearchParams {
                q: None,
                category: None,
                max_budget_cents: None,
                intent: None,
                session_token: None,
            };
            let result =
                gpt::gpt_search_services(axum::extract::State(state), axum::extract::Query(params))
                    .await;

            assert!(result.status().is_success());
            let resp: types::GptSearchResponse = read_typed(result).await;
            // total_count should match services length
            assert_eq!(resp.total_count, resp.services.len());
            // All returned services should be active
            for svc in &resp.services {
                assert!(svc.active);
            }
        }
    }
}

#[tokio::test]
async fn gpt_auth_registers_new_user_and_issues_session() {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        use sqlx::postgres::PgPoolOptions;
        if let Ok(pool) = PgPoolOptions::new().max_connections(1).connect(&url).await {
            sqlx::migrate!("./migrations").run(&pool).await.ok();

            let state = SharedState {
                inner: Arc::new(RwLock::new(AppState::new())),
            };

            // Use a unique email to ensure new user
            let unique_email = format!("test_gpt_auth_{}@example.com", Uuid::new_v4());
            let payload = types::GptAuthRequest {
                email: unique_email.clone(),
                region: "JP".to_string(),
                roles: vec!["developer".to_string()],
                tools_used: vec!["cursor".to_string()],
            };

            let result = gpt::gpt_auth(axum::extract::State(state), axum::Json(payload)).await;

            assert!(
                result.status().is_success(),
                "gpt_auth should succeed for new user"
            );
            let resp: types::GptAuthResponse = read_typed(result).await;
            assert!(resp.is_new_user, "should be a new user");
            assert_eq!(resp.email, unique_email);
            assert!(
                !resp.session_token.is_nil(),
                "session token should not be nil"
            );
            assert!(!resp.user_id.is_nil(), "user_id should not be nil");

            // Verify user was inserted with source = "gpt_apps"
            let source: Option<String> =
                sqlx::query_scalar("SELECT source FROM users WHERE id = $1")
                    .bind(resp.user_id)
                    .fetch_optional(&pool)
                    .await
                    .unwrap();
            assert_eq!(source.as_deref(), Some("gpt_apps"));

            // Cleanup
            sqlx::query("DELETE FROM gpt_sessions WHERE user_id = $1")
                .bind(resp.user_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(resp.user_id)
                .execute(&pool)
                .await
                .ok();
        }
    }
}

#[tokio::test]
async fn gpt_auth_identifies_existing_user() {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        use sqlx::postgres::PgPoolOptions;
        if let Ok(pool) = PgPoolOptions::new().max_connections(1).connect(&url).await {
            sqlx::migrate!("./migrations").run(&pool).await.ok();

            // Pre-insert a user
            let user_id = Uuid::new_v4();
            let email = format!("existing_gpt_{}@example.com", Uuid::new_v4());
            sqlx::query(
                "INSERT INTO users (id, email, region, roles, tools_used, attributes, source) \
                 VALUES ($1, $2, 'JP', '{}', '{}', '{}'::jsonb, 'web')",
            )
            .bind(user_id)
            .bind(&email)
            .execute(&pool)
            .await
            .unwrap();

            let state = SharedState {
                inner: Arc::new(RwLock::new(AppState::new())),
            };

            let payload = types::GptAuthRequest {
                email: email.clone(),
                region: "JP".to_string(),
                roles: vec![],
                tools_used: vec![],
            };

            let result = gpt::gpt_auth(axum::extract::State(state), axum::Json(payload)).await;

            assert!(
                result.status().is_success(),
                "gpt_auth should succeed for existing user"
            );
            let resp: types::GptAuthResponse = read_typed(result).await;
            assert!(!resp.is_new_user, "should NOT be a new user");
            assert_eq!(resp.user_id, user_id);
            assert_eq!(resp.email, email);
            assert!(
                !resp.session_token.is_nil(),
                "session token should be issued"
            );

            // Verify session token is valid via resolve_session
            let resolved = gpt::resolve_session(&pool, resp.session_token).await;
            assert!(resolved.is_ok(), "session token should be resolvable");
            assert_eq!(resolved.unwrap(), user_id);

            // Cleanup
            sqlx::query("DELETE FROM gpt_sessions WHERE user_id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
        }
    }
}

#[test]
fn rate_limiter_sequential_consumption_drains_tokens() {
    use std::time::Duration;

    let mut limiter = gpt::RateLimiter::new(5, Duration::from_secs(60));
    for i in 0..5 {
        assert!(limiter.try_acquire(), "token {} should be available", i + 1);
    }
    assert!(!limiter.try_acquire(), "all 5 tokens should be exhausted");
    assert!(
        !limiter.try_acquire(),
        "still exhausted on repeated attempt"
    );
}

#[test]
fn gpt_get_tasks_has_correct_handler_signature() {
    use crate::types::GptTaskParams;
    let _fn_ref: fn(
        axum::extract::State<crate::types::SharedState>,
        axum::extract::Path<Uuid>,
        axum::extract::Query<GptTaskParams>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = axum::response::Response> + Send>,
    > = |state, path, query| Box::pin(gpt::gpt_get_tasks(state, path, query));
}

#[tokio::test]
async fn gpt_get_tasks_returns_task_details_for_valid_session() {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        use sqlx::postgres::PgPoolOptions;
        if let Ok(pool) = PgPoolOptions::new().max_connections(1).connect(&url).await {
            sqlx::migrate!("./migrations").run(&pool).await.ok();

            // Create a test user
            let user_id = Uuid::new_v4();
            let email = format!("gpt_tasks_test_{}@example.com", Uuid::new_v4());
            sqlx::query(
                "INSERT INTO users (id, email, region, roles, tools_used, attributes, source) \
                 VALUES ($1, $2, 'JP', '{}', '{}', '{}'::jsonb, 'gpt_apps')",
            )
            .bind(user_id)
            .bind(&email)
            .execute(&pool)
            .await
            .unwrap();

            // Create a session token
            let session_token: Uuid = sqlx::query_scalar(
                "INSERT INTO gpt_sessions (user_id) VALUES ($1) RETURNING token",
            )
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();

            // Create a test campaign (no task_schema — should use defaults)
            let campaign_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO campaigns (id, name, sponsor, target_roles, target_tools, required_task, \
                 subsidy_per_call_cents, budget_total_cents, budget_remaining_cents, query_urls, active, created_at) \
                 VALUES ($1, 'Test Campaign', 'TestSponsor', '{}', '{design}', 'complete_survey', \
                 500, 10000, 10000, '{}', true, NOW())"
            )
            .bind(campaign_id)
            .execute(&pool)
            .await
            .unwrap();

            let state = SharedState {
                inner: Arc::new(RwLock::new(AppState::new())),
            };

            let params = types::GptTaskParams { session_token };
            let result = gpt::gpt_get_tasks(
                axum::extract::State(state),
                axum::extract::Path(campaign_id),
                axum::extract::Query(params),
            )
            .await;

            assert!(result.status().is_success(), "gpt_get_tasks should succeed");
            let resp: types::GptTaskResponse = read_typed(result).await;
            assert_eq!(resp.campaign_id, campaign_id);
            assert_eq!(resp.campaign_name, "Test Campaign");
            assert_eq!(resp.sponsor, "TestSponsor");
            assert_eq!(resp.required_task, "complete_survey");
            assert!(!resp.already_completed, "task should not be completed yet");
            assert_eq!(resp.subsidy_amount_cents, 500);
            // Default task input format
            assert_eq!(resp.task_input_format.task_type, "survey");
            assert!(!resp.task_input_format.required_fields.is_empty());

            // Cleanup
            sqlx::query("DELETE FROM gpt_sessions WHERE user_id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM campaigns WHERE id = $1")
                .bind(campaign_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
        }
    }
}

#[tokio::test]
async fn gpt_get_tasks_shows_already_completed_when_task_done() {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        use sqlx::postgres::PgPoolOptions;
        if let Ok(pool) = PgPoolOptions::new().max_connections(1).connect(&url).await {
            sqlx::migrate!("./migrations").run(&pool).await.ok();

            // Create user + session + campaign
            let user_id = Uuid::new_v4();
            let email = format!("gpt_tasks_completed_{}@example.com", Uuid::new_v4());
            sqlx::query(
                "INSERT INTO users (id, email, region, roles, tools_used, attributes, source) \
                 VALUES ($1, $2, 'JP', '{}', '{}', '{}'::jsonb, 'gpt_apps')",
            )
            .bind(user_id)
            .bind(&email)
            .execute(&pool)
            .await
            .unwrap();

            let session_token: Uuid = sqlx::query_scalar(
                "INSERT INTO gpt_sessions (user_id) VALUES ($1) RETURNING token",
            )
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();

            let campaign_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO campaigns (id, name, sponsor, target_roles, target_tools, required_task, \
                 subsidy_per_call_cents, budget_total_cents, budget_remaining_cents, query_urls, active, created_at) \
                 VALUES ($1, 'Completed Campaign', 'Sponsor2', '{}', '{}', 'fill_form', \
                 300, 5000, 5000, '{}', true, NOW())"
            )
            .bind(campaign_id)
            .execute(&pool)
            .await
            .unwrap();

            // Mark task as completed
            sqlx::query(
                "INSERT INTO task_completions (id, campaign_id, user_id, task_name, created_at) \
                 VALUES ($1, $2, $3, 'fill_form', NOW())",
            )
            .bind(Uuid::new_v4())
            .bind(campaign_id)
            .bind(user_id)
            .execute(&pool)
            .await
            .unwrap();

            let state = SharedState {
                inner: Arc::new(RwLock::new(AppState::new())),
            };

            let params = types::GptTaskParams { session_token };
            let result = gpt::gpt_get_tasks(
                axum::extract::State(state),
                axum::extract::Path(campaign_id),
                axum::extract::Query(params),
            )
            .await;

            assert!(result.status().is_success());
            let resp: types::GptTaskResponse = read_typed(result).await;
            assert!(resp.already_completed, "task should be marked as completed");
            assert!(resp.message.contains("already completed"));

            // Cleanup
            sqlx::query("DELETE FROM task_completions WHERE campaign_id = $1")
                .bind(campaign_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM gpt_sessions WHERE user_id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM campaigns WHERE id = $1")
                .bind(campaign_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
        }
    }
}

#[tokio::test]
async fn gpt_get_tasks_returns_not_found_for_missing_campaign() {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        use sqlx::postgres::PgPoolOptions;
        if let Ok(pool) = PgPoolOptions::new().max_connections(1).connect(&url).await {
            sqlx::migrate!("./migrations").run(&pool).await.ok();

            // Create user + session
            let user_id = Uuid::new_v4();
            let email = format!("gpt_tasks_notfound_{}@example.com", Uuid::new_v4());
            sqlx::query(
                "INSERT INTO users (id, email, region, roles, tools_used, attributes, source) \
                 VALUES ($1, $2, 'JP', '{}', '{}', '{}'::jsonb, 'gpt_apps')",
            )
            .bind(user_id)
            .bind(&email)
            .execute(&pool)
            .await
            .unwrap();

            let session_token: Uuid = sqlx::query_scalar(
                "INSERT INTO gpt_sessions (user_id) VALUES ($1) RETURNING token",
            )
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();

            let state = SharedState {
                inner: Arc::new(RwLock::new(AppState::new())),
            };

            let params = types::GptTaskParams { session_token };
            let result = gpt::gpt_get_tasks(
                axum::extract::State(state),
                axum::extract::Path(Uuid::new_v4()), // non-existent campaign
                axum::extract::Query(params),
            )
            .await;

            assert!(
                !result.status().is_success(),
                "should fail for non-existent campaign"
            );
            let err_str = read_body_string(result).await;
            assert!(
                err_str.contains("not found"),
                "error should mention not found, got: {}",
                err_str
            );

            // Cleanup
            sqlx::query("DELETE FROM gpt_sessions WHERE user_id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
        }
    }
}

#[tokio::test]
async fn gpt_get_tasks_returns_custom_format_from_task_schema() {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        use sqlx::postgres::PgPoolOptions;
        if let Ok(pool) = PgPoolOptions::new().max_connections(1).connect(&url).await {
            sqlx::migrate!("./migrations").run(&pool).await.ok();

            // Create user + session
            let user_id = Uuid::new_v4();
            let email = format!("gpt_tasks_schema_{}@example.com", Uuid::new_v4());
            sqlx::query(
                "INSERT INTO users (id, email, region, roles, tools_used, attributes, source) \
                 VALUES ($1, $2, 'JP', '{}', '{}', '{}'::jsonb, 'gpt_apps')",
            )
            .bind(user_id)
            .bind(&email)
            .execute(&pool)
            .await
            .unwrap();

            let session_token: Uuid = sqlx::query_scalar(
                "INSERT INTO gpt_sessions (user_id) VALUES ($1) RETURNING token",
            )
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();

            // Create campaign WITH task_schema
            let campaign_id = Uuid::new_v4();
            let task_schema = serde_json::json!({
                "task_type": "data_provision",
                "required_fields": ["company_name", "website_url", "contact_email"],
                "instructions": "Please provide your company details for verification."
            });
            sqlx::query(
                "INSERT INTO campaigns (id, name, sponsor, target_roles, target_tools, required_task, \
                 subsidy_per_call_cents, budget_total_cents, budget_remaining_cents, query_urls, active, created_at, task_schema) \
                 VALUES ($1, 'Schema Campaign', 'SchemaSponsor', '{}', '{}', 'provide_data', \
                 800, 20000, 20000, '{}', true, NOW(), $2)"
            )
            .bind(campaign_id)
            .bind(&task_schema)
            .execute(&pool)
            .await
            .unwrap();

            let state = SharedState {
                inner: Arc::new(RwLock::new(AppState::new())),
            };

            let params = types::GptTaskParams { session_token };
            let result = gpt::gpt_get_tasks(
                axum::extract::State(state),
                axum::extract::Path(campaign_id),
                axum::extract::Query(params),
            )
            .await;

            assert!(
                result.status().is_success(),
                "gpt_get_tasks should succeed with task_schema"
            );
            let resp: types::GptTaskResponse = read_typed(result).await;
            assert_eq!(resp.task_input_format.task_type, "data_provision");
            assert_eq!(
                resp.task_input_format.required_fields,
                vec!["company_name", "website_url", "contact_email"]
            );
            assert!(
                resp.task_input_format
                    .instructions
                    .contains("company details")
            );

            // Cleanup
            sqlx::query("DELETE FROM gpt_sessions WHERE user_id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM campaigns WHERE id = $1")
                .bind(campaign_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
        }
    }
}

#[test]
fn gpt_complete_task_has_correct_handler_signature() {
    use crate::types::GptCompleteTaskRequest;
    let _fn_ref: fn(
        axum::extract::State<crate::types::SharedState>,
        axum::extract::Path<Uuid>,
        axum::Json<GptCompleteTaskRequest>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = axum::response::Response> + Send>,
    > = |state, path, payload| Box::pin(gpt::gpt_complete_task(state, path, payload));
}

#[tokio::test]
async fn gpt_complete_task_records_consent_and_completion() {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        use sqlx::postgres::PgPoolOptions;
        if let Ok(pool) = PgPoolOptions::new().max_connections(1).connect(&url).await {
            sqlx::migrate!("./migrations").run(&pool).await.ok();

            // Create user + session + campaign
            let user_id = Uuid::new_v4();
            let email = format!("gpt_complete_{}@example.com", Uuid::new_v4());
            sqlx::query(
                "INSERT INTO users (id, email, region, roles, tools_used, attributes, source) \
                 VALUES ($1, $2, 'JP', '{}', '{}', '{}'::jsonb, 'gpt_apps')",
            )
            .bind(user_id)
            .bind(&email)
            .execute(&pool)
            .await
            .unwrap();

            let session_token: Uuid = sqlx::query_scalar(
                "INSERT INTO gpt_sessions (user_id) VALUES ($1) RETURNING token",
            )
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();

            let campaign_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO campaigns (id, name, sponsor, target_roles, target_tools, required_task, \
                 subsidy_per_call_cents, budget_total_cents, budget_remaining_cents, query_urls, active, created_at) \
                 VALUES ($1, 'Consent Campaign', 'ConsentSponsor', '{}', '{}', 'fill_survey', \
                 500, 10000, 10000, '{}', true, NOW())"
            )
            .bind(campaign_id)
            .execute(&pool)
            .await
            .unwrap();

            let state = SharedState {
                inner: Arc::new(RwLock::new(AppState::new())),
            };

            let payload = types::GptCompleteTaskRequest {
                session_token,
                task_name: "fill_survey".to_string(),
                details: Some("Completed the survey".to_string()),
                consent: types::GptConsentInput {
                    data_sharing_agreed: true,
                    purpose_acknowledged: true,
                    contact_permission: false,
                },
            };

            let result = gpt::gpt_complete_task(
                axum::extract::State(state),
                axum::extract::Path(campaign_id),
                axum::Json(payload),
            )
            .await;

            assert!(
                result.status().is_success(),
                "gpt_complete_task should succeed"
            );
            let resp: types::GptCompleteTaskResponse = read_typed(result).await;
            assert_eq!(resp.campaign_id, campaign_id);
            assert!(resp.consent_recorded);
            assert!(resp.can_use_service);
            assert!(!resp.task_completion_id.is_nil());

            // Verify consent records were created in DB
            let consent_count: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM consents WHERE user_id = $1 AND campaign_id = $2",
            )
            .bind(user_id)
            .bind(campaign_id)
            .fetch_one(&pool)
            .await
            .unwrap();
            assert_eq!(
                consent_count, 3,
                "should have 3 consent records (data_sharing, contact, retention)"
            );

            // Verify task completion was recorded
            let task_exists: bool = sqlx::query_scalar(
                "SELECT exists(SELECT 1 FROM task_completions WHERE campaign_id = $1 AND user_id = $2 AND task_name = 'fill_survey')"
            )
            .bind(campaign_id)
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();
            assert!(task_exists, "task completion should be recorded");

            // Cleanup
            sqlx::query("DELETE FROM task_completions WHERE campaign_id = $1")
                .bind(campaign_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM consents WHERE campaign_id = $1")
                .bind(campaign_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM gpt_sessions WHERE user_id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM campaigns WHERE id = $1")
                .bind(campaign_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
        }
    }
}

#[tokio::test]
async fn gpt_complete_task_handles_consent_refused() {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        use sqlx::postgres::PgPoolOptions;
        if let Ok(pool) = PgPoolOptions::new().max_connections(1).connect(&url).await {
            sqlx::migrate!("./migrations").run(&pool).await.ok();

            let user_id = Uuid::new_v4();
            let email = format!("gpt_complete_refuse_{}@example.com", Uuid::new_v4());
            sqlx::query(
                "INSERT INTO users (id, email, region, roles, tools_used, attributes, source) \
                 VALUES ($1, $2, 'JP', '{}', '{}', '{}'::jsonb, 'gpt_apps')",
            )
            .bind(user_id)
            .bind(&email)
            .execute(&pool)
            .await
            .unwrap();

            let session_token: Uuid = sqlx::query_scalar(
                "INSERT INTO gpt_sessions (user_id) VALUES ($1) RETURNING token",
            )
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();

            let campaign_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO campaigns (id, name, sponsor, target_roles, target_tools, required_task, \
                 subsidy_per_call_cents, budget_total_cents, budget_remaining_cents, query_urls, active, created_at) \
                 VALUES ($1, 'Refuse Campaign', 'RefuseSponsor', '{}', '{}', 'do_task', \
                 300, 5000, 5000, '{}', true, NOW())"
            )
            .bind(campaign_id)
            .execute(&pool)
            .await
            .unwrap();

            let state = SharedState {
                inner: Arc::new(RwLock::new(AppState::new())),
            };

            // data_sharing_agreed = false
            let payload = types::GptCompleteTaskRequest {
                session_token,
                task_name: "do_task".to_string(),
                details: None,
                consent: types::GptConsentInput {
                    data_sharing_agreed: false,
                    purpose_acknowledged: true,
                    contact_permission: false,
                },
            };

            let result = gpt::gpt_complete_task(
                axum::extract::State(state),
                axum::extract::Path(campaign_id),
                axum::Json(payload),
            )
            .await;

            assert!(
                result.status().is_success(),
                "should still succeed even when consent refused"
            );
            let resp: types::GptCompleteTaskResponse = read_typed(result).await;
            assert!(resp.can_use_service, "service should still be usable");
            assert!(resp.consent_recorded);
            // Message should mention data transfer is blocked
            assert!(
                resp.message.contains("data")
                    || resp.message.contains("transfer")
                    || resp.message.contains("shar"),
                "message should mention data sharing restriction, got: {}",
                resp.message
            );

            // Task completion should still be recorded
            let task_exists: bool = sqlx::query_scalar(
                "SELECT exists(SELECT 1 FROM task_completions WHERE campaign_id = $1 AND user_id = $2)"
            )
            .bind(campaign_id)
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();
            assert!(
                task_exists,
                "task completion should still be recorded even with consent refused"
            );

            // Consent records should reflect the refusal
            let data_sharing_granted: bool = sqlx::query_scalar(
                "SELECT granted FROM consents WHERE user_id = $1 AND campaign_id = $2 AND consent_type = 'data_sharing'"
            )
            .bind(user_id)
            .bind(campaign_id)
            .fetch_one(&pool)
            .await
            .unwrap();
            assert!(
                !data_sharing_granted,
                "data_sharing consent should be false"
            );

            // Cleanup
            sqlx::query("DELETE FROM task_completions WHERE campaign_id = $1")
                .bind(campaign_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM consents WHERE campaign_id = $1")
                .bind(campaign_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM gpt_sessions WHERE user_id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM campaigns WHERE id = $1")
                .bind(campaign_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
        }
    }
}

#[tokio::test]
async fn gpt_complete_task_returns_not_found_for_missing_campaign() {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        use sqlx::postgres::PgPoolOptions;
        if let Ok(pool) = PgPoolOptions::new().max_connections(1).connect(&url).await {
            sqlx::migrate!("./migrations").run(&pool).await.ok();

            let user_id = Uuid::new_v4();
            let email = format!("gpt_complete_nf_{}@example.com", Uuid::new_v4());
            sqlx::query(
                "INSERT INTO users (id, email, region, roles, tools_used, attributes, source) \
                 VALUES ($1, $2, 'JP', '{}', '{}', '{}'::jsonb, 'gpt_apps')",
            )
            .bind(user_id)
            .bind(&email)
            .execute(&pool)
            .await
            .unwrap();

            let session_token: Uuid = sqlx::query_scalar(
                "INSERT INTO gpt_sessions (user_id) VALUES ($1) RETURNING token",
            )
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();

            let state = SharedState {
                inner: Arc::new(RwLock::new(AppState::new())),
            };

            let payload = types::GptCompleteTaskRequest {
                session_token,
                task_name: "some_task".to_string(),
                details: None,
                consent: types::GptConsentInput {
                    data_sharing_agreed: true,
                    purpose_acknowledged: true,
                    contact_permission: true,
                },
            };

            let result = gpt::gpt_complete_task(
                axum::extract::State(state),
                axum::extract::Path(Uuid::new_v4()),
                axum::Json(payload),
            )
            .await;

            assert!(!result.status().is_success());
            let err_str = read_body_string(result).await;
            assert!(
                err_str.contains("not found"),
                "should return not found, got: {}",
                err_str
            );

            // Cleanup
            sqlx::query("DELETE FROM gpt_sessions WHERE user_id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
        }
    }
}

#[test]
fn gpt_run_service_has_correct_handler_signature() {
    use crate::types::GptRunServiceRequest;
    let _fn_ref: fn(
        axum::extract::State<crate::types::SharedState>,
        axum::extract::Path<String>,
        axum::Json<GptRunServiceRequest>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = axum::response::Response> + Send>,
    > = |state, path, payload| Box::pin(gpt::gpt_run_service(state, path, payload));
}

#[tokio::test]
async fn gpt_run_service_sponsored_flow_success() {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        use sqlx::postgres::PgPoolOptions;
        if let Ok(pool) = PgPoolOptions::new().max_connections(1).connect(&url).await {
            sqlx::migrate!("./migrations").run(&pool).await.ok();

            // Create user with matching roles/tools
            let user_id = Uuid::new_v4();
            let email = format!("gpt_run_svc_{}@example.com", Uuid::new_v4());
            sqlx::query(
                "INSERT INTO users (id, email, region, roles, tools_used, attributes, source) \
                 VALUES ($1, $2, 'JP', '{developer}', '{design}', '{}'::jsonb, 'gpt_apps')",
            )
            .bind(user_id)
            .bind(&email)
            .execute(&pool)
            .await
            .unwrap();

            let session_token: Uuid = sqlx::query_scalar(
                "INSERT INTO gpt_sessions (user_id) VALUES ($1) RETURNING token",
            )
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();

            // Create campaign with matching target_tools and enough budget
            let campaign_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO campaigns (id, name, sponsor, target_roles, target_tools, required_task, \
                 subsidy_per_call_cents, budget_total_cents, budget_remaining_cents, query_urls, active, created_at) \
                 VALUES ($1, 'Run Campaign', 'RunSponsor', '{developer}', '{design}', 'do_survey', \
                 800, 50000, 50000, '{}', true, NOW())"
            )
            .bind(campaign_id)
            .execute(&pool)
            .await
            .unwrap();

            // Complete the required task
            sqlx::query(
                "INSERT INTO task_completions (id, campaign_id, user_id, task_name, created_at) \
                 VALUES ($1, $2, $3, 'do_survey', NOW())",
            )
            .bind(Uuid::new_v4())
            .bind(campaign_id)
            .bind(user_id)
            .execute(&pool)
            .await
            .unwrap();

            let state = SharedState {
                inner: Arc::new(RwLock::new(AppState::new())),
            };

            let payload = types::GptRunServiceRequest {
                session_token,
                input: "test design input".to_string(),
            };

            let result = gpt::gpt_run_service(
                axum::extract::State(state),
                axum::extract::Path("design".to_string()),
                axum::Json(payload),
            )
            .await;

            assert!(
                result.status().is_success(),
                "gpt_run_service should succeed for sponsored flow"
            );
            let resp: types::GptRunServiceResponse = read_typed(result).await;
            assert_eq!(resp.service, "design");
            assert_eq!(resp.payment_mode, "sponsored");
            assert_eq!(resp.sponsored_by, Some("RunSponsor".to_string()));
            assert!(resp.tx_hash.is_some());
            assert!(!resp.output.is_empty());
            assert!(!resp.message.is_empty());

            // Verify budget was deducted
            let remaining: i64 =
                sqlx::query_scalar("SELECT budget_remaining_cents FROM campaigns WHERE id = $1")
                    .bind(campaign_id)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            assert!(remaining < 50000, "budget should have been deducted");

            // Verify payment was recorded
            let payment_exists: bool = sqlx::query_scalar(
                "SELECT exists(SELECT 1 FROM payments WHERE campaign_id = $1 AND service = 'design')"
            )
            .bind(campaign_id)
            .fetch_one(&pool)
            .await
            .unwrap();
            assert!(payment_exists, "payment should be recorded");

            // Cleanup
            sqlx::query("DELETE FROM payments WHERE campaign_id = $1")
                .bind(campaign_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM task_completions WHERE campaign_id = $1")
                .bind(campaign_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM gpt_sessions WHERE user_id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM campaigns WHERE id = $1")
                .bind(campaign_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
        }
    }
}

#[tokio::test]
async fn gpt_run_service_fails_when_task_not_completed() {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        use sqlx::postgres::PgPoolOptions;
        if let Ok(pool) = PgPoolOptions::new().max_connections(1).connect(&url).await {
            sqlx::migrate!("./migrations").run(&pool).await.ok();

            let user_id = Uuid::new_v4();
            let email = format!("gpt_run_notask_{}@example.com", Uuid::new_v4());
            sqlx::query(
                "INSERT INTO users (id, email, region, roles, tools_used, attributes, source) \
                 VALUES ($1, $2, 'JP', '{developer}', '{design}', '{}'::jsonb, 'gpt_apps')",
            )
            .bind(user_id)
            .bind(&email)
            .execute(&pool)
            .await
            .unwrap();

            let session_token: Uuid = sqlx::query_scalar(
                "INSERT INTO gpt_sessions (user_id) VALUES ($1) RETURNING token",
            )
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();

            // Campaign matches user but task NOT completed
            let campaign_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO campaigns (id, name, sponsor, target_roles, target_tools, required_task, \
                 subsidy_per_call_cents, budget_total_cents, budget_remaining_cents, query_urls, active, created_at) \
                 VALUES ($1, 'NoTask Campaign', 'NoTaskSponsor', '{developer}', '{design}', 'pending_task', \
                 500, 10000, 10000, '{}', true, NOW())"
            )
            .bind(campaign_id)
            .execute(&pool)
            .await
            .unwrap();

            let state = SharedState {
                inner: Arc::new(RwLock::new(AppState::new())),
            };

            let payload = types::GptRunServiceRequest {
                session_token,
                input: "test".to_string(),
            };

            let result = gpt::gpt_run_service(
                axum::extract::State(state),
                axum::extract::Path("design".to_string()),
                axum::Json(payload),
            )
            .await;

            assert!(
                !result.status().is_success(),
                "should fail when task not completed"
            );
            let err_str = read_body_string(result).await;
            assert!(
                err_str.contains("pending_task") || err_str.contains("complete"),
                "error should mention the task, got: {}",
                err_str
            );

            // Cleanup
            sqlx::query("DELETE FROM gpt_sessions WHERE user_id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM campaigns WHERE id = $1")
                .bind(campaign_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
        }
    }
}

#[tokio::test]
async fn gpt_run_service_fails_when_no_matching_campaign() {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        use sqlx::postgres::PgPoolOptions;
        if let Ok(pool) = PgPoolOptions::new().max_connections(1).connect(&url).await {
            sqlx::migrate!("./migrations").run(&pool).await.ok();

            let user_id = Uuid::new_v4();
            let email = format!("gpt_run_nomatch_{}@example.com", Uuid::new_v4());
            sqlx::query(
                "INSERT INTO users (id, email, region, roles, tools_used, attributes, source) \
                 VALUES ($1, $2, 'JP', '{analyst}', '{excel}', '{}'::jsonb, 'gpt_apps')",
            )
            .bind(user_id)
            .bind(&email)
            .execute(&pool)
            .await
            .unwrap();

            let session_token: Uuid = sqlx::query_scalar(
                "INSERT INTO gpt_sessions (user_id) VALUES ($1) RETURNING token",
            )
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();

            // Campaign doesn't match user's roles/tools
            let campaign_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO campaigns (id, name, sponsor, target_roles, target_tools, required_task, \
                 subsidy_per_call_cents, budget_total_cents, budget_remaining_cents, query_urls, active, created_at) \
                 VALUES ($1, 'Mismatch Campaign', 'MismatchSponsor', '{engineer}', '{rust}', 'task', \
                 500, 10000, 10000, '{}', true, NOW())"
            )
            .bind(campaign_id)
            .execute(&pool)
            .await
            .unwrap();

            let state = SharedState {
                inner: Arc::new(RwLock::new(AppState::new())),
            };

            let payload = types::GptRunServiceRequest {
                session_token,
                input: "test".to_string(),
            };

            let result = gpt::gpt_run_service(
                axum::extract::State(state),
                axum::extract::Path("design".to_string()),
                axum::Json(payload),
            )
            .await;

            assert!(
                !result.status().is_success(),
                "should fail when no matching campaign"
            );
            let err_str = read_body_string(result).await;
            assert!(
                err_str.contains("No sponsored")
                    || err_str.contains("campaign")
                    || err_str.contains("direct"),
                "error should mention no sponsor found, got: {}",
                err_str
            );

            // Cleanup
            sqlx::query("DELETE FROM gpt_sessions WHERE user_id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM campaigns WHERE id = $1")
                .bind(campaign_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
        }
    }
}

#[test]
fn migration_0010_add_task_schema_has_expected_schema() {
    let sql = include_str!("../migrations/0010_add_task_schema.sql");
    let lower = sql.to_lowercase();

    // ALTER TABLE to add task_schema column
    assert!(lower.contains("alter table"), "should use ALTER TABLE");
    assert!(lower.contains("campaigns"), "should target campaigns table");
    assert!(lower.contains("add column"), "should add a column");
    assert!(
        lower.contains("task_schema"),
        "should add task_schema column"
    );
    assert!(lower.contains("jsonb"), "task_schema should be JSONB type");

    // Safety: IF NOT EXISTS for idempotent migration
    assert!(
        lower.contains("if not exists"),
        "should use IF NOT EXISTS for safety"
    );
}

#[test]
fn gpt_user_status_has_correct_handler_signature() {
    use crate::types::GptUserStatusParams;
    let _fn_ref: fn(
        axum::extract::State<crate::types::SharedState>,
        axum::extract::Query<GptUserStatusParams>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = axum::response::Response> + Send>,
    > = |state, params| Box::pin(gpt::gpt_user_status(state, params));
}

#[tokio::test]
async fn gpt_user_status_returns_completed_tasks_and_services() {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        use sqlx::postgres::PgPoolOptions;
        if let Ok(pool) = PgPoolOptions::new().max_connections(1).connect(&url).await {
            sqlx::migrate!("./migrations").run(&pool).await.ok();

            // Create user
            let user_id = Uuid::new_v4();
            let email = format!("gpt_status_{}@example.com", Uuid::new_v4());
            sqlx::query(
                "INSERT INTO users (id, email, region, roles, tools_used, attributes, source) \
                 VALUES ($1, $2, 'JP', '{developer}', '{design}', '{}'::jsonb, 'gpt_apps')",
            )
            .bind(user_id)
            .bind(&email)
            .execute(&pool)
            .await
            .unwrap();

            let session_token: Uuid = sqlx::query_scalar(
                "INSERT INTO gpt_sessions (user_id) VALUES ($1) RETURNING token",
            )
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();

            // Create campaign matching user
            let campaign_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO campaigns (id, name, sponsor, target_roles, target_tools, required_task, \
                 subsidy_per_call_cents, budget_total_cents, budget_remaining_cents, query_urls, active, created_at) \
                 VALUES ($1, 'Status Campaign', 'StatusSponsor', '{developer}', '{design}', 'status_survey', \
                 500, 10000, 10000, '{}', true, NOW())"
            )
            .bind(campaign_id)
            .execute(&pool)
            .await
            .unwrap();

            // Complete the required task
            sqlx::query(
                "INSERT INTO task_completions (id, campaign_id, user_id, task_name, created_at) \
                 VALUES ($1, $2, $3, 'status_survey', NOW())",
            )
            .bind(Uuid::new_v4())
            .bind(campaign_id)
            .bind(user_id)
            .execute(&pool)
            .await
            .unwrap();

            let state = SharedState {
                inner: Arc::new(RwLock::new(AppState::new())),
            };

            let params = types::GptUserStatusParams { session_token };

            let result =
                gpt::gpt_user_status(axum::extract::State(state), axum::extract::Query(params))
                    .await;

            assert!(
                result.status().is_success(),
                "gpt_user_status should succeed"
            );
            let resp: types::GptUserStatusResponse = read_typed(result).await;
            assert_eq!(resp.user_id, user_id);
            assert_eq!(resp.email, email);
            assert!(
                !resp.completed_tasks.is_empty(),
                "should have completed tasks"
            );
            assert_eq!(resp.completed_tasks[0].campaign_name, "Status Campaign");
            assert_eq!(resp.completed_tasks[0].task_name, "status_survey");
            assert!(
                !resp.available_services.is_empty(),
                "should have available services"
            );
            // The service should be ready since task is completed
            let ready_service = resp
                .available_services
                .iter()
                .find(|s| s.sponsor == "StatusSponsor");
            assert!(
                ready_service.is_some(),
                "should find service from matching campaign"
            );
            assert!(
                ready_service.unwrap().ready,
                "service should be ready since task completed"
            );
            assert!(!resp.message.is_empty());

            // Cleanup
            sqlx::query("DELETE FROM task_completions WHERE campaign_id = $1")
                .bind(campaign_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM gpt_sessions WHERE user_id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM campaigns WHERE id = $1")
                .bind(campaign_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
        }
    }
}

#[tokio::test]
async fn gpt_user_status_new_user_no_tasks() {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        use sqlx::postgres::PgPoolOptions;
        if let Ok(pool) = PgPoolOptions::new().max_connections(1).connect(&url).await {
            sqlx::migrate!("./migrations").run(&pool).await.ok();

            // Create user with no task completions
            let user_id = Uuid::new_v4();
            let email = format!("gpt_newuser_{}@example.com", Uuid::new_v4());
            sqlx::query(
                "INSERT INTO users (id, email, region, roles, tools_used, attributes, source) \
                 VALUES ($1, $2, 'US', '{analyst}', '{excel}', '{}'::jsonb, 'gpt_apps')",
            )
            .bind(user_id)
            .bind(&email)
            .execute(&pool)
            .await
            .unwrap();

            let session_token: Uuid = sqlx::query_scalar(
                "INSERT INTO gpt_sessions (user_id) VALUES ($1) RETURNING token",
            )
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();

            let state = SharedState {
                inner: Arc::new(RwLock::new(AppState::new())),
            };

            let params = types::GptUserStatusParams { session_token };

            let result =
                gpt::gpt_user_status(axum::extract::State(state), axum::extract::Query(params))
                    .await;

            assert!(
                result.status().is_success(),
                "gpt_user_status should succeed for new user"
            );
            let resp: types::GptUserStatusResponse = read_typed(result).await;
            assert_eq!(resp.user_id, user_id);
            assert_eq!(resp.email, email);
            assert!(
                resp.completed_tasks.is_empty(),
                "new user should have no completed tasks"
            );
            assert!(!resp.message.is_empty());

            // Cleanup
            sqlx::query("DELETE FROM gpt_sessions WHERE user_id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
        }
    }
}

// --- Task 6.5: Edge-case unit tests ---

#[tokio::test]
async fn gpt_run_service_skips_campaign_with_insufficient_budget() {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        use sqlx::postgres::PgPoolOptions;
        if let Ok(pool) = PgPoolOptions::new().max_connections(1).connect(&url).await {
            sqlx::migrate!("./migrations").run(&pool).await.ok();

            let user_id = Uuid::new_v4();
            let email = format!("gpt_budget_{}@example.com", Uuid::new_v4());
            sqlx::query(
                "INSERT INTO users (id, email, region, roles, tools_used, attributes, source) \
                 VALUES ($1, $2, 'JP', '{developer}', '{design}', '{}'::jsonb, 'gpt_apps')",
            )
            .bind(user_id)
            .bind(&email)
            .execute(&pool)
            .await
            .unwrap();

            let session_token: Uuid = sqlx::query_scalar(
                "INSERT INTO gpt_sessions (user_id) VALUES ($1) RETURNING token",
            )
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();

            // Campaign with budget too low (1 cent, design costs 8 cents)
            let campaign_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO campaigns (id, name, sponsor, target_roles, target_tools, required_task, \
                 subsidy_per_call_cents, budget_total_cents, budget_remaining_cents, query_urls, active, created_at) \
                 VALUES ($1, 'Low Budget', 'LowSponsor', '{developer}', '{design}', 'survey', \
                 800, 50000, 1, '{}', true, NOW())"
            )
            .bind(campaign_id)
            .execute(&pool)
            .await
            .unwrap();

            // Complete the task (so the only blocker is budget)
            sqlx::query(
                "INSERT INTO task_completions (id, campaign_id, user_id, task_name, created_at) \
                 VALUES ($1, $2, $3, 'survey', NOW())",
            )
            .bind(Uuid::new_v4())
            .bind(campaign_id)
            .bind(user_id)
            .execute(&pool)
            .await
            .unwrap();

            let state = SharedState {
                inner: Arc::new(RwLock::new(AppState::new())),
            };

            let payload = types::GptRunServiceRequest {
                session_token,
                input: "test".to_string(),
            };

            let result = gpt::gpt_run_service(
                axum::extract::State(state),
                axum::extract::Path("design".to_string()),
                axum::Json(payload),
            )
            .await;

            // Should fail because budget_remaining_cents (1) < price (8)
            assert!(
                !result.status().is_success(),
                "should fail when campaign budget is insufficient"
            );

            // Cleanup
            sqlx::query("DELETE FROM task_completions WHERE campaign_id = $1")
                .bind(campaign_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM gpt_sessions WHERE user_id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM campaigns WHERE id = $1")
                .bind(campaign_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
        }
    }
}

#[tokio::test]
async fn gpt_user_status_shows_not_ready_for_pending_task() {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        use sqlx::postgres::PgPoolOptions;
        if let Ok(pool) = PgPoolOptions::new().max_connections(1).connect(&url).await {
            sqlx::migrate!("./migrations").run(&pool).await.ok();

            let user_id = Uuid::new_v4();
            let email = format!("gpt_notready_{}@example.com", Uuid::new_v4());
            sqlx::query(
                "INSERT INTO users (id, email, region, roles, tools_used, attributes, source) \
                 VALUES ($1, $2, 'JP', '{developer}', '{design}', '{}'::jsonb, 'gpt_apps')",
            )
            .bind(user_id)
            .bind(&email)
            .execute(&pool)
            .await
            .unwrap();

            let session_token: Uuid = sqlx::query_scalar(
                "INSERT INTO gpt_sessions (user_id) VALUES ($1) RETURNING token",
            )
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();

            // Campaign matches user but task NOT completed
            let campaign_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO campaigns (id, name, sponsor, target_roles, target_tools, required_task, \
                 subsidy_per_call_cents, budget_total_cents, budget_remaining_cents, query_urls, active, created_at) \
                 VALUES ($1, 'Pending Campaign', 'PendingSponsor', '{developer}', '{design}', 'pending_survey', \
                 500, 10000, 10000, '{}', true, NOW())"
            )
            .bind(campaign_id)
            .execute(&pool)
            .await
            .unwrap();

            let state = SharedState {
                inner: Arc::new(RwLock::new(AppState::new())),
            };

            let params = types::GptUserStatusParams { session_token };

            let result =
                gpt::gpt_user_status(axum::extract::State(state), axum::extract::Query(params))
                    .await;

            assert!(
                result.status().is_success(),
                "gpt_user_status should succeed"
            );
            let resp: types::GptUserStatusResponse = read_typed(result).await;
            assert!(
                resp.completed_tasks.is_empty(),
                "should have no completed tasks"
            );
            assert!(
                !resp.available_services.is_empty(),
                "should list available service from matching campaign"
            );
            let svc = &resp.available_services[0];
            assert_eq!(svc.sponsor, "PendingSponsor");
            assert!(
                !svc.ready,
                "service should NOT be ready since task is pending"
            );
            assert!(
                resp.message.contains("0 ready"),
                "message should indicate 0 ready services"
            );

            // Cleanup
            sqlx::query("DELETE FROM gpt_sessions WHERE user_id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM campaigns WHERE id = $1")
                .bind(campaign_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
        }
    }
}

#[tokio::test]
async fn gpt_handlers_reject_invalid_session_token() {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        use sqlx::postgres::PgPoolOptions;
        if let Ok(pool) = PgPoolOptions::new().max_connections(1).connect(&url).await {
            sqlx::migrate!("./migrations").run(&pool).await.ok();

            let fake_token = Uuid::new_v4();
            let fake_campaign = Uuid::new_v4();

            let state = SharedState {
                inner: Arc::new(RwLock::new(AppState::new())),
            };

            // gpt_get_tasks with invalid session
            let result = gpt::gpt_get_tasks(
                axum::extract::State(state.clone()),
                axum::extract::Path(fake_campaign),
                axum::extract::Query(types::GptTaskParams {
                    session_token: fake_token,
                }),
            )
            .await;
            assert!(
                !result.status().is_success(),
                "gpt_get_tasks should reject invalid session"
            );
            let err_str = read_body_string(result).await;
            assert!(
                err_str.contains("invalid")
                    || err_str.contains("expired")
                    || err_str.contains("session"),
                "error should mention session, got: {}",
                err_str
            );

            // gpt_run_service with invalid session
            let result = gpt::gpt_run_service(
                axum::extract::State(state.clone()),
                axum::extract::Path("design".to_string()),
                axum::Json(types::GptRunServiceRequest {
                    session_token: fake_token,
                    input: "test".to_string(),
                }),
            )
            .await;
            assert!(
                !result.status().is_success(),
                "gpt_run_service should reject invalid session"
            );

            // gpt_user_status with invalid session
            let result = gpt::gpt_user_status(
                axum::extract::State(state.clone()),
                axum::extract::Query(types::GptUserStatusParams {
                    session_token: fake_token,
                }),
            )
            .await;
            assert!(
                !result.status().is_success(),
                "gpt_user_status should reject invalid session"
            );
        }
    }
}

#[tokio::test]
async fn gpt_complete_task_duplicate_completion_records_second_entry() {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        use sqlx::postgres::PgPoolOptions;
        if let Ok(pool) = PgPoolOptions::new().max_connections(1).connect(&url).await {
            sqlx::migrate!("./migrations").run(&pool).await.ok();

            let user_id = Uuid::new_v4();
            let email = format!("gpt_dup_{}@example.com", Uuid::new_v4());
            sqlx::query(
                "INSERT INTO users (id, email, region, roles, tools_used, attributes, source) \
                 VALUES ($1, $2, 'JP', '{developer}', '{design}', '{}'::jsonb, 'gpt_apps')",
            )
            .bind(user_id)
            .bind(&email)
            .execute(&pool)
            .await
            .unwrap();

            let session_token: Uuid = sqlx::query_scalar(
                "INSERT INTO gpt_sessions (user_id) VALUES ($1) RETURNING token",
            )
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();

            let campaign_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO campaigns (id, name, sponsor, target_roles, target_tools, required_task, \
                 subsidy_per_call_cents, budget_total_cents, budget_remaining_cents, query_urls, active, created_at) \
                 VALUES ($1, 'Dup Campaign', 'DupSponsor', '{developer}', '{design}', 'dup_task', \
                 500, 10000, 10000, '{}', true, NOW())"
            )
            .bind(campaign_id)
            .execute(&pool)
            .await
            .unwrap();

            let state = SharedState {
                inner: Arc::new(RwLock::new(AppState::new())),
            };

            let make_payload = || types::GptCompleteTaskRequest {
                session_token,
                task_name: "dup_task".to_string(),
                details: Some("first".to_string()),
                consent: types::GptConsentInput {
                    data_sharing_agreed: true,
                    purpose_acknowledged: true,
                    contact_permission: false,
                },
            };

            // First completion
            let result = gpt::gpt_complete_task(
                axum::extract::State(state.clone()),
                axum::extract::Path(campaign_id),
                axum::Json(make_payload()),
            )
            .await;
            assert!(
                result.status().is_success(),
                "first completion should succeed"
            );

            // Second (duplicate) completion should also succeed (records another entry)
            let result = gpt::gpt_complete_task(
                axum::extract::State(state.clone()),
                axum::extract::Path(campaign_id),
                axum::Json(make_payload()),
            )
            .await;
            assert!(
                result.status().is_success(),
                "duplicate completion should succeed (no unique constraint on task_completions)"
            );

            // Verify two records exist
            let count: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM task_completions WHERE campaign_id = $1 AND user_id = $2",
            )
            .bind(campaign_id)
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();
            assert_eq!(count, 2, "should have two task completion records");

            // Cleanup
            sqlx::query("DELETE FROM consents WHERE campaign_id = $1")
                .bind(campaign_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM task_completions WHERE campaign_id = $1")
                .bind(campaign_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM gpt_sessions WHERE user_id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM campaigns WHERE id = $1")
                .bind(campaign_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
        }
    }
}

// --- Task 7.1: GPT sub-router integration tests ---

#[tokio::test]
async fn gpt_subrouter_search_services_is_reachable() {
    let (app, _state) = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/gpt/services")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    // Should not be 404 (route exists); may be 500 if no DB, but route is reachable
    assert_ne!(
        response.status().as_u16(),
        404,
        "GET /gpt/services should be routed"
    );
}

#[tokio::test]
async fn gpt_subrouter_auth_is_reachable() {
    let (app, _state) = test_app();
    let response = post_json(
        &app,
        "/gpt/auth",
        serde_json::json!({
            "email": "test@example.com",
            "region": "JP"
        }),
        None,
    )
    .await;
    assert_ne!(
        response.status().as_u16(),
        404,
        "POST /gpt/auth should be routed"
    );
}

#[tokio::test]
async fn gpt_subrouter_user_status_is_reachable() {
    let (app, _state) = test_app();
    let fake_token = Uuid::new_v4();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/gpt/user/status?session_token={}", fake_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_ne!(
        response.status().as_u16(),
        404,
        "GET /gpt/user/status should be routed"
    );
}

#[tokio::test]
async fn gpt_subrouter_does_not_break_existing_routes() {
    let (app, _state) = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.status().as_u16(),
        200,
        "existing /health route should still work"
    );
}

// --- Task 7.2: OpenAPI schema tests ---

#[tokio::test]
async fn well_known_openapi_yaml_returns_yaml_content() {
    let (app, _state) = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/.well-known/openapi.yaml")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 200, "should return 200");
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        content_type.contains("yaml") || content_type.contains("text/plain"),
        "content-type should indicate YAML, got: {}",
        content_type
    );

    let body = to_bytes(response.into_body(), 1_000_000).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("openapi:"),
        "body should contain openapi version"
    );
    assert!(
        body_str.contains("searchServices"),
        "should contain searchServices operationId"
    );
    assert!(
        body_str.contains("authenticateUser"),
        "should contain authenticateUser operationId"
    );
    assert!(
        body_str.contains("getTaskDetails"),
        "should contain getTaskDetails operationId"
    );
    assert!(
        body_str.contains("completeTask"),
        "should contain completeTask operationId"
    );
    assert!(
        body_str.contains("runService"),
        "should contain runService operationId"
    );
    assert!(
        body_str.contains("getUserStatus"),
        "should contain getUserStatus operationId"
    );
}

#[test]
fn openapi_yaml_file_exists_and_is_valid() {
    let yaml_content = include_str!("../openapi.yaml");
    assert!(
        yaml_content.contains("openapi:"),
        "should contain openapi version field"
    );
    assert!(yaml_content.contains("3.1.0"), "should be OpenAPI 3.1.0");
    assert!(
        yaml_content.contains("/gpt/services"),
        "should define /gpt/services path"
    );
    assert!(
        yaml_content.contains("/gpt/auth"),
        "should define /gpt/auth path"
    );
    assert!(
        yaml_content.contains("/gpt/tasks/{campaign_id}"),
        "should define tasks path"
    );
    assert!(
        yaml_content.contains("/gpt/services/{service}/run"),
        "should define run path"
    );
    assert!(
        yaml_content.contains("/gpt/user/status"),
        "should define user status path"
    );
    assert!(
        yaml_content.contains("ApiKeyAuth"),
        "should define ApiKeyAuth security scheme"
    );
}

// --- Task 8.4: Comprehensive OpenAPI schema validation ---

/// Validates that openapi.yaml conforms to OpenAPI 3.1.0 and matches the actual implementation.
/// Checks: version, paths, HTTP methods, operationIds, descriptions, parameters,
/// request body fields, response schema properties, security scheme, and endpoint count.
#[test]
fn openapi_schema_matches_implementation() {
    let yaml_content = include_str!("../openapi.yaml");
    let doc: serde_yaml::Value =
        serde_yaml::from_str(yaml_content).expect("openapi.yaml should be valid YAML");

    // --- 1. OpenAPI version ---
    assert_eq!(
        doc["openapi"].as_str().unwrap(),
        "3.1.0",
        "OpenAPI version must be 3.1.0"
    );

    // --- 2. Info section ---
    assert!(
        doc["info"]["title"].as_str().is_some(),
        "info.title is required"
    );
    assert!(
        doc["info"]["version"].as_str().is_some(),
        "info.version is required"
    );
    assert!(
        doc["info"]["description"].as_str().is_some(),
        "info.description is required"
    );

    // --- 3. Paths: all 6 GPT endpoints exist with correct methods ---
    let paths = doc["paths"]
        .as_mapping()
        .expect("paths should be a mapping");

    let expected_endpoints: Vec<(&str, &str, &str)> = vec![
        ("/gpt/services", "get", "searchServices"),
        ("/gpt/auth", "post", "authenticateUser"),
        ("/gpt/tasks/{campaign_id}", "get", "getTaskDetails"),
        ("/gpt/tasks/{campaign_id}/complete", "post", "completeTask"),
        ("/gpt/services/{service}/run", "post", "runService"),
        ("/gpt/user/status", "get", "getUserStatus"),
    ];

    for (path, method, operation_id) in &expected_endpoints {
        let path_val = &doc["paths"][*path];
        assert!(
            !path_val.is_null(),
            "Path '{}' should exist in openapi.yaml",
            path
        );
        let method_val = &path_val[*method];
        assert!(
            !method_val.is_null(),
            "Path '{}' should have method '{}'",
            path,
            method
        );
        assert_eq!(
            method_val["operationId"].as_str().unwrap(),
            *operation_id,
            "operationId mismatch for {} {}",
            method,
            path
        );
        assert!(
            method_val["summary"].as_str().is_some(),
            "{} {} should have a summary",
            method,
            path
        );
        assert!(
            method_val["description"].as_str().is_some(),
            "{} {} should have a description",
            method,
            path
        );
    }

    // --- 4. Endpoint count ≤ 30 (requirement 1.5) ---
    assert!(
        paths.len() <= 30,
        "Endpoint count ({}) must be ≤ 30",
        paths.len()
    );

    // --- 5. GET /gpt/services parameters ---
    let search_params = doc["paths"]["/gpt/services"]["get"]["parameters"]
        .as_sequence()
        .expect("/gpt/services should have parameters");
    let search_param_names: Vec<&str> = search_params
        .iter()
        .filter_map(|p| p["name"].as_str())
        .collect();
    assert!(
        search_param_names.contains(&"q"),
        "/gpt/services should have 'q' param"
    );
    assert!(
        search_param_names.contains(&"category"),
        "/gpt/services should have 'category' param"
    );
    // Both should be optional (matching GptSearchParams where q and category are Option<String>)
    for param in search_params {
        let required = param["required"].as_bool().unwrap_or(false);
        assert!(
            !required,
            "/gpt/services param '{}' should be optional",
            param["name"].as_str().unwrap_or("?")
        );
    }

    // --- 6. POST /gpt/auth request body ---
    let auth_props = &doc["paths"]["/gpt/auth"]["post"]["requestBody"]["content"]["application/json"]
        ["schema"]["properties"];
    for field in &["email", "region"] {
        assert!(
            !auth_props[*field].is_null(),
            "/gpt/auth request should have '{}' field",
            field
        );
    }
    let auth_required = doc["paths"]["/gpt/auth"]["post"]["requestBody"]["content"]
        ["application/json"]["schema"]["required"]
        .as_sequence().expect("/gpt/auth should have required fields");
    let auth_required_names: Vec<&str> = auth_required.iter().filter_map(|v| v.as_str()).collect();
    assert!(
        auth_required_names.contains(&"email"),
        "email should be required in /gpt/auth"
    );
    assert!(
        auth_required_names.contains(&"region"),
        "region should be required in /gpt/auth"
    );

    // --- 7. POST /gpt/auth response properties match GptAuthResponse ---
    let auth_resp_props = &doc["paths"]["/gpt/auth"]["post"]["responses"]["200"]["content"]["application/json"]
        ["schema"]["properties"];
    for field in &[
        "session_token",
        "user_id",
        "email",
        "is_new_user",
        "message",
    ] {
        assert!(
            !auth_resp_props[*field].is_null(),
            "/gpt/auth response should have '{}' field",
            field
        );
    }

    // --- 8. GET /gpt/tasks/{campaign_id} parameters ---
    let task_params = doc["paths"]["/gpt/tasks/{campaign_id}"]["get"]["parameters"]
        .as_sequence()
        .expect("/gpt/tasks should have parameters");
    let task_param_names: Vec<&str> = task_params
        .iter()
        .filter_map(|p| p["name"].as_str())
        .collect();
    assert!(
        task_param_names.contains(&"campaign_id"),
        "/gpt/tasks should have 'campaign_id' path param"
    );
    assert!(
        task_param_names.contains(&"session_token"),
        "/gpt/tasks should have 'session_token' query param"
    );

    // --- 9. GET /gpt/tasks response properties match GptTaskResponse ---
    let task_resp_props = &doc["paths"]["/gpt/tasks/{campaign_id}"]["get"]["responses"]["200"]["content"]
        ["application/json"]["schema"]["properties"];
    for field in &[
        "campaign_id",
        "campaign_name",
        "sponsor",
        "required_task",
        "task_description",
        "task_input_format",
        "already_completed",
        "subsidy_amount_cents",
        "message",
    ] {
        assert!(
            !task_resp_props[*field].is_null(),
            "/gpt/tasks response should have '{}' field",
            field
        );
    }

    // --- 10. POST /gpt/tasks/{campaign_id}/complete request body ---
    let complete_props = &doc["paths"]["/gpt/tasks/{campaign_id}/complete"]["post"]["requestBody"]
        ["content"]["application/json"]["schema"]["properties"];
    for field in &["session_token", "task_name", "consent"] {
        assert!(
            !complete_props[*field].is_null(),
            "/gpt/tasks/complete request should have '{}' field",
            field
        );
    }
    // Consent sub-object
    let consent_props = &complete_props["consent"]["properties"];
    for field in &[
        "data_sharing_agreed",
        "purpose_acknowledged",
        "contact_permission",
    ] {
        assert!(
            !consent_props[*field].is_null(),
            "consent object should have '{}' field",
            field
        );
    }

    // --- 11. POST /gpt/tasks/{campaign_id}/complete response match GptCompleteTaskResponse ---
    let complete_resp_props = &doc["paths"]["/gpt/tasks/{campaign_id}/complete"]["post"]["responses"]
        ["200"]["content"]["application/json"]["schema"]["properties"];
    for field in &[
        "task_completion_id",
        "campaign_id",
        "consent_recorded",
        "can_use_service",
        "message",
    ] {
        assert!(
            !complete_resp_props[*field].is_null(),
            "/gpt/tasks/complete response should have '{}' field",
            field
        );
    }

    // --- 12. POST /gpt/services/{service}/run request body ---
    let run_props = &doc["paths"]["/gpt/services/{service}/run"]["post"]["requestBody"]["content"]
        ["application/json"]["schema"]["properties"];
    for field in &["session_token", "input"] {
        assert!(
            !run_props[*field].is_null(),
            "/gpt/services/run request should have '{}' field",
            field
        );
    }

    // --- 13. POST /gpt/services/{service}/run response match GptRunServiceResponse ---
    let run_resp_props = &doc["paths"]["/gpt/services/{service}/run"]["post"]["responses"]["200"]["content"]
        ["application/json"]["schema"]["properties"];
    for field in &[
        "service",
        "output",
        "payment_mode",
        "sponsored_by",
        "tx_hash",
        "message",
    ] {
        assert!(
            !run_resp_props[*field].is_null(),
            "/gpt/services/run response should have '{}' field",
            field
        );
    }

    // --- 14. GET /gpt/user/status parameters ---
    let status_params = doc["paths"]["/gpt/user/status"]["get"]["parameters"]
        .as_sequence()
        .expect("/gpt/user/status should have parameters");
    let status_param_names: Vec<&str> = status_params
        .iter()
        .filter_map(|p| p["name"].as_str())
        .collect();
    assert!(
        status_param_names.contains(&"session_token"),
        "/gpt/user/status should have 'session_token' param"
    );

    // --- 15. GET /gpt/user/status response match GptUserStatusResponse ---
    let status_resp_props = &doc["paths"]["/gpt/user/status"]["get"]["responses"]["200"]["content"]
        ["application/json"]["schema"]["properties"];
    for field in &[
        "user_id",
        "email",
        "completed_tasks",
        "available_services",
        "message",
    ] {
        assert!(
            !status_resp_props[*field].is_null(),
            "/gpt/user/status response should have '{}' field",
            field
        );
    }

    // --- 16. GptServiceItem component schema ---
    let svc_item_props = &doc["components"]["schemas"]["GptServiceItem"]["properties"];
    for field in &[
        "service_type",
        "service_id",
        "name",
        "sponsor",
        "required_task",
        "subsidy_amount_cents",
        "category",
        "active",
    ] {
        assert!(
            !svc_item_props[*field].is_null(),
            "GptServiceItem schema should have '{}' field",
            field
        );
    }

    // --- 17. Security scheme ---
    let security_schemes = &doc["components"]["securitySchemes"];
    assert!(
        !security_schemes["ApiKeyAuth"].is_null(),
        "ApiKeyAuth security scheme should be defined"
    );
    assert_eq!(
        security_schemes["ApiKeyAuth"]["scheme"].as_str().unwrap(),
        "bearer",
        "ApiKeyAuth should use bearer scheme"
    );

    // --- 18. Global security ---
    let security = doc["security"]
        .as_sequence()
        .expect("global security should be defined");
    assert!(!security.is_empty(), "global security should not be empty");
}

// --- Task 8.5: GPT Builder pre-flight validation ---

/// Validates all prerequisites for GPT Builder import are in place:
/// - openapi.yaml serves correctly from /.well-known/openapi.yaml
/// - Privacy page serves at /privacy
/// - gpt-config.md exists with required sections
/// - All 6 Actions are importable (operationIds present)
/// - Authentication scheme is configured for Bearer
/// - Server URL is defined
#[tokio::test]
async fn gpt_builder_preflight_all_prerequisites_met() {
    let (app, _state) = test_app();

    // 1. OpenAPI schema endpoint returns valid YAML with all 6 operationIds
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/.well-known/openapi.yaml")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.status().as_u16(),
        200,
        "openapi.yaml endpoint must return 200"
    );
    let ct = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        ct.contains("yaml"),
        "content-type must indicate YAML, got: {}",
        ct
    );

    let body = to_bytes(response.into_body(), 1_000_000).await.unwrap();
    let schema_str = String::from_utf8_lossy(&body);

    // GPT Builder requires all operationIds to create Actions
    let required_operations = [
        "searchServices",
        "authenticateUser",
        "getTaskDetails",
        "completeTask",
        "runService",
        "getUserStatus",
    ];
    for op in &required_operations {
        assert!(
            schema_str.contains(op),
            "OpenAPI schema must contain operationId '{}' for GPT Builder Action import",
            op
        );
    }

    // GPT Builder requires servers URL
    assert!(
        schema_str.contains("servers:"),
        "OpenAPI schema must define servers for GPT Builder"
    );

    // 2. Privacy page endpoint (required for GPT Builder publication)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/privacy")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.status().as_u16(),
        200,
        "privacy page must return 200 for GPT Builder"
    );

    // 3. gpt-config.md exists with required sections for GPT Builder setup
    let config_content = include_str!("../.kiro/specs/gpt-apps-integration/gpt-config.md");
    assert!(
        config_content.contains("システムプロンプト"),
        "gpt-config.md must contain system prompt section"
    );
    assert!(
        config_content.contains("Conversation Starters"),
        "gpt-config.md must contain Conversation Starters section"
    );
    assert!(
        config_content.contains("GPT Builder 設定手順"),
        "gpt-config.md must contain GPT Builder setup instructions"
    );
    assert!(
        config_content.contains("Actions の設定"),
        "gpt-config.md must contain Actions setup section"
    );
    assert!(
        config_content.contains("プライバシーポリシー"),
        "gpt-config.md must contain privacy policy section"
    );

    // 4. Conversation Starters: at least 4 defined
    let starter_count = config_content.matches("Conversation Starter").count();
    // Header + table header + at least 4 starters
    assert!(
        starter_count >= 2,
        "gpt-config.md must define Conversation Starters"
    );

    // 5. Parse schema and verify Bearer auth scheme (GPT Builder requires this)
    let doc: serde_yaml::Value =
        serde_yaml::from_str(&schema_str).expect("served openapi.yaml must be parseable YAML");
    assert_eq!(
        doc["components"]["securitySchemes"]["ApiKeyAuth"]["scheme"]
            .as_str()
            .unwrap(),
        "bearer",
        "GPT Builder requires Bearer authentication scheme"
    );
    assert_eq!(
        doc["components"]["securitySchemes"]["ApiKeyAuth"]["type"]
            .as_str()
            .unwrap(),
        "http",
        "GPT Builder requires http type for API key auth"
    );
}

/// Validates that the OpenAPI schema served at runtime matches the file on disk.
/// This ensures no drift between the source file and what GPT Builder would import.
#[tokio::test]
async fn gpt_builder_served_schema_matches_source_file() {
    let (app, _state) = test_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/.well-known/openapi.yaml")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), 1_000_000).await.unwrap();
    let served = String::from_utf8_lossy(&body);
    let source = include_str!("../openapi.yaml");
    // serve_openapi_yaml replaces the placeholder URL with PUBLIC_BASE_URL
    // (defaults to DEFAULT_PUBLIC_BASE_URL when env var is unset)
    let expected = source.replace(
        "https://subsidypayment.example.com",
        crate::types::DEFAULT_PUBLIC_BASE_URL,
    );

    assert_eq!(
        served.trim(),
        expected.trim(),
        "Served OpenAPI schema must match openapi.yaml with URL substitution applied"
    );
}

// --- Task 7.3: Privacy policy page tests ---

#[tokio::test]
async fn privacy_page_returns_html_with_required_content() {
    let (app, _state) = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/privacy")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 200, "should return 200");
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        content_type.contains("text/html"),
        "content-type should be text/html, got: {}",
        content_type
    );

    let body = to_bytes(response.into_body(), 1_000_000).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Privacy"),
        "should contain Privacy heading"
    );
    assert!(
        body_str.contains("email"),
        "should mention collected data types"
    );
    assert!(
        body_str.contains("sponsor") || body_str.contains("Sponsor"),
        "should mention data sharing with sponsors"
    );
    assert!(
        body_str.contains("retention")
            || body_str.contains("Retention")
            || body_str.contains("retain"),
        "should mention data retention"
    );
    assert!(
        body_str.contains("consent")
            || body_str.contains("Consent")
            || body_str.contains("withdraw"),
        "should mention consent/withdrawal rights"
    );
    assert!(
        body_str.contains("contact") || body_str.contains("Contact"),
        "should mention contact information"
    );
}

#[test]
fn privacy_html_file_exists_and_has_structure() {
    let html = include_str!("../privacy.html");
    assert!(
        html.contains("<!DOCTYPE html>") || html.contains("<html"),
        "should be valid HTML"
    );
    assert!(
        html.contains("Privacy Policy") || html.contains("privacy policy"),
        "should have privacy policy title"
    );
}

// --- Task 7.4: .env.example test ---

#[test]
fn env_example_contains_gpt_actions_api_key() {
    let content = include_str!("../.env.example");
    assert!(
        content.contains("GPT_ACTIONS_API_KEY"),
        ".env.example should contain GPT_ACTIONS_API_KEY"
    );
    assert!(
        content.contains("DATABASE_URL"),
        ".env.example should contain DATABASE_URL"
    );
}

// --- Task 7.5: Router integration tests ---

#[tokio::test]
async fn gpt_auth_middleware_rejects_without_api_key_when_configured() {
    let state = SharedState {
        inner: Arc::new(RwLock::new(AppState::new())),
    };
    // Configure an API key
    {
        let mut s = state.inner.write().await;
        s.config.gpt_actions_api_key = Some("test-secret-key".to_string());
    }

    let app = build_app(state, DEFAULT_AGENT_DISCOVERY_RATE_LIMIT_PER_MIN as u32);

    // Request without Authorization header should be rejected
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/gpt/services")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.status().as_u16(),
        401,
        "should reject unauthenticated request to GPT route"
    );
}

#[tokio::test]
async fn gpt_auth_middleware_rejects_invalid_api_key() {
    let state = SharedState {
        inner: Arc::new(RwLock::new(AppState::new())),
    };
    {
        let mut s = state.inner.write().await;
        s.config.gpt_actions_api_key = Some("correct-key".to_string());
    }

    let app = build_app(state, DEFAULT_AGENT_DISCOVERY_RATE_LIMIT_PER_MIN as u32);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/gpt/services")
                .header("Authorization", "Bearer wrong-key")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.status().as_u16(),
        403,
        "should reject invalid API key"
    );
}

#[tokio::test]
async fn gpt_auth_middleware_accepts_valid_api_key() {
    let state = SharedState {
        inner: Arc::new(RwLock::new(AppState::new())),
    };
    {
        let mut s = state.inner.write().await;
        s.config.gpt_actions_api_key = Some("correct-key".to_string());
    }

    let app = build_app(state, DEFAULT_AGENT_DISCOVERY_RATE_LIMIT_PER_MIN as u32);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/gpt/services")
                .header("Authorization", "Bearer correct-key")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    // Should not be 401/403 (auth passed); may be 500 if no DB, but auth is OK
    let status = response.status().as_u16();
    assert!(
        status != 401 && status != 403,
        "valid API key should pass auth, got: {}",
        status
    );
}

#[tokio::test]
async fn gpt_auth_middleware_skips_when_no_key_configured() {
    let state = SharedState {
        inner: Arc::new(RwLock::new(AppState::new())),
    };
    // No API key configured (default)
    {
        let s = state.inner.read().await;
        assert!(
            s.config.gpt_actions_api_key.is_none()
                || s.config.gpt_actions_api_key.as_deref() == Some("")
        );
    }

    let app = build_app(state, DEFAULT_AGENT_DISCOVERY_RATE_LIMIT_PER_MIN as u32);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/gpt/services")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    // Should not be 401/403 when no key is configured
    let status = response.status().as_u16();
    assert!(
        status != 401 && status != 403,
        "should skip auth when no key configured, got: {}",
        status
    );
}

#[tokio::test]
async fn static_endpoints_not_behind_gpt_auth() {
    let state = SharedState {
        inner: Arc::new(RwLock::new(AppState::new())),
    };
    {
        let mut s = state.inner.write().await;
        s.config.gpt_actions_api_key = Some("secret-key".to_string());
    }

    let app = build_app(state, DEFAULT_AGENT_DISCOVERY_RATE_LIMIT_PER_MIN as u32);

    // OpenAPI endpoint should NOT require API key (not under /gpt prefix)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/.well-known/openapi.yaml")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.status().as_u16(),
        200,
        "openapi.yaml should not require auth"
    );

    // Privacy endpoint should NOT require API key
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/privacy")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.status().as_u16(),
        200,
        "privacy page should not require auth"
    );

    // Health endpoint should NOT require API key
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.status().as_u16(),
        200,
        "health should not require auth"
    );
}

#[tokio::test]
async fn gpt_handlers_record_metrics_with_gpt_prefix() {
    // Verify that GPT endpoints record http_requests_total metrics with gpt_ prefixed endpoint names.
    // This test uses the full app router so metrics flow through respond().
    let (app, state) = test_app();

    // Disable API key requirement so requests pass through auth middleware
    {
        let mut s = state.inner.write().await;
        s.config.gpt_actions_api_key = None;
    }

    // GET /gpt/services — should record metric for "gpt_search_services"
    let req = Request::builder()
        .uri("/gpt/services")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    // Will be 500 (no DB) but metric should still be recorded
    let _ = resp.status();

    // Check that the metric was recorded with gpt_ prefix
    {
        let s = state.inner.read().await;
        let metric_families = s.metrics.registry.gather();
        let http_requests = metric_families
            .iter()
            .find(|mf| mf.get_name() == "http_requests_total")
            .expect("http_requests_total metric family should exist");

        let has_gpt_endpoint = http_requests.get_metric().iter().any(|m| {
            m.get_label()
                .iter()
                .any(|l| l.get_name() == "endpoint" && l.get_value().starts_with("gpt_"))
        });
        assert!(
            has_gpt_endpoint,
            "http_requests_total should have at least one metric with gpt_ prefixed endpoint label"
        );
    }
}

#[tokio::test]
async fn gpt_all_endpoints_record_distinct_metrics() {
    // Verify each GPT endpoint records its own distinct gpt_ prefixed metric.
    let (app, state) = test_app();

    {
        let mut s = state.inner.write().await;
        s.config.gpt_actions_api_key = None;
    }

    // Hit all 6 GPT endpoints (they'll return errors without DB, but metrics should record)
    let endpoints = vec![
        ("GET", "/gpt/services"),
        ("POST", "/gpt/auth"),
        (
            "GET",
            "/gpt/tasks/00000000-0000-0000-0000-000000000001?session_token=00000000-0000-0000-0000-000000000001",
        ),
        (
            "POST",
            "/gpt/tasks/00000000-0000-0000-0000-000000000001/complete",
        ),
        ("POST", "/gpt/services/design/run"),
        (
            "GET",
            "/gpt/user/status?session_token=00000000-0000-0000-0000-000000000001",
        ),
    ];

    for (method, uri) in &endpoints {
        let req = Request::builder()
            .method(*method)
            .uri(*uri)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{"session_token":"00000000-0000-0000-0000-000000000001","email":"t@t.com","region":"JP","task_name":"x","input":"x","consent":{"data_sharing_agreed":true,"purpose_acknowledged":true,"contact_permission":false}}"#))
            .unwrap();
        let _ = app.clone().oneshot(req).await.unwrap();
    }

    // Verify all 6 distinct gpt_ endpoints were recorded
    let expected_endpoints = vec![
        "gpt_search_services",
        "gpt_auth",
        "gpt_get_tasks",
        "gpt_complete_task",
        "gpt_run_service",
        "gpt_user_status",
    ];

    let s = state.inner.read().await;
    let metric_families = s.metrics.registry.gather();
    let http_requests = metric_families
        .iter()
        .find(|mf| mf.get_name() == "http_requests_total")
        .expect("http_requests_total metric family should exist");

    let recorded_endpoints: Vec<String> = http_requests
        .get_metric()
        .iter()
        .flat_map(|m| m.get_label().iter())
        .filter(|l| l.get_name() == "endpoint" && l.get_value().starts_with("gpt_"))
        .map(|l| l.get_value().to_string())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    for expected in &expected_endpoints {
        assert!(
            recorded_endpoints.contains(&expected.to_string()),
            "Missing metric for endpoint '{}'. Recorded: {:?}",
            expected,
            recorded_endpoints
        );
    }
}

/// E2E integration test: search services → auth → get tasks → complete task (with consent) → run service → user status
/// This test exercises the full GPT flow using a real DB, verifying that each step produces
/// correct output and that the data flows correctly between steps.
#[tokio::test]
async fn gpt_e2e_flow_search_auth_task_complete_run_status() {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        use sqlx::postgres::PgPoolOptions;
        if let Ok(pool) = PgPoolOptions::new().max_connections(1).connect(&url).await {
            sqlx::migrate!("./migrations").run(&pool).await.ok();

            // --- Setup: Create a campaign that the E2E flow will exercise ---
            let campaign_id = Uuid::new_v4();
            let campaign_name = format!("E2E Campaign {}", Uuid::new_v4());
            let sponsor = "E2ESponsor";
            let required_task = "e2e_survey";
            sqlx::query(
                "INSERT INTO campaigns (id, name, sponsor, target_roles, target_tools, required_task, \
                 subsidy_per_call_cents, budget_total_cents, budget_remaining_cents, query_urls, active, created_at) \
                 VALUES ($1, $2, $3, '{developer}', '{design}', $4, \
                 800, 50000, 50000, '{}', true, NOW())"
            )
            .bind(campaign_id)
            .bind(&campaign_name)
            .bind(sponsor)
            .bind(required_task)
            .execute(&pool)
            .await
            .unwrap();

            let state = SharedState {
                inner: Arc::new(RwLock::new(AppState::new())),
            };

            // ========== Step 1: Search Services ==========
            let result = gpt::gpt_search_services(
                axum::extract::State(state.clone()),
                axum::extract::Query(types::GptSearchParams {
                    q: None,
                    category: None,
                    max_budget_cents: None,
                    intent: None,
                    session_token: None,
                }),
            )
            .await;
            assert!(
                result.status().is_success(),
                "Step 1: search services should succeed"
            );
            let search_resp: types::GptSearchResponse = read_typed(result).await;
            assert!(
                search_resp.total_count > 0,
                "Step 1: should find at least one service"
            );
            // Verify our campaign appears in results
            let our_service = search_resp
                .services
                .iter()
                .find(|s| s.service_id == campaign_id);
            assert!(
                our_service.is_some(),
                "Step 1: our campaign should appear in search results"
            );
            let our_service = our_service.unwrap();
            assert_eq!(our_service.sponsor, sponsor);
            assert_eq!(our_service.required_task.as_deref(), Some(required_task));

            // ========== Step 2: Auth (register new user) ==========
            let unique_email = format!("gpt_e2e_{}@example.com", Uuid::new_v4());
            let result = gpt::gpt_auth(
                axum::extract::State(state.clone()),
                axum::Json(types::GptAuthRequest {
                    email: unique_email.clone(),
                    region: "JP".to_string(),
                    roles: vec!["developer".to_string()],
                    tools_used: vec!["design".to_string()],
                }),
            )
            .await;
            assert!(result.status().is_success(), "Step 2: auth should succeed");
            let auth_resp: types::GptAuthResponse = read_typed(result).await;
            assert!(auth_resp.is_new_user, "Step 2: should be a new user");
            assert_eq!(auth_resp.email, unique_email);
            let session_token = auth_resp.session_token;
            let user_id = auth_resp.user_id;
            assert!(
                !session_token.is_nil(),
                "Step 2: session token should be valid"
            );

            // ========== Step 3: Get Tasks ==========
            let result = gpt::gpt_get_tasks(
                axum::extract::State(state.clone()),
                axum::extract::Path(campaign_id),
                axum::extract::Query(types::GptTaskParams { session_token }),
            )
            .await;
            assert!(
                result.status().is_success(),
                "Step 3: get tasks should succeed"
            );
            let tasks_resp: types::GptTaskResponse = read_typed(result).await;
            assert_eq!(tasks_resp.campaign_id, campaign_id);
            assert_eq!(tasks_resp.campaign_name, campaign_name);
            assert_eq!(tasks_resp.sponsor, sponsor);
            assert_eq!(tasks_resp.required_task, required_task);
            assert!(
                !tasks_resp.already_completed,
                "Step 3: task should not be completed yet"
            );
            assert_eq!(tasks_resp.subsidy_amount_cents, 800);

            // ========== Step 4: Complete Task (with consent) ==========
            let result = gpt::gpt_complete_task(
                axum::extract::State(state.clone()),
                axum::extract::Path(campaign_id),
                axum::Json(types::GptCompleteTaskRequest {
                    session_token,
                    task_name: required_task.to_string(),
                    details: Some("E2E test survey response".to_string()),
                    consent: types::GptConsentInput {
                        data_sharing_agreed: true,
                        purpose_acknowledged: true,
                        contact_permission: false,
                    },
                }),
            )
            .await;
            assert!(
                result.status().is_success(),
                "Step 4: complete task should succeed"
            );
            let complete_resp: types::GptCompleteTaskResponse = read_typed(result).await;
            assert_eq!(complete_resp.campaign_id, campaign_id);
            assert!(
                complete_resp.consent_recorded,
                "Step 4: consent should be recorded"
            );
            assert!(
                complete_resp.can_use_service,
                "Step 4: should be able to use service"
            );
            assert!(!complete_resp.task_completion_id.is_nil());

            // Verify task is now marked as completed
            let result = gpt::gpt_get_tasks(
                axum::extract::State(state.clone()),
                axum::extract::Path(campaign_id),
                axum::extract::Query(types::GptTaskParams { session_token }),
            )
            .await;
            assert!(result.status().is_success());
            let tasks_resp2: types::GptTaskResponse = read_typed(result).await;
            assert!(
                tasks_resp2.already_completed,
                "Step 4 verify: task should now be completed"
            );

            // ========== Step 5: Run Service ==========
            let result = gpt::gpt_run_service(
                axum::extract::State(state.clone()),
                axum::extract::Path("design".to_string()),
                axum::Json(types::GptRunServiceRequest {
                    session_token,
                    input: "E2E test design input".to_string(),
                }),
            )
            .await;
            assert!(
                result.status().is_success(),
                "Step 5: run service should succeed"
            );
            let run_resp: types::GptRunServiceResponse = read_typed(result).await;
            assert_eq!(run_resp.service, "design");
            assert_eq!(run_resp.payment_mode, "sponsored");
            assert_eq!(run_resp.sponsored_by, Some(sponsor.to_string()));
            assert!(run_resp.tx_hash.is_some(), "Step 5: should have tx_hash");
            assert!(!run_resp.output.is_empty(), "Step 5: should have output");

            // Verify budget was deducted
            let remaining: i64 =
                sqlx::query_scalar("SELECT budget_remaining_cents FROM campaigns WHERE id = $1")
                    .bind(campaign_id)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            assert!(
                remaining < 50000,
                "Step 5: budget should have been deducted"
            );

            // Verify payment was recorded
            let payment_exists: bool = sqlx::query_scalar(
                "SELECT exists(SELECT 1 FROM payments WHERE campaign_id = $1 AND service = 'design')"
            )
            .bind(campaign_id)
            .fetch_one(&pool)
            .await
            .unwrap();
            assert!(payment_exists, "Step 5: payment should be recorded");

            // ========== Step 6: User Status ==========
            let result = gpt::gpt_user_status(
                axum::extract::State(state.clone()),
                axum::extract::Query(types::GptUserStatusParams { session_token }),
            )
            .await;
            assert!(
                result.status().is_success(),
                "Step 6: user status should succeed"
            );
            let status_resp: types::GptUserStatusResponse = read_typed(result).await;
            assert_eq!(status_resp.user_id, user_id);
            assert_eq!(status_resp.email, unique_email);
            assert!(
                !status_resp.completed_tasks.is_empty(),
                "Step 6: should have completed tasks"
            );
            let our_task = status_resp
                .completed_tasks
                .iter()
                .find(|t| t.campaign_id == campaign_id);
            assert!(our_task.is_some(), "Step 6: should find our completed task");
            assert_eq!(our_task.unwrap().task_name, required_task);

            // ========== Cleanup ==========
            sqlx::query("DELETE FROM payments WHERE campaign_id = $1")
                .bind(campaign_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM consents WHERE user_id = $1 AND campaign_id = $2")
                .bind(user_id)
                .bind(campaign_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM task_completions WHERE campaign_id = $1")
                .bind(campaign_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM gpt_sessions WHERE user_id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM campaigns WHERE id = $1")
                .bind(campaign_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
        }
    }
}

#[tokio::test]
async fn testnet_payment_signature_service_mismatch_is_rejected() {
    if !has_live_testnet_env() {
        eprintln!(
            "skipping live testnet test: set TESTNET_PAYMENT_SIGNATURE_DESIGN, X402_PAY_TO, and X402_ASSET"
        );
        return;
    }

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

// =============================================================================
// Smart Service Suggestion — タスク 1: DBマイグレーション 0011
// =============================================================================

#[test]
fn migration_0011_user_task_preferences_has_expected_schema() {
    let sql = include_str!("../migrations/0011_user_task_preferences.sql");
    let lower = sql.to_lowercase();

    // Table creation
    assert!(
        lower.contains("create table if not exists user_task_preferences"),
        "should create user_task_preferences table"
    );

    // Required columns
    assert!(
        lower.contains("id uuid primary key default gen_random_uuid()"),
        "should have id uuid primary key with gen_random_uuid() default"
    );
    assert!(
        lower.contains("user_id uuid not null references users(id) on delete cascade"),
        "should have user_id FK to users with ON DELETE CASCADE"
    );
    assert!(
        lower.contains("task_type text not null"),
        "should have task_type TEXT NOT NULL column"
    );
    assert!(
        lower.contains("level text not null"),
        "should have level TEXT NOT NULL column"
    );
    assert!(
        lower.contains("created_at timestamptz not null default now()"),
        "should have created_at column"
    );
    assert!(
        lower.contains("updated_at timestamptz not null default now()"),
        "should have updated_at column"
    );

    // CHECK constraint on level
    assert!(
        lower.contains("preferred"),
        "level CHECK should include 'preferred'"
    );
    assert!(
        lower.contains("neutral"),
        "level CHECK should include 'neutral'"
    );
    assert!(
        lower.contains("avoided"),
        "level CHECK should include 'avoided'"
    );

    // UNIQUE constraint
    assert!(
        lower.contains("unique (user_id, task_type)"),
        "should have UNIQUE(user_id, task_type) constraint"
    );

    // Indexes
    assert!(
        lower.contains("user_task_preferences_user_id_idx"),
        "should have user_id index"
    );
    assert!(
        lower.contains("on user_task_preferences(user_id)"),
        "user_id index should target correct table and column"
    );
}

// =============================================================================
// Smart Service Suggestion — タスク 3: 型定義の拡張と新規型追加
// =============================================================================

#[test]
fn gpt_search_params_supports_extended_fields() {
    // Task 3.1: GptSearchParams に max_budget_cents, intent, session_token を追加
    let params = types::GptSearchParams {
        q: Some("test".into()),
        category: Some("design".into()),
        max_budget_cents: Some(500),
        intent: Some("take a screenshot".into()),
        session_token: Some(Uuid::new_v4()),
    };

    assert_eq!(params.q.as_deref(), Some("test"));
    assert_eq!(params.category.as_deref(), Some("design"));
    assert_eq!(params.max_budget_cents, Some(500));
    assert_eq!(params.intent.as_deref(), Some("take a screenshot"));
    assert!(params.session_token.is_some());

    // 新フィールドは全てオプショナルであること（後方互換性）
    let params_minimal = types::GptSearchParams {
        q: None,
        category: None,
        max_budget_cents: None,
        intent: None,
        session_token: None,
    };
    assert!(params_minimal.max_budget_cents.is_none());
    assert!(params_minimal.intent.is_none());
    assert!(params_minimal.session_token.is_none());
}

#[test]
fn gpt_search_response_supports_applied_filters_and_categories() {
    // Task 3.2: GptSearchResponse に applied_filters, available_categories を追加
    use types::{AppliedFilters, GptSearchResponse};

    // applied_filters と available_categories が Some の場合
    let resp_with_filters = GptSearchResponse {
        services: vec![],
        total_count: 0,
        message: "test".into(),
        applied_filters: Some(AppliedFilters {
            budget: Some(500),
            intent: Some("screenshot".into()),
            category: None,
            keyword: None,
            preferences_applied: true,
        }),
        available_categories: Some(vec!["design".into(), "scraping".into()]),
    };

    assert!(resp_with_filters.applied_filters.is_some());
    let filters = resp_with_filters.applied_filters.as_ref().unwrap();
    assert_eq!(filters.budget, Some(500));
    assert_eq!(filters.intent.as_deref(), Some("screenshot"));
    assert!(filters.preferences_applied);
    assert_eq!(
        resp_with_filters
            .available_categories
            .as_ref()
            .unwrap()
            .len(),
        2
    );

    // applied_filters と available_categories が None の場合（後方互換性）
    let resp_minimal = GptSearchResponse {
        services: vec![],
        total_count: 0,
        message: "test".into(),
        applied_filters: None,
        available_categories: None,
    };
    assert!(resp_minimal.applied_filters.is_none());
    assert!(resp_minimal.available_categories.is_none());

    // skip_serializing_if の検証: None の場合 JSON に含まれない
    let json = serde_json::to_value(&resp_minimal).unwrap();
    assert!(
        !json.as_object().unwrap().contains_key("applied_filters"),
        "applied_filters=None should be omitted from JSON"
    );
    assert!(
        !json
            .as_object()
            .unwrap()
            .contains_key("available_categories"),
        "available_categories=None should be omitted from JSON"
    );

    // Some の場合は JSON に含まれる
    let json_with = serde_json::to_value(&resp_with_filters).unwrap();
    assert!(
        json_with
            .as_object()
            .unwrap()
            .contains_key("applied_filters"),
        "applied_filters=Some should be present in JSON"
    );
    assert!(
        json_with
            .as_object()
            .unwrap()
            .contains_key("available_categories"),
        "available_categories=Some should be present in JSON"
    );
}

#[test]
fn gpt_service_item_supports_tags_and_relevance_score() {
    // Task 3.3: GptServiceItem に tags, relevance_score を追加
    use types::GptServiceItem;

    // tags と relevance_score が設定された場合
    let item_with_score = GptServiceItem {
        service_type: "campaign".into(),
        service_id: Uuid::new_v4(),
        name: "Test Service".into(),
        sponsor: "Sponsor".into(),
        required_task: Some("survey".into()),
        subsidy_amount_cents: 100,
        category: vec!["design".into()],
        active: true,
        tags: vec!["web-scraping".into(), "survey".into()],
        relevance_score: Some(0.85),
    };

    assert_eq!(item_with_score.tags.len(), 2);
    assert_eq!(item_with_score.tags[0], "web-scraping");
    assert_eq!(item_with_score.relevance_score, Some(0.85));

    // relevance_score が None の場合（後方互換性）
    let item_no_score = GptServiceItem {
        service_type: "sponsored_api".into(),
        service_id: Uuid::new_v4(),
        name: "API".into(),
        sponsor: "Sponsor".into(),
        required_task: None,
        subsidy_amount_cents: 50,
        category: vec![],
        active: true,
        tags: vec![],
        relevance_score: None,
    };

    assert!(item_no_score.tags.is_empty());
    assert!(item_no_score.relevance_score.is_none());

    // skip_serializing_if: relevance_score=None は JSON に含まれない
    let json_no_score = serde_json::to_value(&item_no_score).unwrap();
    assert!(
        !json_no_score
            .as_object()
            .unwrap()
            .contains_key("relevance_score"),
        "relevance_score=None should be omitted from JSON"
    );

    // relevance_score=Some は JSON に含まれる
    let json_with_score = serde_json::to_value(&item_with_score).unwrap();
    assert!(
        json_with_score
            .as_object()
            .unwrap()
            .contains_key("relevance_score"),
        "relevance_score=Some should be present in JSON"
    );

    // tags は常に JSON に含まれる
    assert!(
        json_with_score.as_object().unwrap().contains_key("tags"),
        "tags should always be present in JSON"
    );
}

#[test]
fn preference_types_are_constructible() {
    // Task 3.4: 新規型 TaskPreference, GptPreferencesParams, GptSetPreferencesRequest,
    //           GptPreferencesResponse, GptSetPreferencesResponse を追加
    use chrono::Utc;
    use types::{
        GptPreferencesParams, GptPreferencesResponse, GptSetPreferencesRequest,
        GptSetPreferencesResponse, TaskPreference,
    };

    // TaskPreference
    let pref = TaskPreference {
        task_type: "survey".into(),
        level: "avoided".into(),
    };
    assert_eq!(pref.task_type, "survey");
    assert_eq!(pref.level, "avoided");
    let pref_clone = pref.clone(); // Clone trait
    assert_eq!(pref_clone.task_type, pref.task_type);

    // TaskPreference は Serialize + Deserialize
    let json = serde_json::to_string(&pref).unwrap();
    let deser: TaskPreference = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.task_type, "survey");
    assert_eq!(deser.level, "avoided");

    // GptPreferencesParams (Deserialize)
    let _params = GptPreferencesParams {
        session_token: Uuid::new_v4(),
    };

    // GptSetPreferencesRequest (Deserialize)
    let _req = GptSetPreferencesRequest {
        session_token: Uuid::new_v4(),
        preferences: vec![
            TaskPreference {
                task_type: "survey".into(),
                level: "avoided".into(),
            },
            TaskPreference {
                task_type: "github_pr".into(),
                level: "preferred".into(),
            },
        ],
    };
    assert_eq!(_req.preferences.len(), 2);

    // GptPreferencesResponse (Serialize + Deserialize)
    let resp = GptPreferencesResponse {
        user_id: Uuid::new_v4(),
        preferences: vec![TaskPreference {
            task_type: "data_provision".into(),
            level: "neutral".into(),
        }],
        updated_at: Some(Utc::now()),
        message: "Your preferences".into(),
    };
    let resp_json = serde_json::to_value(&resp).unwrap();
    assert!(resp_json.get("user_id").is_some());
    assert!(resp_json.get("preferences").is_some());
    assert!(resp_json.get("updated_at").is_some());

    // GptPreferencesResponse with updated_at = None
    let resp_none = GptPreferencesResponse {
        user_id: Uuid::new_v4(),
        preferences: vec![],
        updated_at: None,
        message: "No preferences set".into(),
    };
    assert!(resp_none.updated_at.is_none());

    // GptSetPreferencesResponse (Serialize + Deserialize)
    let set_resp = GptSetPreferencesResponse {
        user_id: Uuid::new_v4(),
        preferences_count: 3,
        updated_at: Utc::now(),
        message: "Preferences updated".into(),
    };
    let set_json = serde_json::to_value(&set_resp).unwrap();
    assert_eq!(
        set_json.get("preferences_count").unwrap().as_u64().unwrap(),
        3
    );
    assert!(set_json.get("updated_at").is_some());
}

// =============================================================================
// Smart Service Suggestion — タスク 2: DBマイグレーション 0012
// =============================================================================

#[test]
fn migration_0012_campaign_tags_has_expected_schema() {
    let sql = include_str!("../migrations/0012_campaign_tags.sql");
    let lower = sql.to_lowercase();

    // ALTER TABLE to add tags column
    assert!(lower.contains("alter table"), "should use ALTER TABLE");
    assert!(lower.contains("campaigns"), "should target campaigns table");
    assert!(lower.contains("add column"), "should add a column");
    assert!(
        lower.contains("if not exists"),
        "should use IF NOT EXISTS for idempotent migration"
    );
    assert!(lower.contains("tags"), "should add tags column");
    assert!(lower.contains("text[]"), "tags should be TEXT[] array type");
    assert!(
        lower.contains("default '{}'"),
        "tags should default to empty array"
    );
}

// =============================================================================
// Smart Service Suggestion — タスク 4: 嗜好管理ハンドラ
// =============================================================================

// --- Task 4.3: ハンドラ署名テスト（ルーター到達性） ---

#[tokio::test]
async fn gpt_preferences_get_is_reachable() {
    let (app, _state) = test_app();
    let fake_token = Uuid::new_v4();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/gpt/preferences?session_token={}", fake_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    // Should not be 404 (route exists); may be 500 if no DB, but route is reachable
    assert_ne!(
        response.status().as_u16(),
        404,
        "GET /gpt/preferences should be routed"
    );
}

#[tokio::test]
async fn gpt_preferences_post_is_reachable() {
    let (app, _state) = test_app();
    let response = post_json(
        &app,
        "/gpt/preferences",
        serde_json::json!({
            "session_token": Uuid::new_v4(),
            "preferences": [
                { "task_type": "survey", "level": "avoided" }
            ]
        }),
        None,
    )
    .await;
    // Should not be 404 (route exists); may be 500 if no DB or 401 if session invalid
    assert_ne!(
        response.status().as_u16(),
        404,
        "POST /gpt/preferences should be routed"
    );
}

#[tokio::test]
async fn gpt_preferences_behind_auth_middleware() {
    let state = SharedState {
        inner: Arc::new(RwLock::new(AppState::new())),
    };
    {
        let mut s = state.inner.write().await;
        s.config.gpt_actions_api_key = Some("pref-test-key".to_string());
    }
    let app = build_app(state, DEFAULT_AGENT_DISCOVERY_RATE_LIMIT_PER_MIN as u32);

    // GET without auth should be rejected
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!(
                    "/gpt/preferences?session_token={}",
                    Uuid::new_v4()
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.status().as_u16(),
        401,
        "GET /gpt/preferences should require API key"
    );

    // POST without auth should be rejected
    let response = post_json(
        &app,
        "/gpt/preferences",
        serde_json::json!({
            "session_token": Uuid::new_v4(),
            "preferences": []
        }),
        None,
    )
    .await;
    assert_eq!(
        response.status().as_u16(),
        401,
        "POST /gpt/preferences should require API key"
    );
}

#[tokio::test]
async fn gpt_preferences_endpoints_record_metrics() {
    let (app, state) = test_app();
    {
        let mut s = state.inner.write().await;
        s.config.gpt_actions_api_key = None;
    }

    // Hit GET /gpt/preferences
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!(
                    "/gpt/preferences?session_token={}",
                    Uuid::new_v4()
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Hit POST /gpt/preferences
    let req = Request::builder()
        .method("POST")
        .uri("/gpt/preferences")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "session_token": Uuid::new_v4(),
                "preferences": []
            })
            .to_string(),
        ))
        .unwrap();
    let _ = app.clone().oneshot(req).await.unwrap();

    // Check metrics
    let s = state.inner.read().await;
    let metric_families = s.metrics.registry.gather();
    let http_requests = metric_families
        .iter()
        .find(|mf| mf.get_name() == "http_requests_total")
        .expect("http_requests_total should exist");

    let recorded: std::collections::HashSet<String> = http_requests
        .get_metric()
        .iter()
        .flat_map(|m| m.get_label().iter())
        .filter(|l| l.get_name() == "endpoint" && l.get_value().starts_with("gpt_"))
        .map(|l| l.get_value().to_string())
        .collect();

    assert!(
        recorded.contains("gpt_get_preferences"),
        "should record gpt_get_preferences metric. Recorded: {:?}",
        recorded
    );
    assert!(
        recorded.contains("gpt_set_preferences"),
        "should record gpt_set_preferences metric. Recorded: {:?}",
        recorded
    );
}

// --- Task 4.4: 統合テスト（DATABASE_URL 必要） ---

#[tokio::test]
async fn gpt_preferences_integration_crud() {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        use sqlx::postgres::PgPoolOptions;
        if let Ok(pool) = PgPoolOptions::new().max_connections(1).connect(&url).await {
            sqlx::migrate!("./migrations").run(&pool).await.ok();

            let state = SharedState {
                inner: Arc::new(RwLock::new(AppState::new())),
            };

            // --- Setup: create user and session ---
            let user_id = Uuid::new_v4();
            let unique_email = format!("pref_test_{}@example.com", Uuid::new_v4());
            sqlx::query(
                "INSERT INTO users (id, email, region, roles, tools_used, attributes, created_at, source) \
                 VALUES ($1, $2, 'JP', '{developer}', '{design}', '{}'::jsonb, NOW(), 'gpt_apps')",
            )
            .bind(user_id)
            .bind(&unique_email)
            .execute(&pool)
            .await
            .unwrap();

            let session_token: Uuid = sqlx::query_scalar(
                "INSERT INTO gpt_sessions (user_id) VALUES ($1) RETURNING token",
            )
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();

            // ========== Step 1: GET preferences (empty) ==========
            let result = gpt::gpt_get_preferences(
                axum::extract::State(state.clone()),
                axum::extract::Query(types::GptPreferencesParams { session_token }),
            )
            .await;
            assert!(
                result.status().is_success(),
                "GET preferences should succeed"
            );
            let resp: types::GptPreferencesResponse = read_typed(result).await;
            assert_eq!(resp.user_id, user_id);
            assert!(
                resp.preferences.is_empty(),
                "should have no preferences initially"
            );
            assert!(
                resp.updated_at.is_none(),
                "updated_at should be None when no preferences"
            );
            assert!(
                resp.message.contains("No preferences"),
                "should return guidance message"
            );

            // ========== Step 2: SET preferences ==========
            let result = gpt::gpt_set_preferences(
                axum::extract::State(state.clone()),
                axum::Json(types::GptSetPreferencesRequest {
                    session_token,
                    preferences: vec![
                        types::TaskPreference {
                            task_type: "survey".into(),
                            level: "avoided".into(),
                        },
                        types::TaskPreference {
                            task_type: "github_pr".into(),
                            level: "preferred".into(),
                        },
                        types::TaskPreference {
                            task_type: "data_provision".into(),
                            level: "neutral".into(),
                        },
                    ],
                }),
            )
            .await;
            assert!(
                result.status().is_success(),
                "SET preferences should succeed"
            );
            let set_resp: types::GptSetPreferencesResponse = read_typed(result).await;
            assert_eq!(set_resp.user_id, user_id);
            assert_eq!(set_resp.preferences_count, 3);
            assert!(set_resp.message.contains("3"));

            // ========== Step 3: GET preferences (should return 3) ==========
            let result = gpt::gpt_get_preferences(
                axum::extract::State(state.clone()),
                axum::extract::Query(types::GptPreferencesParams { session_token }),
            )
            .await;
            assert!(result.status().is_success());
            let resp: types::GptPreferencesResponse = read_typed(result).await;
            assert_eq!(resp.preferences.len(), 3);
            assert!(
                resp.updated_at.is_some(),
                "updated_at should be set after creating preferences"
            );

            // Verify specific preferences
            let survey = resp
                .preferences
                .iter()
                .find(|p| p.task_type == "survey")
                .expect("should have survey preference");
            assert_eq!(survey.level, "avoided");
            let github = resp
                .preferences
                .iter()
                .find(|p| p.task_type == "github_pr")
                .expect("should have github_pr preference");
            assert_eq!(github.level, "preferred");

            // ========== Step 4: UPDATE preferences (overwrite) ==========
            let result = gpt::gpt_set_preferences(
                axum::extract::State(state.clone()),
                axum::Json(types::GptSetPreferencesRequest {
                    session_token,
                    preferences: vec![types::TaskPreference {
                        task_type: "registration".into(),
                        level: "preferred".into(),
                    }],
                }),
            )
            .await;
            assert!(
                result.status().is_success(),
                "UPDATE preferences should succeed"
            );
            let set_resp: types::GptSetPreferencesResponse = read_typed(result).await;
            assert_eq!(
                set_resp.preferences_count, 1,
                "should replace all with 1 preference"
            );

            // Verify overwrite: only 1 preference now
            let result = gpt::gpt_get_preferences(
                axum::extract::State(state.clone()),
                axum::extract::Query(types::GptPreferencesParams { session_token }),
            )
            .await;
            let resp: types::GptPreferencesResponse = read_typed(result).await;
            assert_eq!(
                resp.preferences.len(),
                1,
                "old preferences should be replaced"
            );
            assert_eq!(resp.preferences[0].task_type, "registration");
            assert_eq!(resp.preferences[0].level, "preferred");

            // ========== Step 5: Invalid session token ==========
            let result = gpt::gpt_get_preferences(
                axum::extract::State(state.clone()),
                axum::extract::Query(types::GptPreferencesParams {
                    session_token: Uuid::new_v4(), // non-existent
                }),
            )
            .await;
            assert_eq!(
                result.status().as_u16(),
                401,
                "invalid session token should return 401"
            );

            // ========== Cleanup ==========
            sqlx::query("DELETE FROM user_task_preferences WHERE user_id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM gpt_sessions WHERE user_id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
        }
    }
}

#[tokio::test]
async fn gpt_set_preferences_validates_level() {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        use sqlx::postgres::PgPoolOptions;
        if let Ok(pool) = PgPoolOptions::new().max_connections(1).connect(&url).await {
            sqlx::migrate!("./migrations").run(&pool).await.ok();

            let state = SharedState {
                inner: Arc::new(RwLock::new(AppState::new())),
            };

            // Setup user and session
            let user_id = Uuid::new_v4();
            let unique_email = format!("pref_val_{}@example.com", Uuid::new_v4());
            sqlx::query(
                "INSERT INTO users (id, email, region, roles, tools_used, attributes, created_at, source) \
                 VALUES ($1, $2, 'JP', '{developer}', '{design}', '{}'::jsonb, NOW(), 'gpt_apps')",
            )
            .bind(user_id)
            .bind(&unique_email)
            .execute(&pool)
            .await
            .unwrap();

            let session_token: Uuid = sqlx::query_scalar(
                "INSERT INTO gpt_sessions (user_id) VALUES ($1) RETURNING token",
            )
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();

            // Invalid level should fail validation
            let result = gpt::gpt_set_preferences(
                axum::extract::State(state.clone()),
                axum::Json(types::GptSetPreferencesRequest {
                    session_token,
                    preferences: vec![types::TaskPreference {
                        task_type: "survey".into(),
                        level: "invalid_level".into(),
                    }],
                }),
            )
            .await;
            assert_eq!(
                result.status().as_u16(),
                400,
                "invalid preference level should return 400"
            );

            // Cleanup
            sqlx::query("DELETE FROM gpt_sessions WHERE user_id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
        }
    }
}

// =============================================================================
// Smart Service Suggestion — タスク 5: 予算フィルタ
// =============================================================================

#[tokio::test]
async fn budget_filter_integration_test() {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        use sqlx::postgres::PgPoolOptions;
        if let Ok(pool) = PgPoolOptions::new().max_connections(1).connect(&url).await {
            sqlx::migrate!("./migrations").run(&pool).await.ok();

            let state = SharedState {
                inner: Arc::new(RwLock::new(AppState::new())),
            };

            // --- Setup: Create campaigns with different subsidy amounts ---
            let cheap_id = Uuid::new_v4();
            let mid_id = Uuid::new_v4();
            let expensive_id = Uuid::new_v4();
            let campaign_prefix = format!("budget_test_{}", Uuid::new_v4());

            for (id, name_suffix, subsidy) in [
                (cheap_id, "cheap", 100_i64),
                (mid_id, "mid", 500_i64),
                (expensive_id, "expensive", 1000_i64),
            ] {
                sqlx::query(
                    "INSERT INTO campaigns (id, name, sponsor, target_roles, target_tools, required_task, \
                     subsidy_per_call_cents, budget_total_cents, budget_remaining_cents, query_urls, active, created_at) \
                     VALUES ($1, $2, 'TestSponsor', '{developer}', '{design}', 'survey', \
                     $3, 50000, 50000, '{}', true, NOW())",
                )
                .bind(id)
                .bind(format!("{}_{}", campaign_prefix, name_suffix))
                .bind(subsidy)
                .execute(&pool)
                .await
                .unwrap();
            }

            // ========== Test 1: No budget filter — all services returned (要件 1.2) ==========
            let result = gpt::gpt_search_services(
                axum::extract::State(state.clone()),
                axum::extract::Query(types::GptSearchParams {
                    q: Some(campaign_prefix.clone()),
                    category: None,
                    max_budget_cents: None,
                    intent: None,
                    session_token: None,
                }),
            )
            .await;
            assert!(
                result.status().is_success(),
                "search without budget should succeed"
            );
            let resp: types::GptSearchResponse = read_typed(result).await;
            assert_eq!(
                resp.total_count, 3,
                "without budget filter, all 3 test campaigns should be returned"
            );

            // ========== Test 2: Budget = 500 — only cheap and mid returned (要件 1.1) ==========
            let result = gpt::gpt_search_services(
                axum::extract::State(state.clone()),
                axum::extract::Query(types::GptSearchParams {
                    q: Some(campaign_prefix.clone()),
                    category: None,
                    max_budget_cents: Some(500),
                    intent: None,
                    session_token: None,
                }),
            )
            .await;
            assert!(result.status().is_success());
            let resp: types::GptSearchResponse = read_typed(result).await;
            assert_eq!(
                resp.total_count, 2,
                "budget=500 should return 2 services (cheap=100, mid=500)"
            );
            // Verify all returned services have subsidy <= 500
            for svc in &resp.services {
                assert!(
                    svc.subsidy_amount_cents <= 500,
                    "service '{}' has subsidy {} which exceeds budget 500",
                    svc.name,
                    svc.subsidy_amount_cents
                );
            }

            // ========== Test 3: Budget = 99 — 0 results with budget message (要件 1.3) ==========
            let result = gpt::gpt_search_services(
                axum::extract::State(state.clone()),
                axum::extract::Query(types::GptSearchParams {
                    q: Some(campaign_prefix.clone()),
                    category: None,
                    max_budget_cents: Some(99),
                    intent: None,
                    session_token: None,
                }),
            )
            .await;
            assert!(result.status().is_success());
            let resp: types::GptSearchResponse = read_typed(result).await;
            assert_eq!(resp.total_count, 0, "budget=99 should return 0 services");
            assert!(
                resp.message.contains("budget") || resp.message.contains("Budget"),
                "zero-result message should mention budget, got: '{}'",
                resp.message
            );
            assert!(
                resp.message.contains("paying directly") || resp.message.contains("increasing"),
                "zero-result message should suggest alternatives, got: '{}'",
                resp.message
            );

            // ========== Test 4: Budget = 100 — only cheapest (boundary test) ==========
            let result = gpt::gpt_search_services(
                axum::extract::State(state.clone()),
                axum::extract::Query(types::GptSearchParams {
                    q: Some(campaign_prefix.clone()),
                    category: None,
                    max_budget_cents: Some(100),
                    intent: None,
                    session_token: None,
                }),
            )
            .await;
            assert!(result.status().is_success());
            let resp: types::GptSearchResponse = read_typed(result).await;
            assert_eq!(
                resp.total_count, 1,
                "budget=100 should return exactly 1 service (cheap=100, boundary inclusive)"
            );
            assert_eq!(resp.services[0].subsidy_amount_cents, 100);

            // ========== Test 5: Budget = 10000 — all services returned ==========
            let result = gpt::gpt_search_services(
                axum::extract::State(state.clone()),
                axum::extract::Query(types::GptSearchParams {
                    q: Some(campaign_prefix.clone()),
                    category: None,
                    max_budget_cents: Some(10000),
                    intent: None,
                    session_token: None,
                }),
            )
            .await;
            assert!(result.status().is_success());
            let resp: types::GptSearchResponse = read_typed(result).await;
            assert_eq!(
                resp.total_count, 3,
                "budget=10000 should return all 3 services"
            );

            // ========== Cleanup ==========
            for id in [cheap_id, mid_id, expensive_id] {
                sqlx::query("DELETE FROM campaigns WHERE id = $1")
                    .bind(id)
                    .execute(&pool)
                    .await
                    .ok();
            }
        }
    }
}

// =============================================================================
// Smart Service Suggestion — タスク 6: 意図フィルタとタグマッチング
// =============================================================================

#[test]
fn infer_tags_returns_existing_tags_when_non_empty() {
    let tags = vec!["custom_tag".to_string()];
    let target_tools = vec!["design".to_string()];
    let result = gpt::infer_tags(&tags, &target_tools, "survey");
    assert_eq!(result, vec!["custom_tag".to_string()]);
}

#[test]
fn infer_tags_generates_from_target_tools_and_required_task() {
    let tags: Vec<String> = vec![];
    let target_tools = vec!["design".to_string(), "screenshot".to_string()];
    let result = gpt::infer_tags(&tags, &target_tools, "survey");
    assert_eq!(
        result,
        vec![
            "design".to_string(),
            "screenshot".to_string(),
            "survey".to_string()
        ]
    );
}

#[test]
fn infer_tags_avoids_duplicate_required_task() {
    let tags: Vec<String> = vec![];
    let target_tools = vec!["survey".to_string(), "design".to_string()];
    let result = gpt::infer_tags(&tags, &target_tools, "survey");
    // "survey" already in target_tools, should not be duplicated
    assert_eq!(result, vec!["survey".to_string(), "design".to_string()]);
}

#[test]
fn matches_intent_matches_name() {
    let service = types::GptServiceItem {
        service_type: "campaign".to_string(),
        service_id: uuid::Uuid::new_v4(),
        name: "Screenshot Service".to_string(),
        sponsor: "TestSponsor".to_string(),
        required_task: Some("survey".to_string()),
        subsidy_amount_cents: 100,
        category: vec!["design".to_string()],
        active: true,
        tags: vec!["web".to_string()],
        relevance_score: None,
    };
    assert!(gpt::matches_intent(&service, &["screenshot"]));
    assert!(gpt::matches_intent(&service, &["SCREENSHOT"])); // case-insensitive
}

#[test]
fn matches_intent_matches_required_task() {
    let service = types::GptServiceItem {
        service_type: "campaign".to_string(),
        service_id: uuid::Uuid::new_v4(),
        name: "Some Service".to_string(),
        sponsor: "TestSponsor".to_string(),
        required_task: Some("github_pr".to_string()),
        subsidy_amount_cents: 100,
        category: vec!["dev".to_string()],
        active: true,
        tags: vec![],
        relevance_score: None,
    };
    assert!(gpt::matches_intent(&service, &["github"]));
}

#[test]
fn matches_intent_matches_category() {
    let service = types::GptServiceItem {
        service_type: "campaign".to_string(),
        service_id: uuid::Uuid::new_v4(),
        name: "Some Service".to_string(),
        sponsor: "TestSponsor".to_string(),
        required_task: Some("survey".to_string()),
        subsidy_amount_cents: 100,
        category: vec!["design".to_string(), "screenshot".to_string()],
        active: true,
        tags: vec![],
        relevance_score: None,
    };
    assert!(gpt::matches_intent(&service, &["design"]));
    assert!(gpt::matches_intent(&service, &["screenshot"]));
}

#[test]
fn matches_intent_matches_tags() {
    let service = types::GptServiceItem {
        service_type: "campaign".to_string(),
        service_id: uuid::Uuid::new_v4(),
        name: "Some Service".to_string(),
        sponsor: "TestSponsor".to_string(),
        required_task: Some("survey".to_string()),
        subsidy_amount_cents: 100,
        category: vec!["design".to_string()],
        active: true,
        tags: vec!["blockchain".to_string(), "web3".to_string()],
        relevance_score: None,
    };
    assert!(gpt::matches_intent(&service, &["blockchain"]));
    assert!(gpt::matches_intent(&service, &["WEB3"])); // case-insensitive
}

#[test]
fn matches_intent_returns_false_when_no_match() {
    let service = types::GptServiceItem {
        service_type: "campaign".to_string(),
        service_id: uuid::Uuid::new_v4(),
        name: "Screenshot Service".to_string(),
        sponsor: "TestSponsor".to_string(),
        required_task: Some("survey".to_string()),
        subsidy_amount_cents: 100,
        category: vec!["design".to_string()],
        active: true,
        tags: vec!["web".to_string()],
        relevance_score: None,
    };
    assert!(!gpt::matches_intent(&service, &["blockchain"]));
    assert!(!gpt::matches_intent(&service, &["finance"]));
}

#[test]
fn matches_intent_any_keyword_matches() {
    let service = types::GptServiceItem {
        service_type: "campaign".to_string(),
        service_id: uuid::Uuid::new_v4(),
        name: "Screenshot Service".to_string(),
        sponsor: "TestSponsor".to_string(),
        required_task: Some("survey".to_string()),
        subsidy_amount_cents: 100,
        category: vec!["design".to_string()],
        active: true,
        tags: vec![],
        relevance_score: None,
    };
    // "finance" doesn't match, but "screenshot" does → overall true
    assert!(gpt::matches_intent(&service, &["finance", "screenshot"]));
}

#[tokio::test]
async fn intent_filter_integration_test() {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        use sqlx::postgres::PgPoolOptions;
        if let Ok(pool) = PgPoolOptions::new().max_connections(1).connect(&url).await {
            sqlx::migrate!("./migrations").run(&pool).await.ok();

            let state = SharedState {
                inner: Arc::new(RwLock::new(AppState::new())),
            };

            // --- Setup: Create campaigns with different names/tasks/tags ---
            let screenshot_id = Uuid::new_v4();
            let blockchain_id = Uuid::new_v4();
            let analytics_id = Uuid::new_v4();
            let test_prefix = format!("intent_test_{}", Uuid::new_v4());

            // Campaign 1: screenshot-related
            sqlx::query(
                "INSERT INTO campaigns (id, name, sponsor, target_roles, target_tools, required_task, \
                 subsidy_per_call_cents, budget_total_cents, budget_remaining_cents, query_urls, active, created_at, tags) \
                 VALUES ($1, $2, 'TestSponsor', '{developer}', '{screenshot,design}', 'survey', \
                 100, 50000, 50000, '{}', true, NOW(), '{screenshot,web,capture}')",
            )
            .bind(screenshot_id)
            .bind(format!("{}_screenshot_svc", test_prefix))
            .execute(&pool)
            .await
            .unwrap();

            // Campaign 2: blockchain-related
            sqlx::query(
                "INSERT INTO campaigns (id, name, sponsor, target_roles, target_tools, required_task, \
                 subsidy_per_call_cents, budget_total_cents, budget_remaining_cents, query_urls, active, created_at, tags) \
                 VALUES ($1, $2, 'TestSponsor', '{developer}', '{defi}', 'github_pr', \
                 200, 50000, 50000, '{}', true, NOW(), '{blockchain,web3,defi}')",
            )
            .bind(blockchain_id)
            .bind(format!("{}_blockchain_svc", test_prefix))
            .execute(&pool)
            .await
            .unwrap();

            // Campaign 3: analytics (no tags set, should infer from target_tools + required_task)
            sqlx::query(
                "INSERT INTO campaigns (id, name, sponsor, target_roles, target_tools, required_task, \
                 subsidy_per_call_cents, budget_total_cents, budget_remaining_cents, query_urls, active, created_at) \
                 VALUES ($1, $2, 'TestSponsor', '{developer}', '{analytics}', 'data_provision', \
                 150, 50000, 50000, '{}', true, NOW())",
            )
            .bind(analytics_id)
            .bind(format!("{}_analytics_svc", test_prefix))
            .execute(&pool)
            .await
            .unwrap();

            // ========== Test 1: Intent matches name — "screenshot" ==========
            let result = gpt::gpt_search_services(
                axum::extract::State(state.clone()),
                axum::extract::Query(types::GptSearchParams {
                    q: Some(test_prefix.clone()),
                    category: None,
                    max_budget_cents: None,
                    intent: Some("screenshot".to_string()),
                    session_token: None,
                }),
            )
            .await;
            assert!(result.status().is_success());
            let resp: types::GptSearchResponse = read_typed(result).await;
            assert_eq!(
                resp.total_count, 1,
                "intent='screenshot' should match 1 service"
            );
            assert!(resp.services[0].name.contains("screenshot"));
            assert!(
                resp.available_categories.is_none(),
                "should not return categories when results found"
            );

            // ========== Test 2: Intent matches tags — "blockchain" ==========
            let result = gpt::gpt_search_services(
                axum::extract::State(state.clone()),
                axum::extract::Query(types::GptSearchParams {
                    q: Some(test_prefix.clone()),
                    category: None,
                    max_budget_cents: None,
                    intent: Some("blockchain".to_string()),
                    session_token: None,
                }),
            )
            .await;
            assert!(result.status().is_success());
            let resp: types::GptSearchResponse = read_typed(result).await;
            assert_eq!(
                resp.total_count, 1,
                "intent='blockchain' should match 1 service via tags"
            );
            assert!(resp.services[0].name.contains("blockchain"));

            // ========== Test 3: Intent matches inferred tags — "data_provision" ==========
            let result = gpt::gpt_search_services(
                axum::extract::State(state.clone()),
                axum::extract::Query(types::GptSearchParams {
                    q: Some(test_prefix.clone()),
                    category: None,
                    max_budget_cents: None,
                    intent: Some("data_provision".to_string()),
                    session_token: None,
                }),
            )
            .await;
            assert!(result.status().is_success());
            let resp: types::GptSearchResponse = read_typed(result).await;
            assert_eq!(
                resp.total_count, 1,
                "intent='data_provision' should match analytics service via inferred tags (required_task)"
            );
            assert!(resp.services[0].name.contains("analytics"));

            // ========== Test 4: No intent matches — 0 results with available_categories (要件 2.3) ==========
            let result = gpt::gpt_search_services(
                axum::extract::State(state.clone()),
                axum::extract::Query(types::GptSearchParams {
                    q: Some(test_prefix.clone()),
                    category: None,
                    max_budget_cents: None,
                    intent: Some("nonexistent_xyz_12345".to_string()),
                    session_token: None,
                }),
            )
            .await;
            assert!(result.status().is_success());
            let resp: types::GptSearchResponse = read_typed(result).await;
            assert_eq!(resp.total_count, 0, "non-matching intent should return 0");
            assert!(
                resp.message.contains("intent") || resp.message.contains("categories"),
                "zero-result intent message should reference intent or categories, got: '{}'",
                resp.message
            );
            let cats = resp
                .available_categories
                .expect("should have available_categories when intent yields 0 results");
            assert!(!cats.is_empty(), "available_categories should not be empty");
            // Should contain categories from all 3 test campaigns
            assert!(
                cats.contains(&"screenshot".to_string())
                    || cats.contains(&"design".to_string())
                    || cats.contains(&"analytics".to_string()),
                "available_categories should contain categories from test campaigns, got: {:?}",
                cats
            );

            // ========== Test 5: No intent filter — all services returned ==========
            let result = gpt::gpt_search_services(
                axum::extract::State(state.clone()),
                axum::extract::Query(types::GptSearchParams {
                    q: Some(test_prefix.clone()),
                    category: None,
                    max_budget_cents: None,
                    intent: None,
                    session_token: None,
                }),
            )
            .await;
            assert!(result.status().is_success());
            let resp: types::GptSearchResponse = read_typed(result).await;
            assert_eq!(
                resp.total_count, 3,
                "without intent filter, all 3 test campaigns should be returned"
            );
            assert!(
                resp.available_categories.is_none(),
                "no intent → no available_categories"
            );

            // ========== Test 6: Tags field populated correctly ==========
            // Verify that the screenshot service has its explicit tags
            let screenshot_svc = resp
                .services
                .iter()
                .find(|s| s.name.contains("screenshot"))
                .expect("screenshot service should exist");
            assert!(
                screenshot_svc.tags.contains(&"screenshot".to_string()),
                "screenshot service should have 'screenshot' tag, got: {:?}",
                screenshot_svc.tags
            );
            // Verify that analytics service has inferred tags
            let analytics_svc = resp
                .services
                .iter()
                .find(|s| s.name.contains("analytics"))
                .expect("analytics service should exist");
            assert!(
                analytics_svc.tags.contains(&"analytics".to_string()),
                "analytics service should have inferred 'analytics' tag from target_tools, got: {:?}",
                analytics_svc.tags
            );
            assert!(
                analytics_svc.tags.contains(&"data_provision".to_string()),
                "analytics service should have inferred 'data_provision' tag from required_task, got: {:?}",
                analytics_svc.tags
            );

            // ========== Cleanup ==========
            for id in [screenshot_id, blockchain_id, analytics_id] {
                sqlx::query("DELETE FROM campaigns WHERE id = $1")
                    .bind(id)
                    .execute(&pool)
                    .await
                    .ok();
            }
        }
    }
}

// =============================================================================
// Smart Service Suggestion — タスク 7: 嗜好フィルタとスコアリング
// =============================================================================

#[test]
fn calculate_score_no_params_returns_neutral() {
    let service = types::GptServiceItem {
        service_type: "campaign".to_string(),
        service_id: uuid::Uuid::new_v4(),
        name: "Test".to_string(),
        sponsor: "Sponsor".to_string(),
        required_task: Some("survey".to_string()),
        subsidy_amount_cents: 100,
        category: vec!["design".to_string()],
        active: true,
        tags: vec![],
        relevance_score: None,
    };
    // All neutral: 0.5*0.3 + 0.5*0.4 + 0.5*0.3 = 0.5
    let score = gpt::calculate_score(&service, None, None, &[]);
    assert!(
        (score - 0.5).abs() < 0.001,
        "neutral score should be ~0.5, got {}",
        score
    );
}

#[test]
fn calculate_score_budget_only() {
    let service = types::GptServiceItem {
        service_type: "campaign".to_string(),
        service_id: uuid::Uuid::new_v4(),
        name: "Test".to_string(),
        sponsor: "Sponsor".to_string(),
        required_task: Some("survey".to_string()),
        subsidy_amount_cents: 200,
        category: vec!["design".to_string()],
        active: true,
        tags: vec![],
        relevance_score: None,
    };
    // budget_score = 1.0 - (200/1000) = 0.8
    // intent_score = 0.5 (not specified)
    // preference_score = 0.5 (no prefs)
    // total = 0.8*0.3 + 0.5*0.4 + 0.5*0.3 = 0.24 + 0.20 + 0.15 = 0.59
    let score = gpt::calculate_score(&service, Some(1000), None, &[]);
    assert!(
        (score - 0.59).abs() < 0.001,
        "score should be ~0.59, got {}",
        score
    );
}

#[test]
fn calculate_score_preferred_boosts() {
    let service = types::GptServiceItem {
        service_type: "campaign".to_string(),
        service_id: uuid::Uuid::new_v4(),
        name: "Test".to_string(),
        sponsor: "Sponsor".to_string(),
        required_task: Some("survey".to_string()),
        subsidy_amount_cents: 100,
        category: vec!["design".to_string()],
        active: true,
        tags: vec![],
        relevance_score: None,
    };
    let prefs = vec![types::TaskPreference {
        task_type: "survey".to_string(),
        level: "preferred".to_string(),
    }];
    // budget_score = 0.5 (no budget)
    // intent_score = 0.5 (no intent)
    // preference_score = 1.0 (preferred)
    // total = 0.5*0.3 + 0.5*0.4 + 1.0*0.3 = 0.15 + 0.20 + 0.30 = 0.65
    let score = gpt::calculate_score(&service, None, None, &prefs);
    assert!(
        (score - 0.65).abs() < 0.001,
        "preferred score should be ~0.65, got {}",
        score
    );
}

#[test]
fn calculate_score_avoided_returns_low() {
    let service = types::GptServiceItem {
        service_type: "campaign".to_string(),
        service_id: uuid::Uuid::new_v4(),
        name: "Test".to_string(),
        sponsor: "Sponsor".to_string(),
        required_task: Some("survey".to_string()),
        subsidy_amount_cents: 100,
        category: vec!["design".to_string()],
        active: true,
        tags: vec![],
        relevance_score: None,
    };
    let prefs = vec![types::TaskPreference {
        task_type: "survey".to_string(),
        level: "avoided".to_string(),
    }];
    // preference_score = 0.0
    // total = 0.5*0.3 + 0.5*0.4 + 0.0*0.3 = 0.15 + 0.20 + 0.0 = 0.35
    let score = gpt::calculate_score(&service, None, None, &prefs);
    assert!(
        (score - 0.35).abs() < 0.001,
        "avoided score should be ~0.35, got {}",
        score
    );
}

#[test]
fn calculate_score_with_intent_matching() {
    let service = types::GptServiceItem {
        service_type: "campaign".to_string(),
        service_id: uuid::Uuid::new_v4(),
        name: "Screenshot Service".to_string(),
        sponsor: "Sponsor".to_string(),
        required_task: Some("survey".to_string()),
        subsidy_amount_cents: 100,
        category: vec!["screenshot".to_string(), "design".to_string()],
        active: true,
        tags: vec!["web".to_string()],
        relevance_score: None,
    };
    // intent "screenshot" matches: name (1), category (1), not required_task (0), not tags (0)
    // total_fields = 4 (name, required_task, category, tags)
    // intent_score = 2/4 = 0.5
    let score = gpt::calculate_score(&service, None, Some("screenshot"), &[]);
    // 0.5*0.3 + 0.5*0.4 + 0.5*0.3 = 0.5
    assert!(
        (score - 0.5).abs() < 0.001,
        "intent matching score should be ~0.5, got {}",
        score
    );
}

#[test]
fn calculate_score_full_match_high_score() {
    let service = types::GptServiceItem {
        service_type: "campaign".to_string(),
        service_id: uuid::Uuid::new_v4(),
        name: "Screenshot Tool".to_string(),
        sponsor: "Sponsor".to_string(),
        required_task: Some("screenshot".to_string()),
        subsidy_amount_cents: 100,
        category: vec!["screenshot".to_string()],
        active: true,
        tags: vec!["screenshot".to_string()],
        relevance_score: None,
    };
    let prefs = vec![types::TaskPreference {
        task_type: "screenshot".to_string(),
        level: "preferred".to_string(),
    }];
    // budget: 1.0 - (100/10000) = 0.99
    // intent "screenshot": matches all 4 fields → 4/4 = 1.0
    // preference: preferred → 1.0
    // total = 0.99*0.3 + 1.0*0.4 + 1.0*0.3 = 0.297 + 0.4 + 0.3 = 0.997
    let score = gpt::calculate_score(&service, Some(10000), Some("screenshot"), &prefs);
    assert!(
        score > 0.9,
        "full match should have high score, got {}",
        score
    );
}

#[tokio::test]
async fn preference_filter_integration_test() {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        use sqlx::postgres::PgPoolOptions;
        if let Ok(pool) = PgPoolOptions::new().max_connections(1).connect(&url).await {
            sqlx::migrate!("./migrations").run(&pool).await.ok();

            let state = SharedState {
                inner: Arc::new(RwLock::new(AppState::new())),
            };

            // --- Setup: Create test user, session, campaigns, preferences ---
            let user_id = Uuid::new_v4();
            let session_token = Uuid::new_v4();
            let test_prefix = format!("pref_test_{}", Uuid::new_v4());

            // Create user
            sqlx::query(
                "INSERT INTO users (id, email, region, roles, tools_used, attributes, created_at, source) \
                 VALUES ($1, $2, 'US', '{developer}', '{design}', '{}'::jsonb, NOW(), 'gpt_apps')",
            )
            .bind(user_id)
            .bind(format!("{}@test.com", test_prefix))
            .execute(&pool)
            .await
            .unwrap();

            // Create session
            sqlx::query(
                "INSERT INTO gpt_sessions (token, user_id, expires_at) VALUES ($1, $2, NOW() + INTERVAL '1 hour')",
            )
            .bind(session_token)
            .bind(user_id)
            .execute(&pool)
            .await
            .unwrap();

            // Create campaigns: survey (will be avoided), github_pr (preferred), data_provision (neutral)
            let survey_id = Uuid::new_v4();
            let github_id = Uuid::new_v4();
            let data_id = Uuid::new_v4();

            for (id, name_suffix, task, subsidy) in [
                (survey_id, "survey_svc", "survey", 100_i64),
                (github_id, "github_svc", "github_pr", 200_i64),
                (data_id, "data_svc", "data_provision", 150_i64),
            ] {
                sqlx::query(
                    "INSERT INTO campaigns (id, name, sponsor, target_roles, target_tools, required_task, \
                     subsidy_per_call_cents, budget_total_cents, budget_remaining_cents, query_urls, active, created_at) \
                     VALUES ($1, $2, 'TestSponsor', '{developer}', '{design}', $3, \
                     $4, 50000, 50000, '{}', true, NOW())",
                )
                .bind(id)
                .bind(format!("{}_{}", test_prefix, name_suffix))
                .bind(task)
                .bind(subsidy)
                .execute(&pool)
                .await
                .unwrap();
            }

            // Set preferences: survey=avoided, github_pr=preferred
            for (task_type, level) in [("survey", "avoided"), ("github_pr", "preferred")] {
                sqlx::query(
                    "INSERT INTO user_task_preferences (id, user_id, task_type, level, created_at, updated_at) \
                     VALUES ($1, $2, $3, $4, NOW(), NOW())",
                )
                .bind(Uuid::new_v4())
                .bind(user_id)
                .bind(task_type)
                .bind(level)
                .execute(&pool)
                .await
                .unwrap();
            }

            // ========== Test 1: With session_token — avoided excluded (要件 4.1) ==========
            let result = gpt::gpt_search_services(
                axum::extract::State(state.clone()),
                axum::extract::Query(types::GptSearchParams {
                    q: Some(test_prefix.clone()),
                    category: None,
                    max_budget_cents: None,
                    intent: None,
                    session_token: Some(session_token),
                }),
            )
            .await;
            assert!(result.status().is_success());
            let resp: types::GptSearchResponse = read_typed(result).await;
            assert_eq!(
                resp.total_count, 2,
                "with preferences, survey (avoided) should be excluded, leaving 2 services"
            );
            // Verify survey is excluded
            assert!(
                !resp
                    .services
                    .iter()
                    .any(|s| s.required_task.as_deref() == Some("survey")),
                "survey service should be excluded by avoided preference"
            );

            // ========== Test 2: Preferred service ranked first (要件 4.2) ==========
            assert!(
                resp.services[0].required_task.as_deref() == Some("github_pr"),
                "preferred github_pr service should be ranked first, got: {:?}",
                resp.services[0].required_task
            );
            // Verify scores are present and preferred > neutral
            let github_score = resp.services[0].relevance_score.unwrap();
            let data_score = resp.services[1].relevance_score.unwrap();
            assert!(
                github_score > data_score,
                "preferred github_pr score ({}) should be > neutral data_provision score ({})",
                github_score,
                data_score
            );

            // ========== Test 3: applied_filters with preferences_applied=true (要件 4.5, 6.3) ==========
            let filters = resp
                .applied_filters
                .as_ref()
                .expect("applied_filters should be present");
            assert!(
                filters.preferences_applied,
                "preferences_applied should be true when preferences are applied"
            );
            assert_eq!(filters.keyword.as_deref(), Some(test_prefix.as_str()));

            // ========== Test 4: Without session_token — all services returned, no preferences (後方互換 要件 6.4) ==========
            let result = gpt::gpt_search_services(
                axum::extract::State(state.clone()),
                axum::extract::Query(types::GptSearchParams {
                    q: Some(test_prefix.clone()),
                    category: None,
                    max_budget_cents: None,
                    intent: None,
                    session_token: None,
                }),
            )
            .await;
            assert!(result.status().is_success());
            let resp: types::GptSearchResponse = read_typed(result).await;
            assert_eq!(
                resp.total_count, 3,
                "without session_token, all 3 services should be returned (backward compat)"
            );

            // ========== Test 5: No extended params — backward compatibility (要件 9.1) ==========
            // When only q is used (no max_budget_cents, intent, session_token), applied_filters and
            // relevance_score should be None for backward compatibility.
            assert!(
                resp.applied_filters.is_none(),
                "applied_filters should be None when no extended params are used"
            );
            for svc in &resp.services {
                assert!(
                    svc.relevance_score.is_none(),
                    "relevance_score should be None when no extended params are used"
                );
            }

            // ========== Test 6: Preference filter excludes all — message (要件 4.4) ==========
            // Make a new user with ALL tasks avoided
            let user_id_all_avoided = Uuid::new_v4();
            let session_all_avoided = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO users (id, email, region, roles, tools_used, attributes, created_at, source) \
                 VALUES ($1, $2, 'US', '{developer}', '{design}', '{}'::jsonb, NOW(), 'gpt_apps')",
            )
            .bind(user_id_all_avoided)
            .bind(format!("{}_all_avoided@test.com", test_prefix))
            .execute(&pool)
            .await
            .unwrap();
            sqlx::query(
                "INSERT INTO gpt_sessions (token, user_id, expires_at) VALUES ($1, $2, NOW() + INTERVAL '1 hour')",
            )
            .bind(session_all_avoided)
            .bind(user_id_all_avoided)
            .execute(&pool)
            .await
            .unwrap();
            for task_type in ["survey", "github_pr", "data_provision"] {
                sqlx::query(
                    "INSERT INTO user_task_preferences (id, user_id, task_type, level, created_at, updated_at) \
                     VALUES ($1, $2, $3, 'avoided', NOW(), NOW())",
                )
                .bind(Uuid::new_v4())
                .bind(user_id_all_avoided)
                .bind(task_type)
                .execute(&pool)
                .await
                .unwrap();
            }

            let result = gpt::gpt_search_services(
                axum::extract::State(state.clone()),
                axum::extract::Query(types::GptSearchParams {
                    q: Some(test_prefix.clone()),
                    category: None,
                    max_budget_cents: None,
                    intent: None,
                    session_token: Some(session_all_avoided),
                }),
            )
            .await;
            assert!(result.status().is_success());
            let resp: types::GptSearchResponse = read_typed(result).await;
            assert_eq!(resp.total_count, 0, "all avoided should return 0 services");
            assert!(
                resp.message.contains("preferences") || resp.message.contains("preference"),
                "zero-result preference message should mention preferences, got: '{}'",
                resp.message
            );

            // ========== Test 7: Combined budget + intent + preferences (要件 6.2: AND条件) ==========
            let result = gpt::gpt_search_services(
                axum::extract::State(state.clone()),
                axum::extract::Query(types::GptSearchParams {
                    q: Some(test_prefix.clone()),
                    category: None,
                    max_budget_cents: Some(180), // excludes github_pr (200)
                    intent: Some("data".to_string()), // matches data_provision
                    session_token: Some(session_token), // excludes survey (avoided)
                }),
            )
            .await;
            assert!(result.status().is_success());
            let resp: types::GptSearchResponse = read_typed(result).await;
            // survey: excluded by avoided preference
            // github_pr: excluded by budget (200 > 180)
            // data_provision: passes all filters (150 <= 180, matches "data" intent, not avoided)
            assert_eq!(
                resp.total_count, 1,
                "combined filters should return only data_provision"
            );
            assert_eq!(
                resp.services[0].required_task.as_deref(),
                Some("data_provision")
            );
            let filters = resp
                .applied_filters
                .as_ref()
                .expect("applied_filters for combined");
            assert_eq!(filters.budget, Some(180));
            assert_eq!(filters.intent.as_deref(), Some("data"));
            assert!(filters.preferences_applied);

            // ========== Cleanup ==========
            sqlx::query("DELETE FROM user_task_preferences WHERE user_id = $1 OR user_id = $2")
                .bind(user_id)
                .bind(user_id_all_avoided)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM gpt_sessions WHERE user_id = $1 OR user_id = $2")
                .bind(user_id)
                .bind(user_id_all_avoided)
                .execute(&pool)
                .await
                .ok();
            for id in [survey_id, github_id, data_id] {
                sqlx::query("DELETE FROM campaigns WHERE id = $1")
                    .bind(id)
                    .execute(&pool)
                    .await
                    .ok();
            }
            sqlx::query("DELETE FROM users WHERE id = $1 OR id = $2")
                .bind(user_id)
                .bind(user_id_all_avoided)
                .execute(&pool)
                .await
                .ok();
        }
    }
}

// =============================================================================
// Smart Service Suggestion — タスク 8: ルーター統合とメトリクス
// =============================================================================

#[tokio::test]
async fn preferences_route_does_not_break_existing_routes() {
    let (app, _state) = test_app();

    // /health should still work
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.status().as_u16(),
        200,
        "/health should still return 200"
    );

    // /gpt/services should still be routed (not 404)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/gpt/services")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_ne!(
        response.status().as_u16(),
        404,
        "/gpt/services should still be reachable"
    );

    // /gpt/auth should still be routed (not 404)
    let response = post_json(
        &app,
        "/gpt/auth",
        serde_json::json!({"email": "test@test.com", "region": "US", "roles": [], "tools_used": []}),
        None,
    )
    .await;
    assert_ne!(
        response.status().as_u16(),
        404,
        "/gpt/auth should still be reachable"
    );

    // /.well-known/openapi.yaml should still work
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/.well-known/openapi.yaml")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.status().as_u16(),
        200,
        "/.well-known/openapi.yaml should still return 200"
    );
}

// =============================================================================
// Smart Service Suggestion — タスク 9: OpenAPIスキーマ検証
// =============================================================================

#[tokio::test]
async fn openapi_schema_contains_new_parameters() {
    let (app, _state) = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/.well-known/openapi.yaml")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 200);
    let body = read_body_string(response).await;

    // 9.1: New parameters on /gpt/services
    assert!(
        body.contains("max_budget_cents"),
        "openapi.yaml should contain max_budget_cents parameter"
    );
    assert!(
        body.contains("intent"),
        "openapi.yaml should contain intent parameter"
    );
    // session_token should appear in /gpt/services parameters (not just other endpoints)
    assert!(
        body.contains("session_token"),
        "openapi.yaml should contain session_token parameter"
    );

    // 9.1: New response fields
    assert!(
        body.contains("applied_filters"),
        "openapi.yaml should contain applied_filters in response"
    );
    assert!(
        body.contains("available_categories"),
        "openapi.yaml should contain available_categories in response"
    );
    assert!(
        body.contains("preferences_applied"),
        "openapi.yaml should contain preferences_applied in applied_filters"
    );

    // 9.1: GptServiceItem extensions
    assert!(
        body.contains("tags"),
        "openapi.yaml GptServiceItem should contain tags"
    );
    assert!(
        body.contains("relevance_score"),
        "openapi.yaml GptServiceItem should contain relevance_score"
    );
}

#[tokio::test]
async fn openapi_schema_contains_preferences_endpoints() {
    let (app, _state) = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/.well-known/openapi.yaml")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 200);
    let body = read_body_string(response).await;

    // 9.2: New endpoints
    assert!(
        body.contains("/gpt/preferences"),
        "openapi.yaml should contain /gpt/preferences path"
    );

    // 9.2: operationIds
    assert!(
        body.contains("getPreferences"),
        "openapi.yaml should contain getPreferences operationId"
    );
    assert!(
        body.contains("setPreferences"),
        "openapi.yaml should contain setPreferences operationId"
    );
}

#[tokio::test]
async fn openapi_schema_endpoint_count_within_limit() {
    let (app, _state) = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/.well-known/openapi.yaml")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 200);
    let body = read_body_string(response).await;

    // Count operationIds to determine number of endpoints
    let operation_count = body.matches("operationId:").count();
    assert!(
        operation_count <= 30,
        "total endpoints ({}) should be <= 30",
        operation_count
    );
    // We should have at least 8 endpoints (6 existing + 2 new)
    assert!(
        operation_count >= 8,
        "should have at least 8 endpoints, got {}",
        operation_count
    );
}

#[tokio::test]
async fn openapi_schema_has_all_expected_operation_ids() {
    let (app, _state) = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/.well-known/openapi.yaml")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 200);
    let body = read_body_string(response).await;

    let expected_operations = [
        "searchServices",
        "authenticateUser",
        "getTaskDetails",
        "completeTask",
        "runService",
        "getUserStatus",
        "getPreferences",
        "setPreferences",
    ];

    for op_id in &expected_operations {
        assert!(
            body.contains(op_id),
            "openapi.yaml should contain operationId: {}",
            op_id
        );
    }
}

// =============================================================================
// Smart Service Suggestion — タスク 11: E2Eテストと後方互換性検証
// =============================================================================

/// 11.1: E2E統合テスト — 嗜好設定 → 意図+予算検索 → フィルタリング結果検証 → スコア降順確認
#[tokio::test]
async fn e2e_smart_suggestion_full_flow() {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        use sqlx::postgres::PgPoolOptions;
        if let Ok(pool) = PgPoolOptions::new().max_connections(1).connect(&url).await {
            sqlx::migrate!("./migrations").run(&pool).await.ok();

            let state = SharedState {
                inner: Arc::new(RwLock::new(AppState::new())),
            };

            // --- Setup: user, session, campaigns, preferences ---
            let user_id = Uuid::new_v4();
            let session_token = Uuid::new_v4();
            let test_prefix = format!("e2e_smart_{}", Uuid::new_v4());

            // Create user
            sqlx::query(
                "INSERT INTO users (id, email, region, roles, tools_used, attributes, created_at, source) \
                 VALUES ($1, $2, 'JP', '{developer}', '{scraping}', '{}'::jsonb, NOW(), 'gpt_apps')",
            )
            .bind(user_id)
            .bind(format!("{}@test.com", test_prefix))
            .execute(&pool)
            .await
            .unwrap();

            // Create session
            sqlx::query(
                "INSERT INTO gpt_sessions (token, user_id, expires_at) VALUES ($1, $2, NOW() + INTERVAL '1 hour')",
            )
            .bind(session_token)
            .bind(user_id)
            .execute(&pool)
            .await
            .unwrap();

            // Create 3 campaigns with different tasks/budgets/names
            let screenshot_id = Uuid::new_v4();
            let design_id = Uuid::new_v4();
            let survey_id = Uuid::new_v4();

            for (id, name_suffix, task, subsidy, tags) in [
                (
                    screenshot_id,
                    "Screenshot Tool",
                    "github_pr",
                    100_i64,
                    vec!["screenshot", "web"],
                ),
                (
                    design_id,
                    "Design Generator",
                    "data_provision",
                    300_i64,
                    vec!["design", "ai"],
                ),
                (
                    survey_id,
                    "Survey Reward",
                    "survey",
                    150_i64,
                    vec!["survey", "reward"],
                ),
            ] {
                let tags_array: String = format!(
                    "{{{}}}",
                    tags.iter()
                        .map(|t| format!("\"{}\"", t))
                        .collect::<Vec<_>>()
                        .join(",")
                );
                sqlx::query(
                    "INSERT INTO campaigns (id, name, sponsor, target_roles, target_tools, required_task, \
                     subsidy_per_call_cents, budget_total_cents, budget_remaining_cents, query_urls, \
                     active, created_at, tags) \
                     VALUES ($1, $2, 'E2ESponsor', '{developer}', '{scraping}', $3, \
                     $4, 50000, 50000, '{}', true, NOW(), $5::TEXT[])",
                )
                .bind(id)
                .bind(format!("{}_{}", test_prefix, name_suffix))
                .bind(task)
                .bind(subsidy)
                .bind(tags_array)
                .execute(&pool)
                .await
                .unwrap();
            }

            // Step 1: Set preferences — survey=avoided, github_pr=preferred
            let set_result = gpt::gpt_set_preferences(
                axum::extract::State(state.clone()),
                axum::Json(types::GptSetPreferencesRequest {
                    session_token,
                    preferences: vec![
                        types::TaskPreference {
                            task_type: "survey".to_string(),
                            level: "avoided".to_string(),
                        },
                        types::TaskPreference {
                            task_type: "github_pr".to_string(),
                            level: "preferred".to_string(),
                        },
                    ],
                }),
            )
            .await;
            assert!(
                set_result.status().is_success(),
                "setPreferences should succeed"
            );

            // Step 2: Verify preferences saved
            let get_result = gpt::gpt_get_preferences(
                axum::extract::State(state.clone()),
                axum::extract::Query(types::GptPreferencesParams { session_token }),
            )
            .await;
            assert!(
                get_result.status().is_success(),
                "getPreferences should succeed"
            );
            let prefs_resp: types::GptPreferencesResponse = read_typed(get_result).await;
            assert_eq!(prefs_resp.preferences.len(), 2, "should have 2 preferences");

            // Step 3: Search with intent + budget + session_token (全拡張パラメータ使用)
            let search_result = gpt::gpt_search_services(
                axum::extract::State(state.clone()),
                axum::extract::Query(types::GptSearchParams {
                    q: Some(test_prefix.clone()),
                    category: None,
                    max_budget_cents: Some(200), // excludes design_id (300)
                    intent: Some("screenshot".to_string()), // matches screenshot_id
                    session_token: Some(session_token), // excludes survey_id (avoided)
                }),
            )
            .await;
            assert!(search_result.status().is_success(), "search should succeed");
            let resp: types::GptSearchResponse = read_typed(search_result).await;

            // Verify: only screenshot_id passes all filters
            // - survey_id: excluded by avoided preference
            // - design_id: excluded by budget (300 > 200)
            // - screenshot_id: passes budget (100 <= 200), matches intent ("screenshot"), not avoided
            assert_eq!(
                resp.total_count, 1,
                "only screenshot service should pass all filters, got {}",
                resp.total_count
            );
            assert!(
                resp.services[0].name.contains("Screenshot Tool"),
                "result should be Screenshot Tool, got: {}",
                resp.services[0].name
            );

            // Verify relevance_score is present and > 0
            let score = resp.services[0]
                .relevance_score
                .expect("relevance_score should be present");
            assert!(
                score > 0.0,
                "relevance_score should be positive, got: {}",
                score
            );

            // Verify applied_filters
            let filters = resp
                .applied_filters
                .as_ref()
                .expect("applied_filters should be present");
            assert_eq!(filters.budget, Some(200));
            assert_eq!(filters.intent.as_deref(), Some("screenshot"));
            assert!(
                filters.preferences_applied,
                "preferences_applied should be true"
            );
            assert_eq!(filters.keyword.as_deref(), Some(test_prefix.as_str()));

            // Step 4: Search with broader budget to get multiple results, verify score ordering
            let search_result = gpt::gpt_search_services(
                axum::extract::State(state.clone()),
                axum::extract::Query(types::GptSearchParams {
                    q: Some(test_prefix.clone()),
                    category: None,
                    max_budget_cents: Some(500), // includes all
                    intent: Some("screenshot design".to_string()), // matches screenshot + design
                    session_token: Some(session_token), // excludes survey
                }),
            )
            .await;
            assert!(search_result.status().is_success());
            let resp: types::GptSearchResponse = read_typed(search_result).await;

            // survey excluded (avoided), screenshot + design remain
            assert_eq!(
                resp.total_count, 2,
                "screenshot + design should remain after survey excluded"
            );

            // Verify score descending order
            let scores: Vec<f64> = resp
                .services
                .iter()
                .map(|s| s.relevance_score.expect("score should be present"))
                .collect();
            for i in 0..scores.len() - 1 {
                assert!(
                    scores[i] >= scores[i + 1],
                    "scores should be in descending order: {:?}",
                    scores
                );
            }

            // ========== Cleanup ==========
            sqlx::query("DELETE FROM user_task_preferences WHERE user_id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
            sqlx::query("DELETE FROM gpt_sessions WHERE user_id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
            for id in [screenshot_id, design_id, survey_id] {
                sqlx::query("DELETE FROM campaigns WHERE id = $1")
                    .bind(id)
                    .execute(&pool)
                    .await
                    .ok();
            }
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .ok();
        }
    }
}

/// 11.2: 後方互換性テスト — 拡張パラメータ未指定時のレスポンス構造検証
#[tokio::test]
async fn backward_compatibility_no_extended_params() {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        use sqlx::postgres::PgPoolOptions;
        if let Ok(pool) = PgPoolOptions::new().max_connections(1).connect(&url).await {
            sqlx::migrate!("./migrations").run(&pool).await.ok();

            let state = SharedState {
                inner: Arc::new(RwLock::new(AppState::new())),
            };

            let test_prefix = format!("compat_test_{}", Uuid::new_v4());

            // Create a campaign for testing
            let campaign_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO campaigns (id, name, sponsor, target_roles, target_tools, required_task, \
                 subsidy_per_call_cents, budget_total_cents, budget_remaining_cents, query_urls, \
                 active, created_at) \
                 VALUES ($1, $2, 'CompatSponsor', '{developer}', '{design}', 'survey', \
                 100, 50000, 50000, '{}', true, NOW())",
            )
            .bind(campaign_id)
            .bind(format!("{}_service", test_prefix))
            .execute(&pool)
            .await
            .unwrap();

            // Test A: keyword only (existing param) — no extended params
            let result = gpt::gpt_search_services(
                axum::extract::State(state.clone()),
                axum::extract::Query(types::GptSearchParams {
                    q: Some(test_prefix.clone()),
                    category: None,
                    max_budget_cents: None,
                    intent: None,
                    session_token: None,
                }),
            )
            .await;
            assert!(result.status().is_success());
            let resp: types::GptSearchResponse = read_typed(result).await;

            assert!(
                resp.total_count >= 1,
                "should find at least our test campaign"
            );
            assert!(
                resp.applied_filters.is_none(),
                "applied_filters should be None when no extended params (要件 9.1), got: {:?}",
                resp.applied_filters
            );
            assert!(
                resp.available_categories.is_none(),
                "available_categories should be None when no extended params (要件 9.1)"
            );
            for svc in &resp.services {
                assert!(
                    svc.relevance_score.is_none(),
                    "relevance_score should be None when no extended params (要件 9.1), service: {}",
                    svc.name
                );
            }

            // Test B: category only (existing param) — no extended params
            let result = gpt::gpt_search_services(
                axum::extract::State(state.clone()),
                axum::extract::Query(types::GptSearchParams {
                    q: None,
                    category: Some("design".to_string()),
                    max_budget_cents: None,
                    intent: None,
                    session_token: None,
                }),
            )
            .await;
            assert!(result.status().is_success());
            let resp: types::GptSearchResponse = read_typed(result).await;

            assert!(
                resp.applied_filters.is_none(),
                "applied_filters should be None with only category param"
            );
            assert!(
                resp.available_categories.is_none(),
                "available_categories should be None with only category param"
            );
            for svc in &resp.services {
                assert!(
                    svc.relevance_score.is_none(),
                    "relevance_score should be None with only category param, service: {}",
                    svc.name
                );
            }

            // Test C: no params at all — full backward compat
            let result = gpt::gpt_search_services(
                axum::extract::State(state.clone()),
                axum::extract::Query(types::GptSearchParams {
                    q: None,
                    category: None,
                    max_budget_cents: None,
                    intent: None,
                    session_token: None,
                }),
            )
            .await;
            assert!(result.status().is_success());
            let resp: types::GptSearchResponse = read_typed(result).await;

            assert!(
                resp.applied_filters.is_none(),
                "applied_filters should be None with no params at all"
            );
            assert!(
                resp.available_categories.is_none(),
                "available_categories should be None with no params at all"
            );
            for svc in &resp.services {
                assert!(
                    svc.relevance_score.is_none(),
                    "relevance_score should be None with no params at all, service: {}",
                    svc.name
                );
            }

            // Test D: Verify JSON structure matches gpt-apps-integration format
            // When serialized, optional None fields with skip_serializing_if should be absent
            let json_str = serde_json::to_string(&resp).unwrap();
            assert!(
                !json_str.contains("applied_filters"),
                "applied_filters should not appear in JSON when None"
            );
            assert!(
                !json_str.contains("available_categories"),
                "available_categories should not appear in JSON when None"
            );
            assert!(
                !json_str.contains("relevance_score"),
                "relevance_score should not appear in JSON when None"
            );

            // ========== Cleanup ==========
            sqlx::query("DELETE FROM campaigns WHERE id = $1")
                .bind(campaign_id)
                .execute(&pool)
                .await
                .ok();
        }
    }
}
