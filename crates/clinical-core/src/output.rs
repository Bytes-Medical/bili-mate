//! Serialisable evaluation output (spec 03 derived types). Display values
//! are produced here, after every decision has been made on exact rationals.

use serde::{Serialize, Serializer};

use crate::catalog::{Category, SourceReference, Timeframe};
use crate::rational::Rational;
use crate::types::{Priority, ThresholdRelation, TrendDirection};

/// A one-decimal display value held as tenths so serialisation is exact and
/// deterministic (TEST-011). Never used in comparisons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Display1Dp(pub i64);

impl Serialize for Display1Dp {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(self.0 as f64 / 10.0)
    }
}

/// Reduced exact fraction recorded in the decision trace (spec 03:
/// ExactThreshold), so tests and reviewers can check decisions against exact
/// values rather than display rounding (TEST-007).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ExactFraction {
    pub numerator: i64,
    pub denominator: i64,
}

impl From<&Rational> for ExactFraction {
    fn from(r: &Rational) -> Self {
        Self {
            numerator: r.numerator(),
            denominator: r.denominator(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ThresholdAssessment {
    pub measurement_id: String,
    pub age_minutes: u32,
    pub phototherapy_threshold_umol_l: Option<Display1Dp>,
    pub phototherapy_relation: ThresholdRelation,
    pub phototherapy_distance_umol_l: Option<Display1Dp>,
    pub exchange_threshold_umol_l: Option<Display1Dp>,
    pub exchange_relation: ThresholdRelation,
    pub exchange_distance_umol_l: Option<Display1Dp>,
    /// True only for serum measurements (CLIN-017).
    pub treatment_decision_eligible: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TrendAssessment {
    pub older_measurement_id: String,
    pub newer_measurement_id: String,
    pub interval_minutes: u32,
    pub rate_umol_l_per_hour: Display1Dp,
    pub direction: TrendDirection,
    pub reliable_for_rapid_rise: bool,
    pub rapid_rise_relation: ThresholdRelation,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Recommendation {
    pub code: String,
    pub priority: Priority,
    pub category: Category,
    pub action: String,
    pub timeframe: Option<Timeframe>,
    pub rationale: String,
    pub source_refs: Vec<SourceReference>,
    pub requires_serum_confirmation: bool,
    /// Always true (DATA-017): every recommendation requires clinician
    /// confirmation.
    pub requires_clinician_confirmation: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WarningCategory {
    Clinical,
    Scope,
    Source,
    Assay,
    System,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Warning {
    pub code: String,
    pub category: WarningCategory,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MissingInformation {
    /// RFC 6901 JSON Pointer to the unknown or absent field.
    pub pointer: String,
    pub code: String,
    /// Which clinical behaviour the gap affects (CLIN-050).
    pub impact: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NormalisedInput {
    pub gestational_age_completed_weeks: u8,
    pub assessment_age_minutes: u32,
    pub measurement_count: usize,
    pub latest_measurement_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExactThresholdTrace {
    pub measurement_id: String,
    pub phototherapy: Option<ExactFraction>,
    pub exchange: Option<ExactFraction>,
    pub phototherapy_distance: Option<ExactFraction>,
    pub exchange_distance: Option<ExactFraction>,
}

/// Decision trace (CLIN-048, DATA-022): exact values behind the decision and
/// the rule codes evaluated, without exposing executable internals.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DecisionTrace {
    pub exact_thresholds: Vec<ExactThresholdTrace>,
    pub exact_rate: Option<ExactFraction>,
    pub activated_rules: Vec<String>,
}

/// The complete deterministic clinical result. Operational metadata
/// (evaluation ID, timestamp, rule-pack summary, legal text) is added outside
/// the core (spec 05: exact arithmetic section).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EvaluationOutcome {
    pub normalised_input: NormalisedInput,
    pub thresholds: Vec<ThresholdAssessment>,
    pub trend: Option<TrendAssessment>,
    pub primary_action: Recommendation,
    /// Includes the primary action exactly once as its first item (DATA-020).
    pub recommendations: Vec<Recommendation>,
    pub warnings: Vec<Warning>,
    pub missing_information: Vec<MissingInformation>,
    pub suppressed_rules: Vec<String>,
    pub decision_trace: DecisionTrace,
}
