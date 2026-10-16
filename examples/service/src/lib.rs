pub mod config;
pub mod handlers;
pub mod models;
pub mod service;
pub mod utils;

use std::sync::Arc;
use std::time::Duration;
use axum::{
    extract::DefaultBodyLimit,
    http::header::{HeaderName, CONTENT_TYPE},
    http::{HeaderValue, Method},
    routing::{get, post},
    Router,
};
use tower::limit::ConcurrencyLimitLayer;
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    set_header::SetResponseHeaderLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

use crate::config::Config;
use crate::handlers::{
    get_info, handle_bahttext_get, handle_bahttext_post, handle_break, handle_compare,
    handle_lines, handle_normalize, handle_validate_id_get, handle_validate_id_post,
    handle_words, handle_wrap, health_check,
};
use crate::service::ThaiBreakEngine;

/// CORS policy: no origin unless configured, `*` for any origin.
fn cors_layer(origins: &[String]) -> CorsLayer {
    if origins.iter().any(|o| o.trim() == "*") {
        return CorsLayer::permissive();
    }
    let allowed: Vec<HeaderValue> = origins
        .iter()
        .filter_map(|o| match HeaderValue::from_str(o.trim()) {
            Ok(v) => Some(v),
            Err(_) => {
                tracing::warn!(origin = %o, "Ignoring invalid CORS origin");
                None
            }
        })
        .collect();
    CorsLayer::new()
        .allow_origin(allowed)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([CONTENT_TYPE])
}

/// Builds the Axum router for thai-break-service
pub fn create_router(engine: Arc<ThaiBreakEngine>, config: &Config) -> Router {
    Router::new()
        // Health & Readiness probes
        .route("/health", get(health_check))
        .route("/livez", get(health_check))
        .route("/readyz", get(health_check))
        // Info endpoints
        .route("/", get(get_info))
        .route("/api/v1/info", get(get_info))
        // ThaiBreak API endpoints
        .route("/api/v1/words", post(handle_words))
        .route("/api/v1/tokenize", post(handle_words))
        .route("/api/v1/lines", post(handle_lines))
        .route("/api/v1/wrap", post(handle_wrap))
        .route("/api/v1/break", post(handle_break))
        .route("/api/v1/normalize", post(handle_normalize))
        .route("/api/v1/compare", post(handle_compare))
        // Enterprise Thai utilities
        .route("/api/v1/bahttext", post(handle_bahttext_post).get(handle_bahttext_get))
        .route("/api/v1/validate-id", post(handle_validate_id_post).get(handle_validate_id_get))
        // Global middlewares
        .layer(DefaultBodyLimit::max(config.max_body_mb * 1024 * 1024))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        ))
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(config.request_timeout_secs),
        ))
        .layer(ConcurrencyLimitLayer::new(config.concurrency_limit))
        .layer(cors_layer(&config.cors_allowed_origins))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(engine)
}
