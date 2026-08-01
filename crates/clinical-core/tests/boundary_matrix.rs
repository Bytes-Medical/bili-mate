//! The required boundary matrix from spec 09 (TEST-013, TEST-014). Every
//! cell is a named test asserting the expected rule code and priority.
//! Equality behaviour is safety-critical product policy: exact boundary
//! values are never rounded into a neighbouring branch (CLIN-024).

mod common;

use clinical_core::types::Priority;
use common::{activated, assert_active, assert_inactive, primary, Case};

// ---------------------------------------------------------------------------
// Gestation boundaries
// ---------------------------------------------------------------------------

#[test]
fn gestation_22_is_rejected_and_23_accepted() {
    assert!(clinical_core::GestationalWeeks::new(22).is_err());
    assert!(clinical_core::GestationalWeeks::new(23).is_ok());
}

#[test]
fn gestation_42_accepted_43_rejected() {
    assert!(clinical_core::GestationalWeeks::new(42).is_ok());
    assert!(clinical_core::GestationalWeeks::new(43).is_err());
}

#[test]
fn gestation_34_requires_serum_35_allows_tcb() {
    let at_34 = Case::new(34, 2880).jaundice().serum(2880, 100).eval();
    assert_active(&at_34, "SERUM_REQUIRED_GESTATION");
    assert_inactive(&at_34, "TCB_INITIAL_ALLOWED");

    let at_35 = Case::new(35, 2880).jaundice().serum(2880, 100).eval();
    assert_inactive(&at_35, "SERUM_REQUIRED_GESTATION");
    assert_active(&at_35, "TCB_INITIAL_ALLOWED");
}

#[test]
fn kernicterus_gestation_boundary_36_vs_37() {
    // Serum 341 in a baby whose lines are far above: use a low value later.
    // Here we isolate the gestation term: >340 requires gestation >= 37.
    let at_36 = Case::new(36, 10_000).jaundice().serum(10_000, 341).eval();
    // 36 weeks: plateau photo 260 / exchange 360 — 341 is below exchange but
    // the kernicterus gestation condition still fails at 36 weeks.
    assert_inactive(&at_36, "INCREASED_KERNICTERUS_RISK");

    let at_37 = Case::new(37, 10_000).jaundice().serum(10_000, 341).eval();
    assert_active(&at_37, "INCREASED_KERNICTERUS_RISK");
}

#[test]
fn retest_rules_apply_at_38_weeks_but_not_37() {
    // 38 weeks at 48 h: photo line exactly 250; serum 220 is 30 below.
    let term = Case::new(38, 2880).jaundice().serum(2880, 220).eval();
    assert_active(&term, "RETEST_WITHIN_24H");
    assert_inactive(&term, "RETEST_INTERVAL_LOCAL_PROTOCOL");

    // 37 weeks is outside the retest population: local protocol instead.
    let preterm = Case::new(37, 2880).jaundice().serum(2880, 180).eval();
    assert_inactive(&preterm, "RETEST_WITHIN_24H");
    assert_inactive(&preterm, "NO_ROUTINE_REPEAT");
    assert_active(&preterm, "RETEST_INTERVAL_LOCAL_PROTOCOL");
}

// ---------------------------------------------------------------------------
// Assessment-age boundaries: the 24-hour pathway split (CLIN-051)
// ---------------------------------------------------------------------------

#[test]
fn first_day_pathway_applies_at_1439_and_1440() {
    for age in [1439u32, 1440] {
        let outcome = Case::new(38, age).jaundice().eval();
        assert_active(&outcome, "EARLY_JAUNDICE_MEASURE_2H");
        assert_active(&outcome, "EARLY_JAUNDICE_MEDICAL_REVIEW_6H");
        assert_active(&outcome, "SERUM_REQUIRED_AGE");
        assert_inactive(&outcome, "JAUNDICE_MEASURE_6H");
        assert_eq!(primary(&outcome), "EARLY_JAUNDICE_MEASURE_2H", "age {age}");
        assert_eq!(outcome.primary_action.priority, Priority::Urgent);
    }
}

#[test]
fn more_than_24h_pathway_starts_at_1441() {
    let outcome = Case::new(38, 1441).jaundice().eval();
    assert_inactive(&outcome, "EARLY_JAUNDICE_MEASURE_2H");
    assert_inactive(&outcome, "EARLY_JAUNDICE_MEDICAL_REVIEW_6H");
    assert_inactive(&outcome, "SERUM_REQUIRED_AGE");
    assert_active(&outcome, "JAUNDICE_MEASURE_6H");
    assert_eq!(primary(&outcome), "JAUNDICE_MEASURE_6H");
    assert_eq!(outcome.primary_action.priority, Priority::Timed);
}

// ---------------------------------------------------------------------------
// Assessment-age boundaries: treatment-line availability (CLIN-027)
// ---------------------------------------------------------------------------

#[test]
fn treatment_lines_available_at_20159_and_20160_not_20161() {
    for age in [20_159u32, 20_160] {
        let outcome = Case::new(38, age).jaundice().serum(age, 300).eval();
        assert!(
            outcome.thresholds[0]
                .phototherapy_threshold_umol_l
                .is_some(),
            "age {age}"
        );
        assert!(
            outcome.thresholds[0].exchange_threshold_umol_l.is_some(),
            "age {age}"
        );
    }
    let beyond = Case::new(38, 20_161).jaundice().serum(20_161, 300).eval();
    assert!(beyond.thresholds[0].phototherapy_threshold_umol_l.is_none());
    assert!(beyond.thresholds[0].exchange_threshold_umol_l.is_none());
    assert!(beyond
        .warnings
        .iter()
        .any(|w| w.code == "THRESHOLDS_NOT_CALCULATED_AFTER_336_HOURS"));
}

#[test]
fn age_40319_is_the_maximum_valid_age() {
    assert!(clinical_core::AgeMinutes::new(40_319).is_ok());
    assert!(clinical_core::AgeMinutes::new(40_320).is_err());
}

// ---------------------------------------------------------------------------
// TcB serum-confirmation boundary: 249 / 250 / 251 (strictly greater)
// ---------------------------------------------------------------------------

#[test]
fn tcb_250_boundary() {
    // 38 weeks at 72 h: photo line 300, so these values stay below the line
    // and isolate the 250 rule.
    let at_249 = Case::new(38, 4320).jaundice().tcb(4320, 249).eval();
    assert_inactive(&at_249, "SERUM_CONFIRM_TCB_250");

    let at_250 = Case::new(38, 4320).jaundice().tcb(4320, 250).eval();
    assert_inactive(&at_250, "SERUM_CONFIRM_TCB_250");

    let at_251 = Case::new(38, 4320).jaundice().tcb(4320, 251).eval();
    assert_active(&at_251, "SERUM_CONFIRM_TCB_250");
    assert!(
        at_251
            .recommendations
            .iter()
            .find(|r| r.code == "SERUM_CONFIRM_TCB_250")
            .unwrap()
            .requires_serum_confirmation
    );
}

// ---------------------------------------------------------------------------
// Treatment-distance boundaries: 49 / 50 / 51 below both lines
// ---------------------------------------------------------------------------

#[test]
fn below_phototherapy_line_49_50_51() {
    // 38 weeks at 72 h: photo line exactly 300.
    let below_49 = Case::new(38, 4320).jaundice().serum(4320, 251).eval();
    assert_active(&below_49, "RETEST_WITHIN_24H");
    assert_inactive(&below_49, "NO_ROUTINE_REPEAT");

    // Exactly 50 below stays in the within-50 branch (spec 02).
    let below_50 = Case::new(38, 4320).jaundice().serum(4320, 250).eval();
    assert_active(&below_50, "RETEST_WITHIN_24H");
    assert_inactive(&below_50, "NO_ROUTINE_REPEAT");

    let below_51 = Case::new(38, 4320).jaundice().serum(4320, 249).eval();
    assert_inactive(&below_51, "RETEST_WITHIN_24H");
    assert_active(&below_51, "NO_ROUTINE_REPEAT");
    assert_eq!(primary(&below_51), "NO_ROUTINE_REPEAT");
    assert_eq!(below_51.primary_action.priority, Priority::Routine);
}

#[test]
fn retest_18h_with_risk_factor_24h_without() {
    let with_risk = Case::new(38, 4320)
        .jaundice()
        .sibling_risk()
        .serum(4320, 260)
        .eval();
    assert_active(&with_risk, "RETEST_WITHIN_18H");
    assert_inactive(&with_risk, "RETEST_WITHIN_24H");

    let breastfeeding = Case::new(38, 4320)
        .jaundice()
        .breastfeeding()
        .serum(4320, 260)
        .eval();
    assert_active(&breastfeeding, "RETEST_WITHIN_18H");

    let no_risk = Case::new(38, 4320).jaundice().serum(4320, 260).eval();
    assert_inactive(&no_risk, "RETEST_WITHIN_18H");
    assert_active(&no_risk, "RETEST_WITHIN_24H");
}

#[test]
fn stop_phototherapy_at_exactly_50_below_not_49() {
    // 38 weeks at 72 h: photo 300. Baseline serum before start avoids
    // response-incomplete noise in the priority ranking assertion.
    let at_50 = Case::new(38, 4320)
        .jaundice()
        .serum(4000, 320)
        .serum(4320, 250)
        .phototherapy(4010)
        .eval();
    assert_active(&at_50, "STOP_PHOTOTHERAPY");

    let at_49 = Case::new(38, 4320)
        .jaundice()
        .serum(4000, 320)
        .serum(4320, 251)
        .phototherapy(4010)
        .eval();
    assert_inactive(&at_49, "STOP_PHOTOTHERAPY");
}

#[test]
fn exchange_proximity_and_reduce_intensity_around_50_below_exchange() {
    // 36 weeks (below the kernicterus gestation), exchange plateau 360 from
    // 72 h. Serum values fall from baseline so the failure-to-respond branch
    // stays inactive and only the proximity boundary is exercised.
    // 49 below (311): proximity trigger fires, reduce does not.
    let at_49 = Case::new(36, 4320)
        .jaundice()
        .serum(4000, 320)
        .serum(4320, 311)
        .intensified(4010)
        .eval();
    assert_active(&at_49, "CONSIDER_INTENSIFIED_PHOTOTHERAPY");
    assert_inactive(&at_49, "REDUCE_PHOTOTHERAPY_INTENSITY");
    assert_eq!(primary(&at_49), "CONSIDER_INTENSIFIED_PHOTOTHERAPY");

    // Exactly 50 below (310): both boundary rules apply by specification;
    // the urgent escalation is primary and the de-escalation is suppressed.
    let at_50 = Case::new(36, 4320)
        .jaundice()
        .serum(4000, 320)
        .serum(4320, 310)
        .intensified(4010)
        .eval();
    assert_active(&at_50, "CONSIDER_INTENSIFIED_PHOTOTHERAPY");
    assert_active(&at_50, "REDUCE_PHOTOTHERAPY_INTENSITY");
    assert_eq!(primary(&at_50), "CONSIDER_INTENSIFIED_PHOTOTHERAPY");
    assert!(at_50
        .suppressed_rules
        .contains(&"REDUCE_PHOTOTHERAPY_INTENSITY".to_string()));

    // 51 below (309): reduce fires, proximity does not.
    let at_51 = Case::new(36, 4320)
        .jaundice()
        .serum(4000, 320)
        .serum(4320, 309)
        .intensified(4010)
        .eval();
    assert_inactive(&at_51, "CONSIDER_INTENSIFIED_PHOTOTHERAPY");
    assert_active(&at_51, "REDUCE_PHOTOTHERAPY_INTENSITY");
    assert_eq!(primary(&at_51), "REDUCE_PHOTOTHERAPY_INTENSITY");
}

#[test]
fn exchange_proximity_requires_72_hours_of_age() {
    // 36 weeks at 71 h 59 min (4319): serum falling and within 50 of the
    // exchange line must NOT trigger the proximity branch before 72 h
    // (CLIN-038).
    let before_72h = Case::new(36, 4319)
        .jaundice()
        .serum(4000, 330)
        .serum(4319, 320)
        .intensified(4010)
        .eval();
    assert_inactive(&before_72h, "CONSIDER_INTENSIFIED_PHOTOTHERAPY");

    let at_72h = Case::new(36, 4320)
        .jaundice()
        .serum(4000, 330)
        .serum(4320, 320)
        .intensified(4010)
        .eval();
    assert_active(&at_72h, "CONSIDER_INTENSIFIED_PHOTOTHERAPY");
}

// ---------------------------------------------------------------------------
// Rapid-rise boundary: exactly 8.5 vs strictly greater (CLIN-035)
// ---------------------------------------------------------------------------

#[test]
fn rapid_rise_below_at_and_above_8_5() {
    // Two serum values 120 minutes apart. +16 → 8.0/h, +17 → 8.5/h,
    // +18 → 9.0/h.
    let below = Case::new(38, 2000)
        .jaundice()
        .serum(1800, 100)
        .serum(1920, 116)
        .eval();
    assert_inactive(&below, "AT_RAPID_RISE_BOUNDARY");
    assert_inactive(&below, "INCREASED_KERNICTERUS_RISK");

    let at = Case::new(38, 2000)
        .jaundice()
        .serum(1800, 100)
        .serum(1920, 117)
        .eval();
    assert_active(&at, "AT_RAPID_RISE_BOUNDARY");
    assert_inactive(&at, "INCREASED_KERNICTERUS_RISK");
    assert_inactive(&at, "CONSIDER_INTENSIFIED_PHOTOTHERAPY");

    let above = Case::new(38, 2000)
        .jaundice()
        .serum(1800, 100)
        .serum(1920, 118)
        .eval();
    assert_inactive(&above, "AT_RAPID_RISE_BOUNDARY");
    assert_active(&above, "INCREASED_KERNICTERUS_RISK");
    assert_active(&above, "CONSIDER_INTENSIFIED_PHOTOTHERAPY");
    assert_eq!(primary(&above), "INCREASED_KERNICTERUS_RISK");
    assert_eq!(above.primary_action.priority, Priority::Immediate);
}

#[test]
fn rapid_rise_needs_two_serum_measurements() {
    // Same numbers with a transcutaneous newer value: no confirmation
    // (CLIN-036), no boundary code.
    let tcb_pair = Case::new(38, 2000)
        .jaundice()
        .serum(1800, 100)
        .tcb(1920, 118)
        .eval();
    assert_inactive(&tcb_pair, "INCREASED_KERNICTERUS_RISK");
    assert_inactive(&tcb_pair, "AT_RAPID_RISE_BOUNDARY");
    assert!(!tcb_pair.trend.as_ref().unwrap().reliable_for_rapid_rise);
}

#[test]
fn rapid_rise_applies_at_one_minute_interval() {
    // NICE sets no minimum interval and the engine must not invent one:
    // +9 umol/L in 1 minute is 540/h, rapid.
    let outcome = Case::new(38, 2000)
        .jaundice()
        .serum(1919, 100)
        .serum(1920, 109)
        .eval();
    assert_active(&outcome, "INCREASED_KERNICTERUS_RISK");
    assert_eq!(outcome.trend.as_ref().unwrap().interval_minutes, 1);
}

// ---------------------------------------------------------------------------
// Kernicterus bilirubin boundary: 339 / 340 / 341 (strictly greater)
// ---------------------------------------------------------------------------

#[test]
fn kernicterus_bilirubin_boundary() {
    // 38 weeks at 96 h: photo 350, exchange 450 — all three values sit below
    // the photothrapy line so only the kernicterus rule separates them.
    let at_339 = Case::new(38, 5760).jaundice().serum(5760, 339).eval();
    assert_inactive(&at_339, "INCREASED_KERNICTERUS_RISK");

    let at_340 = Case::new(38, 5760).jaundice().serum(5760, 340).eval();
    assert_inactive(&at_340, "INCREASED_KERNICTERUS_RISK");

    let at_341 = Case::new(38, 5760).jaundice().serum(5760, 341).eval();
    assert_active(&at_341, "INCREASED_KERNICTERUS_RISK");
    assert_eq!(primary(&at_341), "INCREASED_KERNICTERUS_RISK");
}

// ---------------------------------------------------------------------------
// Conjugated bilirubin boundary: 24 / 25 / 26
// ---------------------------------------------------------------------------

#[test]
fn conjugated_bilirubin_boundary() {
    let base = || Case::new(38, 21_000).jaundice();

    let at_24 = base().conjugated(24).eval();
    assert_inactive(&at_24, "EXPERT_LIVER_ADVICE");
    assert_inactive(&at_24, "AT_CONJUGATED_BOUNDARY_REVIEW");

    let at_25 = base().conjugated(25).eval();
    assert_inactive(&at_25, "EXPERT_LIVER_ADVICE");
    assert_active(&at_25, "AT_CONJUGATED_BOUNDARY_REVIEW");

    let at_26 = base().conjugated(26).eval();
    assert_active(&at_26, "EXPERT_LIVER_ADVICE");
    assert_inactive(&at_26, "AT_CONJUGATED_BOUNDARY_REVIEW");
    assert_eq!(primary(&at_26), "EXPERT_LIVER_ADVICE");
    assert_eq!(at_26.primary_action.priority, Priority::Immediate);
}

// ---------------------------------------------------------------------------
// Prolonged jaundice: strictly beyond 14 or 21 days
// ---------------------------------------------------------------------------

#[test]
fn prolonged_jaundice_term_boundary_at_14_days() {
    let at_limit = Case::new(38, 20_160).jaundice().eval();
    assert_inactive(&at_limit, "PROLONGED_JAUNDICE_ASSESSMENT");

    let beyond = Case::new(38, 20_161).jaundice().eval();
    assert_active(&beyond, "PROLONGED_JAUNDICE_ASSESSMENT");
    assert_eq!(primary(&beyond), "PROLONGED_JAUNDICE_ASSESSMENT");
    assert_eq!(beyond.primary_action.priority, Priority::Urgent);
}

#[test]
fn prolonged_jaundice_preterm_boundary_at_21_days() {
    // Below 37 weeks the 14-day rule does not apply.
    let at_15_days = Case::new(36, 21_600).jaundice().eval();
    assert_inactive(&at_15_days, "PROLONGED_JAUNDICE_ASSESSMENT");

    let at_limit = Case::new(36, 30_240).jaundice().eval();
    assert_inactive(&at_limit, "PROLONGED_JAUNDICE_ASSESSMENT");

    let beyond = Case::new(36, 30_241).jaundice().eval();
    assert_active(&beyond, "PROLONGED_JAUNDICE_ASSESSMENT");
}

#[test]
fn gestation_37_uses_the_term_prolonged_rule() {
    // 37 weeks belongs to the >= 37 population for prolonged jaundice.
    let beyond = Case::new(37, 20_161).jaundice().eval();
    assert_active(&beyond, "PROLONGED_JAUNDICE_ASSESSMENT");
}

// ---------------------------------------------------------------------------
// Treatment-line equality (CLIN-024): at is neither below nor above
// ---------------------------------------------------------------------------

#[test]
fn serum_exactly_at_phototherapy_line() {
    // 38 weeks at 48 h: photo line exactly 250.
    let outcome = Case::new(38, 2880).jaundice().serum(2880, 250).eval();
    assert_active(&outcome, "AT_TREATMENT_LINE_REVIEW");
    assert_inactive(&outcome, "START_PHOTOTHERAPY");
    assert_inactive(&outcome, "RETEST_WITHIN_24H");
    assert_eq!(primary(&outcome), "AT_TREATMENT_LINE_REVIEW");
    // Serum equality does not demand serum confirmation.
    assert!(!outcome.primary_action.requires_serum_confirmation);
}

#[test]
fn tcb_exactly_at_phototherapy_line_requires_serum_confirmation() {
    let outcome = Case::new(38, 2880).jaundice().tcb(2880, 250).eval();
    assert_active(&outcome, "AT_TREATMENT_LINE_REVIEW");
    assert_active(&outcome, "SERUM_CONFIRM_TREATMENT_LINE");
    let at_line = outcome
        .recommendations
        .iter()
        .find(|r| r.code == "AT_TREATMENT_LINE_REVIEW")
        .unwrap();
    assert!(at_line.requires_serum_confirmation);
}

#[test]
fn serum_exactly_at_exchange_line() {
    // 38 weeks with a measurement at exactly 24 h, where the exchange line
    // is exactly 300 (the assessment itself is a minute later so the
    // more-than-24-hours recognition pathway applies).
    let outcome = Case::new(38, 1441).jaundice().serum(1440, 300).eval();
    assert_active(&outcome, "AT_EXCHANGE_LINE_EMERGENCY_REVIEW");
    assert_inactive(&outcome, "EXCHANGE_TRANSFUSION_ESCALATION");
    assert_eq!(primary(&outcome), "AT_EXCHANGE_LINE_EMERGENCY_REVIEW");
    assert_eq!(outcome.primary_action.priority, Priority::Immediate);
    // One above escalates.
    let above = Case::new(38, 1441).jaundice().serum(1440, 301).eval();
    assert_eq!(primary(&above), "EXCHANGE_TRANSFUSION_ESCALATION");
    assert_eq!(above.primary_action.priority, Priority::Emergency);
}

// ---------------------------------------------------------------------------
// Phototherapy monitoring boundary: 359 / 360 / 361 minutes after start
// ---------------------------------------------------------------------------

#[test]
fn phototherapy_monitoring_overdue_boundary() {
    let case = |elapsed: u32| {
        let start = 3000;
        Case::new(38, start + elapsed)
            .jaundice()
            .serum(2990, 280)
            .phototherapy(start)
            .eval()
    };
    let at_359 = case(359);
    assert_active(&at_359, "PHOTOTHERAPY_CHECK_4_6H");
    assert_inactive(&at_359, "PHOTOTHERAPY_CHECK_OVERDUE");

    let at_360 = case(360);
    assert_active(&at_360, "PHOTOTHERAPY_CHECK_4_6H");
    assert_inactive(&at_360, "PHOTOTHERAPY_CHECK_OVERDUE");

    let at_361 = case(361);
    assert_inactive(&at_361, "PHOTOTHERAPY_CHECK_4_6H");
    assert_active(&at_361, "PHOTOTHERAPY_CHECK_OVERDUE");
    assert_eq!(at_361.primary_action.priority, Priority::Urgent);
    assert!(activated(&at_361).contains(&"PHOTOTHERAPY_RESPONSE_INCOMPLETE"));
}
