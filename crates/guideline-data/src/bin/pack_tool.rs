//! `pack-tool` — governed rule-pack operations (spec 05 tooling; CLIN-002,
//! CLIN-005, SAFE-019).
//!
//! Commands:
//!   pack-tool verify <pack.yaml>
//!       Parse, run every engine self-test and print identity, status and
//!       content digest.
//!   pack-tool diff <predecessor.yaml> <candidate.yaml>
//!       Print the clinically meaningful differences for reviewer sign-off.
//!   pack-tool promote <pack.yaml> --to <status> --output <path>
//!             [--reviewer NAME]... [--cso NAME]
//!       Record a status transition with the required approvals:
//!         draft -> clinically_validated   requires two distinct --reviewer
//!         clinically_validated -> active  requires --cso (SAFE-020)
//!         active -> retired               no additional approval
//!       The edit touches only the status/reviewer/CSO lines, re-verifies
//!       the result against the engine, and prints the new content digest.

use std::process::ExitCode;

use guideline_data::schema::PackStatus;
use guideline_data::{load_pack, parse_pack, sha256_hex};

fn status_name(status: PackStatus) -> &'static str {
    match status {
        PackStatus::Draft => "draft",
        PackStatus::ClinicallyValidated => "clinically_validated",
        PackStatus::Active => "active",
        PackStatus::Retired => "retired",
    }
}

fn cmd_verify(path: &str) -> ExitCode {
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) => {
            eprintln!("cannot read {path}: {error}");
            return ExitCode::from(66);
        }
    };
    match load_pack(&content) {
        Ok(pack) => {
            let rule_pack = &pack.file.rule_pack;
            println!("pack            {}", rule_pack.id);
            println!(
                "guideline       {} ({})",
                rule_pack.guideline_id, rule_pack.guideline_title
            );
            println!("status          {}", status_name(rule_pack.status));
            println!("source updated  {}", rule_pack.source_updated_on);
            println!("content sha256  {}", pack.content_sha256);
            println!("authors         {:?}", rule_pack.authors);
            println!("reviewers       {:?}", rule_pack.clinical_reviewers);
            println!(
                "safety officer  {}",
                rule_pack
                    .clinical_safety_officer
                    .as_deref()
                    .unwrap_or("(none)")
            );
            println!("self-tests      PASS (engine and pack agree)");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("verification failed: {error}");
            ExitCode::from(1)
        }
    }
}

fn cmd_diff(old_path: &str, new_path: &str) -> ExitCode {
    let read = |path: &str| std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"));
    let (old, new) = match (read(old_path), read(new_path)) {
        (Ok(old), Ok(new)) => (old, new),
        (Err(error), _) | (_, Err(error)) => {
            eprintln!("{error}");
            return ExitCode::from(66);
        }
    };
    let (old_pack, new_pack) = match (parse_pack(&old), parse_pack(&new)) {
        (Ok(old_pack), Ok(new_pack)) => (old_pack, new_pack),
        (Err(error), _) | (_, Err(error)) => {
            eprintln!("parse failed: {error}");
            return ExitCode::from(65);
        }
    };
    let changes = guideline_data::diff::diff_packs(&old_pack, &new_pack);
    if changes.is_empty() {
        println!("no clinically meaningful differences");
    } else {
        println!("clinically meaningful differences requiring review (CLIN-005):");
        for change in &changes {
            println!("  - {change}");
        }
    }
    println!("predecessor sha256  {}", sha256_hex(&old));
    println!("candidate sha256    {}", sha256_hex(&new));
    ExitCode::SUCCESS
}

fn cmd_promote(args: &[String]) -> ExitCode {
    let mut path: Option<&str> = None;
    let mut to: Option<&str> = None;
    let mut output: Option<&str> = None;
    let mut reviewers: Vec<&str> = Vec::new();
    let mut cso: Option<&str> = None;

    let mut iter = args.iter();
    while let Some(argument) = iter.next() {
        match argument.as_str() {
            "--to" => to = iter.next().map(String::as_str),
            "--output" => output = iter.next().map(String::as_str),
            "--reviewer" => {
                if let Some(name) = iter.next() {
                    reviewers.push(name);
                }
            }
            "--cso" => cso = iter.next().map(String::as_str),
            other if path.is_none() => path = Some(other),
            other => {
                eprintln!("unexpected argument {other}");
                return ExitCode::from(64);
            }
        }
    }
    let (Some(path), Some(to), Some(output)) = (path, to, output) else {
        eprintln!("usage: pack-tool promote <pack.yaml> --to <status> --output <path> [--reviewer NAME]... [--cso NAME]");
        return ExitCode::from(64);
    };

    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) => {
            eprintln!("cannot read {path}: {error}");
            return ExitCode::from(66);
        }
    };
    let current = match parse_pack(&content) {
        Ok(file) => file.rule_pack.status,
        Err(error) => {
            eprintln!("pack does not parse: {error}");
            return ExitCode::from(65);
        }
    };

    // Transition rules (CLIN-003 lifecycle, SAFE-019/SAFE-020).
    let permitted = match (current, to) {
        (PackStatus::Draft, "clinically_validated") => {
            let mut distinct = reviewers.clone();
            distinct.sort_unstable();
            distinct.dedup();
            if distinct.len() < 2 {
                eprintln!(
                    "draft -> clinically_validated requires two distinct clinical reviewers (SAFE-019); got {:?}",
                    reviewers
                );
                return ExitCode::from(1);
            }
            true
        }
        (PackStatus::ClinicallyValidated, "active") => {
            if cso.is_none() {
                eprintln!("clinically_validated -> active requires --cso (SAFE-020)");
                return ExitCode::from(1);
            }
            true
        }
        (PackStatus::Active, "retired") => true,
        _ => false,
    };
    if !permitted {
        eprintln!(
            "transition {} -> {to} is not permitted; the lifecycle is draft -> clinically_validated -> active -> retired",
            status_name(current)
        );
        return ExitCode::from(1);
    }

    // Targeted line edits so everything except status/approvals stays
    // byte-identical for review diffs.
    let mut updated = content.replace(
        &format!("  status: {}", status_name(current)),
        &format!("  status: {to}"),
    );
    if !reviewers.is_empty() {
        let list = reviewers
            .iter()
            .map(|name| format!("\"{name}\""))
            .collect::<Vec<_>>()
            .join(", ");
        updated = updated.replace(
            "  clinical_reviewers: []",
            &format!("  clinical_reviewers: [{list}]"),
        );
    }
    if let Some(name) = cso {
        updated = updated.replace(
            "  clinical_safety_officer: null",
            &format!("  clinical_safety_officer: \"{name}\""),
        );
    }

    // The promoted pack must still parse and pass every engine self-test.
    if let Err(error) = load_pack(&updated) {
        eprintln!("promotion produced an invalid pack; nothing written: {error}");
        return ExitCode::from(1);
    }
    if let Err(error) = std::fs::write(output, &updated) {
        eprintln!("cannot write {output}: {error}");
        return ExitCode::from(74);
    }
    println!("promoted {} -> {to}", status_name(current));
    println!("written to      {output}");
    println!("content sha256  {}", sha256_hex(&updated));
    println!("record this digest in the release manifest (SAFE-024, SEC-014)");
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("verify") if args.len() == 2 => cmd_verify(&args[1]),
        Some("diff") if args.len() == 3 => cmd_diff(&args[1], &args[2]),
        Some("promote") => cmd_promote(&args[1..]),
        _ => {
            eprintln!("usage: pack-tool <verify|diff|promote> ...");
            ExitCode::from(64)
        }
    }
}
