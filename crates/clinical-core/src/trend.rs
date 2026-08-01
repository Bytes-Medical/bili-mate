//! Serial-measurement trend (spec 02, CLIN-035–CLIN-037).

use core::cmp::Ordering;

use crate::error::SafetyError;
use crate::input::Measurement;
use crate::rational::Rational;
use crate::types::{MeasurementMethod, ThresholdRelation, TrendDirection};

/// The serum rapid-rise comparison value, exactly 8.5 umol/L/hour.
pub const RAPID_RISE_NUM: i64 = 17;
pub const RAPID_RISE_DEN: i64 = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trend {
    pub older_measurement_id: String,
    pub newer_measurement_id: String,
    pub interval_minutes: u32,
    /// Exact rate in umol/L per hour; display rounding happens at the
    /// serialisation boundary only (CLIN-035).
    pub rate: Rational,
    pub direction: TrendDirection,
    /// Both measurements are serum at distinct validated ages; only such a
    /// pair can confirm the rapid-rise rule (CLIN-036).
    pub reliable_for_rapid_rise: bool,
    /// Exact relation of the rate to 8.5, or `NotAvailable` when the pair is
    /// not reliable for the rapid-rise rule.
    pub rapid_rise_relation: ThresholdRelation,
}

impl Trend {
    /// A confirmed rapid rise: reliable serum pair with a positive rate
    /// strictly greater than 8.5 umol/L/hour.
    pub fn rapid_rise_confirmed(&self) -> bool {
        self.reliable_for_rapid_rise && self.rapid_rise_relation == ThresholdRelation::Above
    }

    pub fn stable_or_falling(&self) -> bool {
        matches!(
            self.direction,
            TrendDirection::Stable | TrendDirection::Falling
        )
    }
}

/// Display trend from the two most recent measurements. `measurements` must
/// already be validated and sorted by age; ages are unique, so the last two
/// entries are the pair. NICE specifies no minimum interval and the engine
/// does not invent one (spec 02).
pub fn calculate_trend(measurements: &[Measurement]) -> Result<Option<Trend>, SafetyError> {
    let [.., older, newer] = measurements else {
        return Ok(None);
    };

    let interval = newer.age_minutes.value() - older.age_minutes.value();
    let delta = i64::from(newer.total_bilirubin_umol_l.value())
        - i64::from(older.total_bilirubin_umol_l.value());
    let rate = Rational::from_int(delta).checked_mul(&Rational::new(60, i64::from(interval))?)?;

    let direction = match delta.cmp(&0) {
        Ordering::Greater => TrendDirection::Rising,
        Ordering::Equal => TrendDirection::Stable,
        Ordering::Less => TrendDirection::Falling,
    };

    let reliable =
        older.method == MeasurementMethod::Serum && newer.method == MeasurementMethod::Serum;
    let rapid_rise_relation = if reliable {
        match rate.cmp_exact(&Rational::new(RAPID_RISE_NUM, RAPID_RISE_DEN)?)? {
            Ordering::Less => ThresholdRelation::Below,
            Ordering::Equal => ThresholdRelation::At,
            Ordering::Greater => ThresholdRelation::Above,
        }
    } else {
        ThresholdRelation::NotAvailable
    };

    Ok(Some(Trend {
        older_measurement_id: older.id.clone(),
        newer_measurement_id: newer.id.clone(),
        interval_minutes: interval,
        rate,
        direction,
        reliable_for_rapid_rise: reliable,
        rapid_rise_relation,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AgeMinutes, BilirubinUmolL};

    fn m(id: &str, age: u32, value: u16, method: MeasurementMethod) -> Measurement {
        Measurement {
            id: id.into(),
            age_minutes: AgeMinutes::new(age).unwrap(),
            total_bilirubin_umol_l: BilirubinUmolL::new(value).unwrap(),
            method,
        }
    }

    #[test]
    fn no_trend_for_fewer_than_two() {
        assert_eq!(calculate_trend(&[]).unwrap(), None);
        assert_eq!(
            calculate_trend(&[m("a", 100, 100, MeasurementMethod::Serum)]).unwrap(),
            None
        );
    }

    #[test]
    fn exact_rate_at_boundary_is_at_not_above() {
        // 8.5 umol/L/hour exactly: +17 over 120 minutes.
        let trend = calculate_trend(&[
            m("a", 1000, 100, MeasurementMethod::Serum),
            m("b", 1120, 117, MeasurementMethod::Serum),
        ])
        .unwrap()
        .unwrap();
        assert_eq!(trend.rapid_rise_relation, ThresholdRelation::At);
        assert!(!trend.rapid_rise_confirmed());
        assert_eq!(trend.direction, TrendDirection::Rising);
    }

    #[test]
    fn strictly_greater_is_rapid() {
        let trend = calculate_trend(&[
            m("a", 1000, 100, MeasurementMethod::Serum),
            m("b", 1120, 118, MeasurementMethod::Serum),
        ])
        .unwrap()
        .unwrap();
        assert_eq!(trend.rapid_rise_relation, ThresholdRelation::Above);
        assert!(trend.rapid_rise_confirmed());
    }

    #[test]
    fn tcb_pair_cannot_confirm_rapid_rise() {
        let trend = calculate_trend(&[
            m("a", 1000, 100, MeasurementMethod::Serum),
            m("b", 1120, 200, MeasurementMethod::Transcutaneous),
        ])
        .unwrap()
        .unwrap();
        assert!(!trend.reliable_for_rapid_rise);
        assert_eq!(trend.rapid_rise_relation, ThresholdRelation::NotAvailable);
        assert!(!trend.rapid_rise_confirmed());
    }

    #[test]
    fn one_minute_interval_is_valid() {
        let trend = calculate_trend(&[
            m("a", 1000, 100, MeasurementMethod::Serum),
            m("b", 1001, 101, MeasurementMethod::Serum),
        ])
        .unwrap()
        .unwrap();
        assert_eq!(trend.interval_minutes, 1);
        // 1 umol/L over 1 minute = 60/hour, rapid.
        assert!(trend.rapid_rise_confirmed());
    }

    #[test]
    fn equal_values_are_stable() {
        let trend = calculate_trend(&[
            m("a", 1000, 150, MeasurementMethod::Serum),
            m("b", 1200, 150, MeasurementMethod::Serum),
        ])
        .unwrap()
        .unwrap();
        assert_eq!(trend.direction, TrendDirection::Stable);
        assert!(trend.stable_or_falling());
    }
}
