//! Threshold oracle tests (TEST-006, TEST-007, TEST-008) and the exhaustive
//! domain sweep from spec 09. Golden values are derived independently from
//! the normative formulas and control points in spec 02 and must match the
//! engine with zero divergence.

use clinical_core::rational::Rational;
use clinical_core::thresholds::{
    assess_against_line, treatment_thresholds, TERM_EXCHANGE_POINTS, TERM_PHOTOTHERAPY_POINTS,
};
use clinical_core::types::{AgeMinutes, BilirubinUmolL, GestationalWeeks, ThresholdRelation};

fn thresholds(g: u8, m: u32) -> Option<(Rational, Rational)> {
    treatment_thresholds(
        GestationalWeeks::new(g).unwrap(),
        AgeMinutes::new(m).unwrap(),
    )
    .unwrap()
    .map(|p| (p.phototherapy, p.exchange))
}

fn rational(num: i64, den: i64) -> Rational {
    Rational::new(num, den).unwrap()
}

// ---------------------------------------------------------------------------
// Preterm oracle (23–37 weeks): CLIN-018, CLIN-019, CLIN-020
// ---------------------------------------------------------------------------

#[test]
fn preterm_birth_values_are_40_and_80_for_every_gestation() {
    for g in 23..=37u8 {
        let (photo, exch) = thresholds(g, 0).unwrap();
        assert_eq!(photo, Rational::from_int(40), "photo at birth, {g} weeks");
        assert_eq!(exch, Rational::from_int(80), "exchange at birth, {g} weeks");
    }
}

#[test]
fn preterm_72h_and_14d_plateau_values() {
    for g in 23..=37u8 {
        let p72 = i64::from(g) * 10 - 100;
        let e72 = i64::from(g) * 10;
        for m in [4320, 10_000, 20_160] {
            let (photo, exch) = thresholds(g, m).unwrap();
            assert_eq!(
                photo,
                Rational::from_int(p72),
                "photo plateau {g} weeks at {m}"
            );
            assert_eq!(
                exch,
                Rational::from_int(e72),
                "exchange plateau {g} weeks at {m}"
            );
        }
    }
}

#[test]
fn preterm_one_minute_around_72h() {
    for g in 23..=37u8 {
        let p72 = i64::from(g) * 10 - 100;
        let e72 = i64::from(g) * 10;
        // One minute before the plateau the line is still interpolating.
        let (photo, exch) = thresholds(g, 4319).unwrap();
        let expected_photo = Rational::from_int(40)
            .checked_add(&rational((p72 - 40) * 4319, 4320))
            .unwrap();
        let expected_exch = Rational::from_int(80)
            .checked_add(&rational((e72 - 80) * 4319, 4320))
            .unwrap();
        assert_eq!(photo, expected_photo, "photo at 4319, {g} weeks");
        assert_eq!(exch, expected_exch, "exchange at 4319, {g} weeks");
        // One minute after 72 h the plateau holds exactly.
        let (photo, exch) = thresholds(g, 4321).unwrap();
        assert_eq!(photo, Rational::from_int(p72));
        assert_eq!(exch, Rational::from_int(e72));
    }
}

#[test]
fn preterm_midpoint_is_exact_mean_of_birth_and_72h() {
    for g in 23..=37u8 {
        let p72 = i64::from(g) * 10 - 100;
        let e72 = i64::from(g) * 10;
        let (photo, exch) = thresholds(g, 2160).unwrap();
        assert_eq!(photo, rational(40 + p72, 2), "photo midpoint {g} weeks");
        assert_eq!(exch, rational(80 + e72, 2), "exchange midpoint {g} weeks");
    }
}

#[test]
fn preterm_non_divisible_age_uses_exact_fraction() {
    // 30 weeks at 1000 minutes: P = 40 + (200-40)*1000/4320 = 40 + 1000/27,
    // an exact non-terminating decimal that must not be rounded (TEST-007).
    let (photo, _) = thresholds(30, 1000).unwrap();
    let expected = Rational::from_int(40)
        .checked_add(&rational(1000, 27))
        .unwrap();
    assert_eq!(photo, expected);
    assert_eq!(photo, rational(40 * 27 + 1000, 27));
}

// ---------------------------------------------------------------------------
// Term oracle (38–42 weeks): CLIN-021, CLIN-022, CLIN-023
// ---------------------------------------------------------------------------

#[test]
fn term_every_control_point_is_exact_for_every_gestation() {
    for g in 38..=42u8 {
        for &(m, v) in TERM_PHOTOTHERAPY_POINTS {
            let (photo, _) = thresholds(g, m).unwrap();
            assert_eq!(
                photo,
                Rational::from_int(v),
                "photo point {m} min, {g} weeks"
            );
        }
        for &(m, v) in TERM_EXCHANGE_POINTS {
            let (_, exch) = thresholds(g, m).unwrap();
            assert_eq!(
                exch,
                Rational::from_int(v),
                "exchange point {m} min, {g} weeks"
            );
        }
    }
}

#[test]
fn term_midpoints_interpolate_exactly() {
    for g in 38..=42u8 {
        for pair in TERM_PHOTOTHERAPY_POINTS.windows(2) {
            let [(m1, v1), (m2, v2)] = [pair[0], pair[1]];
            let mid = m1 + (m2 - m1) / 2;
            let (photo, _) = thresholds(g, mid).unwrap();
            assert_eq!(
                photo,
                rational(v1 + v2, 2),
                "photo midpoint {m1}-{m2}, {g} weeks"
            );
        }
        for pair in TERM_EXCHANGE_POINTS.windows(2) {
            let [(m1, v1), (m2, v2)] = [pair[0], pair[1]];
            let mid = m1 + (m2 - m1) / 2;
            let (_, exch) = thresholds(g, mid).unwrap();
            assert_eq!(
                exch,
                rational(v1 + v2, 2),
                "exchange midpoint {m1}-{m2}, {g} weeks"
            );
        }
    }
}

#[test]
fn term_one_minute_around_every_control_point() {
    for g in 38..=42u8 {
        for pair in TERM_PHOTOTHERAPY_POINTS.windows(2) {
            let [(m1, v1), (m2, v2)] = [pair[0], pair[1]];
            let span = i64::from(m2 - m1);
            let (after, _) = thresholds(g, m1 + 1).unwrap();
            assert_eq!(
                after,
                Rational::from_int(v1)
                    .checked_add(&rational(v2 - v1, span))
                    .unwrap(),
                "photo one minute after {m1}, {g} weeks"
            );
            let (before, _) = thresholds(g, m2 - 1).unwrap();
            assert_eq!(
                before,
                Rational::from_int(v1)
                    .checked_add(&rational((v2 - v1) * (span - 1), span))
                    .unwrap(),
                "photo one minute before {m2}, {g} weeks"
            );
        }
    }
}

#[test]
fn term_96h_and_14d_plateaus() {
    for g in 38..=42u8 {
        for m in [5760, 10_000, 20_160] {
            let (photo, exch) = thresholds(g, m).unwrap();
            assert_eq!(
                photo,
                Rational::from_int(350),
                "photo plateau at {m}, {g} weeks"
            );
            assert_eq!(
                exch,
                Rational::from_int(450),
                "exchange plateau at {m}, {g} weeks"
            );
        }
        // Exchange plateaus earlier, from 42 hours.
        for m in [2520, 3000, 5000] {
            let (_, exch) = thresholds(g, m).unwrap();
            assert_eq!(
                exch,
                Rational::from_int(450),
                "exchange plateau at {m}, {g} weeks"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Relation classification around integral lines (CLIN-024, TEST-006)
// ---------------------------------------------------------------------------

#[test]
fn integral_lines_classify_below_at_above_without_rounding() {
    // Term 38 weeks at 24 h: photo line exactly 200, exchange exactly 300.
    let (photo, exch) = thresholds(38, 1440).unwrap();
    for (line, exact) in [(photo, 200u16), (exch, 300u16)] {
        let below = assess_against_line(BilirubinUmolL::new(exact - 1).unwrap(), &line).unwrap();
        let at = assess_against_line(BilirubinUmolL::new(exact).unwrap(), &line).unwrap();
        let above = assess_against_line(BilirubinUmolL::new(exact + 1).unwrap(), &line).unwrap();
        assert_eq!(below.relation, ThresholdRelation::Below);
        assert_eq!(at.relation, ThresholdRelation::At);
        assert_eq!(above.relation, ThresholdRelation::Above);
        assert_eq!(at.distance, Rational::from_int(0));
        assert_eq!(below.distance, Rational::from_int(-1));
        assert_eq!(above.distance, Rational::from_int(1));
    }
}

#[test]
fn non_integral_line_has_no_at_state_for_integer_input() {
    // 30 weeks at 1000 minutes the photo line is 2080/27, strictly between
    // 77 and 78, so integer inputs can only be below or above.
    let (photo, _) = thresholds(30, 1000).unwrap();
    let below = assess_against_line(BilirubinUmolL::new(77).unwrap(), &photo).unwrap();
    let above = assess_against_line(BilirubinUmolL::new(78).unwrap(), &photo).unwrap();
    assert_eq!(below.relation, ThresholdRelation::Below);
    assert_eq!(above.relation, ThresholdRelation::Above);
}

// ---------------------------------------------------------------------------
// Scope limit (CLIN-027, TEST-008)
// ---------------------------------------------------------------------------

#[test]
fn no_threshold_is_returned_after_minute_20160() {
    for g in [23u8, 30, 37, 38, 42] {
        assert!(
            thresholds(g, 20_160).is_some(),
            "{g} weeks at 20160 must have a line"
        );
        for m in [20_161, 25_000, 40_319] {
            assert!(
                thresholds(g, m).is_none(),
                "{g} weeks at {m} must have no line"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Exhaustive sweep: every gestation, every minute 0–20,160 (spec 09)
// ---------------------------------------------------------------------------

/// Independent half-away-from-zero rounding to tenths using i128, to check
/// the display conversion against a second implementation.
fn independent_display_tenths(r: &Rational) -> i64 {
    let num = i128::from(r.numerator()) * 10;
    let den = i128::from(r.denominator());
    let doubled = num * 2;
    let adjust = if num >= 0 { den } else { -den };
    ((doubled + adjust) / (den * 2)) as i64
}

#[test]
fn exhaustive_domain_sweep() {
    for g in 23..=42u8 {
        let mut previous: Option<(Rational, Rational)> = None;
        for m in 0..=20_160u32 {
            let (photo, exch) =
                thresholds(g, m).unwrap_or_else(|| panic!("line must exist at {m} min, {g} weeks"));

            // Phototherapy never exceeds exchange.
            assert_ne!(
                photo.cmp_exact(&exch).unwrap(),
                std::cmp::Ordering::Greater,
                "photo > exchange at {m} min, {g} weeks"
            );

            // Both lines are non-decreasing with age.
            if let Some((prev_photo, prev_exch)) = previous {
                assert_ne!(
                    prev_photo.cmp_exact(&photo).unwrap(),
                    std::cmp::Ordering::Greater,
                    "photo decreased at {m} min, {g} weeks"
                );
                assert_ne!(
                    prev_exch.cmp_exact(&exch).unwrap(),
                    std::cmp::Ordering::Greater,
                    "exchange decreased at {m} min, {g} weeks"
                );
            }
            previous = Some((photo, exch));

            // Plateaus hold exactly.
            if g < 38 && m >= 4320 {
                assert_eq!(photo, Rational::from_int(i64::from(g) * 10 - 100));
                assert_eq!(exch, Rational::from_int(i64::from(g) * 10));
            }
            if g >= 38 {
                if m >= 5760 {
                    assert_eq!(photo, Rational::from_int(350));
                }
                if m >= 2520 {
                    assert_eq!(exch, Rational::from_int(450));
                }
            }

            // Display rounding matches the independent implementation.
            assert_eq!(
                photo.display_tenths().unwrap(),
                independent_display_tenths(&photo)
            );
            assert_eq!(
                exch.display_tenths().unwrap(),
                independent_display_tenths(&exch)
            );
        }
    }
}

#[test]
fn higher_preterm_gestation_never_has_a_lower_line() {
    for g in 24..=37u8 {
        for m in (0..=20_160u32).step_by(7) {
            let (photo_lo, exch_lo) = thresholds(g - 1, m).unwrap();
            let (photo_hi, exch_hi) = thresholds(g, m).unwrap();
            assert_ne!(
                photo_hi.cmp_exact(&photo_lo).unwrap(),
                std::cmp::Ordering::Less,
                "photo line dropped from {} to {} weeks at {m} min",
                g - 1,
                g
            );
            assert_ne!(
                exch_hi.cmp_exact(&exch_lo).unwrap(),
                std::cmp::Ordering::Less,
                "exchange line dropped from {} to {} weeks at {m} min",
                g - 1,
                g
            );
        }
    }
}
