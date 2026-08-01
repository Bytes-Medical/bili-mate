//! `POST /v1/evaluations` (spec 04): the ten-step processing order from
//! transport controls through evaluation to a complete response, with no
//! partial clinical result on any failure (API-010, API-011).

use std::net::IpAddr;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{header, HeaderMap, HeaderValue};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

use clinical_core::evaluate::{evaluate, EvaluationContext};
use clinical_core::input::{
    Assessment, ClinicalFeatures, Measurement, RiskFactors, TreatmentState,
};
use clinical_core::output::{
    MissingInformation, NormalisedInput, Recommendation, ThresholdAssessment, TrendAssessment,
    Warning,
};
use clinical_core::types::{AgeMinutes, BilirubinUmolL, GestationalWeeks, Mode};
use guideline_data::RulePackSummary;

use crate::legal::{legal_notices, LegalNotices};
use crate::middleware::Ctx;
use crate::problem::{FieldError, Problem};
use crate::receipt::{decision_receipt, DecisionReceipt};
use crate::state::AppState;
use crate::strict_json::{parse_strict, ParseFailure};
use crate::API_VERSION;

/// Wire request per `spec/openapi.yaml` `EvaluationRequest`. Unknown
/// properties are rejected (DATA-001); range validation happens inside the
/// core domain types.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireRequest {
    rule_pack_id: String,
    gestational_age_completed_weeks: GestationalWeeks,
    assessment_age_minutes: AgeMinutes,
    clinical_features: ClinicalFeatures,
    risk_factors: RiskFactors,
    measurements: Vec<Measurement>,
    #[serde(default)]
    conjugated_bilirubin_umol_l: Option<BilirubinUmolL>,
    treatment_state: TreatmentState,
}

/// Wire response per `spec/openapi.yaml` `EvaluationResponse`. Fields are
/// listed explicitly (rather than flattening the core outcome) so the wire
/// shape cannot silently gain internal fields such as the decision trace.
#[derive(Serialize)]
struct WireResponse {
    evaluation_id: String,
    request_id: String,
    evaluated_at: String,
    mode: Mode,
    api_version: &'static str,
    engine_version: &'static str,
    rule_pack: RulePackSummary,
    normalised_input: NormalisedInput,
    thresholds: Vec<ThresholdAssessment>,
    trend: Option<TrendAssessment>,
    primary_action: Recommendation,
    recommendations: Vec<Recommendation>,
    warnings: Vec<Warning>,
    missing_information: Vec<MissingInformation>,
    suppressed_rules: Vec<String>,
    decision_receipt: DecisionReceipt,
    legal: LegalNotices,
}

fn rfc3339_utc_now() -> String {
    let now = time::OffsetDateTime::now_utc();
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        now.year(),
        u8::from(now.month()),
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    )
}

const MEASUREMENT_ID_PATTERN_MESSAGE: &str =
    "Measurement ids must use only letters, digits, hyphen and underscore.";

pub async fn evaluate_assessment(
    State(state): State<AppState>,
    ctx: Ctx,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let rid = ctx.request_id;

    // 1. Transport and payload controls (body size is enforced by the
    // router's 64 KiB limit; encoding and media type here).
    let source_ip: IpAddr = ctx.source_ip.unwrap_or(IpAddr::from([127, 0, 0, 1]));
    if let Err(retry_after) = state.limiter.check(source_ip) {
        return Problem::rate_limited(retry_after).into_response_with_request_id(&rid);
    }
    if headers.contains_key(header::CONTENT_ENCODING) {
        return Problem::content_encoding_rejected().into_response_with_request_id(&rid);
    }
    let content_type_ok = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| {
            let essence = v.split(';').next().unwrap_or("").trim();
            essence.eq_ignore_ascii_case("application/json")
        });
    if !content_type_ok {
        return Problem::content_type_invalid().into_response_with_request_id(&rid);
    }

    // 2–3. Duplicate-key rejection, then schema validation.
    let wire: WireRequest = match parse_strict(&body) {
        Ok(wire) => wire,
        Err(ParseFailure::Malformed) => {
            return Problem::malformed_json().into_response_with_request_id(&rid)
        }
        Err(ParseFailure::DuplicateKey) => {
            return Problem::duplicate_json_key().into_response_with_request_id(&rid)
        }
        Err(ParseFailure::Schema(errors)) => {
            return Problem::validation_failed(errors).into_response_with_request_id(&rid)
        }
    };

    // Wire-level pattern checks the domain types do not encode.
    let mut field_errors: Vec<FieldError> = Vec::new();
    for (i, m) in wire.measurements.iter().enumerate() {
        if !m
            .id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        {
            field_errors.push(FieldError {
                pointer: format!("/measurements/{i}/id"),
                code: "PATTERN_MISMATCH".into(),
                message: MEASUREMENT_ID_PATTERN_MESSAGE.into(),
            });
        }
    }
    if !field_errors.is_empty() {
        return Problem::validation_failed(field_errors).into_response_with_request_id(&rid);
    }

    // 4. Resolve the exact requested rule pack (CLIN-006, API-013). An
    // unready service serves no clinical result at all: a draft pack must
    // never answer in clinical mode (CLIN-003), even for a request that
    // arrives while orchestration is removing the instance (OPS-003).
    if !state.ready() {
        return Problem::engine_unavailable().into_response_with_request_id(&rid);
    }
    let Some(pack) = state.pack() else {
        return Problem::engine_unavailable().into_response_with_request_id(&rid);
    };
    if wire.rule_pack_id != pack.file.rule_pack.id {
        return Problem::stale_rule_pack(pack.file.rule_pack.id.clone())
            .into_response_with_request_id(&rid);
    }

    // Domain validation (spec 03 layer two).
    let assessment = match Assessment::new(
        wire.gestational_age_completed_weeks,
        wire.assessment_age_minutes,
        wire.clinical_features,
        wire.risk_factors,
        wire.measurements,
        wire.conjugated_bilirubin_umol_l,
        wire.treatment_state,
    ) {
        Ok(assessment) => assessment,
        Err(errors) => {
            let field_errors = errors
                .into_iter()
                .map(|e| FieldError {
                    pointer: e.pointer,
                    code: serde_json::to_value(e.code)
                        .ok()
                        .and_then(|v| v.as_str().map(str::to_owned))
                        .unwrap_or_else(|| "DOMAIN_INVALID".into()),
                    message: e.message,
                })
                .collect();
            return Problem::validation_failed(field_errors).into_response_with_request_id(&rid);
        }
    };

    // 5–8. Normalise, calculate and evaluate; any safety failure produces no
    // partial result (API-011).
    let mode = state.config.mode;
    let outcome = match evaluate(&assessment, &EvaluationContext { mode }) {
        Ok(outcome) => outcome,
        Err(_) => return Problem::engine_safety_check_failed().into_response_with_request_id(&rid),
    };

    // 9. Receipt, versions and notices.
    let summary = pack.summary();
    let receipt = match decision_receipt(
        &outcome,
        mode,
        clinical_core::ENGINE_VERSION,
        &summary.id,
        &summary.content_sha256,
    ) {
        Ok(receipt) => receipt,
        Err(_) => return Problem::engine_safety_check_failed().into_response_with_request_id(&rid),
    };

    let response_body = WireResponse {
        evaluation_id: uuid::Uuid::now_v7().to_string(),
        request_id: rid.clone(),
        evaluated_at: rfc3339_utc_now(),
        mode,
        api_version: API_VERSION,
        engine_version: clinical_core::ENGINE_VERSION,
        rule_pack: summary,
        normalised_input: outcome.normalised_input,
        thresholds: outcome.thresholds,
        trend: outcome.trend,
        primary_action: outcome.primary_action,
        recommendations: outcome.recommendations,
        warnings: outcome.warnings,
        missing_information: outcome.missing_information,
        suppressed_rules: outcome.suppressed_rules,
        decision_receipt: receipt,
        legal: legal_notices(),
    };

    // 10. Respond no-store; request/response clinical content is dropped
    // with this stack frame (PRD-014).
    (
        [(header::CACHE_CONTROL, HeaderValue::from_static("no-store"))],
        axum::Json(response_body),
    )
        .into_response()
}
