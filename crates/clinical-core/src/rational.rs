//! Exact rational arithmetic for clinical comparisons (PRD-031, CLIN-018,
//! CLIN-022, CLIN-035).
//!
//! Every value is a reduced fraction of `i64` with a strictly positive
//! denominator. All operations use checked arithmetic; overflow is a typed
//! safety failure and never a wrapped or saturated value (TEST-012).
//!
//! Magnitude analysis for the supported clinical domain: numerators are
//! bounded by `40 × 4320 + 320 × 20160 < 6.7 × 10^6` and denominators by
//! `4320`, so cross-multiplication products stay below `3 × 10^10` — far
//! inside `i64`. Checked operations remain mandatory so any future domain
//! change fails safe instead of silently wrapping.

use core::cmp::Ordering;

use crate::error::SafetyError;

/// A reduced exact fraction with `den > 0`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rational {
    num: i64,
    den: i64,
}

fn gcd(mut a: i64, mut b: i64) -> i64 {
    a = a.abs();
    b = b.abs();
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

impl Rational {
    pub fn new(num: i64, den: i64) -> Result<Self, SafetyError> {
        if den == 0 {
            return Err(SafetyError::ZeroDenominator);
        }
        // i64::MIN.abs() would overflow; the clinical domain never reaches it,
        // but reject rather than panic if it ever appears.
        if num == i64::MIN || den == i64::MIN {
            return Err(SafetyError::ArithmeticOverflow);
        }
        let sign = if den < 0 { -1 } else { 1 };
        let (num, den) = (num * sign, den * sign);
        let g = gcd(num, den);
        if g == 0 {
            return Ok(Self { num: 0, den: 1 });
        }
        Ok(Self {
            num: num / g,
            den: den / g,
        })
    }

    pub fn from_int(v: i64) -> Self {
        Self { num: v, den: 1 }
    }

    pub fn numerator(&self) -> i64 {
        self.num
    }

    pub fn denominator(&self) -> i64 {
        self.den
    }

    pub fn checked_add(&self, other: &Self) -> Result<Self, SafetyError> {
        let lhs = self
            .num
            .checked_mul(other.den)
            .ok_or(SafetyError::ArithmeticOverflow)?;
        let rhs = other
            .num
            .checked_mul(self.den)
            .ok_or(SafetyError::ArithmeticOverflow)?;
        let num = lhs
            .checked_add(rhs)
            .ok_or(SafetyError::ArithmeticOverflow)?;
        let den = self
            .den
            .checked_mul(other.den)
            .ok_or(SafetyError::ArithmeticOverflow)?;
        Self::new(num, den)
    }

    pub fn checked_sub(&self, other: &Self) -> Result<Self, SafetyError> {
        let neg = Self {
            num: other
                .num
                .checked_neg()
                .ok_or(SafetyError::ArithmeticOverflow)?,
            den: other.den,
        };
        self.checked_add(&neg)
    }

    pub fn checked_mul(&self, other: &Self) -> Result<Self, SafetyError> {
        let num = self
            .num
            .checked_mul(other.num)
            .ok_or(SafetyError::ArithmeticOverflow)?;
        let den = self
            .den
            .checked_mul(other.den)
            .ok_or(SafetyError::ArithmeticOverflow)?;
        Self::new(num, den)
    }

    pub fn checked_div(&self, other: &Self) -> Result<Self, SafetyError> {
        if other.num == 0 {
            return Err(SafetyError::ZeroDenominator);
        }
        let num = self
            .num
            .checked_mul(other.den)
            .ok_or(SafetyError::ArithmeticOverflow)?;
        let den = self
            .den
            .checked_mul(other.num)
            .ok_or(SafetyError::ArithmeticOverflow)?;
        Self::new(num, den)
    }

    /// Exact comparison by checked cross-multiplication (CLIN-024: equality is
    /// its own state and is never rounded into `below` or `above`).
    pub fn cmp_exact(&self, other: &Self) -> Result<Ordering, SafetyError> {
        let lhs = self
            .num
            .checked_mul(other.den)
            .ok_or(SafetyError::ArithmeticOverflow)?;
        let rhs = other
            .num
            .checked_mul(self.den)
            .ok_or(SafetyError::ArithmeticOverflow)?;
        Ok(lhs.cmp(&rhs))
    }

    pub fn cmp_int(&self, v: i64) -> Result<Ordering, SafetyError> {
        self.cmp_exact(&Self::from_int(v))
    }

    pub fn is_zero(&self) -> bool {
        self.num == 0
    }

    pub fn is_positive(&self) -> bool {
        self.num > 0
    }

    /// Display value in tenths, rounded half away from zero (spec 02:
    /// "round-half-away-from-zero" to one decimal place). Display values are
    /// formatting only and never drive a decision.
    pub fn display_tenths(&self) -> Result<i64, SafetyError> {
        let scaled = self
            .num
            .checked_mul(10)
            .ok_or(SafetyError::ArithmeticOverflow)?;
        // (2a ± b) / 2b truncated toward zero rounds a/b half away from zero.
        let twice = scaled
            .checked_mul(2)
            .ok_or(SafetyError::ArithmeticOverflow)?;
        let adjust = if scaled >= 0 { self.den } else { -self.den };
        let numer = twice
            .checked_add(adjust)
            .ok_or(SafetyError::ArithmeticOverflow)?;
        let denom = self
            .den
            .checked_mul(2)
            .ok_or(SafetyError::ArithmeticOverflow)?;
        Ok(numer / denom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(num: i64, den: i64) -> Rational {
        Rational::new(num, den).unwrap()
    }

    #[test]
    fn reduces_and_normalises_sign() {
        assert_eq!(r(6, 4), r(3, 2));
        assert_eq!(r(-6, -4), r(3, 2));
        assert_eq!(r(6, -4), r(-3, 2));
        assert_eq!(r(0, -7), r(0, 1));
    }

    #[test]
    fn zero_denominator_is_a_safety_error() {
        assert_eq!(Rational::new(1, 0), Err(SafetyError::ZeroDenominator));
    }

    #[test]
    fn exact_comparison_preserves_equality() {
        assert_eq!(r(1, 3).cmp_exact(&r(2, 6)).unwrap(), Ordering::Equal);
        assert_eq!(r(1, 3).cmp_exact(&r(334, 1000)).unwrap(), Ordering::Less);
        assert_eq!(r(17, 2).cmp_int(8).unwrap(), Ordering::Greater);
        assert_eq!(r(17, 2).cmp_exact(&r(17, 2)).unwrap(), Ordering::Equal);
    }

    #[test]
    fn overflow_is_typed_not_wrapped() {
        let big = r(i64::MAX - 1, 1);
        assert_eq!(
            big.checked_mul(&r(3, 1)),
            Err(SafetyError::ArithmeticOverflow)
        );
        assert_eq!(big.checked_add(&big), Err(SafetyError::ArithmeticOverflow));
    }

    #[test]
    fn display_rounds_half_away_from_zero() {
        assert_eq!(r(25, 100).display_tenths().unwrap(), 3); // 0.25 -> 0.3
        assert_eq!(r(-25, 100).display_tenths().unwrap(), -3); // -0.25 -> -0.3
        assert_eq!(r(24, 100).display_tenths().unwrap(), 2); // 0.24 -> 0.2
        assert_eq!(r(212, 1).display_tenths().unwrap(), 2120);
        assert_eq!(r(1275, 10).display_tenths().unwrap(), 1275); // 127.5 stays
        assert_eq!(r(337, 2).display_tenths().unwrap(), 1685); // 168.5
        assert_eq!(r(1, 3).display_tenths().unwrap(), 3); // 0.333 -> 0.3
        assert_eq!(r(2, 3).display_tenths().unwrap(), 7); // 0.667 -> 0.7
    }

    #[test]
    fn arithmetic_is_exact() {
        // 40 + (280 - 40) * 2160 / 4320 = 160
        let interpolated = Rational::from_int(40)
            .checked_add(
                &Rational::from_int(280 - 40)
                    .checked_mul(&r(2160, 4320))
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(interpolated, Rational::from_int(160));
    }
}
