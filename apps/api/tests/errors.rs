//! Error-path tests (TEST-018–TEST-022): strict parsing, safe validation
//! errors that never echo submitted values, rule-pack conflicts without
//! clinical content, size and rate controls.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};

use bili_mate_api::Config;
use common::{get, post_evaluation, send, test_app, test_app_with, ACTIVE_PACK_ID, NORMAL_REQUEST};

#[tokio::test]
async fn malformed_json_is_400() {
    let response = post_evaluation(test_app(), "{ not json").await;
    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert_eq!(response.body["code"], "MALFORMED_JSON");
}

#[tokio::test]
async fn duplicate_json_key_is_400() {
    let duplicated = NORMAL_REQUEST.replace(
        "\"gestational_age_completed_weeks\": 38,",
        "\"gestational_age_completed_weeks\": 38, \"gestational_age_completed_weeks\": 39,",
    );
    assert_ne!(duplicated, NORMAL_REQUEST);
    let response = post_evaluation(test_app(), &duplicated).await;
    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert_eq!(response.body["code"], "DUPLICATE_JSON_KEY");
}

#[tokio::test]
async fn wrong_content_type_is_400() {
    let request = Request::builder()
        .method("POST")
        .uri("/v1/evaluations")
        .header("content-type", "text/plain")
        .body(Body::from(NORMAL_REQUEST))
        .unwrap();
    let response = send(test_app(), request).await;
    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert_eq!(response.body["code"], "CONTENT_TYPE_INVALID");
}

#[tokio::test]
async fn content_encoding_is_rejected() {
    let request = Request::builder()
        .method("POST")
        .uri("/v1/evaluations")
        .header("content-type", "application/json")
        .header("content-encoding", "gzip")
        .body(Body::from(NORMAL_REQUEST))
        .unwrap();
    let response = send(test_app(), request).await;
    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert_eq!(response.body["code"], "CONTENT_ENCODING_NOT_SUPPORTED");
}

#[tokio::test]
async fn unknown_property_is_422_with_pointer_and_no_echo() {
    let with_identifier = NORMAL_REQUEST.replace(
        "\"rule_pack_id\"",
        "\"nhs_number\": \"SENTINEL-4529\", \"rule_pack_id\"",
    );
    let response = post_evaluation(test_app(), &with_identifier).await;
    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(response.body["code"], "VALIDATION_FAILED");
    let raw = String::from_utf8(response.raw_body.clone()).unwrap();
    assert!(
        !raw.contains("SENTINEL-4529"),
        "submitted values must never be echoed (API-017)"
    );
}

#[tokio::test]
async fn invalid_enum_value_is_422_with_field_pointer() {
    let invalid = NORMAL_REQUEST.replace("\"serum\"", "\"capillary\"");
    let response = post_evaluation(test_app(), &invalid).await;
    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    let pointer = response.body["errors"][0]["pointer"].as_str().unwrap();
    assert!(
        pointer.starts_with("/measurements/0"),
        "pointer must identify the field, got {pointer}"
    );
    let raw = String::from_utf8(response.raw_body.clone()).unwrap();
    assert!(
        !raw.contains("capillary"),
        "invalid value must not be echoed"
    );
}

#[tokio::test]
async fn out_of_range_value_is_422() {
    let invalid = NORMAL_REQUEST.replace(
        "\"total_bilirubin_umol_l\": 180",
        "\"total_bilirubin_umol_l\": 1001",
    );
    let response = post_evaluation(test_app(), &invalid).await;
    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    let raw = String::from_utf8(response.raw_body.clone()).unwrap();
    assert!(
        !raw.contains("1001"),
        "out-of-range value must not be echoed"
    );
}

#[tokio::test]
async fn invalid_treatment_state_is_422_with_domain_code() {
    let invalid = NORMAL_REQUEST.replace(
        "\"treatment_state\": {\n    \"mode\": \"none\"\n  }",
        "\"treatment_state\": { \"mode\": \"none\", \"started_age_minutes\": 100 }",
    );
    assert_ne!(
        invalid, NORMAL_REQUEST,
        "fixture text must match for replacement"
    );
    let response = post_evaluation(test_app(), &invalid).await;
    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    let errors = response.body["errors"].as_array().unwrap();
    assert!(
        errors.iter().any(|e| {
            e["pointer"] == "/treatment_state/started_age_minutes"
                && e["code"] == "TREATMENT_STATE_FIELD_FORBIDDEN"
        }),
        "expected treatment-state domain error, got {errors:?}"
    );
}

#[tokio::test]
async fn duplicate_measurement_ages_are_422() {
    let duplicated = NORMAL_REQUEST.replace(
        "\"measurements\": [",
        "\"measurements\": [
    { \"id\": \"dup\", \"age_minutes\": 2880, \"total_bilirubin_umol_l\": 170, \"method\": \"serum\" },",
    );
    let response = post_evaluation(test_app(), &duplicated).await;
    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    let errors = response.body["errors"].as_array().unwrap();
    assert!(errors
        .iter()
        .any(|e| e["code"] == "DUPLICATE_MEASUREMENT_AGE"));
}

#[tokio::test]
async fn stale_rule_pack_is_409_with_active_pack_and_no_clinical_result() {
    let stale = NORMAL_REQUEST.replace(ACTIVE_PACK_ID, "nice-cg98-2010-05-19.1");
    let response = post_evaluation(test_app(), &stale).await;
    assert_eq!(response.status, StatusCode::CONFLICT);
    assert_eq!(response.body["code"], "RULE_PACK_NOT_ACTIVE");
    assert_eq!(response.body["active_rule_pack_id"], ACTIVE_PACK_ID);
    // No clinical content in a conflict (API-014, TEST-019).
    assert!(response.body.get("thresholds").is_none());
    assert!(response.body.get("recommendations").is_none());
    assert!(response.body.get("primary_action").is_none());
}

#[tokio::test]
async fn oversized_body_is_413_problem() {
    let padding = "x".repeat(70 * 1024);
    let oversized = NORMAL_REQUEST.replace(
        "\"rule_pack_id\"",
        &format!("\"padding\": \"{padding}\", \"rule_pack_id\""),
    );
    let response = post_evaluation(test_app(), &oversized).await;
    assert_eq!(response.status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(response.body["code"], "PAYLOAD_TOO_LARGE");
}

#[tokio::test]
async fn rate_limit_returns_429_with_retry_after() {
    let config = Config {
        rate_limit_per_minute: 60,
        rate_limit_burst: 2,
        ..Config::for_tests()
    };
    let app = test_app_with(config);
    let first = post_evaluation(app.clone(), NORMAL_REQUEST).await;
    assert_eq!(first.status, StatusCode::OK);
    let second = post_evaluation(app.clone(), NORMAL_REQUEST).await;
    assert_eq!(second.status, StatusCode::OK);
    let third = post_evaluation(app, NORMAL_REQUEST).await;
    assert_eq!(third.status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(third.body["code"], "RATE_LIMITED");
    let retry_after = third
        .headers
        .get("retry-after")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u32>().ok())
        .expect("Retry-After header required (API-006)");
    assert!(retry_after >= 1);
    assert!(
        third.body.get("primary_action").is_none(),
        "no clinical result when limited"
    );
}

#[tokio::test]
async fn unknown_curve_pack_is_404_and_bad_query_is_422() {
    let not_found = get(
        test_app(),
        "/v1/threshold-curves/nice-cg98-2010-05-19.1?gestational_age_completed_weeks=38",
    )
    .await;
    assert_eq!(not_found.status, StatusCode::NOT_FOUND);

    let missing_gestation = get(
        test_app(),
        &format!("/v1/threshold-curves/{ACTIVE_PACK_ID}"),
    )
    .await;
    assert_eq!(missing_gestation.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        missing_gestation.body["errors"][0]["pointer"],
        "/gestational_age_completed_weeks"
    );

    let bad_resolution = get(
        test_app(),
        &format!(
            "/v1/threshold-curves/{ACTIVE_PACK_ID}?gestational_age_completed_weeks=38&resolution_minutes=7"
        ),
    )
    .await;
    assert_eq!(bad_resolution.status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn wrong_method_is_a_problem_response() {
    let request = Request::builder()
        .method("GET")
        .uri("/v1/evaluations")
        .body(Body::empty())
        .unwrap();
    let response = send(test_app(), request).await;
    assert_eq!(response.status, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(response.body["code"], "METHOD_NOT_ALLOWED");
}
