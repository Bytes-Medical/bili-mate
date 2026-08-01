//! Typed failures. A `SafetyError` means no clinical result may be returned
//! (API-011, TEST-012); a `ValidationError` is a field-level domain rejection
//! reported with an RFC 6901 JSON Pointer.

use serde::Serialize;

/// Internal safety failure. The caller must map this to an unavailable
/// response with no partial clinical content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SafetyError {
    /// A checked arithmetic operation overflowed (DATA: ExactThreshold).
    ArithmeticOverflow,
    /// A rational was constructed with a zero denominator.
    ZeroDenominator,
}

impl core::fmt::Display for SafetyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SafetyError::ArithmeticOverflow => {
                write!(f, "engine safety check failed: arithmetic overflow")
            }
            SafetyError::ZeroDenominator => {
                write!(f, "engine safety check failed: zero denominator")
            }
        }
    }
}

impl std::error::Error for SafetyError {}

/// Field-level domain validation error (DATA-013, spec 03 validation layers).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidationError {
    /// RFC 6901 JSON Pointer to the offending field.
    pub pointer: String,
    /// Stable machine code.
    pub code: ValidationCode,
    /// Safe human explanation; never reproduces the submitted value (API-017).
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ValidationCode {
    OutOfRange,
    DuplicateMeasurementAge,
    DuplicateMeasurementId,
    MeasurementAfterAssessment,
    TreatmentAgeAfterAssessment,
    TreatmentStateFieldRequired,
    TreatmentStateFieldForbidden,
    TreatmentStopNotAfterStart,
    TooManyMeasurements,
    InvalidIdentifier,
}
