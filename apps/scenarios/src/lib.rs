//! Clinical scenario harness (spec 09 clinical scenario validation).
//!
//! Scenarios are the independently reviewable encoding of what the engine
//! is expected to do: inputs, the expected primary and supporting actions
//! and the NICE references, in a form two clinicians can sign. The harness
//! runs each scenario through the clinical core and reports divergence;
//! the exported review document adds reviewer identity, outcome and
//! discrepancy-disposition fields and carries a digest of the scenario set
//! so an approval signs exactly one content state.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use serde::Deserialize;
use sha2::{Digest, Sha256};

use clinical_core::evaluate::{evaluate, EvaluationContext};
use clinical_core::input::{
    Assessment, ClinicalFeatures, Measurement, RiskFactors, TreatmentState,
};
use clinical_core::output::EvaluationOutcome;
use clinical_core::types::{
    AgeMinutes, BilirubinUmolL, GestationalWeeks, MeasurementMethod, Mode, ThresholdRelation,
    TreatmentMode, TriState,
};

pub const SCENARIO_FILE: &str = include_str!("../../../validation/cg98-scenarios.yaml");

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioFile {
    pub scenarios: Vec<Scenario>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub id: String,
    pub title: String,
    pub category: String,
    #[serde(default)]
    pub description: String,
    /// NICE CG98 recommendation references supporting the expectation.
    #[serde(default)]
    pub refs: Vec<String>,
    pub input: ScenarioInput,
    pub expect: Expectations,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioInput {
    pub gestation: u8,
    pub age: u32,
    /// Baseline: jaundice suspected and visible. Set false for a baby with
    /// both confirmed absent.
    #[serde(default = "default_true")]
    pub jaundice: bool,
    /// Overrides on the baseline features (baseline: clinically well
    /// present, every danger sign absent, metabolic screen completed).
    #[serde(default)]
    pub features: BTreeMap<String, String>,
    #[serde(default)]
    pub risks: BTreeMap<String, String>,
    #[serde(default)]
    pub measurements: Vec<MeasurementInput>,
    #[serde(default)]
    pub conjugated: Option<u16>,
    #[serde(default)]
    pub treatment: Option<TreatmentInput>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeasurementInput {
    pub age: u32,
    pub value: u16,
    pub method: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TreatmentInput {
    pub mode: String,
    #[serde(default)]
    pub started: Option<u32>,
    #[serde(default)]
    pub stopped: Option<u32>,
    #[serde(default)]
    pub exchange_completed: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Expectations {
    pub primary: String,
    #[serde(default)]
    pub active: Vec<String>,
    #[serde(default)]
    pub inactive: Vec<String>,
    #[serde(default)]
    pub suppressed: Vec<String>,
    #[serde(default)]
    pub missing: Vec<String>,
    #[serde(default)]
    pub photo_relation: Option<String>,
    #[serde(default)]
    pub exchange_relation: Option<String>,
}

pub struct ScenarioResult {
    pub id: String,
    pub title: String,
    pub category: String,
    pub refs: Vec<String>,
    pub failures: Vec<String>,
    pub actual_primary: String,
    pub actual_active: Vec<String>,
    pub expected_primary: String,
}

impl ScenarioResult {
    pub fn passed(&self) -> bool {
        self.failures.is_empty()
    }
}

fn tri(value: &str) -> Result<TriState, String> {
    match value {
        "present" => Ok(TriState::Present),
        "absent" => Ok(TriState::Absent),
        "unknown" => Ok(TriState::Unknown),
        other => Err(format!("invalid tri-state {other}")),
    }
}

fn relation(value: &str) -> Result<ThresholdRelation, String> {
    match value {
        "below" => Ok(ThresholdRelation::Below),
        "at" => Ok(ThresholdRelation::At),
        "above" => Ok(ThresholdRelation::Above),
        "not_available" => Ok(ThresholdRelation::NotAvailable),
        other => Err(format!("invalid relation {other}")),
    }
}

fn build_assessment(input: &ScenarioInput) -> Result<Assessment, String> {
    let jaundice = if input.jaundice {
        TriState::Present
    } else {
        TriState::Absent
    };
    let mut features = ClinicalFeatures {
        suspected_or_obvious_jaundice: jaundice,
        visible_jaundice: jaundice,
        clinically_well: TriState::Present,
        acute_bilirubin_encephalopathy: TriState::Absent,
        pale_chalky_stools: TriState::Absent,
        dark_urine_stains_nappy: TriState::Absent,
        rhesus_haemolytic_disease: TriState::Absent,
        abo_haemolytic_disease: TriState::Absent,
        infection_suspected: TriState::Absent,
        urinary_tract_infection_suspected: TriState::Absent,
        routine_metabolic_screen_completed: TriState::Present,
    };
    for (key, value) in &input.features {
        let state = tri(value)?;
        match key.as_str() {
            "suspected_or_obvious_jaundice" => features.suspected_or_obvious_jaundice = state,
            "visible_jaundice" => features.visible_jaundice = state,
            "clinically_well" => features.clinically_well = state,
            "acute_bilirubin_encephalopathy" => features.acute_bilirubin_encephalopathy = state,
            "pale_chalky_stools" => features.pale_chalky_stools = state,
            "dark_urine_stains_nappy" => features.dark_urine_stains_nappy = state,
            "rhesus_haemolytic_disease" => features.rhesus_haemolytic_disease = state,
            "abo_haemolytic_disease" => features.abo_haemolytic_disease = state,
            "infection_suspected" => features.infection_suspected = state,
            "urinary_tract_infection_suspected" => {
                features.urinary_tract_infection_suspected = state
            }
            "routine_metabolic_screen_completed" => {
                features.routine_metabolic_screen_completed = state
            }
            other => return Err(format!("unknown clinical feature {other}")),
        }
    }

    let mut risks = RiskFactors {
        previous_sibling_required_phototherapy: TriState::Absent,
        exclusive_breastfeeding_intended: TriState::Absent,
    };
    for (key, value) in &input.risks {
        let state = tri(value)?;
        match key.as_str() {
            "previous_sibling_required_phototherapy" => {
                risks.previous_sibling_required_phototherapy = state
            }
            "exclusive_breastfeeding_intended" => risks.exclusive_breastfeeding_intended = state,
            other => return Err(format!("unknown risk factor {other}")),
        }
    }

    let measurements = input
        .measurements
        .iter()
        .enumerate()
        .map(|(index, m)| {
            Ok(Measurement {
                id: format!("m{}", index + 1),
                age_minutes: AgeMinutes::new(m.age).map_err(|e| e.to_string())?,
                total_bilirubin_umol_l: BilirubinUmolL::new(m.value).map_err(|e| e.to_string())?,
                method: match m.method.as_str() {
                    "serum" => MeasurementMethod::Serum,
                    "transcutaneous" | "tcb" => MeasurementMethod::Transcutaneous,
                    other => return Err(format!("invalid method {other}")),
                },
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    let age = |value: Option<u32>| -> Result<Option<AgeMinutes>, String> {
        value
            .map(|v| AgeMinutes::new(v).map_err(|e| e.to_string()))
            .transpose()
    };
    let treatment = match &input.treatment {
        None => TreatmentState {
            mode: TreatmentMode::None,
            started_age_minutes: None,
            stopped_age_minutes: None,
            exchange_completed_age_minutes: None,
        },
        Some(t) => TreatmentState {
            mode: match t.mode.as_str() {
                "none" => TreatmentMode::None,
                "phototherapy" => TreatmentMode::Phototherapy,
                "intensified_phototherapy" => TreatmentMode::IntensifiedPhototherapy,
                "post_phototherapy" => TreatmentMode::PostPhototherapy,
                "post_exchange" => TreatmentMode::PostExchange,
                other => return Err(format!("invalid treatment mode {other}")),
            },
            started_age_minutes: age(t.started)?,
            stopped_age_minutes: age(t.stopped)?,
            exchange_completed_age_minutes: age(t.exchange_completed)?,
        },
    };

    Assessment::new(
        GestationalWeeks::new(input.gestation).map_err(|e| e.to_string())?,
        AgeMinutes::new(input.age).map_err(|e| e.to_string())?,
        features,
        risks,
        measurements,
        input
            .conjugated
            .map(|v| BilirubinUmolL::new(v).map_err(|e| e.to_string()))
            .transpose()?,
        treatment,
    )
    .map_err(|errors| {
        errors
            .iter()
            .map(|e| format!("{}: {}", e.pointer, e.message))
            .collect::<Vec<_>>()
            .join("; ")
    })
}

fn check(outcome: &EvaluationOutcome, expect: &Expectations) -> Vec<String> {
    let mut failures = Vec::new();
    let active = &outcome.decision_trace.activated_rules;

    if outcome.primary_action.code != expect.primary {
        failures.push(format!(
            "primary: expected {}, engine produced {}",
            expect.primary, outcome.primary_action.code
        ));
    }
    for code in &expect.active {
        if !active.contains(code) {
            failures.push(format!("expected active rule {code} did not activate"));
        }
    }
    for code in &expect.inactive {
        if active.contains(code) {
            failures.push(format!("rule {code} activated but was expected inactive"));
        }
    }
    for code in &expect.suppressed {
        if !outcome.suppressed_rules.contains(code) {
            failures.push(format!("expected suppression of {code} was not recorded"));
        }
    }
    for pointer in &expect.missing {
        if !outcome
            .missing_information
            .iter()
            .any(|m| &m.pointer == pointer)
        {
            failures.push(format!("expected missing-information pointer {pointer}"));
        }
    }
    // Relations refer to the latest (last, age-sorted) measurement.
    if let Some(expected) = &expect.photo_relation {
        match (outcome.thresholds.last(), relation(expected)) {
            (Some(row), Ok(rel)) if row.phototherapy_relation == rel => {}
            (Some(row), Ok(rel)) => failures.push(format!(
                "phototherapy relation: expected {rel:?}, engine produced {:?}",
                row.phototherapy_relation
            )),
            (None, _) => failures.push("no threshold row for photo_relation".into()),
            (_, Err(e)) => failures.push(e),
        }
    }
    if let Some(expected) = &expect.exchange_relation {
        match (outcome.thresholds.last(), relation(expected)) {
            (Some(row), Ok(rel)) if row.exchange_relation == rel => {}
            (Some(row), Ok(rel)) => failures.push(format!(
                "exchange relation: expected {rel:?}, engine produced {:?}",
                row.exchange_relation
            )),
            (None, _) => failures.push("no threshold row for exchange_relation".into()),
            (_, Err(e)) => failures.push(e),
        }
    }
    failures
}

pub fn load_scenarios() -> Result<ScenarioFile, String> {
    let file: ScenarioFile =
        serde_yaml::from_str(SCENARIO_FILE).map_err(|e| format!("scenario file: {e}"))?;
    let mut seen = std::collections::HashSet::new();
    for scenario in &file.scenarios {
        if !seen.insert(scenario.id.clone()) {
            return Err(format!("duplicate scenario id {}", scenario.id));
        }
    }
    Ok(file)
}

pub fn scenario_set_digest() -> String {
    hex::encode(Sha256::digest(SCENARIO_FILE.as_bytes()))
}

pub fn run_all(file: &ScenarioFile) -> Vec<ScenarioResult> {
    file.scenarios
        .iter()
        .map(|scenario| {
            let mut failures = Vec::new();
            let (actual_primary, actual_active) = match build_assessment(&scenario.input) {
                Err(error) => {
                    failures.push(format!("scenario input invalid: {error}"));
                    (String::from("-"), Vec::new())
                }
                Ok(assessment) => {
                    match evaluate(
                        &assessment,
                        &EvaluationContext {
                            mode: Mode::Demonstration,
                        },
                    ) {
                        Err(error) => {
                            failures.push(format!("engine safety failure: {error}"));
                            (String::from("-"), Vec::new())
                        }
                        Ok(outcome) => {
                            failures.extend(check(&outcome, &scenario.expect));
                            (
                                outcome.primary_action.code.clone(),
                                outcome.decision_trace.activated_rules.clone(),
                            )
                        }
                    }
                }
            };
            ScenarioResult {
                id: scenario.id.clone(),
                title: scenario.title.clone(),
                category: scenario.category.clone(),
                refs: scenario.refs.clone(),
                failures,
                actual_primary,
                actual_active,
                expected_primary: scenario.expect.primary.clone(),
            }
        })
        .collect()
}

/// The clinical review document (spec 09: each scenario records inputs,
/// expected actions, source references, actual output, reviewer identity,
/// outcome and discrepancy disposition).
pub fn render_review(file: &ScenarioFile, results: &[ScenarioResult]) -> String {
    let mut document = String::new();
    let divergent = results.iter().filter(|r| !r.passed()).count();
    document.push_str("# Clinical scenario validation review\n\n");
    document.push_str(&format!(
        "Scenario set digest (SHA-256): `{}`\n\nScenarios: {} · engine divergences at export time: {}\n\n",
        scenario_set_digest(),
        results.len(),
        divergent
    ));
    document.push_str(
        "Two clinical reviewers, independent of the rule transcription, review every scenario \
         (spec 08). An approval signs the exact scenario set identified by the digest above.\n\n",
    );
    document.push_str("| # | Category | Scenario | Expected primary | Engine primary | Match |\n|---|---|---|---|---|---|\n");
    for result in results {
        document.push_str(&format!(
            "| {} | {} | {} | `{}` | `{}` | {} |\n",
            result.id,
            result.category,
            result.title,
            result.expected_primary,
            result.actual_primary,
            if result.passed() { "yes" } else { "DIVERGENT" }
        ));
    }
    document.push('\n');

    for (scenario, result) in file.scenarios.iter().zip(results) {
        document.push_str(&format!("---\n\n## {} — {}\n\n", result.id, result.title));
        if !scenario.description.is_empty() {
            document.push_str(&format!("{}\n\n", scenario.description));
        }
        document.push_str(&format!(
            "**Category:** {} · **NICE references:** {}\n\n",
            result.category,
            if result.refs.is_empty() {
                "—".to_string()
            } else {
                result.refs.join(", ")
            }
        ));
        document.push_str("**Inputs**\n\n```yaml\n");
        document.push_str(
            &serde_yaml::to_string(&InputEcho::from(&scenario.input)).unwrap_or_default(),
        );
        document.push_str("```\n\n");
        document.push_str(&format!(
            "**Expected primary action:** `{}`\n\n**Engine primary action:** `{}`\n\n",
            result.expected_primary, result.actual_primary
        ));
        if !scenario.expect.active.is_empty() {
            document.push_str(&format!(
                "**Expected supporting/active:** `{}`\n\n",
                scenario.expect.active.join("`, `")
            ));
        }
        document.push_str(&format!(
            "**Engine active rules:** `{}`\n\n",
            result.actual_active.join("`, `")
        ));
        if result.passed() {
            document.push_str("**Harness result:** PASS\n\n");
        } else {
            document.push_str("**Harness result:** DIVERGENT\n\n");
            for failure in &result.failures {
                document.push_str(&format!("- {failure}\n"));
            }
            document.push('\n');
        }
        document.push_str(
            "**Reviewer 1 (name, date):** ______________________  **Outcome:** approve / discrepancy\n\n\
             **Reviewer 2 (name, date):** ______________________  **Outcome:** approve / discrepancy\n\n\
             **Discrepancy disposition (if any):** ______________________\n\n",
        );
    }
    document
}

/// Serializable echo of the scenario input for the review document.
#[derive(serde::Serialize)]
struct InputEcho {
    gestation: u8,
    age_minutes: u32,
    jaundice_suspected_and_visible: bool,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    feature_overrides: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    risk_overrides: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    measurements: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    conjugated_bilirubin_umol_l: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    treatment: Option<String>,
}

impl From<&ScenarioInput> for InputEcho {
    fn from(input: &ScenarioInput) -> Self {
        Self {
            gestation: input.gestation,
            age_minutes: input.age,
            jaundice_suspected_and_visible: input.jaundice,
            feature_overrides: input.features.clone(),
            risk_overrides: input.risks.clone(),
            measurements: input
                .measurements
                .iter()
                .map(|m| format!("{} µmol/L {} at {} min", m.value, m.method, m.age))
                .collect(),
            conjugated_bilirubin_umol_l: input.conjugated,
            treatment: input.treatment.as_ref().map(|t| {
                format!(
                    "{} (started {:?}, stopped {:?}, exchange {:?})",
                    t.mode, t.started, t.stopped, t.exchange_completed
                )
            }),
        }
    }
}
