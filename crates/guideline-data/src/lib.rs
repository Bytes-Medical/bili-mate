//! Rule-pack loading, integrity verification and startup self-tests
//! (spec 05 `guideline-data` responsibilities; CLIN-001–CLIN-006, DATA-027).
//!
//! The approved pack is embedded in the binary at build time; production
//! never retrieves or scrapes NICE content at startup or runtime. The v1
//! engine implements rules as typed Rust functions, so the self-tests here
//! prove the embedded pack and the compiled engine agree before the service
//! reports readiness: mismatch is a startup safety fault, not a runtime
//! surprise.

pub mod diff;
pub mod schema;

use sha2::{Digest, Sha256};

use clinical_core::catalog::RuleCode;
use clinical_core::rational::Rational;
use clinical_core::thresholds::{
    treatment_thresholds, TERM_EXCHANGE_POINTS, TERM_PHOTOTHERAPY_POINTS,
};
use clinical_core::types::{AgeMinutes, GestationalWeeks, Priority};

use schema::{PackStatus, RulePackFile};

/// The identifier of the embedded approved pack (CLIN-001).
pub const RULE_PACK_ID: &str = "nice-cg98-2023-10-31.1";

/// Raw content of the embedded pack. The `spec/clinical` file is the single
/// normative source; embedding it directly prevents transcription drift.
pub const EMBEDDED_PACK_YAML: &str =
    include_str!("../../../spec/clinical/nice-cg98-2023-10-31.1.yaml");

/// Metadata surfaced through the API and decision receipts.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RulePackSummary {
    pub id: String,
    pub guideline_id: String,
    pub source_updated_on: String,
    pub status: String,
    pub content_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    Parse(String),
    /// The pack disagrees with the compiled engine; each entry is one failed
    /// self-test. The service must refuse readiness (SEC-014, OPS-004).
    SelfTest(Vec<String>),
}

impl core::fmt::Display for LoadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LoadError::Parse(e) => write!(f, "rule pack parse failure: {e}"),
            LoadError::SelfTest(failures) => {
                write!(f, "rule pack self-test failures: {}", failures.join("; "))
            }
        }
    }
}

impl std::error::Error for LoadError {}

/// A parsed and self-test-verified rule pack.
#[derive(Debug, Clone)]
pub struct VerifiedPack {
    pub file: RulePackFile,
    pub content_sha256: String,
}

impl VerifiedPack {
    pub fn summary(&self) -> RulePackSummary {
        let pack = &self.file.rule_pack;
        RulePackSummary {
            id: pack.id.clone(),
            guideline_id: pack.guideline_id.clone(),
            source_updated_on: pack.source_updated_on.clone(),
            status: match pack.status {
                PackStatus::Draft => "draft",
                PackStatus::Candidate => "candidate",
                PackStatus::Active => "active",
                PackStatus::Retired => "retired",
            }
            .to_string(),
            content_sha256: self.content_sha256.clone(),
        }
    }

    /// Only an `active` pack may serve clinical-mode evaluations (CLIN-003).
    pub fn clinical_mode_allowed(&self) -> bool {
        self.file.rule_pack.status == PackStatus::Active
    }
}

pub fn sha256_hex(content: &str) -> String {
    hex::encode(Sha256::digest(content.as_bytes()))
}

/// Parse a pack without running self-tests (used by the diff tool).
pub fn parse_pack(yaml: &str) -> Result<RulePackFile, LoadError> {
    serde_yaml::from_str(yaml).map_err(|e| LoadError::Parse(e.to_string()))
}

/// Load and verify the embedded approved pack.
pub fn load_embedded_pack() -> Result<VerifiedPack, LoadError> {
    load_pack(EMBEDDED_PACK_YAML)
}

/// Parse a pack and run every startup self-test vector against the compiled
/// engine.
pub fn load_pack(yaml: &str) -> Result<VerifiedPack, LoadError> {
    let file = parse_pack(yaml)?;
    let failures = self_test(&file);
    if !failures.is_empty() {
        return Err(LoadError::SelfTest(failures));
    }
    Ok(VerifiedPack {
        content_sha256: sha256_hex(yaml),
        file,
    })
}

fn priority_from_str(s: &str) -> Option<Priority> {
    Some(match s {
        "emergency" => Priority::Emergency,
        "immediate" => Priority::Immediate,
        "urgent" => Priority::Urgent,
        "treatment" => Priority::Treatment,
        "timed" => Priority::Timed,
        "routine" => Priority::Routine,
        _ => return None,
    })
}

/// Startup self-test vectors (spec 05: `guideline-data`; health readiness).
fn self_test(file: &RulePackFile) -> Vec<String> {
    let mut failures = Vec::new();
    let pack = &file.rule_pack;

    if file.schema_version != 1 {
        failures.push(format!(
            "unsupported schema_version {}",
            file.schema_version
        ));
    }
    if pack.id != RULE_PACK_ID {
        failures.push(format!(
            "pack id {} does not match expected {RULE_PACK_ID}",
            pack.id
        ));
    }
    if pack.unit != "umol/L" {
        failures.push(format!("unexpected unit {}", pack.unit));
    }

    // Scope must match the compiled domain types.
    let scope = &pack.scope;
    let scope_checks = [
        (
            "gestation minimum",
            scope.gestational_age_completed_weeks.minimum,
            23,
        ),
        (
            "gestation maximum",
            scope.gestational_age_completed_weeks.maximum,
            42,
        ),
        (
            "assessment age minimum",
            scope.assessment_age_minutes.minimum,
            0,
        ),
        (
            "assessment age maximum",
            scope.assessment_age_minutes.maximum,
            40_319,
        ),
        (
            "treatment age minimum",
            scope.treatment_threshold_age_minutes.minimum,
            0,
        ),
        (
            "treatment age maximum",
            scope.treatment_threshold_age_minutes.maximum,
            20_160,
        ),
        ("bilirubin minimum", scope.bilirubin_umol_l.minimum, 0),
        ("bilirubin maximum", scope.bilirubin_umol_l.maximum, 1_000),
    ];
    for (name, actual, expected) in scope_checks {
        if actual != expected {
            failures.push(format!(
                "scope {name}: pack has {actual}, engine has {expected}"
            ));
        }
    }

    // Constants must match the values compiled into the engine.
    let c = &pack.constants;
    let constant_checks = [
        ("first_day_minutes", i64::from(c.first_day_minutes), 1_440),
        (
            "serum_gestation_cutoff_weeks",
            i64::from(c.serum_gestation_cutoff_weeks),
            35,
        ),
        (
            "term_retest_gestation_weeks",
            i64::from(c.term_retest_gestation_weeks),
            38,
        ),
        (
            "tcb_serum_confirmation",
            c.tcb_serum_confirmation_umol_l_strictly_greater_than,
            250,
        ),
        ("treatment_margin_umol_l", c.treatment_margin_umol_l, 50),
        (
            "intensified_proximity_age_minimum",
            i64::from(c.intensified_exchange_proximity_age_minutes_minimum),
            4_320,
        ),
        (
            "kernicterus_bilirubin",
            c.kernicterus_bilirubin_umol_l_strictly_greater_than,
            340,
        ),
        (
            "kernicterus_gestation_minimum",
            i64::from(c.kernicterus_gestation_weeks_minimum),
            37,
        ),
        (
            "prolonged_term_age",
            i64::from(c.prolonged_term_age_minutes_strictly_greater_than),
            20_160,
        ),
        (
            "prolonged_preterm_age",
            i64::from(c.prolonged_preterm_age_minutes_strictly_greater_than),
            30_240,
        ),
        (
            "prolonged_gestation_boundary",
            i64::from(c.prolonged_gestation_boundary_weeks),
            37,
        ),
        (
            "conjugated_bilirubin",
            c.conjugated_bilirubin_umol_l_strictly_greater_than,
            25,
        ),
    ];
    for (name, actual, expected) in constant_checks {
        if actual != expected {
            failures.push(format!(
                "constant {name}: pack has {actual}, engine has {expected}"
            ));
        }
    }
    // The rapid-rise rate is exactly 17/2 in the engine.
    if c.rapid_rise_umol_l_per_hour_strictly_greater_than != 8.5 {
        failures.push(format!(
            "constant rapid_rise: pack has {}, engine has 8.5",
            c.rapid_rise_umol_l_per_hour_strictly_greater_than
        ));
    }

    // Term control points must match the compiled tables exactly (H-005).
    let term = &pack.thresholds.term_38_plus;
    if term.phototherapy_points != TERM_PHOTOTHERAPY_POINTS {
        failures.push("term phototherapy control points differ from engine".into());
    }
    if term.exchange_transfusion_points != TERM_EXCHANGE_POINTS {
        failures.push("term exchange control points differ from engine".into());
    }

    // Preterm birth values and 72-hour formulas must reproduce the engine's
    // calculated lines for every supported preterm gestation.
    let preterm = &pack.thresholds.preterm_23_to_37;
    if preterm.use_corrected_gestation {
        failures.push(
            "pack demands corrected gestation; engine uses actual gestation (CLIN-009)".into(),
        );
    }
    for g in 23..=37u8 {
        let gestation = GestationalWeeks::new(g).expect("supported gestation");
        for (age, expect_photo, expect_exch) in [
            (
                0u32,
                preterm.phototherapy.birth_umol_l,
                preterm.exchange_transfusion.birth_umol_l,
            ),
            (4_320, 10 * i64::from(g) - 100, 10 * i64::from(g)),
        ] {
            match treatment_thresholds(gestation, AgeMinutes::new(age).expect("valid age")) {
                Ok(Some(pair)) => {
                    if pair.phototherapy != Rational::from_int(expect_photo) {
                        failures.push(format!(
                            "preterm photo self-test failed at {g} weeks, {age} min"
                        ));
                    }
                    if pair.exchange != Rational::from_int(expect_exch) {
                        failures.push(format!(
                            "preterm exchange self-test failed at {g} weeks, {age} min"
                        ));
                    }
                }
                _ => failures.push(format!("no threshold produced at {g} weeks, {age} min")),
            }
        }
    }

    // Priority order must match the engine's fixed order (CLIN-047).
    let expected_priorities = [
        "emergency",
        "immediate",
        "urgent",
        "treatment",
        "timed",
        "routine",
    ];
    if pack.priority_order != expected_priorities {
        failures.push("priority order differs from engine".into());
    }

    // Every pack rule must exist in the compiled catalogue with the same
    // stable order and priority, and vice versa (CLIN-005: changing a
    // mapping requires a new pack revision AND a reviewed engine change).
    let catalogue: std::collections::HashMap<&str, (u32, Priority)> = RuleCode::all()
        .iter()
        .map(|code| (code.as_str(), (code.spec().order, code.spec().priority)))
        .collect();
    for rule in &pack.rules {
        match catalogue.get(rule.code.as_str()) {
            None => failures.push(format!(
                "pack rule {} is not implemented by the engine",
                rule.code
            )),
            Some((order, priority)) => {
                if *order != rule.order {
                    failures.push(format!(
                        "rule {}: pack order {} differs from engine order {order}",
                        rule.code, rule.order
                    ));
                }
                match priority_from_str(&rule.priority) {
                    Some(p) if p == *priority => {}
                    _ => failures.push(format!(
                        "rule {}: pack priority {} differs from engine priority {priority:?}",
                        rule.code, rule.priority
                    )),
                }
            }
        }
    }
    if pack.rules.len() != RuleCode::all().len() {
        failures.push(format!(
            "pack defines {} rules, engine implements {}",
            pack.rules.len(),
            RuleCode::all().len()
        ));
    }

    failures
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_pack_parses_and_passes_every_self_test() {
        let pack = load_embedded_pack().expect("embedded pack must verify");
        assert_eq!(pack.file.rule_pack.id, RULE_PACK_ID);
        assert_eq!(pack.content_sha256, sha256_hex(EMBEDDED_PACK_YAML));
    }

    #[test]
    fn draft_pack_refuses_clinical_mode() {
        let pack = load_embedded_pack().unwrap();
        // The committed pack is still draft: clinical mode must refuse it
        // (CLIN-003) until two clinical approvals set it active.
        assert!(!pack.clinical_mode_allowed());
        assert_eq!(pack.summary().status, "draft");
    }

    #[test]
    fn tampered_threshold_fails_the_self_test() {
        // Flip one term control point: integrity checking must catch it.
        let tampered = EMBEDDED_PACK_YAML.replace("- [1440, 200]", "- [1440, 201]");
        assert_ne!(tampered, EMBEDDED_PACK_YAML, "tamper target must exist");
        match load_pack(&tampered) {
            Err(LoadError::SelfTest(failures)) => {
                assert!(
                    failures.iter().any(|f| f.contains("control points")),
                    "{failures:?}"
                );
            }
            other => panic!("tampered pack must fail self-test, got {other:?}"),
        }
    }

    #[test]
    fn tampered_constant_fails_the_self_test() {
        let tampered = EMBEDDED_PACK_YAML.replace(
            "kernicterus_bilirubin_umol_l_strictly_greater_than: 340",
            "kernicterus_bilirubin_umol_l_strictly_greater_than: 350",
        );
        assert_ne!(tampered, EMBEDDED_PACK_YAML);
        match load_pack(&tampered) {
            Err(LoadError::SelfTest(failures)) => {
                assert!(
                    failures.iter().any(|f| f.contains("kernicterus_bilirubin")),
                    "{failures:?}"
                );
            }
            other => panic!("tampered constant must fail self-test, got {other:?}"),
        }
    }

    #[test]
    fn unknown_yaml_field_is_rejected() {
        let extended =
            EMBEDDED_PACK_YAML.replace("schema_version: 1", "schema_version: 1\nextra_field: 1");
        assert!(matches!(load_pack(&extended), Err(LoadError::Parse(_))));
    }
}
