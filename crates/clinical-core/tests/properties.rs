//! Property-based tests over the valid input domain (spec 09):
//! measurement-order invariance (TEST-009), duplicate-age rejection
//! (TEST-010) and byte-equivalent determinism (TEST-011).

use proptest::prelude::*;

use clinical_core::error::ValidationCode;
use clinical_core::evaluate::{evaluate, EvaluationContext};
use clinical_core::input::{
    Assessment, ClinicalFeatures, Measurement, RiskFactors, TreatmentState,
};
use clinical_core::types::{
    AgeMinutes, BilirubinUmolL, GestationalWeeks, MeasurementMethod, Mode, TreatmentMode, TriState,
};

fn tri_state() -> impl Strategy<Value = TriState> {
    prop_oneof![
        Just(TriState::Present),
        Just(TriState::Absent),
        Just(TriState::Unknown),
    ]
}

fn method() -> impl Strategy<Value = MeasurementMethod> {
    prop_oneof![
        Just(MeasurementMethod::Serum),
        Just(MeasurementMethod::Transcutaneous),
    ]
}

fn clinical_features() -> impl Strategy<Value = ClinicalFeatures> {
    (
        (
            tri_state(),
            tri_state(),
            tri_state(),
            tri_state(),
            tri_state(),
            tri_state(),
        ),
        (
            tri_state(),
            tri_state(),
            tri_state(),
            tri_state(),
            tri_state(),
        ),
    )
        .prop_map(|((a, b, c, d, e, f), (g, h, i, j, k))| ClinicalFeatures {
            suspected_or_obvious_jaundice: a,
            visible_jaundice: b,
            clinically_well: c,
            acute_bilirubin_encephalopathy: d,
            pale_chalky_stools: e,
            dark_urine_stains_nappy: f,
            rhesus_haemolytic_disease: g,
            abo_haemolytic_disease: h,
            infection_suspected: i,
            urinary_tract_infection_suspected: j,
            routine_metabolic_screen_completed: k,
        })
}

fn risk_factors() -> impl Strategy<Value = RiskFactors> {
    (tri_state(), tri_state()).prop_map(|(s, b)| RiskFactors {
        previous_sibling_required_phototherapy: s,
        exclusive_breastfeeding_intended: b,
    })
}

/// Measurements with unique ages no later than the assessment age.
fn measurements(assessment_age: u32) -> impl Strategy<Value = Vec<Measurement>> {
    (
        proptest::collection::btree_set(0..=assessment_age, 0..5),
        proptest::collection::vec((0..=1000u16, method()), 5),
    )
        .prop_map(|(ages, values)| {
            ages.into_iter()
                .zip(values)
                .enumerate()
                .map(|(i, (age, (value, method)))| Measurement {
                    id: format!("m{i}"),
                    age_minutes: AgeMinutes::new(age).unwrap(),
                    total_bilirubin_umol_l: BilirubinUmolL::new(value).unwrap(),
                    method,
                })
                .collect()
        })
}

fn treatment_state(assessment_age: u32) -> impl Strategy<Value = TreatmentState> {
    prop_oneof![
        Just(TreatmentState {
            mode: TreatmentMode::None,
            started_age_minutes: None,
            stopped_age_minutes: None,
            exchange_completed_age_minutes: None,
        }),
        (0..=assessment_age, prop::bool::ANY).prop_map(move |(start, intensified)| {
            TreatmentState {
                mode: if intensified {
                    TreatmentMode::IntensifiedPhototherapy
                } else {
                    TreatmentMode::Phototherapy
                },
                started_age_minutes: Some(AgeMinutes::new(start).unwrap()),
                stopped_age_minutes: None,
                exchange_completed_age_minutes: None,
            }
        }),
        // Stop must be strictly later than start (DATA-013), so stop is at
        // least 1 and start strictly below it. `assessment_age` is >= 1 in
        // every generated case.
        (1..=assessment_age).prop_flat_map(move |stop| {
            (0..stop, Just(stop)).prop_map(|(start, stop)| TreatmentState {
                mode: TreatmentMode::PostPhototherapy,
                started_age_minutes: Some(AgeMinutes::new(start).unwrap()),
                stopped_age_minutes: Some(AgeMinutes::new(stop).unwrap()),
                exchange_completed_age_minutes: None,
            })
        }),
        (0..=assessment_age).prop_map(move |done| TreatmentState {
            mode: TreatmentMode::PostExchange,
            started_age_minutes: None,
            stopped_age_minutes: None,
            exchange_completed_age_minutes: Some(AgeMinutes::new(done).unwrap()),
        }),
    ]
}

#[derive(Debug, Clone)]
struct Case {
    gestation: u8,
    age: u32,
    features: ClinicalFeatures,
    risks: RiskFactors,
    measurements: Vec<Measurement>,
    conjugated: Option<u16>,
    treatment: TreatmentState,
}

fn valid_case() -> impl Strategy<Value = Case> {
    (23..=42u8, 1..=40_319u32).prop_flat_map(|(gestation, age)| {
        (
            clinical_features(),
            risk_factors(),
            measurements(age),
            proptest::option::of(0..=1000u16),
            treatment_state(age),
        )
            .prop_map(
                move |(features, risks, measurements, conjugated, treatment)| Case {
                    gestation,
                    age,
                    features,
                    risks,
                    measurements,
                    conjugated,
                    treatment,
                },
            )
    })
}

fn build(case: &Case, measurements: Vec<Measurement>) -> Assessment {
    Assessment::new(
        GestationalWeeks::new(case.gestation).unwrap(),
        AgeMinutes::new(case.age).unwrap(),
        case.features,
        case.risks,
        measurements,
        case.conjugated.map(|v| BilirubinUmolL::new(v).unwrap()),
        case.treatment,
    )
    .expect("generated case must be valid")
}

const CTX: EvaluationContext = EvaluationContext {
    mode: Mode::Demonstration,
};

proptest! {
    /// TEST-009: permuting a valid measurement array never changes the
    /// normalised clinical result.
    #[test]
    fn measurement_order_never_changes_the_result(case in valid_case()) {
        let baseline = evaluate(&build(&case, case.measurements.clone()), &CTX).unwrap();
        let mut reversed = case.measurements.clone();
        reversed.reverse();
        let permuted = evaluate(&build(&case, reversed), &CTX).unwrap();
        prop_assert_eq!(baseline, permuted);
    }

    /// TEST-011: equivalent inputs produce byte-equivalent canonical clinical
    /// payloads before operational metadata is added.
    #[test]
    fn evaluation_is_byte_deterministic(case in valid_case()) {
        let first = serde_json::to_vec(&evaluate(&build(&case, case.measurements.clone()), &CTX).unwrap()).unwrap();
        let second = serde_json::to_vec(&evaluate(&build(&case, case.measurements.clone()), &CTX).unwrap()).unwrap();
        prop_assert_eq!(first, second);
    }

    /// TEST-010: duplicate measurement ages are always rejected, never
    /// averaged or resolved automatically (DATA-011).
    #[test]
    fn duplicate_ages_are_always_rejected(case in valid_case()) {
        prop_assume!(!case.measurements.is_empty());
        let mut duplicated = case.measurements.clone();
        let mut copy = duplicated[0].clone();
        copy.id = "dup".into();
        duplicated.push(copy);
        let result = Assessment::new(
            GestationalWeeks::new(case.gestation).unwrap(),
            AgeMinutes::new(case.age).unwrap(),
            case.features,
            case.risks,
            duplicated,
            case.conjugated.map(|v| BilirubinUmolL::new(v).unwrap()),
            case.treatment,
        );
        let errors = result.expect_err("duplicate age must be rejected");
        prop_assert!(errors.iter().any(|e| e.code == ValidationCode::DuplicateMeasurementAge));
    }

    /// Every valid input yields a complete outcome with a primary action
    /// that appears exactly once, first, in the recommendation list
    /// (DATA-019, DATA-020), and every recommendation requires clinician
    /// confirmation (DATA-017).
    #[test]
    fn every_valid_input_yields_a_primary_action(case in valid_case()) {
        let outcome = evaluate(&build(&case, case.measurements.clone()), &CTX).unwrap();
        prop_assert!(!outcome.recommendations.is_empty());
        prop_assert_eq!(&outcome.recommendations[0], &outcome.primary_action);
        let occurrences = outcome
            .recommendations
            .iter()
            .filter(|r| r.code == outcome.primary_action.code)
            .count();
        prop_assert_eq!(occurrences, 1);
        for rec in &outcome.recommendations {
            prop_assert!(rec.requires_clinician_confirmation);
            prop_assert!(!rec.source_refs.is_empty());
        }
    }

    /// No threshold row ever reports a line past 20,160 minutes (CLIN-027)
    /// and TcB rows are never treatment-decision eligible (CLIN-017).
    #[test]
    fn threshold_rows_respect_scope_and_method(case in valid_case()) {
        let assessment = build(&case, case.measurements.clone());
        let outcome = evaluate(&assessment, &CTX).unwrap();
        for row in &outcome.thresholds {
            if row.age_minutes > 20_160 {
                prop_assert!(row.phototherapy_threshold_umol_l.is_none());
                prop_assert!(row.exchange_threshold_umol_l.is_none());
            } else {
                prop_assert!(row.phototherapy_threshold_umol_l.is_some());
                prop_assert!(row.exchange_threshold_umol_l.is_some());
            }
            let source = assessment
                .measurements
                .iter()
                .find(|m| m.id == row.measurement_id)
                .unwrap();
            prop_assert_eq!(
                row.treatment_decision_eligible,
                source.method == MeasurementMethod::Serum
            );
        }
    }
}
