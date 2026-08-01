//! Decision receipt (spec 03): a SHA-256 digest over the RFC 8785 (JCS)
//! canonicalisation of the clinical payload, so clients can detect
//! accidental changes. The server retains neither the receipt nor the
//! digest (PRD-014).

use serde::Serialize;
use sha2::{Digest, Sha256};

use clinical_core::output::EvaluationOutcome;
use clinical_core::types::Mode;

#[derive(Debug, Clone, Serialize)]
pub struct DecisionReceipt {
    pub schema_version: u32,
    pub digest_sha256: String,
    pub canonicalisation: &'static str,
    pub retained_by_server: bool,
}

/// The digest covers the normalised input and every clinical decision field
/// plus the engine and rule-pack identity — everything that must not change
/// for the same input — and excludes operational metadata (evaluation ID,
/// timestamp, request ID), which varies per request.
#[derive(Serialize)]
struct DigestPayload<'a> {
    mode: Mode,
    engine_version: &'a str,
    rule_pack_id: &'a str,
    rule_pack_sha256: &'a str,
    normalised_input: &'a clinical_core::output::NormalisedInput,
    thresholds: &'a [clinical_core::output::ThresholdAssessment],
    trend: &'a Option<clinical_core::output::TrendAssessment>,
    primary_action: &'a clinical_core::output::Recommendation,
    recommendations: &'a [clinical_core::output::Recommendation],
    warnings: &'a [clinical_core::output::Warning],
    missing_information: &'a [clinical_core::output::MissingInformation],
    suppressed_rules: &'a [String],
}

pub fn decision_receipt(
    outcome: &EvaluationOutcome,
    mode: Mode,
    engine_version: &str,
    rule_pack_id: &str,
    rule_pack_sha256: &str,
) -> Result<DecisionReceipt, String> {
    let payload = DigestPayload {
        mode,
        engine_version,
        rule_pack_id,
        rule_pack_sha256,
        normalised_input: &outcome.normalised_input,
        thresholds: &outcome.thresholds,
        trend: &outcome.trend,
        primary_action: &outcome.primary_action,
        recommendations: &outcome.recommendations,
        warnings: &outcome.warnings,
        missing_information: &outcome.missing_information,
        suppressed_rules: &outcome.suppressed_rules,
    };
    let canonical = serde_jcs::to_vec(&payload).map_err(|e| e.to_string())?;
    Ok(DecisionReceipt {
        schema_version: 1,
        digest_sha256: hex::encode(Sha256::digest(&canonical)),
        canonicalisation: "JCS-RFC8785",
        retained_by_server: false,
    })
}
