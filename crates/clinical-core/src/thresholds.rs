//! Exact CG98 treatment-threshold calculation (spec 02, CLIN-018–CLIN-027).
//!
//! Thresholds are exact rationals; a rounded display value never drives a
//! decision. No threshold exists after minute 20,160 (CLIN-027, TEST-008).

use core::cmp::Ordering;

use crate::error::SafetyError;
use crate::rational::Rational;
use crate::types::{
    AgeMinutes, BilirubinUmolL, GestationalWeeks, ThresholdRelation, PRETERM_PLATEAU_MINUTES,
    TREATMENT_LINE_MAX_AGE_MINUTES,
};

/// Term (>= 38 weeks) phototherapy control points in (minutes, umol/L),
/// transcribed from the normative rule pack `nice-cg98-2023-10-31.1`.
pub const TERM_PHOTOTHERAPY_POINTS: &[(u32, i64)] = &[
    (0, 100),
    (360, 125),
    (720, 150),
    (1080, 175),
    (1440, 200),
    (1800, 212),
    (2160, 225),
    (2520, 237),
    (2880, 250),
    (3240, 262),
    (3600, 275),
    (3960, 287),
    (4320, 300),
    (4680, 312),
    (5040, 325),
    (5400, 337),
    (5760, 350),
    (20160, 350),
];

/// Term (>= 38 weeks) exchange-transfusion control points.
pub const TERM_EXCHANGE_POINTS: &[(u32, i64)] = &[
    (0, 100),
    (360, 150),
    (720, 200),
    (1080, 250),
    (1440, 300),
    (1800, 350),
    (2160, 400),
    (2520, 450),
    (20160, 450),
];

/// Both treatment lines at one age for one gestation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThresholdPair {
    pub phototherapy: Rational,
    pub exchange: Rational,
}

/// Exact treatment lines, or `None` past 336 hours (CLIN-012, CLIN-027).
pub fn treatment_thresholds(
    gestation: GestationalWeeks,
    age: AgeMinutes,
) -> Result<Option<ThresholdPair>, SafetyError> {
    if !age.within_treatment_line_range() {
        return Ok(None);
    }
    let pair = if gestation.is_preterm() {
        ThresholdPair {
            phototherapy: preterm_line(gestation, age, 40)?,
            exchange: preterm_line(gestation, age, 80)?,
        }
    } else {
        ThresholdPair {
            phototherapy: interpolate(TERM_PHOTOTHERAPY_POINTS, age)?,
            exchange: interpolate(TERM_EXCHANGE_POINTS, age)?,
        }
    };
    Ok(Some(pair))
}

/// Preterm line (23–37 weeks, CLIN-018–CLIN-020). `birth_value` is 40 for
/// phototherapy and 80 for exchange; the 72-hour value is `10g − 100` and
/// `10g` respectively, i.e. `birth_value` maps 40→(10g−100) and 80→10g.
fn preterm_line(
    gestation: GestationalWeeks,
    age: AgeMinutes,
    birth_value: i64,
) -> Result<Rational, SafetyError> {
    let g = i64::from(gestation.value());
    let value_72h = if birth_value == 40 {
        10 * g - 100
    } else {
        10 * g
    };
    let m = i64::from(age.value());
    if age.value() >= PRETERM_PLATEAU_MINUTES {
        return Ok(Rational::from_int(value_72h));
    }
    // birth_value + (value_72h - birth_value) * m / 4320, without
    // intermediate rounding (CLIN-018).
    let rise = Rational::from_int(value_72h - birth_value)
        .checked_mul(&Rational::new(m, i64::from(PRETERM_PLATEAU_MINUTES))?)?;
    Rational::from_int(birth_value).checked_add(&rise)
}

/// Straight-line interpolation between control points using elapsed minutes
/// and exact rational arithmetic (CLIN-022).
fn interpolate(points: &[(u32, i64)], age: AgeMinutes) -> Result<Rational, SafetyError> {
    let m = age.value();
    debug_assert!(m <= TREATMENT_LINE_MAX_AGE_MINUTES);
    let mut previous = points[0];
    for &(px, pv) in points {
        match m.cmp(&px) {
            Ordering::Equal => return Ok(Rational::from_int(pv)),
            Ordering::Less => {
                let (x1, v1) = (i64::from(previous.0), previous.1);
                let (x2, v2) = (i64::from(px), pv);
                // v1 + (v2 - v1) * (m - x1) / (x2 - x1)
                let rise = Rational::from_int(v2 - v1)
                    .checked_mul(&Rational::new(i64::from(m) - x1, x2 - x1)?)?;
                return Rational::from_int(v1).checked_add(&rise);
            }
            Ordering::Greater => previous = (px, pv),
        }
    }
    // Unreachable while the tables end at 20,160 and the age is range-checked;
    // fail safe rather than extrapolate if either ever changes.
    Err(SafetyError::ArithmeticOverflow)
}

/// Exact three-state relation of a measurement to a line (CLIN-024) plus the
/// unrounded signed distance, measurement minus threshold (CLIN-026).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineAssessment {
    pub threshold: Rational,
    pub relation: ThresholdRelation,
    pub distance: Rational,
}

pub fn assess_against_line(
    value: BilirubinUmolL,
    line: &Rational,
) -> Result<LineAssessment, SafetyError> {
    let value = Rational::from_int(i64::from(value.value()));
    let relation = match value.cmp_exact(line)? {
        Ordering::Less => ThresholdRelation::Below,
        Ordering::Equal => ThresholdRelation::At,
        Ordering::Greater => ThresholdRelation::Above,
    };
    Ok(LineAssessment {
        threshold: *line,
        relation,
        distance: value.checked_sub(line)?,
    })
}
