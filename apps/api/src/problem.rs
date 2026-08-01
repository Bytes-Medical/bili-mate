//! `application/problem+json` responses (spec 04). Stable machine codes,
//! request-URN instances, optional RFC 6901 field errors. Problem details
//! never include stack traces, source paths, internal hostnames or echoed
//! clinical values (API-016, API-017).

use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Serialize;

pub const PROBLEM_TYPE_BASE: &str = "https://bili-mate.uk/problems/";

#[derive(Debug, Clone, Serialize)]
pub struct FieldError {
    pub pointer: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProblemBody {
    #[serde(rename = "type")]
    pub problem_type: String,
    pub title: String,
    pub status: u16,
    pub detail: String,
    pub instance: String,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<FieldError>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_rule_pack_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Problem {
    pub status: StatusCode,
    pub code: &'static str,
    pub title: &'static str,
    pub detail: String,
    pub errors: Option<Vec<FieldError>>,
    pub active_rule_pack_id: Option<String>,
    pub retry_after_seconds: Option<u32>,
}

impl Problem {
    pub fn new(
        status: StatusCode,
        code: &'static str,
        title: &'static str,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            status,
            code,
            title,
            detail: detail.into(),
            errors: None,
            active_rule_pack_id: None,
            retry_after_seconds: None,
        }
    }

    pub fn with_errors(mut self, errors: Vec<FieldError>) -> Self {
        self.errors = Some(errors);
        self
    }

    pub fn with_active_rule_pack(mut self, id: impl Into<String>) -> Self {
        self.active_rule_pack_id = Some(id.into());
        self
    }

    pub fn with_retry_after(mut self, seconds: u32) -> Self {
        self.retry_after_seconds = Some(seconds);
        self
    }

    pub fn malformed_json() -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "MALFORMED_JSON",
            "Malformed request body",
            "The request body is not well-formed JSON.",
        )
    }

    pub fn duplicate_json_key() -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "DUPLICATE_JSON_KEY",
            "Malformed request body",
            "The request body contains a duplicate JSON object key.",
        )
    }

    pub fn content_type_invalid() -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "CONTENT_TYPE_INVALID",
            "Invalid content type",
            "Evaluation requests must use Content-Type: application/json.",
        )
    }

    pub fn content_encoding_rejected() -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "CONTENT_ENCODING_NOT_SUPPORTED",
            "Content encoding not supported",
            "Request content encoding is not accepted; send an identity JSON body.",
        )
    }

    pub fn payload_too_large() -> Self {
        Self::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            "PAYLOAD_TOO_LARGE",
            "Request body too large",
            "The request body exceeds the 64 KiB limit.",
        )
    }

    pub fn validation_failed(errors: Vec<FieldError>) -> Self {
        Self::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "VALIDATION_FAILED",
            "Validation failed",
            "The request did not satisfy the evaluation schema or domain rules.",
        )
        .with_errors(errors)
    }

    pub fn stale_rule_pack(active: impl Into<String>) -> Self {
        Self::new(
            StatusCode::CONFLICT,
            "RULE_PACK_NOT_ACTIVE",
            "Requested rule pack cannot be evaluated",
            "The requested rule pack is not the active rule pack. Refresh guideline metadata and review the assessment before resubmitting.",
        )
        .with_active_rule_pack(active)
    }

    pub fn rate_limited(retry_after: u32) -> Self {
        Self::new(
            StatusCode::TOO_MANY_REQUESTS,
            "RATE_LIMITED",
            "Rate limit exceeded",
            "The request rate limit for this source was exceeded. No clinical result was produced.",
        )
        .with_retry_after(retry_after)
    }

    pub fn engine_unavailable() -> Self {
        Self::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "ENGINE_UNAVAILABLE",
            "Clinical engine unavailable",
            "The clinical engine is unavailable. No clinical result was produced; follow the locally approved procedure.",
        )
        .with_retry_after(30)
    }

    pub fn engine_safety_check_failed() -> Self {
        Self::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "ENGINE_SAFETY_CHECK_FAILED",
            "Engine safety check failed",
            "An internal safety check failed. No clinical result was produced; follow the locally approved procedure.",
        )
        .with_retry_after(30)
    }

    pub fn internal() -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_FAILURE",
            "Internal failure",
            "An unexpected internal error occurred. No clinical result was produced.",
        )
    }

    pub fn not_found() -> Self {
        Self::new(
            StatusCode::NOT_FOUND,
            "NOT_FOUND",
            "Resource not found",
            "The requested resource does not exist.",
        )
    }

    /// Render with the request ID from middleware; the ID forms the
    /// problem `instance` URN.
    pub fn into_response_with_request_id(self, request_id: &str) -> Response {
        let body = ProblemBody {
            problem_type: format!(
                "{PROBLEM_TYPE_BASE}{}",
                self.code.to_lowercase().replace('_', "-")
            ),
            title: self.title.to_string(),
            status: self.status.as_u16(),
            detail: self.detail,
            instance: format!("urn:bili-mate:request:{request_id}"),
            code: self.code.to_string(),
            errors: self.errors,
            active_rule_pack_id: self.active_rule_pack_id,
        };
        let mut response = (
            self.status,
            [
                (
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("application/problem+json"),
                ),
                (header::CACHE_CONTROL, HeaderValue::from_static("no-store")),
            ],
            axum::Json(body),
        )
            .into_response();
        if let Some(seconds) = self.retry_after_seconds {
            if let Ok(value) = HeaderValue::from_str(&seconds.to_string()) {
                response.headers_mut().insert(header::RETRY_AFTER, value);
            }
        }
        response
    }
}

/// Handlers return `Problem` directly; the request-ID middleware has already
/// stored the ID in request extensions, and `into_response` here uses a
/// placeholder that the middleware replaces when building the final
/// `instance`. To keep a single construction path, handlers instead use the
/// `RequestContext` extension and call `into_response_with_request_id`.
impl IntoResponse for Problem {
    fn into_response(self) -> Response {
        // Fallback path only; the request-ID middleware rewrites `instance`
        // via the stored extension when available.
        self.into_response_with_request_id("unknown")
    }
}
