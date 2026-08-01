//! Readiness mode gates (OPS-004, OPS-011, CLIN-003, CLIN-004): the draft
//! embedded pack serves demonstration mode but can never serve clinical
//! mode, with or without a release authorisation.

mod common;

use axum::http::StatusCode;
use clinical_core::types::Mode;

use bili_mate_api::Config;
use common::{get, post_evaluation, test_app_with, NORMAL_REQUEST};

fn clinical_config(release_ref: Option<&str>) -> Config {
    Config {
        mode: Mode::Clinical,
        release_authorisation_ref: release_ref.map(String::from),
        ..Config::for_tests()
    }
}

#[tokio::test]
async fn demonstration_mode_is_ready_with_a_draft_pack() {
    let response = get(test_app_with(Config::for_tests()), "/health/ready").await;
    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body["status"], "ready");
}

#[tokio::test]
async fn demonstration_responses_are_labelled_not_for_patient_care() {
    let response = post_evaluation(test_app_with(Config::for_tests()), NORMAL_REQUEST).await;
    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body["mode"], "demonstration");
    let warnings = response.body["warnings"].as_array().unwrap();
    assert!(
        warnings.iter().any(|w| w["code"] == "DEMONSTRATION_ONLY"),
        "CLIN-004: every demonstration response is labelled"
    );
}

#[tokio::test]
async fn clinical_mode_with_draft_pack_is_never_ready() {
    // Even with a release-authorisation reference, a draft pack cannot serve
    // clinical mode (CLIN-003).
    let response = get(
        test_app_with(clinical_config(Some("REL-2026-001"))),
        "/health/ready",
    )
    .await;
    assert_eq!(response.status, StatusCode::SERVICE_UNAVAILABLE);
    // Readiness discloses no dependency or internal details.
    let raw = String::from_utf8(response.raw_body.clone()).unwrap();
    for leak in ["spec/", "yaml", "path", "self_test"] {
        assert!(
            !raw.contains(leak),
            "readiness must not disclose internals: {leak}"
        );
    }
}

#[tokio::test]
async fn clinical_mode_without_release_authorisation_is_not_ready() {
    let response = get(test_app_with(clinical_config(None)), "/health/ready").await;
    assert_eq!(response.status, StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn liveness_does_not_depend_on_clinical_readiness() {
    let response = get(test_app_with(clinical_config(None)), "/health/live").await;
    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body["status"], "live");
}
