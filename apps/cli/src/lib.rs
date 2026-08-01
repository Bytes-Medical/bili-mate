//! Engineering-only synthetic evaluator (Stage 1 deliverable, spec 11).
//!
//! Parses an `EvaluationRequest` JSON fixture, converts wire types to domain
//! types (DATA-025), evaluates the embedded rule pack in demonstration mode
//! and prints the outcome. Never deployed; the production HTTP layer (M3)
//! additionally enforces transport controls and duplicate-JSON-key rejection.

use serde::{Deserialize, Serialize};

use clinical_core::evaluate::{evaluate, EvaluationContext};
use clinical_core::input::{
    Assessment, ClinicalFeatures, Measurement, RiskFactors, TreatmentState,
};
use clinical_core::output::EvaluationOutcome;
use clinical_core::types::{AgeMinutes, BilirubinUmolL, GestationalWeeks, Mode};
use clinical_core::ValidationError;
use guideline_data::{load_embedded_pack, RulePackSummary};

/// Wire request per `spec/openapi.yaml`. Unknown properties are rejected
/// (DATA-001); range validation happens inside the domain types.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireRequest {
    pub rule_pack_id: String,
    pub gestational_age_completed_weeks: GestationalWeeks,
    pub assessment_age_minutes: AgeMinutes,
    pub clinical_features: ClinicalFeatures,
    pub risk_factors: RiskFactors,
    pub measurements: Vec<Measurement>,
    #[serde(default)]
    pub conjugated_bilirubin_umol_l: Option<BilirubinUmolL>,
    pub treatment_state: TreatmentState,
}

#[derive(Debug, Serialize)]
pub struct CliResponse {
    pub mode: Mode,
    pub engine_version: String,
    pub rule_pack: RulePackSummary,
    #[serde(flatten)]
    pub outcome: EvaluationOutcome,
}

#[derive(Debug)]
pub enum CliError {
    /// Malformed JSON, unknown property or out-of-range scalar.
    Schema(String),
    /// Domain validation failures with JSON Pointers.
    Domain(Vec<ValidationError>),
    /// The requested pack is not the embedded active-candidate pack
    /// (CLIN-006); no clinical result is produced.
    RulePackMismatch {
        requested: String,
        available: String,
    },
    /// Engine safety failure: no result (API-011).
    Safety(String),
    /// The embedded pack failed its integrity self-tests.
    PackIntegrity(String),
}

pub fn run(request_json: &str) -> Result<CliResponse, CliError> {
    let pack = load_embedded_pack().map_err(|e| CliError::PackIntegrity(e.to_string()))?;

    let wire: WireRequest =
        serde_json::from_str(request_json).map_err(|e| CliError::Schema(e.to_string()))?;

    if wire.rule_pack_id != pack.file.rule_pack.id {
        return Err(CliError::RulePackMismatch {
            requested: wire.rule_pack_id,
            available: pack.file.rule_pack.id.clone(),
        });
    }

    let assessment = Assessment::new(
        wire.gestational_age_completed_weeks,
        wire.assessment_age_minutes,
        wire.clinical_features,
        wire.risk_factors,
        wire.measurements,
        wire.conjugated_bilirubin_umol_l,
        wire.treatment_state,
    )
    .map_err(CliError::Domain)?;

    let outcome = evaluate(
        &assessment,
        &EvaluationContext {
            mode: Mode::Demonstration,
        },
    )
    .map_err(|e| CliError::Safety(e.to_string()))?;

    Ok(CliResponse {
        mode: Mode::Demonstration,
        engine_version: clinical_core::ENGINE_VERSION.to_string(),
        rule_pack: pack.summary(),
        outcome,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const NORMAL: &str = include_str!("../../../spec/examples/normal-below-threshold-request.json");
    const EARLY: &str = include_str!("../../../spec/examples/early-jaundice-request.json");
    const PHOTOTHERAPY: &str = include_str!("../../../spec/examples/phototherapy-request.json");
    const INTENSIFIED: &str =
        include_str!("../../../spec/examples/intensified-phototherapy-request.json");
    const EXCHANGE: &str = include_str!("../../../spec/examples/exchange-escalation-request.json");
    const PROLONGED: &str = include_str!("../../../spec/examples/prolonged-jaundice-request.json");

    fn primary(json: &str) -> String {
        run(json)
            .expect("fixture must evaluate")
            .outcome
            .primary_action
            .code
    }

    #[test]
    fn normal_below_threshold_fixture_matches_published_response() {
        let response = run(NORMAL).unwrap();
        assert_eq!(response.outcome.primary_action.code, "NO_ROUTINE_REPEAT");
        assert_eq!(
            response.outcome.primary_action.action,
            "Do not routinely repeat the bilirubin measurement solely on the basis of this result."
        );
        let row = &response.outcome.thresholds[0];
        assert_eq!(row.phototherapy_threshold_umol_l.unwrap().0, 2500);
        assert_eq!(row.phototherapy_distance_umol_l.unwrap().0, -700);
        assert_eq!(row.exchange_threshold_umol_l.unwrap().0, 4500);
        assert_eq!(row.exchange_distance_umol_l.unwrap().0, -2700);
        assert_eq!(response.rule_pack.id, "nice-cg98-2023-10-31.1");
        assert_eq!(response.rule_pack.source_updated_on, "2023-10-31");
    }

    #[test]
    fn early_jaundice_fixture_continues_serial_serum_measurement() {
        assert_eq!(primary(EARLY), "EARLY_JAUNDICE_REPEAT_6H");
        let response = run(EARLY).unwrap();
        // Metabolic screen was submitted unknown: reported, not assumed.
        assert!(response
            .outcome
            .missing_information
            .iter()
            .any(|m| m.field == "/clinical_features/routine_metabolic_screen_completed"));
    }

    #[test]
    fn phototherapy_fixture_starts_phototherapy() {
        assert_eq!(primary(PHOTOTHERAPY), "START_PHOTOTHERAPY");
    }

    #[test]
    fn intensified_fixture_flags_kernicterus_risk_and_intensification() {
        let response = run(INTENSIFIED).unwrap();
        // The serum rise is 10 umol/L/hour: kernicterus risk outranks the
        // intensification advice (spec 02 priority order).
        assert_eq!(
            response.outcome.primary_action.code,
            "INCREASED_KERNICTERUS_RISK"
        );
        let codes: Vec<&str> = response
            .outcome
            .recommendations
            .iter()
            .map(|r| r.code.as_str())
            .collect();
        assert!(codes.contains(&"CONSIDER_INTENSIFIED_PHOTOTHERAPY"));
        assert!(codes.contains(&"PHOTOTHERAPY_CHECK_OVERDUE"));
    }

    #[test]
    fn exchange_fixture_escalates() {
        let response = run(EXCHANGE).unwrap();
        assert_eq!(
            response.outcome.primary_action.code,
            "EXCHANGE_TRANSFUSION_ESCALATION"
        );
        // Unknown danger fields in the fixture are surfaced.
        assert!(response
            .outcome
            .missing_information
            .iter()
            .any(|m| m.field == "/clinical_features/infection_suspected"));
    }

    #[test]
    fn prolonged_fixture_reaches_expert_liver_advice() {
        let response = run(PROLONGED).unwrap();
        assert_eq!(response.outcome.primary_action.code, "EXPERT_LIVER_ADVICE");
        let codes: Vec<&str> = response
            .outcome
            .recommendations
            .iter()
            .map(|r| r.code.as_str())
            .collect();
        assert!(codes.contains(&"PROLONGED_JAUNDICE_ASSESSMENT"));
        assert!(response.outcome.thresholds.is_empty());
    }

    #[test]
    fn unknown_property_is_rejected() {
        let with_identifier = NORMAL.replace(
            "\"rule_pack_id\"",
            "\"nhs_number\": \"999\", \"rule_pack_id\"",
        );
        assert!(matches!(run(&with_identifier), Err(CliError::Schema(_))));
    }

    #[test]
    fn stale_rule_pack_is_a_conflict_with_no_clinical_result() {
        let stale = NORMAL.replace("nice-cg98-2023-10-31.1", "nice-cg98-2010-05-19.1");
        match run(&stale) {
            Err(CliError::RulePackMismatch {
                requested,
                available,
            }) => {
                assert_eq!(requested, "nice-cg98-2010-05-19.1");
                assert_eq!(available, "nice-cg98-2023-10-31.1");
            }
            other => panic!("expected rule-pack mismatch, got {other:?}"),
        }
    }

    #[test]
    fn out_of_range_gestation_is_a_schema_error() {
        let invalid = NORMAL.replace(
            "\"gestational_age_completed_weeks\": 38",
            "\"gestational_age_completed_weeks\": 22",
        );
        assert!(matches!(run(&invalid), Err(CliError::Schema(_))));
    }
}
