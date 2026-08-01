//! Shared helpers for API tests: an in-memory app instance driven through
//! `tower::ServiceExt::oneshot`, and response validation against the
//! committed OpenAPI document so the runtime contract cannot drift from
//! `spec/openapi.yaml` (spec 04 contract testing).
//!
//! Each test binary compiles this module separately and uses a different
//! subset of the helpers.
#![allow(dead_code)]

use axum::body::Body;
use axum::http::{HeaderMap, Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use tower::ServiceExt;

use bili_mate_api::{app, AppState, Config};

pub const ACTIVE_PACK_ID: &str = "nice-cg98-2023-10-31.1";

pub const NORMAL_REQUEST: &str =
    include_str!("../../../../spec/examples/normal-below-threshold-request.json");
pub const EARLY_REQUEST: &str =
    include_str!("../../../../spec/examples/early-jaundice-request.json");
pub const PHOTOTHERAPY_REQUEST: &str =
    include_str!("../../../../spec/examples/phototherapy-request.json");
pub const INTENSIFIED_REQUEST: &str =
    include_str!("../../../../spec/examples/intensified-phototherapy-request.json");
pub const EXCHANGE_REQUEST: &str =
    include_str!("../../../../spec/examples/exchange-escalation-request.json");
pub const PROLONGED_REQUEST: &str =
    include_str!("../../../../spec/examples/prolonged-jaundice-request.json");

pub fn all_example_requests() -> Vec<(&'static str, &'static str)> {
    vec![
        ("normal-below-threshold", NORMAL_REQUEST),
        ("early-jaundice", EARLY_REQUEST),
        ("phototherapy", PHOTOTHERAPY_REQUEST),
        ("intensified-phototherapy", INTENSIFIED_REQUEST),
        ("exchange-escalation", EXCHANGE_REQUEST),
        ("prolonged-jaundice", PROLONGED_REQUEST),
    ]
}

pub fn test_app() -> Router {
    app(AppState::new(Config::for_tests()))
}

pub fn test_app_with(config: Config) -> Router {
    app(AppState::new(config))
}

pub struct TestResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: serde_json::Value,
    pub raw_body: Vec<u8>,
}

pub async fn send(app: Router, request: Request<Body>) -> TestResponse {
    let response = app.oneshot(request).await.expect("infallible service");
    let status = response.status();
    let headers = response.headers().clone();
    let raw_body = response
        .into_body()
        .collect()
        .await
        .expect("collect body")
        .to_bytes()
        .to_vec();
    let body = serde_json::from_slice(&raw_body).unwrap_or(serde_json::Value::Null);
    TestResponse {
        status,
        headers,
        body,
        raw_body,
    }
}

pub async fn post_evaluation(app: Router, body: &str) -> TestResponse {
    let request = Request::builder()
        .method("POST")
        .uri("/v1/evaluations")
        .header("content-type", "application/json")
        .header("accept", "application/json")
        .body(Body::from(body.to_owned()))
        .unwrap();
    send(app, request).await
}

pub async fn get(app: Router, path: &str) -> TestResponse {
    let request = Request::builder()
        .method("GET")
        .uri(path)
        .header("accept", "application/json")
        .body(Body::empty())
        .unwrap();
    send(app, request).await
}

/// Validator backed by the committed OpenAPI document. A response body is
/// checked against a named component schema with `$ref` resolution across
/// the whole document.
pub struct OpenApiDoc {
    doc: serde_json::Value,
}

impl OpenApiDoc {
    pub fn load() -> Self {
        let yaml = include_str!("../../../../spec/openapi.yaml");
        let doc: serde_json::Value =
            serde_yaml::from_str(yaml).expect("committed OpenAPI document must parse");
        Self { doc }
    }

    pub fn assert_valid(&self, schema_name: &str, instance: &serde_json::Value) {
        let mut root = self.doc.clone();
        root["$ref"] = serde_json::json!(format!("#/components/schemas/{schema_name}"));
        let validator = jsonschema::validator_for(&root)
            .unwrap_or_else(|e| panic!("schema {schema_name} must compile: {e}"));
        let errors: Vec<String> = validator
            .iter_errors(instance)
            .map(|e| format!("{} at {}", e, e.instance_path()))
            .collect();
        assert!(
            errors.is_empty(),
            "response does not match {schema_name}:\n{}",
            errors.join("\n")
        );
    }
}
