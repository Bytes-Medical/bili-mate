//! Privacy log test (API-015, SEC-007; an early, in-process version of the
//! sentinel discipline in TEST-028): capture everything the service logs
//! while evaluating a distinctive assessment and prove no clinical value,
//! field name or body fragment appears.

mod common;

use std::io::Write;
use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::http::{Request, StatusCode};

use common::{send, test_app, NORMAL_REQUEST};

#[derive(Clone)]
struct CaptureWriter(Arc<Mutex<Vec<u8>>>);

impl Write for CaptureWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn logs_never_contain_clinical_content() {
    let captured = Arc::new(Mutex::new(Vec::new()));
    let writer = CaptureWriter(captured.clone());
    let subscriber = tracing_subscriber::fmt()
        .with_writer(move || writer.clone())
        .with_max_level(tracing::Level::TRACE)
        .finish();
    let _guard = tracing::subscriber::set_default(subscriber);

    // Sentinel values chosen to be unmistakable in any log line.
    let sentinel_body = NORMAL_REQUEST
        .replace(
            "\"total_bilirubin_umol_l\": 180",
            "\"total_bilirubin_umol_l\": 777",
        )
        .replace(
            "\"assessment_age_minutes\": 2880",
            "\"assessment_age_minutes\": 33333",
        );
    // 33333 exceeds nothing? (max 40319) — still a valid, distinctive age.
    let request = Request::builder()
        .method("POST")
        .uri("/v1/evaluations")
        .header("content-type", "application/json")
        .header("x-request-id", "privacy-probe-1")
        .body(Body::from(
            sentinel_body.replace("\"age_minutes\": 2880", "\"age_minutes\": 33333"),
        ))
        .unwrap();
    let response = send(test_app(), request).await;
    assert_eq!(response.status, StatusCode::OK);

    let logs = String::from_utf8(captured.lock().unwrap().clone()).expect("utf8 logs");
    assert!(
        logs.contains("request completed") && logs.contains("/v1/evaluations"),
        "the allowlisted request event must be logged; got: {logs}"
    );
    assert!(
        logs.contains("privacy-probe-1"),
        "request id is an allowed field"
    );

    // No clinical values, field names or recommendation codes (API-015).
    for forbidden in [
        "777",
        "33333",
        "total_bilirubin",
        "suspected_or_obvious_jaundice",
        "gestational_age",
        "NO_ROUTINE_REPEAT",
        "recommendation",
        "digest",
    ] {
        assert!(
            !logs.contains(forbidden),
            "log output must not contain {forbidden:?}; got: {logs}"
        );
    }
}
