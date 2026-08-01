//! Pure deterministic NICE CG98 clinical evaluation core.
//!
//! This crate implements the clinical behaviour in `spec/02-clinical-rule-engine.md`
//! against the rule pack `nice-cg98-2023-10-31.1`. It is deterministic and
//! independent of wall clock, network, filesystem and environment (DATA-024):
//! it accepts a fully validated assessment plus an evaluation context and
//! returns either a complete result or a typed safety error — never a partial
//! clinical result (API-011).

#![forbid(unsafe_code)]

pub mod catalog;
pub mod error;
pub mod evaluate;
pub mod input;
pub mod output;
pub mod rational;
pub mod thresholds;
pub mod trend;
pub mod types;

pub use catalog::RuleCode;
pub use error::{SafetyError, ValidationCode, ValidationError};
pub use evaluate::{evaluate, EvaluationContext};
pub use input::{Assessment, ClinicalFeatures, Measurement, RiskFactors, TreatmentState};
pub use output::EvaluationOutcome;
pub use rational::Rational;
pub use types::{
    AgeMinutes, BilirubinUmolL, GestationalWeeks, MeasurementMethod, Mode, Priority,
    ThresholdRelation, TreatmentMode, TrendDirection, TriState,
};

/// Engine version, independent of the rule-pack version (spec 04 versioning).
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");
