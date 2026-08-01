//! Bili Mate HTTP service (milestone M3): Axum implementation of the
//! committed OpenAPI 3.1 contract in `spec/openapi.yaml`.

pub mod config;
pub mod legal;
pub mod middleware;
pub mod problem;
pub mod rate_limit;
pub mod receipt;
pub mod state;
pub mod strict_json;

mod routes {
    pub mod evaluations;
    pub mod guidelines;
    pub mod health;
    pub mod legal_route;
}

use axum::extract::DefaultBodyLimit;
use axum::http::{header, HeaderValue, Method};
use axum::routing::{get, post};
use axum::Router;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::cors::CorsLayer;

pub use config::Config;
pub use state::AppState;

/// The API version served under `/v1` (spec 04 versioning; matches the
/// committed OpenAPI `info.version`).
pub const API_VERSION: &str = "1.0.0-draft";

/// Maximum request body size (API-004, spec 07 resource controls).
pub const MAX_BODY_BYTES: usize = 64 * 1024;

pub fn app(state: AppState) -> Router {
    let cors = build_cors(&state.config.allowed_origins);
    Router::new()
        .route("/v1/guidelines/active", get(routes::guidelines::active))
        .route(
            "/v1/evaluations",
            post(routes::evaluations::evaluate_assessment),
        )
        .route(
            "/v1/threshold-curves/{rule_pack_id}",
            get(routes::guidelines::threshold_curve),
        )
        .route("/v1/legal", get(routes::legal_route::legal))
        .route("/health/live", get(routes::health::live))
        .route("/health/ready", get(routes::health::ready))
        .fallback(middleware::not_found)
        .layer(
            tower::ServiceBuilder::new()
                .layer(axum::middleware::from_fn(middleware::request_id))
                .layer(axum::middleware::from_fn(middleware::log_request))
                .layer(axum::middleware::from_fn(middleware::security_headers))
                .layer(axum::middleware::from_fn(middleware::problem_fallback))
                .layer(CatchPanicLayer::new())
                .layer(cors)
                .layer(DefaultBodyLimit::max(MAX_BODY_BYTES)),
        )
        .with_state(state)
}

/// Browser CORS policy (spec 04): explicit origin allowlist, no wildcard,
/// no credentials, GET/POST/OPTIONS only, one-hour preflight cache.
fn build_cors(allowed_origins: &[String]) -> CorsLayer {
    let origins: Vec<HeaderValue> = allowed_origins
        .iter()
        .filter_map(|origin| HeaderValue::from_str(origin).ok())
        .collect();
    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([
            header::ACCEPT,
            header::CONTENT_TYPE,
            header::HeaderName::from_static("x-request-id"),
        ])
        // Browser clients must be able to read the request ID on every
        // response (API-003) and Retry-After on 429/503 (API-006).
        .expose_headers([
            header::RETRY_AFTER,
            header::HeaderName::from_static("x-request-id"),
        ])
        .max_age(std::time::Duration::from_secs(3600))
}
