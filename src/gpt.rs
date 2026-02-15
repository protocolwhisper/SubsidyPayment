use axum::{
    Json,
    extract::{Path, Query, State},
    http::HeaderMap,
    response::Response,
};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use uuid::Uuid;

use chrono::Utc;

use crate::error::{ApiError, ApiResult};
use crate::types::{
    AppliedFilters, Campaign, CampaignRow as FullCampaignRow, GptAuthRequest, GptAuthResponse,
    GptAvailableService, GptCompleteTaskRequest, GptCompleteTaskResponse, GptCompletedTaskSummary,
    GptPreferencesParams, GptPreferencesResponse, GptRunServiceRequest, GptRunServiceResponse,
    GptSearchParams, GptSearchResponse, GptServiceItem, GptSetPreferencesRequest,
    GptSetPreferencesResponse, GptTaskInputFormat, GptTaskParams, GptTaskResponse,
    GptUserStatusParams, GptUserStatusResponse, SharedState, TaskPreference, UserProfile,
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

pub async fn resolve_session(db: &PgPool, session_token: Uuid) -> ApiResult<Uuid> {
    let row = sqlx::query_scalar::<_, Uuid>(
        "SELECT user_id FROM gpt_sessions WHERE token = $1 AND expires_at > NOW()",
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
        "SELECT id, name, sponsor, required_task, subsidy_per_call_cents, target_tools, active, tags \
         FROM campaigns WHERE active = true"
    )
    .fetch_all(&db)
    .await
    .map_err(|e| ApiError::internal(format!("campaign query failed: {e}")))?;

    for row in campaign_rows {
        let inferred_tags = infer_tags(&row.tags, &row.target_tools, &row.required_task);
        services.push(GptServiceItem {
            service_type: "campaign".to_string(),
            service_id: row.id,
            name: row.name.clone(),
            sponsor: row.sponsor,
            required_task: Some(row.required_task),
            subsidy_amount_cents: row.subsidy_per_call_cents as u64,
            category: row.target_tools,
            active: row.active,
            tags: inferred_tags,
            relevance_score: None,
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
            tags: vec![],
            relevance_score: None,
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

    // Budget filter (要件 1.1): retain only services within budget
    let budget_applied = params.max_budget_cents.is_some();
    if let Some(max_budget) = params.max_budget_cents {
        services.retain(|s| s.subsidy_amount_cents <= max_budget);
    }

    // Intent filter (要件 2.1): retain only services matching intent keywords
    let intent_applied = params.intent.is_some();
    // Save all categories before intent filter for available_categories fallback (要件 2.3)
    let all_categories: Vec<String> = if intent_applied {
        let mut cats: Vec<String> = services
            .iter()
            .flat_map(|s| s.category.clone())
            .collect();
        cats.sort();
        cats.dedup();
        cats
    } else {
        vec![]
    };

    if let Some(ref intent) = params.intent {
        let keywords: Vec<&str> = intent.split_whitespace().collect();
        if !keywords.is_empty() {
            services.retain(|s| matches_intent(s, &keywords));
        }
    }

    // 要件 2.3: Intent search returned 0 results → provide available_categories
    let available_categories = if intent_applied && services.is_empty() && !all_categories.is_empty() {
        Some(all_categories)
    } else {
        None
    };

    // 嗜好フィルタ (要件 4.1–4.3): session_token がある場合に嗜好を適用
    let mut preferences_applied = false;
    let preferences: Vec<TaskPreference> = if let Some(session_token) = params.session_token {
        let user_id = resolve_session(&db, session_token).await?;
        let rows = sqlx::query_as::<_, TaskPreferenceRow>(
            "SELECT task_type, level, updated_at FROM user_task_preferences \
             WHERE user_id = $1 ORDER BY task_type",
        )
        .bind(user_id)
        .fetch_all(&db)
        .await
        .map_err(|e| ApiError::internal(format!("preferences query failed: {e}")))?;

        let prefs: Vec<TaskPreference> = rows
            .into_iter()
            .map(|r| TaskPreference {
                task_type: r.task_type,
                level: r.level,
            })
            .collect();

        if !prefs.is_empty() {
            preferences_applied = true;
            // 要件 4.1: avoided タスクタイプのサービスを除外
            let avoided_tasks: Vec<&str> = prefs
                .iter()
                .filter(|p| p.level == "avoided")
                .map(|p| p.task_type.as_str())
                .collect();
            services.retain(|s| {
                match s.required_task.as_ref() {
                    Some(task) => !avoided_tasks.contains(&task.as_str()),
                    None => true, // sponsored_api without required_task is never excluded
                }
            });
        }

        prefs
    } else {
        vec![]
    };

    // 後方互換性 (要件 9.1): 拡張パラメータが1つでもある場合のみスコア算出・AppliedFilters構築
    let has_extended_params = budget_applied || intent_applied || params.session_token.is_some();

    // スコア算出 (要件 6.5) + スコア降順ソート (要件 1.4)
    if has_extended_params {
        for service in &mut services {
            service.relevance_score = Some(calculate_score(
                service,
                params.max_budget_cents,
                params.intent.as_deref(),
                &preferences,
            ));
        }
        services.sort_by(|a, b| {
            b.relevance_score
                .unwrap_or(0.0)
                .partial_cmp(&a.relevance_score.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    // AppliedFilters 構築 (要件 6.3)
    let applied_filters = if has_extended_params {
        Some(AppliedFilters {
            budget: params.max_budget_cents,
            intent: params.intent.clone(),
            category: params.category.clone(),
            keyword: params.q.clone(),
            preferences_applied,
        })
    } else {
        None
    };

    let total_count = services.len();
    let message = if total_count == 0 {
        if preferences_applied && budget_applied {
            "No services found within your budget and preferences. Consider adjusting your budget or updating your preferences.".to_string()
        } else if preferences_applied {
            // 要件 4.4: 嗜好フィルタで全除外
            "No services found matching your preferences. Consider updating your task preferences to see more services.".to_string()
        } else if budget_applied {
            // 要件 1.3: 予算フィルタで0件 → 予算緩和・直接支払いの案内
            "No services found within your budget. Consider increasing your budget or paying directly for the service.".to_string()
        } else if intent_applied {
            "No services found matching your intent. Check available categories for alternatives.".to_string()
        } else {
            "No services found matching your criteria.".to_string()
        }
    } else {
        format!("Found {} service(s) matching your criteria.", total_count)
    };

    Ok(Json(GptSearchResponse {
        services,
        total_count,
        message,
        applied_filters,
        available_categories,
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
    tags: Vec<String>,
}

/// 意図キーワードがサービスの name, required_task, category, tags にマッチするか判定 (要件 2.1)
pub fn matches_intent(service: &GptServiceItem, keywords: &[&str]) -> bool {
    keywords.iter().any(|kw| {
        let kw_lower = kw.to_lowercase();
        service.name.to_lowercase().contains(&kw_lower)
            || service
                .required_task
                .as_ref()
                .map(|rt| rt.to_lowercase().contains(&kw_lower))
                .unwrap_or(false)
            || service
                .category
                .iter()
                .any(|c| c.to_lowercase().contains(&kw_lower))
            || service
                .tags
                .iter()
                .any(|t| t.to_lowercase().contains(&kw_lower))
    })
}

/// タグ未設定時に target_tools + required_task からデフォルトタグを推定する (要件 5.3)
pub fn infer_tags(tags: &[String], target_tools: &[String], required_task: &str) -> Vec<String> {
    if !tags.is_empty() {
        return tags.to_vec();
    }
    let mut inferred = target_tools.to_vec();
    let rt = required_task.to_string();
    if !inferred.contains(&rt) {
        inferred.push(rt);
    }
    inferred
}

/// スコア算出関数 (要件 6.5)
/// budget_score (重み 0.3) + intent_score (重み 0.4) + preference_score (重み 0.3)
pub fn calculate_score(
    service: &GptServiceItem,
    max_budget_cents: Option<u64>,
    intent: Option<&str>,
    preferences: &[TaskPreference],
) -> f64 {
    // budget_score: max_budget未指定→0.5, 指定あり→1.0 - (cost/budget).min(1.0)
    let budget_score = match max_budget_cents {
        None => 0.5,
        Some(0) => 0.0,
        Some(budget) => 1.0 - (service.subsidy_amount_cents as f64 / budget as f64).min(1.0),
    };

    // intent_score: intent未指定→0.5, 指定あり→matched_fields / total_searchable_fields
    let intent_score = match intent {
        None => 0.5,
        Some(intent_str) => {
            let keywords: Vec<&str> = intent_str.split_whitespace().collect();
            if keywords.is_empty() {
                0.5
            } else {
                // Count how many searchable fields match
                let mut matched = 0u32;
                let mut total = 0u32;

                // name field
                total += 1;
                if keywords
                    .iter()
                    .any(|kw| service.name.to_lowercase().contains(&kw.to_lowercase()))
                {
                    matched += 1;
                }

                // required_task field
                if let Some(ref rt) = service.required_task {
                    total += 1;
                    if keywords
                        .iter()
                        .any(|kw| rt.to_lowercase().contains(&kw.to_lowercase()))
                    {
                        matched += 1;
                    }
                }

                // category fields (count as one field)
                if !service.category.is_empty() {
                    total += 1;
                    if keywords.iter().any(|kw| {
                        service
                            .category
                            .iter()
                            .any(|c| c.to_lowercase().contains(&kw.to_lowercase()))
                    }) {
                        matched += 1;
                    }
                }

                // tags fields (count as one field)
                if !service.tags.is_empty() {
                    total += 1;
                    if keywords.iter().any(|kw| {
                        service
                            .tags
                            .iter()
                            .any(|t| t.to_lowercase().contains(&kw.to_lowercase()))
                    }) {
                        matched += 1;
                    }
                }

                if total == 0 {
                    0.5
                } else {
                    matched as f64 / total as f64
                }
            }
        }
    };

    // preference_score: 嗜好未登録→0.5, preferred→1.0, neutral→0.5, avoided→0.0
    let preference_score = if preferences.is_empty() {
        0.5
    } else {
        match service.required_task.as_ref() {
            Some(task) => {
                match preferences.iter().find(|p| p.task_type == *task) {
                    Some(pref) => match pref.level.as_str() {
                        "preferred" => 1.0,
                        "neutral" => 0.5,
                        "avoided" => 0.0,
                        _ => 0.5,
                    },
                    None => 0.5, // no preference for this task type
                }
            }
            None => 0.5, // sponsored_api without required_task
        }
    };

    budget_score * 0.3 + intent_score * 0.4 + preference_score * 0.3
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
            s.db.clone()
                .ok_or_else(|| ApiError::internal("database not configured"))?
        };

        let user_id = resolve_session(&db, params.session_token).await?;

        let campaign = sqlx::query_as::<_, CampaignDetailRow>(
            "SELECT id, name, sponsor, required_task, subsidy_per_call_cents, task_schema \
         FROM campaigns WHERE id = $1",
        )
        .bind(campaign_id)
        .fetch_optional(&db)
        .await
        .map_err(|e| ApiError::internal(format!("campaign query failed: {e}")))?
        .ok_or_else(|| ApiError::not_found("campaign not found"))?;

        let already_completed =
            crate::utils::has_completed_task(&db, campaign_id, user_id, &campaign.required_task)
                .await?;

        let task_input_format = match campaign.task_schema {
            Some(schema) => {
                let task_type = schema
                    .get("task_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("survey")
                    .to_string();
                let required_fields = schema
                    .get("required_fields")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_else(|| vec!["email".to_string(), "region".to_string()]);
                let instructions = schema
                    .get("instructions")
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
                instructions: "Please provide the required information to complete this task."
                    .to_string(),
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
            task_description: "Complete the required task to access the sponsored service."
                .to_string(),
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
            s.db.clone()
                .ok_or_else(|| ApiError::internal("database not configured"))?
        };

        let user_id = resolve_session(&db, params.session_token).await?;

        // Load user profile
        let user = sqlx::query_as::<_, UserProfile>(
            "SELECT id, email, region, roles, tools_used, attributes, created_at, source \
         FROM users WHERE id = $1",
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
         ORDER BY tc.created_at DESC",
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
         ORDER BY created_at DESC",
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

            let task_done = crate::utils::has_completed_task(
                &db,
                campaign.id,
                user_id,
                &campaign.required_task,
            )
            .await
            .unwrap_or(false);

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

// --- Smart Service Suggestion: 嗜好管理ハンドラ (タスク 4) ---

#[derive(sqlx::FromRow)]
struct TaskPreferenceRow {
    task_type: String,
    level: String,
    updated_at: chrono::DateTime<Utc>,
}

pub async fn gpt_get_preferences(
    State(state): State<SharedState>,
    Query(params): Query<GptPreferencesParams>,
) -> Response {
    let metrics = { state.inner.read().await.metrics.clone() };
    let result: ApiResult<Json<GptPreferencesResponse>> = async {
        let db = {
            let s = state.inner.read().await;
            s.db.clone()
                .ok_or_else(|| ApiError::internal("database not configured"))?
        };

        let user_id = resolve_session(&db, params.session_token).await?;

        let rows = sqlx::query_as::<_, TaskPreferenceRow>(
            "SELECT task_type, level, updated_at FROM user_task_preferences \
             WHERE user_id = $1 ORDER BY task_type",
        )
        .bind(user_id)
        .fetch_all(&db)
        .await
        .map_err(|e| ApiError::internal(format!("preferences query failed: {e}")))?;

        let updated_at = rows.first().map(|r| r.updated_at);

        let preferences: Vec<TaskPreference> = rows
            .into_iter()
            .map(|r| TaskPreference {
                task_type: r.task_type,
                level: r.level,
            })
            .collect();

        let message = if preferences.is_empty() {
            "No preferences set yet. You can set your task preferences to get personalized service recommendations.".to_string()
        } else {
            format!(
                "You have {} preference(s) configured.",
                preferences.len()
            )
        };

        Ok(Json(GptPreferencesResponse {
            user_id,
            preferences,
            updated_at,
            message,
        }))
    }
    .await;
    respond(&metrics, "gpt_get_preferences", result)
}

pub async fn gpt_set_preferences(
    State(state): State<SharedState>,
    Json(payload): Json<GptSetPreferencesRequest>,
) -> Response {
    let metrics = { state.inner.read().await.metrics.clone() };
    let result: ApiResult<Json<GptSetPreferencesResponse>> = async {
        let db = {
            let s = state.inner.read().await;
            s.db.clone()
                .ok_or_else(|| ApiError::internal("database not configured"))?
        };

        let user_id = resolve_session(&db, payload.session_token).await?;

        // Validate preference levels
        for pref in &payload.preferences {
            match pref.level.as_str() {
                "preferred" | "neutral" | "avoided" => {}
                _ => {
                    return Err(ApiError::validation(format!(
                        "invalid preference level '{}' for task_type '{}'. Must be one of: preferred, neutral, avoided",
                        pref.level, pref.task_type
                    )));
                }
            }
        }

        // Delete existing preferences
        sqlx::query("DELETE FROM user_task_preferences WHERE user_id = $1")
            .bind(user_id)
            .execute(&db)
            .await
            .map_err(|e| ApiError::internal(format!("preferences delete failed: {e}")))?;

        // Insert new preferences
        let now = Utc::now();
        for pref in &payload.preferences {
            sqlx::query(
                "INSERT INTO user_task_preferences (id, user_id, task_type, level, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, $5, $5)",
            )
            .bind(Uuid::new_v4())
            .bind(user_id)
            .bind(&pref.task_type)
            .bind(&pref.level)
            .bind(now)
            .execute(&db)
            .await
            .map_err(|e| ApiError::internal(format!("preference insert failed: {e}")))?;
        }

        Ok(Json(GptSetPreferencesResponse {
            user_id,
            preferences_count: payload.preferences.len(),
            updated_at: now,
            message: format!(
                "Successfully updated {} preference(s).",
                payload.preferences.len()
            ),
        }))
    }
    .await;
    respond(&metrics, "gpt_set_preferences", result)
}
