use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::Response,
    Json,
};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use uuid::Uuid;

use chrono::Utc;

use crate::error::{ApiError, ApiResult};
use crate::types::{
    Campaign, CampaignRow as FullCampaignRow, GptAuthRequest, GptAuthResponse,
    GptAvailableService, GptCompleteTaskRequest, GptCompleteTaskResponse,
    GptCompletedTaskSummary, GptRunServiceRequest, GptRunServiceResponse, GptSearchParams,
    GptSearchResponse, GptServiceItem, GptTaskInputFormat, GptTaskParams, GptTaskResponse,
    GptUserStatusParams, GptUserStatusResponse, SharedState, UserProfile,
};
use crate::utils::respond;

pub struct RateLimiter {
    tokens: u32,
    max_tokens: u32,
    last_refill: Instant,
    refill_interval: Duration,
}

impl RateLimiter {
    pub fn new(max_tokens: u32, refill_interval: Duration) -> Self {
        Self {
            tokens: max_tokens,
            max_tokens,
            last_refill: Instant::now(),
            refill_interval,
        }
    }

    pub fn try_acquire(&mut self) -> bool {
        self.refill();
        if self.tokens > 0 {
            self.tokens -= 1;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        if elapsed >= self.refill_interval {
            let new_tokens = (elapsed.as_millis() / self.refill_interval.as_millis()) as u32;
            self.tokens = (self.tokens + new_tokens).min(self.max_tokens);
            self.last_refill = now;
        }
    }

    fn retry_after_secs(&self) -> u64 {
        self.refill_interval.as_secs().max(1)
    }
}

pub async fn rate_limit_middleware(
    State(limiter): State<Arc<Mutex<RateLimiter>>>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<Response, ApiError> {
    let retry_after = {
        let mut lim = limiter.lock().await;
        if lim.try_acquire() {
            None
        } else {
            Some(lim.retry_after_secs())
        }
    };

    match retry_after {
        Some(secs) => Err(ApiError::rate_limited(secs)),
        None => Ok(next.run(request).await),
    }
}

pub async fn verify_gpt_api_key(
    State(state): State<SharedState>,
    headers: HeaderMap,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<Response, ApiError> {
    let expected_key = {
        let s = state.inner.read().await;
        s.config.gpt_actions_api_key.clone()
    };

    let expected_key = match expected_key {
        Some(k) if !k.is_empty() => k,
        _ => return Ok(next.run(request).await),
    };

    let auth_header = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::unauthorized("API key required"))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| ApiError::unauthorized("Invalid authorization format"))?;

    if token != expected_key {
        return Err(ApiError::forbidden("Invalid API key"));
    }

    Ok(next.run(request).await)
}

pub async fn resolve_session(
    db: &PgPool,
    session_token: Uuid,
) -> ApiResult<Uuid> {
    let row = sqlx::query_scalar::<_, Uuid>(
        "SELECT user_id FROM gpt_sessions WHERE token = $1 AND expires_at > NOW()"
    )
    .bind(session_token)
    .fetch_optional(db)
    .await
    .map_err(|e| ApiError::internal(format!("session lookup failed: {e}")))?;

    row.ok_or_else(|| ApiError::unauthorized("invalid or expired session token"))
}

pub async fn gpt_search_services(
    State(state): State<SharedState>,
    Query(params): Query<GptSearchParams>,
) -> Response {
    let metrics = { state.inner.read().await.metrics.clone() };
    let result: ApiResult<Json<GptSearchResponse>> = async {
    let db = {
        let s = state.inner.read().await;
        s.db.clone().ok_or_else(|| ApiError::internal("database not configured"))?
    };

    let mut services: Vec<GptServiceItem> = Vec::new();

    // Query campaigns (active=true)
    let campaign_rows = sqlx::query_as::<_, CampaignRow>(
        "SELECT id, name, sponsor, required_task, subsidy_per_call_cents, target_tools, active \
         FROM campaigns WHERE active = true"
    )
    .fetch_all(&db)
    .await
    .map_err(|e| ApiError::internal(format!("campaign query failed: {e}")))?;

    for row in campaign_rows {
        services.push(GptServiceItem {
            service_type: "campaign".to_string(),
            service_id: row.id,
            name: row.name,
            sponsor: row.sponsor,
            required_task: Some(row.required_task),
            subsidy_amount_cents: row.subsidy_per_call_cents as u64,
            category: row.target_tools,
            active: row.active,
        });
    }

    // Query sponsored_apis (active=true)
    let api_rows = sqlx::query_as::<_, SponsoredApiRow>(
        "SELECT id, name, sponsor, service_key, price_cents, active \
         FROM sponsored_apis WHERE active = true"
    )
    .fetch_all(&db)
    .await
    .map_err(|e| ApiError::internal(format!("sponsored_api query failed: {e}")))?;

    for row in api_rows {
        services.push(GptServiceItem {
            service_type: "sponsored_api".to_string(),
            service_id: row.id,
            name: row.name,
            sponsor: row.sponsor,
            required_task: None,
            subsidy_amount_cents: row.price_cents as u64,
            category: vec![row.service_key],
            active: row.active,
        });
    }

    // Filter by q (keyword search on name/sponsor)
    if let Some(ref q) = params.q {
        let q_lower = q.to_lowercase();
        services.retain(|s| {
            s.name.to_lowercase().contains(&q_lower)
                || s.sponsor.to_lowercase().contains(&q_lower)
        });
    }

    // Filter by category (match against category/target_tools)
    if let Some(ref category) = params.category {
        let cat_lower = category.to_lowercase();
        services.retain(|s| {
            s.category.iter().any(|c| c.to_lowercase() == cat_lower)
        });
    }

    let total_count = services.len();
    let message = if total_count == 0 {
        "No services found matching your criteria.".to_string()
    } else {
        format!("Found {} service(s) matching your criteria.", total_count)
    };

    Ok(Json(GptSearchResponse {
        services,
        total_count,
        message,
    }))
    }
    .await;
    respond(&metrics, "gpt_search_services", result)
}

#[derive(sqlx::FromRow)]
struct CampaignRow {
    id: Uuid,
    name: String,
    sponsor: String,
    required_task: String,
    subsidy_per_call_cents: i64,
    target_tools: Vec<String>,
    active: bool,
}

#[derive(sqlx::FromRow)]
struct SponsoredApiRow {
    id: Uuid,
    name: String,
    sponsor: String,
    service_key: String,
    price_cents: i64,
    active: bool,
}

pub async fn gpt_auth(
    State(state): State<SharedState>,
    Json(payload): Json<GptAuthRequest>,
) -> Response {
    let metrics = { state.inner.read().await.metrics.clone() };
    let result: ApiResult<Json<GptAuthResponse>> = async {
    let db = {
        let s = state.inner.read().await;
        s.db.clone().ok_or_else(|| ApiError::internal("database not configured"))?
    };

    // Look up existing user by email
    let existing_user: Option<(Uuid, String)> = sqlx::query_as(
        "SELECT id, email FROM users WHERE email = $1"
    )
    .bind(&payload.email)
    .fetch_optional(&db)
    .await
    .map_err(|e| ApiError::internal(format!("user lookup failed: {e}")))?;

    let (user_id, is_new_user) = match existing_user {
        Some((id, _)) => (id, false),
        None => {
            // Insert new user with source = "gpt_apps"
            let new_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO users (id, email, region, roles, tools_used, attributes, created_at, source) \
                 VALUES ($1, $2, $3, $4, $5, '{}'::jsonb, NOW(), 'gpt_apps')"
            )
            .bind(new_id)
            .bind(&payload.email)
            .bind(&payload.region)
            .bind(&payload.roles)
            .bind(&payload.tools_used)
            .execute(&db)
            .await
            .map_err(|e| ApiError::internal(format!("user insert failed: {e}")))?;

            (new_id, true)
        }
    };

    // Issue session token (token is auto-generated by DB default)
    let session_token: Uuid = sqlx::query_scalar(
        "INSERT INTO gpt_sessions (user_id) VALUES ($1) RETURNING token"
    )
    .bind(user_id)
    .fetch_one(&db)
    .await
    .map_err(|e| ApiError::internal(format!("session creation failed: {e}")))?;

    let message = if is_new_user {
        "Welcome! Your account has been created.".to_string()
    } else {
        "Welcome back! You have been authenticated.".to_string()
    };

    Ok(Json(GptAuthResponse {
        session_token,
        user_id,
        email: payload.email,
        is_new_user,
        message,
    }))
    }
    .await;
    respond(&metrics, "gpt_auth", result)
}

#[derive(sqlx::FromRow)]
struct CampaignDetailRow {
    #[allow(dead_code)]
    id: Uuid,
    name: String,
    sponsor: String,
    required_task: String,
    subsidy_per_call_cents: i64,
    task_schema: Option<serde_json::Value>,
}

pub async fn gpt_get_tasks(
    State(state): State<SharedState>,
    Path(campaign_id): Path<Uuid>,
    Query(params): Query<GptTaskParams>,
) -> Response {
    let metrics = { state.inner.read().await.metrics.clone() };
    let result: ApiResult<Json<GptTaskResponse>> = async {
    let db = {
        let s = state.inner.read().await;
        s.db.clone().ok_or_else(|| ApiError::internal("database not configured"))?
    };

    let user_id = resolve_session(&db, params.session_token).await?;

    let campaign = sqlx::query_as::<_, CampaignDetailRow>(
        "SELECT id, name, sponsor, required_task, subsidy_per_call_cents, task_schema \
         FROM campaigns WHERE id = $1"
    )
    .bind(campaign_id)
    .fetch_optional(&db)
    .await
    .map_err(|e| ApiError::internal(format!("campaign query failed: {e}")))?
    .ok_or_else(|| ApiError::not_found("campaign not found"))?;

    let already_completed = crate::utils::has_completed_task(
        &db, campaign_id, user_id, &campaign.required_task
    ).await?;

    let task_input_format = match campaign.task_schema {
        Some(schema) => {
            let task_type = schema.get("task_type")
                .and_then(|v| v.as_str())
                .unwrap_or("survey")
                .to_string();
            let required_fields = schema.get("required_fields")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_else(|| vec!["email".to_string(), "region".to_string()]);
            let instructions = schema.get("instructions")
                .and_then(|v| v.as_str())
                .unwrap_or("Please provide the required information.")
                .to_string();
            GptTaskInputFormat {
                task_type,
                required_fields,
                instructions,
            }
        }
        None => GptTaskInputFormat {
            task_type: "survey".to_string(),
            required_fields: vec!["email".to_string(), "region".to_string()],
            instructions: "Please provide the required information to complete this task.".to_string(),
        },
    };

    let message = if already_completed {
        format!(
            "You have already completed the task '{}'. You can proceed to use the service.",
            campaign.required_task
        )
    } else {
        format!(
            "Please complete the task '{}' to unlock the sponsored service.",
            campaign.required_task
        )
    };

    Ok(Json(GptTaskResponse {
        campaign_id,
        campaign_name: campaign.name,
        sponsor: campaign.sponsor,
        required_task: campaign.required_task,
        task_description: "Complete the required task to access the sponsored service.".to_string(),
        task_input_format,
        already_completed,
        subsidy_amount_cents: campaign.subsidy_per_call_cents as u64,
        message,
    }))
    }
    .await;
    respond(&metrics, "gpt_get_tasks", result)
}

pub async fn gpt_complete_task(
    State(state): State<SharedState>,
    Path(campaign_id): Path<Uuid>,
    Json(payload): Json<GptCompleteTaskRequest>,
) -> Response {
    let metrics = { state.inner.read().await.metrics.clone() };
    let result: ApiResult<Json<GptCompleteTaskResponse>> = async {
    let db = {
        let s = state.inner.read().await;
        s.db.clone().ok_or_else(|| ApiError::internal("database not configured"))?
    };

    let user_id = resolve_session(&db, payload.session_token).await?;

    // Verify campaign exists
    let campaign_exists = sqlx::query_scalar::<_, bool>(
        "SELECT exists(SELECT 1 FROM campaigns WHERE id = $1)"
    )
    .bind(campaign_id)
    .fetch_one(&db)
    .await
    .map_err(|e| ApiError::internal(format!("campaign check failed: {e}")))?;

    if !campaign_exists {
        return Err(ApiError::not_found("campaign not found"));
    }

    // Record consent (3 types: data_sharing, contact, retention)
    let now = Utc::now();
    let consent_records = [
        ("data_sharing", payload.consent.data_sharing_agreed, Some("Data sharing with sponsor")),
        ("contact", payload.consent.contact_permission, Some("Contact permission")),
        ("retention", payload.consent.purpose_acknowledged, Some("Data retention acknowledgement")),
    ];

    for (consent_type, granted, purpose) in &consent_records {
        sqlx::query(
            "INSERT INTO consents (id, user_id, campaign_id, consent_type, granted, purpose, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)"
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(campaign_id)
        .bind(*consent_type)
        .bind(*granted)
        .bind(*purpose)
        .bind(now)
        .execute(&db)
        .await
        .map_err(|e| ApiError::internal(format!("consent insert failed: {e}")))?;
    }

    // Record task completion (reuse existing logic pattern)
    let task_completion_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO task_completions (id, campaign_id, user_id, task_name, details, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(task_completion_id)
    .bind(campaign_id)
    .bind(user_id)
    .bind(&payload.task_name)
    .bind(payload.details.as_deref())
    .bind(now)
    .execute(&db)
    .await
    .map_err(|e| ApiError::internal(format!("task completion insert failed: {e}")))?;

    let message = if payload.consent.data_sharing_agreed {
        "Task completed successfully. Consent recorded. You can now use the sponsored service.".to_string()
    } else {
        "Task completed successfully. However, data sharing was not agreed, so your data will not be transferred to the sponsor. You can still use the service.".to_string()
    };

    Ok(Json(GptCompleteTaskResponse {
        task_completion_id,
        campaign_id,
        consent_recorded: true,
        can_use_service: true,
        message,
    }))
    }
    .await;
    respond(&metrics, "gpt_complete_task", result)
}

pub async fn gpt_run_service(
    State(state): State<SharedState>,
    Path(service): Path<String>,
    Json(payload): Json<GptRunServiceRequest>,
) -> Response {
    let metrics = { state.inner.read().await.metrics.clone() };
    let result: ApiResult<Json<GptRunServiceResponse>> = async {
    let (db, price) = {
        let s = state.inner.read().await;
        let db = s.db.clone().ok_or_else(|| ApiError::internal("database not configured"))?;
        let price = s.service_price(&service);
        (db, price)
    };

    let user_id = resolve_session(&db, payload.session_token).await?;

    // Load user profile
    let user = sqlx::query_as::<_, UserProfile>(
        "SELECT id, email, region, roles, tools_used, attributes, created_at, source \
         FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(&db)
    .await
    .map_err(|e| ApiError::internal(format!("user lookup failed: {e}")))?
    .ok_or_else(|| ApiError::not_found("user not found"))?;

    // Load active campaigns with enough budget
    let campaign_rows = sqlx::query_as::<_, FullCampaignRow>(
        "SELECT id, name, sponsor, sponsor_wallet_address, target_roles, target_tools, \
         required_task, subsidy_per_call_cents, budget_total_cents, budget_remaining_cents, \
         query_urls, active, created_at \
         FROM campaigns WHERE active = true AND budget_remaining_cents >= $1 \
         ORDER BY created_at DESC"
    )
    .bind(price as i64)
    .fetch_all(&db)
    .await
    .map_err(|e| ApiError::internal(format!("campaign query failed: {e}")))?;

    let campaigns: Vec<Campaign> = campaign_rows
        .into_iter()
        .filter_map(|row| Campaign::try_from(row).ok())
        .collect();

    // Match campaigns: find one where user matches AND task is completed
    let mut match_without_task: Option<Campaign> = None;
    let mut match_with_task: Option<Campaign> = None;

    for campaign in campaigns {
        if !campaign
            .target_tools
            .iter()
            .any(|tool| tool == &service)
        {
            continue;
        }

        if !crate::utils::user_matches_campaign(&user, &campaign) {
            continue;
        }

        match crate::utils::has_completed_task(&db, campaign.id, user_id, &campaign.required_task).await {
            Ok(true) => {
                match_with_task = Some(campaign);
                break;
            }
            Ok(false) => {
                if match_without_task.is_none() {
                    match_without_task = Some(campaign);
                }
            }
            Err(e) => return Err(e),
        }
    }

    // Sponsored match found: deduct budget and record payment
    if let Some(campaign) = match_with_task {
        let updated = sqlx::query(
            "UPDATE campaigns
             SET budget_remaining_cents = budget_remaining_cents - $1,
                 active = (budget_remaining_cents - $1) >= $1 AND (budget_remaining_cents - $1) > 0
             WHERE id = $2
               AND active = true
               AND budget_remaining_cents >= $1"
        )
        .bind(price as i64)
        .bind(campaign.id)
        .execute(&db)
        .await
        .map_err(|e| ApiError::internal(format!("budget update failed: {e}")))?;

        if updated.rows_affected() == 0 {
            return Err(ApiError::precondition(
                "Sponsor budget is no longer sufficient for this service call.",
            ));
        }

        let tx_hash = format!("sponsor-{}", Uuid::new_v4());

        sqlx::query(
            "INSERT INTO payments (tx_hash, campaign_id, service, amount_cents, payer, source, status, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
        )
        .bind(&tx_hash)
        .bind(campaign.id)
        .bind(&service)
        .bind(price as i64)
        .bind(&campaign.sponsor)
        .bind("sponsor")
        .bind("settled")
        .bind(Utc::now())
        .execute(&db)
        .await
        .map_err(|e| ApiError::internal(format!("payment insert failed: {e}")))?;

        let output = format!(
            "Executed '{}' task for user {} with input: {}",
            service, user_id, payload.input
        );

        return Ok(Json(GptRunServiceResponse {
            service,
            output,
            payment_mode: "sponsored".to_string(),
            sponsored_by: Some(campaign.sponsor),
            tx_hash: Some(tx_hash),
            message: "Service executed successfully. This call was sponsored.".to_string(),
        }));
    }

    // Campaign matches but task not yet completed
    if let Some(campaign) = match_without_task {
        return Err(ApiError::precondition(format!(
            "Please complete the required task '{}' for campaign '{}' before using this service.",
            campaign.required_task, campaign.name
        )));
    }

    // No matching campaign
    Err(ApiError::precondition(format!(
        "No sponsored campaign found for service '{}'. You may need to pay directly or check available campaigns.",
        service
    )))
    }
    .await;
    respond(&metrics, "gpt_run_service", result)
}

pub async fn gpt_user_status(
    State(state): State<SharedState>,
    Query(params): Query<GptUserStatusParams>,
) -> Response {
    let metrics = { state.inner.read().await.metrics.clone() };
    let result: ApiResult<Json<GptUserStatusResponse>> = async {
    let db = {
        let s = state.inner.read().await;
        s.db.clone().ok_or_else(|| ApiError::internal("database not configured"))?
    };

    let user_id = resolve_session(&db, params.session_token).await?;

    // Load user profile
    let user = sqlx::query_as::<_, UserProfile>(
        "SELECT id, email, region, roles, tools_used, attributes, created_at, source \
         FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(&db)
    .await
    .map_err(|e| ApiError::internal(format!("user lookup failed: {e}")))?
    .ok_or_else(|| ApiError::not_found("user not found"))?;

    // Load completed tasks with campaign names
    let completed_tasks = sqlx::query_as::<_, CompletedTaskRow>(
        "SELECT tc.campaign_id, c.name AS campaign_name, tc.task_name, tc.created_at \
         FROM task_completions tc \
         JOIN campaigns c ON c.id = tc.campaign_id \
         WHERE tc.user_id = $1 \
         ORDER BY tc.created_at DESC"
    )
    .bind(user_id)
    .fetch_all(&db)
    .await
    .map_err(|e| ApiError::internal(format!("task completions query failed: {e}")))?;

    let completed_task_summaries: Vec<GptCompletedTaskSummary> = completed_tasks
        .into_iter()
        .map(|row| GptCompletedTaskSummary {
            campaign_id: row.campaign_id,
            campaign_name: row.campaign_name,
            task_name: row.task_name,
            completed_at: row.created_at,
        })
        .collect();

    // Load active campaigns and determine available services
    let campaign_rows = sqlx::query_as::<_, FullCampaignRow>(
        "SELECT id, name, sponsor, sponsor_wallet_address, target_roles, target_tools, \
         required_task, subsidy_per_call_cents, budget_total_cents, budget_remaining_cents, \
         query_urls, active, created_at \
         FROM campaigns WHERE active = true \
         ORDER BY created_at DESC"
    )
    .fetch_all(&db)
    .await
    .map_err(|e| ApiError::internal(format!("campaign query failed: {e}")))?;

    let campaigns: Vec<Campaign> = campaign_rows
        .into_iter()
        .filter_map(|row| Campaign::try_from(row).ok())
        .collect();

    let mut available_services: Vec<GptAvailableService> = Vec::new();
    for campaign in &campaigns {
        if !crate::utils::user_matches_campaign(&user, campaign) {
            continue;
        }

        let task_done =
            crate::utils::has_completed_task(&db, campaign.id, user_id, &campaign.required_task)
                .await?;

        for tool in &campaign.target_tools {
            available_services.push(GptAvailableService {
                service: tool.clone(),
                sponsor: campaign.sponsor.clone(),
                ready: task_done,
            });
        }
    }

    let task_count = completed_task_summaries.len();
    let service_count = available_services.len();
    let ready_count = available_services.iter().filter(|s| s.ready).count();

    let message = format!(
        "You have completed {} task(s). {} service(s) available ({} ready to use).",
        task_count, service_count, ready_count
    );

    Ok(Json(GptUserStatusResponse {
        user_id,
        email: user.email,
        completed_tasks: completed_task_summaries,
        available_services,
        message,
    }))
    }
    .await;
    respond(&metrics, "gpt_user_status", result)
}

#[derive(sqlx::FromRow)]
struct CompletedTaskRow {
    campaign_id: Uuid,
    campaign_name: String,
    task_name: String,
    created_at: chrono::DateTime<Utc>,
}
