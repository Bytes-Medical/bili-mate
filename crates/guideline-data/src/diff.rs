//! Candidate-vs-predecessor pack comparison (spec 05: tooling to compare a
//! candidate pack with its predecessor). Used during rule-pack review to
//! show clinical reviewers exactly what changed.

use crate::schema::RulePackFile;

/// Human-readable list of clinically meaningful differences between two
/// packs. An empty result means the packs are clinically equivalent even if
/// metadata such as reviewers differs.
pub fn diff_packs(predecessor: &RulePackFile, candidate: &RulePackFile) -> Vec<String> {
    let mut changes = Vec::new();
    let old = &predecessor.rule_pack;
    let new = &candidate.rule_pack;

    if old.id != new.id {
        changes.push(format!("pack id: {} -> {}", old.id, new.id));
    }
    if old.source_updated_on != new.source_updated_on {
        changes.push(format!(
            "source update date: {} -> {}",
            old.source_updated_on, new.source_updated_on
        ));
    }

    let old_term = &old.thresholds.term_38_plus;
    let new_term = &new.thresholds.term_38_plus;
    if old_term.phototherapy_points != new_term.phototherapy_points {
        changes.push("term phototherapy control points changed".into());
    }
    if old_term.exchange_transfusion_points != new_term.exchange_transfusion_points {
        changes.push("term exchange control points changed".into());
    }

    let old_pre = &old.thresholds.preterm_23_to_37;
    let new_pre = &new.thresholds.preterm_23_to_37;
    if old_pre.phototherapy.birth_umol_l != new_pre.phototherapy.birth_umol_l
        || old_pre.phototherapy.age_72h_formula != new_pre.phototherapy.age_72h_formula
        || old_pre.exchange_transfusion.birth_umol_l != new_pre.exchange_transfusion.birth_umol_l
        || old_pre.exchange_transfusion.age_72h_formula
            != new_pre.exchange_transfusion.age_72h_formula
    {
        changes.push("preterm threshold formulas changed".into());
    }

    // Constants: compare the serialised view field by field.
    let old_c = format!("{:?}", old.constants);
    let new_c = format!("{:?}", new.constants);
    if old_c != new_c {
        changes.push("clinical constants changed".into());
    }

    let old_rules: std::collections::BTreeMap<&str, (u32, &str)> = old
        .rules
        .iter()
        .map(|r| (r.code.as_str(), (r.order, r.priority.as_str())))
        .collect();
    let new_rules: std::collections::BTreeMap<&str, (u32, &str)> = new
        .rules
        .iter()
        .map(|r| (r.code.as_str(), (r.order, r.priority.as_str())))
        .collect();
    for (code, meta) in &old_rules {
        match new_rules.get(code) {
            None => changes.push(format!("rule removed: {code}")),
            Some(new_meta) if new_meta != meta => {
                changes.push(format!("rule changed: {code} {meta:?} -> {new_meta:?}"))
            }
            _ => {}
        }
    }
    for code in new_rules.keys() {
        if !old_rules.contains_key(code) {
            changes.push(format!("rule added: {code}"));
        }
    }

    changes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse_pack, EMBEDDED_PACK_YAML};

    #[test]
    fn identical_packs_have_no_differences() {
        let a = parse_pack(EMBEDDED_PACK_YAML).unwrap();
        let b = parse_pack(EMBEDDED_PACK_YAML).unwrap();
        assert!(diff_packs(&a, &b).is_empty());
    }

    #[test]
    fn changed_control_point_is_reported() {
        let a = parse_pack(EMBEDDED_PACK_YAML).unwrap();
        let tampered = EMBEDDED_PACK_YAML.replace("- [1440, 200]", "- [1440, 205]");
        let b = parse_pack(&tampered).unwrap();
        let changes = diff_packs(&a, &b);
        assert!(
            changes
                .iter()
                .any(|c| c.contains("phototherapy control points")),
            "{changes:?}"
        );
    }
}
