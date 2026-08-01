//! Rule-branch tests (spec 09): positive activation, nearest negative,
//! unknown-input behaviour, priority and suppression for the CG98 rule set,
//! plus the named multi-rule scenarios.

mod common;

use clinical_core::types::{Priority, TriState};
use common::{activated, assert_active, assert_inactive, primary, recommended, Case};

// ---------------------------------------------------------------------------
// Recognition and measurement-method rules
// ---------------------------------------------------------------------------

#[test]
fn no_routine_bilirubin_requires_confirmed_absence() {
    let absent = Case::new(38, 2880).eval();
    assert_active(&absent, "NO_ROUTINE_BILIRUBIN");
    assert_eq!(primary(&absent), "NO_ROUTINE_BILIRUBIN");

    // Unknown is not absence (PRD-008).
    let unknown = Case::new(38, 2880)
        .feature(|f| f.suspected_or_obvious_jaundice = TriState::Unknown)
        .eval();
    assert_inactive(&unknown, "NO_ROUTINE_BILIRUBIN");
    assert!(unknown
        .missing_information
        .iter()
        .any(|m| m.pointer == "/clinical_features/suspected_or_obvious_jaundice"));
}

#[test]
fn early_jaundice_measure_2h_only_until_a_serum_result_exists() {
    let no_result = Case::new(38, 600).jaundice().eval();
    assert_active(&no_result, "EARLY_JAUNDICE_MEASURE_2H");
    assert_eq!(primary(&no_result), "EARLY_JAUNDICE_MEASURE_2H");

    // A TcB result does not satisfy the serum requirement in the first day.
    let tcb_only = Case::new(38, 600).jaundice().tcb(500, 90).eval();
    assert_active(&tcb_only, "EARLY_JAUNDICE_MEASURE_2H");

    let with_serum = Case::new(38, 600).jaundice().serum(500, 90).eval();
    assert_inactive(&with_serum, "EARLY_JAUNDICE_MEASURE_2H");
    assert_active(&with_serum, "EARLY_JAUNDICE_MEDICAL_REVIEW_6H");
}

#[test]
fn early_jaundice_repeat_6h_until_below_and_stable_or_falling() {
    // Single below-line result, no trend yet: keep repeating.
    let no_trend = Case::new(38, 700).jaundice().serum(600, 90).eval();
    assert_active(&no_trend, "EARLY_JAUNDICE_REPEAT_6H");

    // Below the line and falling: repeat rule stands down.
    let falling = Case::new(38, 900)
        .jaundice()
        .serum(600, 95)
        .serum(840, 90)
        .eval();
    assert_inactive(&falling, "EARLY_JAUNDICE_REPEAT_6H");

    // Below the line but rising: keep repeating.
    let rising = Case::new(38, 900)
        .jaundice()
        .serum(600, 90)
        .serum(840, 100)
        .eval();
    assert_active(&rising, "EARLY_JAUNDICE_REPEAT_6H");
}

#[test]
fn jaundice_measure_6h_only_when_no_measurement_supplied() {
    let no_measurement = Case::new(38, 2880).jaundice().eval();
    assert_active(&no_measurement, "JAUNDICE_MEASURE_6H");
    assert_eq!(primary(&no_measurement), "JAUNDICE_MEASURE_6H");

    let with_measurement = Case::new(38, 2880).jaundice().serum(2800, 180).eval();
    assert_inactive(&with_measurement, "JAUNDICE_MEASURE_6H");
}

#[test]
fn serum_required_subsequent_after_line_reached_or_treatment() {
    // A previous result at the line forces serum thereafter, even though the
    // latest value is lower.
    let line_reached = Case::new(38, 4320)
        .jaundice()
        .serum(2880, 250)
        .serum(4320, 240)
        .eval();
    assert_active(&line_reached, "SERUM_REQUIRED_SUBSEQUENT");
    assert_inactive(&line_reached, "TCB_INITIAL_ALLOWED");

    let on_treatment = Case::new(38, 4320)
        .jaundice()
        .serum(4000, 200)
        .phototherapy(4100)
        .eval();
    assert_active(&on_treatment, "SERUM_REQUIRED_SUBSEQUENT");
    assert_inactive(&on_treatment, "TCB_INITIAL_ALLOWED");

    let neither = Case::new(38, 4320).jaundice().serum(4320, 200).eval();
    assert_inactive(&neither, "SERUM_REQUIRED_SUBSEQUENT");
    assert_active(&neither, "TCB_INITIAL_ALLOWED");
}

#[test]
fn measurement_guidance_brings_icterometer_and_prediction_prohibitions() {
    let outcome = Case::new(38, 2880).jaundice().eval();
    assert_active(&outcome, "NO_ICETEROMETER");
    assert_active(&outcome, "DO_NOT_USE_PREDICTION_TESTS");
}

// ---------------------------------------------------------------------------
// Unknown danger signs (CLIN-030, CLIN-033, PRD-025)
// ---------------------------------------------------------------------------

#[test]
fn unknown_encephalopathy_blocks_reassuring_primary() {
    let outcome = Case::new(38, 2880)
        .feature(|f| f.acute_bilirubin_encephalopathy = TriState::Unknown)
        .eval();
    assert_active(&outcome, "INCOMPLETE_DANGER_ASSESSMENT");
    assert_eq!(primary(&outcome), "INCOMPLETE_DANGER_ASSESSMENT");
    assert_eq!(outcome.primary_action.priority, Priority::Urgent);
    // The reassuring no-routine code is suppressed, not merely outranked.
    assert!(outcome
        .suppressed_rules
        .contains(&"NO_ROUTINE_BILIRUBIN".to_string()));
    assert!(!recommended(&outcome).contains(&"NO_ROUTINE_BILIRUBIN"));
    // The unknown rule itself never activates the positive emergency.
    assert_inactive(&outcome, "ACUTE_BILIRUBIN_ENCEPHALOPATHY_EMERGENCY");
}

#[test]
fn unknown_clinical_state_prevents_no_routine_repeat() {
    // Well-baby repeat rules need confirmed wellbeing (CLIN-033).
    let outcome = Case::new(38, 2880)
        .jaundice()
        .feature(|f| f.clinically_well = TriState::Unknown)
        .serum(2880, 180)
        .eval();
    assert_inactive(&outcome, "NO_ROUTINE_REPEAT");
    assert_active(&outcome, "RETEST_INTERVAL_LOCAL_PROTOCOL");
    assert_active(&outcome, "INCOMPLETE_DANGER_ASSESSMENT");
    assert_eq!(primary(&outcome), "INCOMPLETE_DANGER_ASSESSMENT");
}

#[test]
fn every_danger_field_unknown_is_reported() {
    let outcome = Case::new(38, 2880)
        .feature(|f| {
            f.acute_bilirubin_encephalopathy = TriState::Unknown;
            f.clinically_well = TriState::Unknown;
            f.pale_chalky_stools = TriState::Unknown;
            f.dark_urine_stains_nappy = TriState::Unknown;
            f.infection_suspected = TriState::Unknown;
        })
        .eval();
    assert_active(&outcome, "INCOMPLETE_DANGER_ASSESSMENT");
    for field in [
        "acute_bilirubin_encephalopathy",
        "clinically_well",
        "pale_chalky_stools",
        "dark_urine_stains_nappy",
        "infection_suspected",
    ] {
        assert!(
            outcome
                .missing_information
                .iter()
                .any(|m| m.pointer == format!("/clinical_features/{field}")),
            "missing_information must list {field}"
        );
    }
}

// ---------------------------------------------------------------------------
// Treatment pathway
// ---------------------------------------------------------------------------

#[test]
fn start_phototherapy_between_the_lines() {
    // 38 weeks at 48 h: photo 250, exchange 450.
    let outcome = Case::new(38, 2880).jaundice().serum(2880, 270).eval();
    assert_active(&outcome, "START_PHOTOTHERAPY");
    assert_eq!(primary(&outcome), "START_PHOTOTHERAPY");
    assert_eq!(outcome.primary_action.priority, Priority::Treatment);
    assert_active(&outcome, "ASSESS_UNDERLYING_DISEASE");
    assert_active(&outcome, "PHOTOTHERAPY_CARE_INFORMATION");
    assert_active(&outcome, "DO_NOT_USE_SUNLIGHT");
    assert_active(&outcome, "SERUM_REQUIRED_SUBSEQUENT");
}

#[test]
fn start_phototherapy_needs_serum_not_tcb() {
    let outcome = Case::new(38, 2880).jaundice().tcb(2880, 270).eval();
    assert_inactive(&outcome, "START_PHOTOTHERAPY");
    assert_active(&outcome, "SERUM_CONFIRM_TCB_250");
    assert_active(&outcome, "SERUM_CONFIRM_TREATMENT_LINE");
    assert!(outcome
        .warnings
        .iter()
        .any(|w| w.code == "TCB_NOT_TREATMENT_DECISION_ELIGIBLE"));
    assert!(!outcome.thresholds[0].treatment_decision_eligible);
}

#[test]
fn stop_phototherapy_suppresses_continuing_checks() {
    // Falling well below the line during phototherapy.
    let outcome = Case::new(38, 4320)
        .jaundice()
        .serum(4000, 320)
        .serum(4320, 240)
        .phototherapy(4010)
        .eval();
    assert_active(&outcome, "STOP_PHOTOTHERAPY");
    assert_eq!(primary(&outcome), "STOP_PHOTOTHERAPY");
    // The 6–12 h continuing check contradicts stopping and is suppressed.
    assert_active(&outcome, "PHOTOTHERAPY_CHECK_6_12H");
    assert!(outcome
        .suppressed_rules
        .contains(&"PHOTOTHERAPY_CHECK_6_12H".to_string()));
}

#[test]
fn rebound_check_after_stopping() {
    let outcome = Case::new(38, 4400)
        .jaundice()
        .serum(4300, 200)
        .post_phototherapy(3000, 4000)
        .eval();
    assert_active(&outcome, "REBOUND_CHECK_12_18H");
    assert_eq!(primary(&outcome), "REBOUND_CHECK_12_18H");
    assert_active(&outcome, "SERUM_REQUIRED_SUBSEQUENT");
}

#[test]
fn phototherapy_check_6_12h_needs_serum_trend() {
    let serum_falling = Case::new(38, 4320)
        .jaundice()
        .serum(4000, 320)
        .serum(4320, 300)
        .phototherapy(4010)
        .eval();
    assert_active(&serum_falling, "PHOTOTHERAPY_CHECK_6_12H");

    // A TcB pair cannot show a reliable serum trend.
    let tcb_involved = Case::new(38, 4320)
        .jaundice()
        .serum(4000, 320)
        .tcb(4320, 300)
        .phototherapy(4010)
        .eval();
    assert_inactive(&tcb_involved, "PHOTOTHERAPY_CHECK_6_12H");
}

// ---------------------------------------------------------------------------
// Multi-rule scenario: phototherapy failure to respond (CLIN-039/040)
// ---------------------------------------------------------------------------

#[test]
fn nonresponse_detected_before_six_hours() {
    // Post-start serum has not fallen 2 hours in: do not wait for 6 hours.
    let outcome = Case::new(38, 3120)
        .jaundice()
        .serum(2990, 280)
        .serum(3120, 285)
        .phototherapy(3000)
        .eval();
    assert_active(&outcome, "CONSIDER_INTENSIFIED_PHOTOTHERAPY");
    assert_inactive(&outcome, "PHOTOTHERAPY_CHECK_OVERDUE");
    assert_eq!(primary(&outcome), "CONSIDER_INTENSIFIED_PHOTOTHERAPY");
}

#[test]
fn nonresponse_at_exactly_six_hours() {
    // An unchanged value counts as failure to fall.
    let outcome = Case::new(38, 3360)
        .jaundice()
        .serum(2990, 280)
        .serum(3360, 280)
        .phototherapy(3000)
        .eval();
    assert_active(&outcome, "CONSIDER_INTENSIFIED_PHOTOTHERAPY");
    assert_inactive(&outcome, "PHOTOTHERAPY_CHECK_OVERDUE");
}

#[test]
fn late_result_still_activates_nonresponse_and_flags_overdue_monitoring() {
    // First post-start serum arrives after the 6-hour deadline and shows no
    // fall: both the intensification branch and the overdue flag apply.
    let outcome = Case::new(38, 3500)
        .jaundice()
        .serum(2990, 280)
        .serum(3450, 285)
        .phototherapy(3000)
        .eval();
    assert_active(&outcome, "CONSIDER_INTENSIFIED_PHOTOTHERAPY");
    assert_active(&outcome, "PHOTOTHERAPY_CHECK_OVERDUE");
}

#[test]
fn falling_response_is_not_nonresponse() {
    let outcome = Case::new(38, 3120)
        .jaundice()
        .serum(2990, 280)
        .serum(3120, 270)
        .phototherapy(3000)
        .eval();
    assert_inactive(&outcome, "CONSIDER_INTENSIFIED_PHOTOTHERAPY");
}

#[test]
fn missing_baseline_returns_incomplete_not_nonresponse() {
    let outcome = Case::new(38, 3200)
        .jaundice()
        .serum(3100, 285)
        .phototherapy(3000)
        .eval();
    assert_inactive(&outcome, "CONSIDER_INTENSIFIED_PHOTOTHERAPY");
    assert_active(&outcome, "PHOTOTHERAPY_RESPONSE_INCOMPLETE");
    assert!(outcome
        .missing_information
        .iter()
        .any(|m| m.code == "PHOTOTHERAPY_RESPONSE_COMPARISON_UNAVAILABLE"));
}

#[test]
fn missing_post_start_result_after_six_hours_is_overdue_and_incomplete() {
    let outcome = Case::new(38, 3400)
        .jaundice()
        .serum(2990, 280)
        .phototherapy(3000)
        .eval();
    assert_active(&outcome, "PHOTOTHERAPY_CHECK_OVERDUE");
    assert_active(&outcome, "PHOTOTHERAPY_RESPONSE_INCOMPLETE");
    assert_inactive(&outcome, "CONSIDER_INTENSIFIED_PHOTOTHERAPY");
    assert_inactive(&outcome, "PHOTOTHERAPY_CHECK_4_6H");
}

#[test]
fn within_six_hours_awaiting_first_check() {
    let outcome = Case::new(38, 3100)
        .jaundice()
        .serum(2990, 280)
        .phototherapy(3000)
        .eval();
    assert_active(&outcome, "PHOTOTHERAPY_CHECK_4_6H");
    assert_inactive(&outcome, "PHOTOTHERAPY_CHECK_OVERDUE");
}

// ---------------------------------------------------------------------------
// Multi-rule scenario: early jaundice above the exchange line
// ---------------------------------------------------------------------------

#[test]
fn early_jaundice_above_exchange_line() {
    // 38 weeks at 12 h: exchange line 200. Serum 250 exceeds it.
    let outcome = Case::new(38, 720).jaundice().serum(720, 250).eval();
    assert_eq!(primary(&outcome), "EXCHANGE_TRANSFUSION_ESCALATION");
    assert_eq!(outcome.primary_action.priority, Priority::Emergency);
    // Early-jaundice supporting actions stay visible (CLIN-049).
    assert_active(&outcome, "EARLY_JAUNDICE_MEDICAL_REVIEW_6H");
    assert!(recommended(&outcome).contains(&"EARLY_JAUNDICE_MEDICAL_REVIEW_6H"));
    assert_active(&outcome, "ASSESS_UNDERLYING_DISEASE");
    assert_active(&outcome, "EXCHANGE_TRANSFUSION_INFORMATION");
    // Exchange precedence over phototherapy (CLIN-025): no start-phototherapy
    // instruction alongside the escalation.
    assert_inactive(&outcome, "START_PHOTOTHERAPY");
}

// ---------------------------------------------------------------------------
// Multi-rule scenario: rapid rise during standard phototherapy
// ---------------------------------------------------------------------------

#[test]
fn rapid_rise_during_standard_phototherapy() {
    // +30 umol/L over 2 hours = 15/h during phototherapy.
    let outcome = Case::new(38, 3120)
        .jaundice()
        .serum(2990, 250)
        .serum(3110, 280)
        .phototherapy(3000)
        .eval();
    assert_active(&outcome, "CONSIDER_INTENSIFIED_PHOTOTHERAPY");
    assert_active(&outcome, "INCREASED_KERNICTERUS_RISK");
    assert_eq!(primary(&outcome), "INCREASED_KERNICTERUS_RISK");
    // Standard, not intensified, phototherapy: the IVIG pathway needs
    // intensified treatment.
    assert_inactive(&outcome, "IVIG_SPECIALIST_PATHWAY");
}

// ---------------------------------------------------------------------------
// Multi-rule scenario: encephalopathy below the numeric lines (CLIN-041)
// ---------------------------------------------------------------------------

#[test]
fn encephalopathy_outranks_every_numeric_line() {
    // Serum far below both lines; encephalopathy still an emergency.
    let outcome = Case::new(38, 2880)
        .jaundice()
        .feature(|f| f.acute_bilirubin_encephalopathy = TriState::Present)
        .serum(2880, 150)
        .eval();
    assert_eq!(
        primary(&outcome),
        "ACUTE_BILIRUBIN_ENCEPHALOPATHY_EMERGENCY"
    );
    assert_eq!(outcome.primary_action.priority, Priority::Emergency);
    assert_active(&outcome, "EXCHANGE_TRANSFUSION_ESCALATION");
    assert_active(&outcome, "INCREASED_KERNICTERUS_RISK");
    assert_active(&outcome, "EXCHANGE_TRANSFUSION_INFORMATION");
    // The reassuring below-line codes are suppressed (CLIN-048).
    assert!(outcome
        .suppressed_rules
        .contains(&"NO_ROUTINE_REPEAT".to_string()));
    assert!(!recommended(&outcome).contains(&"NO_ROUTINE_REPEAT"));
}

// ---------------------------------------------------------------------------
// Multi-rule scenario: IVIG pathway
// ---------------------------------------------------------------------------

#[test]
fn ivig_requires_every_precondition() {
    let rapid_pair = |case: Case| case.serum(4000, 300).serum(4120, 330);

    // All present: haemolytic disease + intensified + rapid rise.
    let complete = rapid_pair(
        Case::new(38, 4200)
            .jaundice()
            .feature(|f| f.rhesus_haemolytic_disease = TriState::Present),
    )
    .intensified(3900)
    .eval();
    assert_active(&complete, "IVIG_SPECIALIST_PATHWAY");
    assert_active(&complete, "IVIG_INFORMATION");
    let ivig = complete
        .recommendations
        .iter()
        .find(|r| r.code == "IVIG_SPECIALIST_PATHWAY")
        .unwrap();
    assert!(ivig.action.contains("not an order"));
    assert!(ivig.action.contains("neonatal specialist"));

    // Haemolytic disease unknown: the conjunction fails and the gap is
    // reported (spec 09: IVIG preconditions partly unknown).
    let unknown_disease = rapid_pair(
        Case::new(38, 4200)
            .jaundice()
            .feature(|f| f.rhesus_haemolytic_disease = TriState::Unknown),
    )
    .intensified(3900)
    .eval();
    assert_inactive(&unknown_disease, "IVIG_SPECIALIST_PATHWAY");
    assert_inactive(&unknown_disease, "IVIG_INFORMATION");
    assert!(unknown_disease
        .missing_information
        .iter()
        .any(|m| m.pointer == "/clinical_features/rhesus_haemolytic_disease"));

    // Standard phototherapy: not intensified, no IVIG.
    let standard = rapid_pair(
        Case::new(38, 4200)
            .jaundice()
            .feature(|f| f.rhesus_haemolytic_disease = TriState::Present),
    )
    .phototherapy(3900)
    .eval();
    assert_inactive(&standard, "IVIG_SPECIALIST_PATHWAY");
}

// ---------------------------------------------------------------------------
// Multi-rule scenario: prolonged jaundice with liver danger signs
// ---------------------------------------------------------------------------

#[test]
fn prolonged_jaundice_with_dark_urine_and_high_conjugated_bilirubin() {
    let outcome = Case::new(38, 21_601)
        .jaundice()
        .feature(|f| f.dark_urine_stains_nappy = TriState::Present)
        .conjugated(30)
        .eval();
    assert_active(&outcome, "PROLONGED_JAUNDICE_ASSESSMENT");
    assert_active(&outcome, "EXPERT_LIVER_ADVICE");
    assert_eq!(primary(&outcome), "EXPERT_LIVER_ADVICE");
    assert_eq!(outcome.primary_action.priority, Priority::Immediate);
    // No treatment line after 14 days (CLIN-044) and no reassuring output.
    assert!(outcome.thresholds.is_empty());
    assert_inactive(&outcome, "NO_ROUTINE_BILIRUBIN");
}

#[test]
fn prolonged_jaundice_without_conjugated_result_reports_the_gap() {
    let outcome = Case::new(38, 21_601).jaundice().eval();
    assert_active(&outcome, "PROLONGED_JAUNDICE_ASSESSMENT");
    assert!(outcome
        .missing_information
        .iter()
        .any(|m| m.pointer == "/conjugated_bilirubin_umol_l"));
}

// ---------------------------------------------------------------------------
// Multi-rule scenario: the specification's normal-below-threshold example
// ---------------------------------------------------------------------------

#[test]
fn well_term_baby_more_than_50_below_the_line_matches_the_spec_example() {
    let outcome = Case::new(38, 2880).jaundice().serum(2880, 180).eval();
    assert_eq!(primary(&outcome), "NO_ROUTINE_REPEAT");
    assert_eq!(outcome.primary_action.priority, Priority::Routine);
    let row = &outcome.thresholds[0];
    // Display values match the published example: 250.0 / -70.0 / 450.0 / -270.0.
    assert_eq!(row.phototherapy_threshold_umol_l.unwrap().0, 2500);
    assert_eq!(row.phototherapy_distance_umol_l.unwrap().0, -700);
    assert_eq!(row.exchange_threshold_umol_l.unwrap().0, 4500);
    assert_eq!(row.exchange_distance_umol_l.unwrap().0, -2700);
    assert!(row.treatment_decision_eligible);
    assert!(outcome.missing_information.is_empty());
    assert!(outcome.suppressed_rules.is_empty());
    assert!(outcome
        .warnings
        .iter()
        .any(|w| w.code == "LOCAL_PATHOLOGY_ASSAY_WARNING"));
}

// ---------------------------------------------------------------------------
// Supporting content rules
// ---------------------------------------------------------------------------

#[test]
fn breastfeeding_support_never_advises_stopping() {
    let outcome = Case::new(38, 2880).breastfeeding().eval();
    assert_active(&outcome, "BREASTFEEDING_SUPPORT");
    let support = outcome
        .recommendations
        .iter()
        .find(|r| r.code == "BREASTFEEDING_SUPPORT")
        .unwrap();
    assert!(support.action.contains("not a reason to stop"));
}

#[test]
fn additional_visual_inspection_within_48_hours_for_risk_factors() {
    let preterm = Case::new(37, 2000).eval();
    assert_active(&preterm, "ADDITIONAL_VISUAL_INSPECTION_48H");

    let sibling = Case::new(40, 2000).sibling_risk().eval();
    assert_active(&sibling, "ADDITIONAL_VISUAL_INSPECTION_48H");

    let first_day_jaundice = Case::new(40, 1200).jaundice().eval();
    assert_active(&first_day_jaundice, "ADDITIONAL_VISUAL_INSPECTION_48H");

    // No risk factor, or outside the 48-hour window: inactive.
    let no_risk = Case::new(40, 2000).eval();
    assert_inactive(&no_risk, "ADDITIONAL_VISUAL_INSPECTION_48H");
    let too_old = Case::new(37, 3000).eval();
    assert_inactive(&too_old, "ADDITIONAL_VISUAL_INSPECTION_48H");
}

#[test]
fn universal_warnings_are_always_present() {
    let outcome = Case::new(38, 2880).eval();
    for code in [
        "DEMONSTRATION_ONLY",
        "LOCAL_PATHOLOGY_ASSAY_WARNING",
        "DARKER_SKIN_RECOGNITION",
    ] {
        assert!(
            outcome.warnings.iter().any(|w| w.code == code),
            "warning {code} must always be present"
        );
    }
    assert_active(&outcome, "VISUAL_ASSESSMENT_LIMITATIONS");
    assert_active(&outcome, "PARENT_CARER_INFORMATION");
}

#[test]
fn activated_rules_trace_matches_recommendations_plus_suppressed() {
    let outcome = Case::new(38, 2880)
        .jaundice()
        .feature(|f| f.acute_bilirubin_encephalopathy = TriState::Present)
        .serum(2880, 150)
        .eval();
    let mut from_parts: Vec<&str> = recommended(&outcome);
    from_parts.extend(outcome.suppressed_rules.iter().map(String::as_str));
    from_parts.sort_unstable();
    let mut trace = activated(&outcome);
    trace.sort_unstable();
    assert_eq!(trace, from_parts);
}
