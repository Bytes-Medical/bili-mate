//! Request-scoped middleware: request-ID handling (API-002/003), security
//! headers (SEC-015), problem+json fallback for framework-generated errors,
//! and privacy-safe request logging (SEC-007, API-015).

use axum::body::Body;
use axum::extract::{FromRequestParts, MatchedPath, Request};
use axum::http::request::Parts;
use axum::http::{header, HeaderName, HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::problem::Problem;

/// The validated or generated request ID, stored in request extensions.
#[derive(Debug, Clone)]
pub struct RequestId(pub String);

/// Extractor for handlers: the request ID plus the connection source IP
/// when the server was started with connect-info (absent under test
/// `oneshot` calls).
pub struct Ctx {
    pub request_id: String,
    pub source_ip: Option<std::net::IpAddr>,
}

impl<S: Send + Sync> FromRequestParts<S> for Ctx {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let request_id = parts
            .extensions
            .get::<RequestId>()
            .map(|id| id.0.clone())
            .unwrap_or_else(|| "unknown".to_string());
        let source_ip = parts
            .extensions
            .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
            .map(|info| info.0.ip());
        Ok(Ctx {
            request_id,
            source_ip,
        })
    }
}

fn valid_request_id(value: &str) -> bool {
    (1..=64).contains(&value.len()) && value.bytes().all(|b| (0x20..=0x7e).contains(&b))
}

/// Validate or generate `X-Request-ID`, store it for handlers and echo it on
/// every response (API-002, API-003).
pub async fn request_id(mut request: Request, next: Next) -> Response {
    let id = request
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .filter(|v| valid_request_id(v))
        .map(str::to_owned)
        .unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
    request.extensions_mut().insert(RequestId(id.clone()));
    let mut response = next.run(request).await;
    if let Ok(value) = HeaderValue::from_str(&id) {
        response.headers_mut().insert("x-request-id", value);
    }
    response
}

/// Minimum API response header policy (SEC-015 and the API JSON CSP from
/// spec 07).
pub async fn security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("camera=(), microphone=(), geolocation=(), payment=(), usb=()"),
    );
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'none'; frame-ancestors 'none'"),
    );
    response
}

/// Convert framework-generated plain error responses (body-limit 413,
/// method-not-allowed 405, panic 500, unmatched 404, …) into the documented
/// problem+json shape so no route can leak a non-contract error body.
pub async fn problem_fallback(request: Request, next: Next) -> Response {
    let request_id = request
        .extensions()
        .get::<RequestId>()
        .map(|id| id.0.clone())
        .unwrap_or_else(|| "unknown".to_string());
    let response = next.run(request).await;
    let status = response.status();
    let is_problem = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("application/problem+json"));
    if !status.is_client_error() && !status.is_server_error() || is_problem {
        return response;
    }
    let problem = match status {
        StatusCode::PAYLOAD_TOO_LARGE => Problem::payload_too_large(),
        StatusCode::NOT_FOUND => Problem::not_found(),
        StatusCode::METHOD_NOT_ALLOWED => Problem::new(
            StatusCode::METHOD_NOT_ALLOWED,
            "METHOD_NOT_ALLOWED",
            "Method not allowed",
            "The HTTP method is not supported for this resource.",
        ),
        StatusCode::UNSUPPORTED_MEDIA_TYPE => Problem::content_type_invalid(),
        s if s.is_server_error() => Problem::internal(),
        s => Problem::new(
            s,
            "REQUEST_REJECTED",
            "Request rejected",
            "The request could not be processed.",
        ),
    };
    problem.into_response_with_request_id(&request_id)
}

/// One allowlisted log event per request: route template, method, status,
/// request ID and duration only. Bodies, clinical values, query strings and
/// user agents are never logged (SEC-007, API-015, OPS log policy).
pub async fn log_request(request: Request<Body>, next: Next) -> Response {
    let method = request.method().as_str().to_owned();
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map(|p| p.as_str().to_owned())
        .unwrap_or_else(|| "unmatched".to_string());
    let request_id = request
        .extensions()
        .get::<RequestId>()
        .map(|id| id.0.clone())
        .unwrap_or_else(|| "unknown".to_string());
    let started = std::time::Instant::now();
    let response = next.run(request).await;
    tracing::info!(
        target: "bili_mate_api::request",
        method = %method,
        route = %route,
        status = response.status().as_u16(),
        request_id = %request_id,
        duration_ms = started.elapsed().as_millis() as u64,
        "request completed"
    );
    response
}

/// Shared 404 for unmatched routes, converted by `problem_fallback`.
pub async fn not_found() -> Response {
    StatusCode::NOT_FOUND.into_response()
}
