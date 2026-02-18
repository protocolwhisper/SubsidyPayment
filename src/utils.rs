use axum::{
    Json,
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use reqwest::{Client, Method};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    time::Duration,
};
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};
use crate::onchain::{VerifiedX402Payment, verify_and_settle_x402_payment};
use crate::types::{
    AgentServiceMetadata, AppConfig, Campaign, GptServiceItem, Metrics, PAYMENT_RESPONSE_HEADER,
    PAYMENT_SIGNATURE_HEADER, PaymentRequired, SPONSORED_API_SERVICE_PREFIX, ServiceCatalogItem,
    ServiceRunRequest, ServiceRunResponse, SponsorCatalogItem, SponsoredApi, UserProfile,
    X402_VERSION_HEADER, X402PaymentRequirement,
};
use sqlx::PgPool;

const USDC_BASE_UNITS_PER_CENT: u128 = 10_000;

pub fn respond<T: IntoResponse>(
    metrics: &Metrics,
    endpoint: &str,
    result: ApiResult<T>,
) -> Response {
    let response = match result {
        Ok(value) => value.into_response(),
        Err(err) => err.into_response(),
    };

    mark_request(metrics, endpoint, response.status());
    response
}

pub fn user_matches_campaign(user: &UserProfile, campaign: &Campaign) -> bool {
    let role_match = if campaign.target_roles.is_empty() {
        true
    } else {
        user.roles
            .iter()
            .any(|role| campaign.target_roles.iter().any(|target| target == role))
    };

    let tool_match = if campaign.target_tools.is_empty() {
        true
    } else {
        user.tools_used
            .iter()
            .any(|tool| campaign.target_tools.iter().any(|target| target == tool))
    };

    role_match && tool_match
}

pub async fn has_completed_task(
    db: &PgPool,
    campaign_id: Uuid,
    user_id: Uuid,
    required_task: &str,
) -> ApiResult<bool> {
    let exists = sqlx::query_scalar::<_, bool>(
        r#"
        select exists(
            select 1 from task_completions
            where campaign_id = $1
              and user_id = $2
              and task_name = $3
        )
        "#,
    )
    .bind(campaign_id)
    .bind(user_id)
    .bind(required_task)
    .fetch_one(db)
    .await
    .map_err(|err| ApiError::database(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok(exists)
}

pub async fn verify_x402_payment(
    http: &Client,
    config: &AppConfig,
    service: &str,
    amount_cents: u64,
    resource_path: &str,
    headers: &HeaderMap,
) -> ApiResult<VerifiedX402Payment> {
    let Some(signature) = headers
        .get(PAYMENT_SIGNATURE_HEADER)
        .and_then(|value| value.to_str().ok())
    else {
        return Err(payment_required_error(
            config,
            service,
            amount_cents,
            resource_path,
            "missing PAYMENT-SIGNATURE header",
            "create a payment from the PAYMENT-REQUIRED challenge and retry",
        ));
    };

    let requirement = build_payment_requirement(config, service, amount_cents, resource_path)?;
    match verify_and_settle_x402_payment(http, config, signature, &requirement).await {
        Ok(payment) => Ok(payment),
        Err(err) => match err {
            ApiError::Config { .. } => Err(err),
            _ => Err(payment_required_error(
                config,
                service,
                amount_cents,
                resource_path,
                format!("payment rejected: {err}"),
                "regenerate PAYMENT-SIGNATURE from the latest challenge and retry",
            )),
        },
    }
}

pub fn payment_required_error(
    config: &AppConfig,
    service: &str,
    amount_cents: u64,
    resource_path: &str,
    message: impl Into<String>,
    next_step: impl Into<String>,
) -> ApiError {
    let requirement = match build_payment_requirement(config, service, amount_cents, resource_path)
    {
        Ok(value) => value,
        Err(err) => return err,
    };

    let payment_required = match encode_payment_required_header(&requirement) {
        Ok(value) => value,
        Err(err) => return ApiError::internal(err),
    };

    ApiError::PaymentRequired(PaymentRequired {
        service: service.to_string(),
        amount_cents,
        accepted_header: PAYMENT_SIGNATURE_HEADER.to_string(),
        payment_required,
        message: message.into(),
        next_step: next_step.into(),
    })
}

fn build_payment_requirement(
    config: &AppConfig,
    service: &str,
    amount_cents: u64,
    resource_path: &str,
) -> ApiResult<X402PaymentRequirement> {
    let pay_to = required_non_empty_env_like(config.x402_pay_to.as_deref(), "X402_PAY_TO")?;
    let asset = required_non_empty_env_like(config.x402_asset.as_deref(), "X402_ASSET")?;

    let resource = format!(
        "{}{}",
        config.public_base_url.trim_end_matches('/'),
        resource_path
    );

    Ok(X402PaymentRequirement {
        scheme: "exact".to_string(),
        network: config.x402_network.clone(),
        max_amount_required: amount_to_base_units(amount_cents),
        resource,
        description: format!("Access paid service '{service}'"),
        mime_type: "application/json".to_string(),
        pay_to,
        max_timeout_seconds: 300,
        asset,
        output_schema: None,
        extra: HashMap::new(),
    })
}

fn required_non_empty_env_like(value: Option<&str>, key: &str) -> ApiResult<String> {
    let Some(raw) = value else {
        return Err(ApiError::config(format!("{key} is required for x402")));
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(ApiError::config(format!("{key} is required for x402")));
    }
    Ok(trimmed.to_string())
}

fn amount_to_base_units(amount_cents: u64) -> String {
    (u128::from(amount_cents) * USDC_BASE_UNITS_PER_CENT).to_string()
}

fn encode_payment_required_header(requirement: &X402PaymentRequirement) -> Result<String, String> {
    let bytes = serde_json::to_vec(&vec![requirement]).map_err(|err| err.to_string())?;
    Ok(STANDARD.encode(bytes))
}

pub fn build_paid_tool_response(
    service: String,
    request: ServiceRunRequest,
    payment_mode: String,
    sponsored_by: Option<String>,
    tx_hash: Option<String>,
    payment_response_header: Option<&str>,
) -> Response {
    let payload = ServiceRunResponse {
        service: service.clone(),
        output: format!(
            "Executed '{}' task for user {} with input: {}",
            service, request.user_id, request.input
        ),
        payment_mode,
        sponsored_by,
        tx_hash,
    };

    let mut response = (StatusCode::OK, Json(payload)).into_response();
    response.headers_mut().insert(
        HeaderName::from_static(X402_VERSION_HEADER),
        HeaderValue::from_static("2"),
    );

    if let Some(payment_response) = payment_response_header
        && let Ok(header_value) = HeaderValue::from_str(payment_response)
    {
        response.headers_mut().insert(
            HeaderName::from_static(PAYMENT_RESPONSE_HEADER),
            header_value,
        );
    }

    response
}

pub fn mark_request(metrics: &Metrics, endpoint: &str, status: StatusCode) {
    let status_label = status.as_u16().to_string();
    metrics
        .http_requests_total
        .with_label_values(&[endpoint, status_label.as_str()])
        .inc();
}

#[derive(Default)]
struct ServiceCatalogAgg {
    sponsor_names: BTreeSet<String>,
    offer_count: usize,
    min_subsidy_cents: Option<u64>,
    max_subsidy_cents: Option<u64>,
}

#[derive(Default)]
struct SponsorCatalogAgg {
    service_keys: BTreeSet<String>,
    required_tasks: BTreeSet<String>,
    offer_count: usize,
}

fn apply_catalog_offer(
    service_catalog: &mut BTreeMap<String, ServiceCatalogAgg>,
    sponsor_catalog: &mut BTreeMap<String, SponsorCatalogAgg>,
    service_key: &str,
    sponsor: &str,
    subsidy_cents: u64,
    required_task: Option<&str>,
) {
    let normalized_key = normalize_service_key(service_key);
    if normalized_key.is_empty() {
        return;
    }

    let service_entry = service_catalog.entry(normalized_key.clone()).or_default();
    service_entry.sponsor_names.insert(sponsor.to_string());
    service_entry.offer_count += 1;
    service_entry.min_subsidy_cents = Some(
        service_entry
            .min_subsidy_cents
            .map(|v| v.min(subsidy_cents))
            .unwrap_or(subsidy_cents),
    );
    service_entry.max_subsidy_cents = Some(
        service_entry
            .max_subsidy_cents
            .map(|v| v.max(subsidy_cents))
            .unwrap_or(subsidy_cents),
    );

    let sponsor_entry = sponsor_catalog.entry(sponsor.to_string()).or_default();
    sponsor_entry.service_keys.insert(normalized_key);
    sponsor_entry.offer_count += 1;
    if let Some(task) = required_task {
        let task = task.trim();
        if !task.is_empty() {
            sponsor_entry.required_tasks.insert(task.to_string());
        }
    }
}

pub fn build_marketplace_catalog_from_gpt_services(
    services: &[GptServiceItem],
) -> (Vec<ServiceCatalogItem>, Vec<SponsorCatalogItem>) {
    let mut service_catalog: BTreeMap<String, ServiceCatalogAgg> = BTreeMap::new();
    let mut sponsor_catalog: BTreeMap<String, SponsorCatalogAgg> = BTreeMap::new();

    for service in services {
        let mut keys: Vec<String> = if service.category.is_empty() {
            vec![service.name.clone()]
        } else {
            service.category.clone()
        };
        keys.sort();
        keys.dedup();

        for key in keys {
            apply_catalog_offer(
                &mut service_catalog,
                &mut sponsor_catalog,
                &key,
                &service.sponsor,
                service.subsidy_amount_cents,
                service.required_task.as_deref(),
            );
        }
    }

    finalize_marketplace_catalog(service_catalog, sponsor_catalog)
}

pub fn build_marketplace_catalog_from_agent_services(
    services: &[AgentServiceMetadata],
) -> (Vec<ServiceCatalogItem>, Vec<SponsorCatalogItem>) {
    let mut service_catalog: BTreeMap<String, ServiceCatalogAgg> = BTreeMap::new();
    let mut sponsor_catalog: BTreeMap<String, SponsorCatalogAgg> = BTreeMap::new();

    for service in services {
        apply_catalog_offer(
            &mut service_catalog,
            &mut sponsor_catalog,
            &service.service_key,
            &service.sponsor,
            service.subsidy_cents,
            service.required_task.as_deref(),
        );
    }

    finalize_marketplace_catalog(service_catalog, sponsor_catalog)
}

fn finalize_marketplace_catalog(
    service_catalog: BTreeMap<String, ServiceCatalogAgg>,
    sponsor_catalog: BTreeMap<String, SponsorCatalogAgg>,
) -> (Vec<ServiceCatalogItem>, Vec<SponsorCatalogItem>) {
    let services = service_catalog
        .into_iter()
        .map(|(service_key, agg)| ServiceCatalogItem {
            display_name: display_name_for_service_key(&service_key),
            sponsor_count: agg.sponsor_names.len(),
            sponsor_names: agg.sponsor_names.into_iter().collect(),
            offer_count: agg.offer_count,
            min_subsidy_cents: agg.min_subsidy_cents.unwrap_or(0),
            max_subsidy_cents: agg.max_subsidy_cents.unwrap_or(0),
            service_key,
        })
        .collect::<Vec<_>>();

    let sponsors = sponsor_catalog
        .into_iter()
        .map(|(sponsor_name, agg)| SponsorCatalogItem {
            sponsor_archetype: infer_sponsor_archetype(
                &sponsor_name,
                &agg.required_tasks,
                &agg.service_keys,
            ),
            service_keys: agg.service_keys.into_iter().collect(),
            required_tasks: agg.required_tasks.into_iter().collect(),
            offer_count: agg.offer_count,
            sponsor_name,
        })
        .collect::<Vec<_>>();

    (services, sponsors)
}

fn normalize_service_key(raw: &str) -> String {
    raw.trim().to_lowercase().replace(['_', ' '], "-")
}

fn display_name_for_service_key(service_key: &str) -> String {
    match normalize_service_key(service_key).as_str() {
        "claude" | "claude-code" => "Claude Code".to_string(),
        "coinmarketcap" => "CoinMarketCap".to_string(),
        "nansen" => "Nansen".to_string(),
        "vercel" => "Vercel".to_string(),
        "figma" => "Figma".to_string(),
        "canva" => "Canva".to_string(),
        "moralis" => "Moralis".to_string(),
        "alchemy" => "Alchemy".to_string(),
        "the-graph" | "graph" => "The Graph".to_string(),
        "infura" => "Infura".to_string(),
        "x-api" | "xapi" | "twitter-api" => "X API".to_string(),
        "supabase" => "Supabase".to_string(),
        "render" => "Render".to_string(),
        "neon" => "Neon".to_string(),
        "railway" => "Railway".to_string(),
        "hugging-face" | "huggingface" => "Hugging Face".to_string(),
        "midjourney" => "Midjourney".to_string(),
        "pinata" => "Pinata".to_string(),
        "coingecko" => "CoinGecko".to_string(),
        "thirdweb" => "thirdweb".to_string(),
        "firecrawl" => "Firecrawl".to_string(),
        "browserbase" => "Browserbase".to_string(),
        "neynar" => "Neynar".to_string(),
        "quicknode" => "QuickNode".to_string(),
        "coinbase" => "Coinbase".to_string(),
        "api-key" => "API Key".to_string(),
        other => other
            .split('-')
            .filter(|part| !part.is_empty())
            .map(|part| {
                let mut chars = part.chars();
                match chars.next() {
                    Some(first) => {
                        let mut out = first.to_ascii_uppercase().to_string();
                        out.push_str(chars.as_str());
                        out
                    }
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" "),
    }
}

fn infer_sponsor_archetype(
    sponsor_name: &str,
    required_tasks: &BTreeSet<String>,
    service_keys: &BTreeSet<String>,
) -> String {
    let mut context = sponsor_name.to_lowercase();
    for task in required_tasks {
        context.push(' ');
        context.push_str(&task.to_lowercase());
    }
    for service in service_keys {
        context.push(' ');
        context.push_str(&service.to_lowercase());
    }

    if contains_any(&context, &["hackathon", "conference", "event signup"]) {
        return "hackathon/conference provider".to_string();
    }
    if contains_any(
        &context,
        &[
            "api key",
            "sdk",
            "cli",
            "github",
            "pull request",
            "pr",
            "bug report",
        ],
    ) {
        return "development company".to_string();
    }
    if contains_any(&context, &["hiring", "job", "talent"]) {
        return "hiring platform".to_string();
    }
    if contains_any(&context, &["wallet", "credit card", "card activation"]) {
        return "wallet or credit-card provider".to_string();
    }
    if contains_any(&context, &["exchange", "cex"]) {
        return "centralized exchange".to_string();
    }
    if contains_any(
        &context,
        &[
            "tweet",
            "post",
            "sns",
            "ugc",
            "video share",
            "share content",
        ],
    ) {
        return "creator or marketing sponsor".to_string();
    }
    if contains_any(&context, &["survey", "questionnaire", "research"]) {
        return "research or think-tank sponsor".to_string();
    }
    if contains_any(&context, &["annotation", "labeling", "label data"]) {
        return "ai data provider".to_string();
    }
    if contains_any(&context, &["onsite", "on-site", "local research", "photo"]) {
        return "infrastructure company".to_string();
    }
    if contains_any(
        &context,
        &["store review", "dining", "customer service review"],
    ) {
        return "blockchain/chain ecosystem sponsor".to_string();
    }

    "general sponsor".to_string()
}

fn contains_any(text: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|pattern| text.contains(pattern))
}

pub fn sponsored_api_service_key(api_id: Uuid) -> String {
    format!("{}-{}", SPONSORED_API_SERVICE_PREFIX, api_id)
}

pub fn normalize_upstream_method(method: Option<String>) -> ApiResult<String> {
    let value = method.unwrap_or_else(|| "POST".to_string());
    let normalized = value.trim().to_uppercase();
    match normalized.as_str() {
        "GET" | "POST" => Ok(normalized),
        _ => Err(ApiError::validation("upstream_method must be GET or POST")),
    }
}

pub async fn call_upstream(
    http: &Client,
    api: &SponsoredApi,
    payload: Value,
    timeout_secs: u64,
) -> ApiResult<(u16, String)> {
    let method = match api.upstream_method.as_str() {
        "GET" => Method::GET,
        "POST" => Method::POST,
        other => {
            return Err(ApiError::internal(format!(
                "unsupported upstream method: {other}"
            )));
        }
    };

    let mut request = http
        .request(method.clone(), &api.upstream_url)
        .timeout(Duration::from_secs(timeout_secs));

    for (header, value) in &api.upstream_headers {
        request = request.header(header, value);
    }

    if matches!(method, Method::GET) {
        if let Some(params) = payload.as_object() {
            request = request.query(params);
        }
    } else {
        request = request.json(&payload);
    }

    let response = request
        .send()
        .await
        .map_err(|err| ApiError::upstream(StatusCode::BAD_GATEWAY, err.to_string()))?;

    let status = response.status().as_u16();
    let body = response.text().await.unwrap_or_default();
    Ok((status, body))
}
