//! Serde schema for the machine-readable rule pack
//! (`spec/clinical/nice-cg98-2023-10-31.1.yaml`). Unknown fields are
//! rejected so an edited pack cannot smuggle unreviewed content past the
//! loader (SEC-006 spirit at the data boundary).

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RulePackFile {
    pub schema_version: u32,
    pub rule_pack: RulePack,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RulePack {
    pub id: String,
    pub guideline_id: String,
    pub guideline_title: String,
    pub source_updated_on: String,
    pub specification_created_on: String,
    pub status: PackStatus,
    pub market: String,
    pub language: String,
    pub unit: String,
    pub authors: Vec<String>,
    pub clinical_reviewers: Vec<String>,
    pub clinical_safety_officer: Option<String>,
    pub supersedes: Option<String>,
    pub sources: Vec<Source>,
    pub scope: Scope,
    pub arithmetic: Arithmetic,
    pub thresholds: Thresholds,
    pub constants: Constants,
    pub priority_order: Vec<String>,
    pub rules: Vec<Rule>,
    pub universal_notices: Vec<String>,
}

/// Only an `active` pack may serve clinical-mode evaluations (CLIN-003).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackStatus {
    Draft,
    Candidate,
    Active,
    Retired,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub id: String,
    pub url: String,
    pub retrieved_on: String,
    pub sha256: String,
    pub verification_status: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub gestational_age_completed_weeks: Range,
    pub assessment_age_minutes: Range,
    pub treatment_threshold_age_minutes: Range,
    pub bilirubin_umol_l: BilirubinScope,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Range {
    pub minimum: i64,
    pub maximum: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BilirubinScope {
    pub minimum: i64,
    pub maximum: i64,
    pub integer_only: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Arithmetic {
    pub internal: String,
    pub decision_rounding: String,
    pub display_decimal_places: u32,
    pub display_rounding: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Thresholds {
    pub preterm_23_to_37: PretermThresholds,
    pub term_38_plus: TermThresholds,
    pub comparison: Comparison,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PretermThresholds {
    pub gestation_selector: String,
    pub use_corrected_gestation: bool,
    pub phototherapy: PretermLine,
    pub exchange_transfusion: PretermLine,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PretermLine {
    pub birth_umol_l: i64,
    pub age_72h_formula: String,
    pub interpolation: Interpolation,
    pub plateau: Plateau,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Interpolation {
    pub from_age_minutes: u32,
    pub to_age_minutes: u32,
    pub method: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plateau {
    pub from_age_minutes: u32,
    pub through_age_minutes: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TermThresholds {
    pub gestation_bucket_minimum: u8,
    pub interpolation: String,
    pub phototherapy_points: Vec<(u32, i64)>,
    pub exchange_transfusion_points: Vec<(u32, i64)>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Comparison {
    pub below: String,
    pub at: String,
    pub above: String,
    pub equality_policy: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Constants {
    pub first_day_minutes: u32,
    pub serum_gestation_cutoff_weeks: u8,
    pub term_retest_gestation_weeks: u8,
    pub tcb_serum_confirmation_umol_l_strictly_greater_than: i64,
    pub rapid_rise_umol_l_per_hour_strictly_greater_than: f64,
    pub treatment_margin_umol_l: i64,
    pub intensified_exchange_proximity_age_minutes_minimum: u32,
    pub kernicterus_bilirubin_umol_l_strictly_greater_than: i64,
    pub kernicterus_gestation_weeks_minimum: u8,
    pub prolonged_term_age_minutes_strictly_greater_than: u32,
    pub prolonged_preterm_age_minutes_strictly_greater_than: u32,
    pub prolonged_gestation_boundary_weeks: u8,
    pub conjugated_bilirubin_umol_l_strictly_greater_than: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    pub code: String,
    pub order: u32,
    pub priority: String,
    pub when: String,
    pub source_refs: Vec<String>,
    #[serde(default)]
    pub policy_refs: Vec<String>,
}
