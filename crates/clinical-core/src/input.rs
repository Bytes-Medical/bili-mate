//! Validated assessment input (spec 03). The HTTP/CLI layer converts wire
//! types into these domain types before the core runs (DATA-025, DATA-026);
//! `Assessment::new` enforces the domain invariants that schema validation
//! cannot express.

use serde::{Deserialize, Serialize};

use crate::error::{ValidationCode, ValidationError};
use crate::types::{
    AgeMinutes, BilirubinUmolL, GestationalWeeks, MeasurementMethod, TreatmentMode, TriState,
};

pub const MAX_MEASUREMENTS: usize = 64;

/// Every field is required tri-state; unknown is data, not absence (spec 03).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClinicalFeatures {
    pub suspected_or_obvious_jaundice: TriState,
    pub visible_jaundice: TriState,
    pub clinically_well: TriState,
    pub acute_bilirubin_encephalopathy: TriState,
    pub pale_chalky_stools: TriState,
    pub dark_urine_stains_nappy: TriState,
    pub rhesus_haemolytic_disease: TriState,
    pub abo_haemolytic_disease: TriState,
    pub infection_suspected: TriState,
    pub urinary_tract_infection_suspected: TriState,
    pub routine_metabolic_screen_completed: TriState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RiskFactors {
    pub previous_sibling_required_phototherapy: TriState,
    pub exclusive_breastfeeding_intended: TriState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Measurement {
    /// Request-local identifier, 1–32 characters (DATA-007).
    pub id: String,
    pub age_minutes: AgeMinutes,
    pub total_bilirubin_umol_l: BilirubinUmolL,
    pub method: MeasurementMethod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TreatmentState {
    pub mode: TreatmentMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_age_minutes: Option<AgeMinutes>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stopped_age_minutes: Option<AgeMinutes>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exchange_completed_age_minutes: Option<AgeMinutes>,
}

/// A fully validated assessment. Constructed only through [`Assessment::new`],
/// after which measurements are sorted by age (DATA-009) and every domain
/// invariant holds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Assessment {
    pub gestational_age: GestationalWeeks,
    pub assessment_age: AgeMinutes,
    pub clinical_features: ClinicalFeatures,
    pub risk_factors: RiskFactors,
    pub measurements: Vec<Measurement>,
    pub conjugated_bilirubin_umol_l: Option<BilirubinUmolL>,
    pub treatment_state: TreatmentState,
}

impl Assessment {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        gestational_age: GestationalWeeks,
        assessment_age: AgeMinutes,
        clinical_features: ClinicalFeatures,
        risk_factors: RiskFactors,
        mut measurements: Vec<Measurement>,
        conjugated_bilirubin_umol_l: Option<BilirubinUmolL>,
        treatment_state: TreatmentState,
    ) -> Result<Self, Vec<ValidationError>> {
        let mut errors = Vec::new();

        if measurements.len() > MAX_MEASUREMENTS {
            errors.push(ValidationError {
                pointer: "/measurements".into(),
                code: ValidationCode::TooManyMeasurements,
                message: format!(
                    "a request must contain no more than {MAX_MEASUREMENTS} measurements"
                ),
            });
        }

        for (i, m) in measurements.iter().enumerate() {
            if m.id.is_empty() || m.id.len() > 32 {
                errors.push(ValidationError {
                    pointer: format!("/measurements/{i}/id"),
                    code: ValidationCode::InvalidIdentifier,
                    message: "measurement id must be 1 through 32 characters".into(),
                });
            }
            if m.age_minutes > assessment_age {
                errors.push(ValidationError {
                    pointer: format!("/measurements/{i}/age_minutes"),
                    code: ValidationCode::MeasurementAfterAssessment,
                    message: "measurement age must be no later than the assessment age".into(),
                });
            }
            for (j, other) in measurements.iter().enumerate().take(i) {
                if other.age_minutes == m.age_minutes {
                    // Duplicate ages are rejected, never averaged (DATA-011).
                    errors.push(ValidationError {
                        pointer: format!("/measurements/{i}/age_minutes"),
                        code: ValidationCode::DuplicateMeasurementAge,
                        message: "measurement ages must be unique".into(),
                    });
                }
                if other.id == m.id {
                    errors.push(ValidationError {
                        pointer: format!("/measurements/{i}/id"),
                        code: ValidationCode::DuplicateMeasurementId,
                        message: format!("measurement id duplicates /measurements/{j}/id"),
                    });
                }
            }
        }

        errors.extend(validate_treatment_state(&treatment_state, assessment_age));

        if !errors.is_empty() {
            return Err(errors);
        }

        measurements.sort_by_key(|m| m.age_minutes);

        Ok(Self {
            gestational_age,
            assessment_age,
            clinical_features,
            risk_factors,
            measurements,
            conjugated_bilirubin_umol_l,
            treatment_state,
        })
    }
}

/// Treatment-state invariants from spec 03 (DATA-013). Missing treatment
/// times are never inferred from measurement times (DATA-014).
fn validate_treatment_state(
    state: &TreatmentState,
    assessment_age: AgeMinutes,
) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    let require = |errors: &mut Vec<ValidationError>, field: &str, value: Option<AgeMinutes>| {
        if value.is_none() {
            errors.push(ValidationError {
                pointer: format!("/treatment_state/{field}"),
                code: ValidationCode::TreatmentStateFieldRequired,
                message: format!("{field} is required for this treatment mode"),
            });
        }
    };
    let forbid = |errors: &mut Vec<ValidationError>, field: &str, value: Option<AgeMinutes>| {
        if value.is_some() {
            errors.push(ValidationError {
                pointer: format!("/treatment_state/{field}"),
                code: ValidationCode::TreatmentStateFieldForbidden,
                message: format!("{field} must be absent for this treatment mode"),
            });
        }
    };

    match state.mode {
        TreatmentMode::None => {
            forbid(
                &mut errors,
                "started_age_minutes",
                state.started_age_minutes,
            );
            forbid(
                &mut errors,
                "stopped_age_minutes",
                state.stopped_age_minutes,
            );
            forbid(
                &mut errors,
                "exchange_completed_age_minutes",
                state.exchange_completed_age_minutes,
            );
        }
        TreatmentMode::Phototherapy | TreatmentMode::IntensifiedPhototherapy => {
            require(
                &mut errors,
                "started_age_minutes",
                state.started_age_minutes,
            );
            forbid(
                &mut errors,
                "stopped_age_minutes",
                state.stopped_age_minutes,
            );
            forbid(
                &mut errors,
                "exchange_completed_age_minutes",
                state.exchange_completed_age_minutes,
            );
        }
        TreatmentMode::PostPhototherapy => {
            require(
                &mut errors,
                "started_age_minutes",
                state.started_age_minutes,
            );
            require(
                &mut errors,
                "stopped_age_minutes",
                state.stopped_age_minutes,
            );
            forbid(
                &mut errors,
                "exchange_completed_age_minutes",
                state.exchange_completed_age_minutes,
            );
            if let (Some(start), Some(stop)) =
                (state.started_age_minutes, state.stopped_age_minutes)
            {
                if stop <= start {
                    errors.push(ValidationError {
                        pointer: "/treatment_state/stopped_age_minutes".into(),
                        code: ValidationCode::TreatmentStopNotAfterStart,
                        message: "stop age must be later than start age".into(),
                    });
                }
            }
        }
        TreatmentMode::PostExchange => {
            require(
                &mut errors,
                "exchange_completed_age_minutes",
                state.exchange_completed_age_minutes,
            );
            forbid(
                &mut errors,
                "stopped_age_minutes",
                state.stopped_age_minutes,
            );
            // Phototherapy start may be present alongside a completed exchange.
        }
    }

    for (field, value) in [
        ("started_age_minutes", state.started_age_minutes),
        ("stopped_age_minutes", state.stopped_age_minutes),
        (
            "exchange_completed_age_minutes",
            state.exchange_completed_age_minutes,
        ),
    ] {
        if let Some(age) = value {
            if age > assessment_age {
                errors.push(ValidationError {
                    pointer: format!("/treatment_state/{field}"),
                    code: ValidationCode::TreatmentAgeAfterAssessment,
                    message: format!("{field} must be no later than the assessment age"),
                });
            }
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    fn features() -> ClinicalFeatures {
        ClinicalFeatures {
            suspected_or_obvious_jaundice: TriState::Present,
            visible_jaundice: TriState::Present,
            clinically_well: TriState::Present,
            acute_bilirubin_encephalopathy: TriState::Absent,
            pale_chalky_stools: TriState::Absent,
            dark_urine_stains_nappy: TriState::Absent,
            rhesus_haemolytic_disease: TriState::Absent,
            abo_haemolytic_disease: TriState::Absent,
            infection_suspected: TriState::Absent,
            urinary_tract_infection_suspected: TriState::Absent,
            routine_metabolic_screen_completed: TriState::Present,
        }
    }

    fn risks() -> RiskFactors {
        RiskFactors {
            previous_sibling_required_phototherapy: TriState::Absent,
            exclusive_breastfeeding_intended: TriState::Absent,
        }
    }

    fn measurement(id: &str, age: u32, value: u16) -> Measurement {
        Measurement {
            id: id.into(),
            age_minutes: AgeMinutes::new(age).unwrap(),
            total_bilirubin_umol_l: BilirubinUmolL::new(value).unwrap(),
            method: MeasurementMethod::Serum,
        }
    }

    fn none_treatment() -> TreatmentState {
        TreatmentState {
            mode: TreatmentMode::None,
            started_age_minutes: None,
            stopped_age_minutes: None,
            exchange_completed_age_minutes: None,
        }
    }

    #[test]
    fn sorts_measurements_by_age() {
        let a = Assessment::new(
            GestationalWeeks::new(38).unwrap(),
            AgeMinutes::new(3000).unwrap(),
            features(),
            risks(),
            vec![measurement("b", 2000, 200), measurement("a", 1000, 150)],
            None,
            none_treatment(),
        )
        .unwrap();
        assert_eq!(a.measurements[0].id, "a");
        assert_eq!(a.measurements[1].id, "b");
    }

    #[test]
    fn rejects_duplicate_ages() {
        let err = Assessment::new(
            GestationalWeeks::new(38).unwrap(),
            AgeMinutes::new(3000).unwrap(),
            features(),
            risks(),
            vec![measurement("a", 2000, 200), measurement("b", 2000, 210)],
            None,
            none_treatment(),
        )
        .unwrap_err();
        assert!(err
            .iter()
            .any(|e| e.code == ValidationCode::DuplicateMeasurementAge));
    }

    #[test]
    fn rejects_measurement_after_assessment() {
        let err = Assessment::new(
            GestationalWeeks::new(38).unwrap(),
            AgeMinutes::new(1000).unwrap(),
            features(),
            risks(),
            vec![measurement("a", 2000, 200)],
            None,
            none_treatment(),
        )
        .unwrap_err();
        assert!(err
            .iter()
            .any(|e| e.code == ValidationCode::MeasurementAfterAssessment));
    }

    #[test]
    fn phototherapy_requires_start_age() {
        let err = Assessment::new(
            GestationalWeeks::new(38).unwrap(),
            AgeMinutes::new(3000).unwrap(),
            features(),
            risks(),
            vec![],
            None,
            TreatmentState {
                mode: TreatmentMode::Phototherapy,
                started_age_minutes: None,
                stopped_age_minutes: None,
                exchange_completed_age_minutes: None,
            },
        )
        .unwrap_err();
        assert!(err
            .iter()
            .any(|e| e.code == ValidationCode::TreatmentStateFieldRequired));
    }

    #[test]
    fn post_phototherapy_stop_must_follow_start() {
        let err = Assessment::new(
            GestationalWeeks::new(38).unwrap(),
            AgeMinutes::new(3000).unwrap(),
            features(),
            risks(),
            vec![],
            None,
            TreatmentState {
                mode: TreatmentMode::PostPhototherapy,
                started_age_minutes: Some(AgeMinutes::new(2000).unwrap()),
                stopped_age_minutes: Some(AgeMinutes::new(2000).unwrap()),
                exchange_completed_age_minutes: None,
            },
        )
        .unwrap_err();
        assert!(err
            .iter()
            .any(|e| e.code == ValidationCode::TreatmentStopNotAfterStart));
    }

    #[test]
    fn none_mode_forbids_ages() {
        let err = Assessment::new(
            GestationalWeeks::new(38).unwrap(),
            AgeMinutes::new(3000).unwrap(),
            features(),
            risks(),
            vec![],
            None,
            TreatmentState {
                mode: TreatmentMode::None,
                started_age_minutes: Some(AgeMinutes::new(100).unwrap()),
                stopped_age_minutes: None,
                exchange_completed_age_minutes: None,
            },
        )
        .unwrap_err();
        assert!(err
            .iter()
            .any(|e| e.code == ValidationCode::TreatmentStateFieldForbidden));
    }
}
