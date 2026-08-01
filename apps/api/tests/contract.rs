//! Contract tests (TEST-015–TEST-017 runtime side, TEST-021): every live
//! response is validated against the committed `spec/openapi.yaml`, so the
//! implementation cannot drift from the published contract.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};

use common::{
    all_example_requests, get, post_evaluation, send, test_app, OpenApiDoc, ACTIVE_PACK_ID,
    NORMAL_REQUEST,
};

#[tokio::test]
async fn every_example_request_evaluates_and_matches_the_response_schema() {
    let openapi = OpenApiDoc::load();
    for (name, fixture) in all_example_requests() {
        let response = post_evaluation(test_app(), fixture).await;
        assert_eq!(response.status, StatusCode::OK, "{name} must evaluate");
        openapi.assert_valid("EvaluationResponse", &response.body);
        assert_eq!(
            response
                .headers
                .get("cache-control")
                .and_then(|v| v.to_str().ok()),
            Some("no-store"),
            "{name}: evaluations are never cacheable (API-007)"
        );
        assert!(
            response.headers.contains_key("x-request-id"),
            "{name}: every response returns X-Request-ID (API-003)"
        );
        assert_eq!(response.body["rule_pack"]["id"], ACTIVE_PACK_ID);
        assert_eq!(response.body["mode"], "demonstration");
        // DATA-020: primary action appears first in recommendations.
        assert_eq!(
            response.body["primary_action"]["code"], response.body["recommendations"][0]["code"],
            "{name}"
        );
    }
}

#[tokio::test]
async fn evaluation_echoes_a_supplied_request_id() {
    let request = Request::builder()
        .method("POST")
        .uri("/v1/evaluations")
        .header("content-type", "application/json")
        .header("x-request-id", "contract-test-042")
        .body(Body::from(NORMAL_REQUEST))
        .unwrap();
    let response = send(test_app(), request).await;
    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(
        response
            .headers
            .get("x-request-id")
            .and_then(|v| v.to_str().ok()),
        Some("contract-test-042")
    );
    assert_eq!(response.body["request_id"], "contract-test-042");
}

#[tokio::test]
async fn decision_receipt_is_deterministic_for_equivalent_requests() {
    let first = post_evaluation(test_app(), NORMAL_REQUEST).await;
    let second = post_evaluation(test_app(), NORMAL_REQUEST).await;
    assert_eq!(
        first.body["decision_receipt"]["canonicalisation"],
        "JCS-RFC8785"
    );
    assert_eq!(first.body["decision_receipt"]["retained_by_server"], false);
    // Same input, same rule pack: identical digest even though the
    // evaluation IDs differ (PRD-030 at the receipt level).
    assert_eq!(
        first.body["decision_receipt"]["digest_sha256"],
        second.body["decision_receipt"]["digest_sha256"]
    );
    assert_ne!(first.body["evaluation_id"], second.body["evaluation_id"]);
}

#[tokio::test]
async fn active_guideline_metadata_matches_schema() {
    let openapi = OpenApiDoc::load();
    let response = get(test_app(), "/v1/guidelines/active").await;
    assert_eq!(response.status, StatusCode::OK);
    openapi.assert_valid("RulePackMetadata", &response.body);
    assert_eq!(response.body["id"], ACTIVE_PACK_ID);
    assert_eq!(
        response
            .headers
            .get("cache-control")
            .and_then(|v| v.to_str().ok()),
        Some("public, max-age=3600")
    );
}

#[tokio::test]
async fn threshold_curve_matches_schema_and_control_points() {
    let openapi = OpenApiDoc::load();
    let response = get(
        test_app(),
        &format!("/v1/threshold-curves/{ACTIVE_PACK_ID}?gestational_age_completed_weeks=38"),
    )
    .await;
    assert_eq!(response.status, StatusCode::OK);
    openapi.assert_valid("ThresholdCurve", &response.body);
    assert_eq!(response.body["display_only"], true);
    assert_eq!(response.body["resolution_minutes"], 60);
    let points = response.body["points"].as_array().unwrap();
    assert_eq!(
        points.len(),
        337,
        "hourly points from 0 through 20160 minutes"
    );
    assert_eq!(points[0]["age_minutes"], 0);
    assert_eq!(points[0]["phototherapy_threshold_umol_l"], 100.0);
    assert_eq!(points[0]["exchange_threshold_umol_l"], 100.0);
    let last = points.last().unwrap();
    assert_eq!(last["age_minutes"], 20160);
    assert_eq!(last["phototherapy_threshold_umol_l"], 350.0);
    assert_eq!(last["exchange_threshold_umol_l"], 450.0);
}

#[tokio::test]
async fn preterm_curve_starts_at_40_and_80() {
    let response = get(
        test_app(),
        &format!(
            "/v1/threshold-curves/{ACTIVE_PACK_ID}?gestational_age_completed_weeks=30&resolution_minutes=15"
        ),
    )
    .await;
    assert_eq!(response.status, StatusCode::OK);
    let points = response.body["points"].as_array().unwrap();
    assert_eq!(points[0]["phototherapy_threshold_umol_l"], 40.0);
    assert_eq!(points[0]["exchange_threshold_umol_l"], 80.0);
    let last = points.last().unwrap();
    assert_eq!(last["age_minutes"], 20160);
    assert_eq!(last["phototherapy_threshold_umol_l"], 200.0);
    assert_eq!(last["exchange_threshold_umol_l"], 300.0);
}

#[tokio::test]
async fn legal_notices_match_schema() {
    let openapi = OpenApiDoc::load();
    let response = get(test_app(), "/v1/legal").await;
    assert_eq!(response.status, StatusCode::OK);
    openapi.assert_valid("LegalNotices", &response.body);
    assert_eq!(response.body["uk_only"], true);
}

#[tokio::test]
async fn health_endpoints_match_schema() {
    let openapi = OpenApiDoc::load();
    let live = get(test_app(), "/health/live").await;
    assert_eq!(live.status, StatusCode::OK);
    openapi.assert_valid("HealthStatus", &live.body);
    assert_eq!(live.body["status"], "live");

    let ready = get(test_app(), "/health/ready").await;
    assert_eq!(ready.status, StatusCode::OK);
    openapi.assert_valid("HealthStatus", &ready.body);
    assert_eq!(ready.body["status"], "ready");
}

#[tokio::test]
async fn problem_responses_match_the_problem_schema() {
    let openapi = OpenApiDoc::load();
    // Stale rule pack.
    let stale = NORMAL_REQUEST.replace(ACTIVE_PACK_ID, "nice-cg98-2010-05-19.1");
    let response = post_evaluation(test_app(), &stale).await;
    assert_eq!(response.status, StatusCode::CONFLICT);
    openapi.assert_valid("Problem", &response.body);
    // Validation failure.
    let invalid = NORMAL_REQUEST.replace(
        "\"gestational_age_completed_weeks\": 38",
        "\"gestational_age_completed_weeks\": 22",
    );
    let response = post_evaluation(test_app(), &invalid).await;
    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    openapi.assert_valid("Problem", &response.body);
    // Unmatched route.
    let response = get(test_app(), "/v1/unknown").await;
    assert_eq!(response.status, StatusCode::NOT_FOUND);
    openapi.assert_valid("Problem", &response.body);
}

#[tokio::test]
async fn security_headers_are_present_on_api_responses() {
    let response = get(test_app(), "/v1/legal").await;
    for (name, expected) in [
        (
            "strict-transport-security",
            "max-age=31536000; includeSubDomains",
        ),
        ("x-content-type-options", "nosniff"),
        ("referrer-policy", "no-referrer"),
        (
            "content-security-policy",
            "default-src 'none'; frame-ancestors 'none'",
        ),
    ] {
        assert_eq!(
            response.headers.get(name).and_then(|v| v.to_str().ok()),
            Some(expected),
            "header {name}"
        );
    }
}
