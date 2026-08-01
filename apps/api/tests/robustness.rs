//! JSON-boundary robustness (spec 09 security testing: property/fuzz
//! testing of JSON and domain boundaries). Whatever bytes arrive, the
//! parser never panics and the evaluation route always answers with a
//! documented status — an error is always problem+json, never a partial
//! clinical result.

mod common;

use axum::body::Body;
use axum::http::Request;
use proptest::prelude::*;

use bili_mate_api::strict_json::parse_strict;
use common::{send, test_app, NORMAL_REQUEST};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    /// Arbitrary bytes never panic the strict parser.
    #[test]
    fn strict_parser_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..2048)) {
        let _ = parse_strict::<serde_json::Value>(&bytes);
    }

    /// Arbitrary JSON-looking text never panics the strict parser.
    #[test]
    fn strict_parser_never_panics_on_jsonish_text(text in "[\\{\\}\\[\\]\",:0-9a-z \\.\\-]{0,512}") {
        let _ = parse_strict::<serde_json::Value>(text.as_bytes());
    }
}

proptest! {
    // Router round-trips are slower; fewer cases, still meaningful.
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// Byte-level mutations of a valid request always produce a documented
    /// response: 200 for still-valid bodies, otherwise a problem+json error
    /// with no clinical content.
    #[test]
    fn mutated_requests_always_get_a_documented_answer(
        mutations in proptest::collection::vec(
            (0usize..NORMAL_REQUEST.len(), any::<u8>()),
            1..6,
        ),
        truncate_at in proptest::option::of(0usize..NORMAL_REQUEST.len()),
    ) {
        let mut body = NORMAL_REQUEST.as_bytes().to_vec();
        for (index, byte) in mutations {
            body[index] = byte;
        }
        if let Some(cut) = truncate_at {
            body.truncate(cut);
        }

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        runtime.block_on(async {
            let request = Request::builder()
                .method("POST")
                .uri("/v1/evaluations")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap();
            let response = send(test_app(), request).await;
            let status = response.status.as_u16();
            prop_assert!(
                [200, 400, 409, 413, 422].contains(&status),
                "undocumented status {status}"
            );
            if status != 200 {
                let content_type = response
                    .headers
                    .get("content-type")
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or("");
                prop_assert!(
                    content_type.starts_with("application/problem+json"),
                    "error responses are problem+json, got {content_type}"
                );
                prop_assert!(
                    response.body.get("primary_action").is_none(),
                    "no clinical content in an error"
                );
            }
            Ok(())
        })?;
    }
}
