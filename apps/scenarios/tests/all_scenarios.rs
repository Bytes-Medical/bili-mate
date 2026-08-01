//! CI gate: every scenario in the signed clinical set must match the
//! engine. A divergence is either an engine defect or a scenario-set
//! change, and both require review before merge (OPS-017 spirit).

use bili_mate_scenarios::{load_scenarios, run_all};

#[test]
fn scenario_set_is_valid_and_large_enough() {
    let file = load_scenarios().expect("scenario set parses");
    assert!(
        file.scenarios.len() >= 60,
        "spec 09 requires at least 60 scenarios, found {}",
        file.scenarios.len()
    );
    // Every spec 09 distribution category is represented.
    for category in [
        "preterm-thresholds",
        "term-thresholds",
        "recognition",
        "measurement-method",
        "below-line-monitoring",
        "trends",
        "phototherapy",
        "intensified-phototherapy",
        "exchange-and-encephalopathy",
        "haemolysis-ivig",
        "underlying-disease",
        "prolonged-jaundice",
        "missing-and-out-of-scope",
    ] {
        assert!(
            file.scenarios.iter().any(|s| s.category == category),
            "no scenario covers category {category}"
        );
    }
}

#[test]
fn every_scenario_matches_the_engine() {
    let file = load_scenarios().expect("scenario set parses");
    let results = run_all(&file);
    let divergent: Vec<String> = results
        .iter()
        .filter(|r| !r.passed())
        .map(|r| format!("{} {}: {}", r.id, r.title, r.failures.join("; ")))
        .collect();
    assert!(
        divergent.is_empty(),
        "{} divergent scenario(s):\n{}",
        divergent.len(),
        divergent.join("\n")
    );
}
