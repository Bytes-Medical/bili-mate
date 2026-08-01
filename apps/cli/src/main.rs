//! `bili-eval` — engineering-only synthetic evaluator (Stage 1, spec 11).
//!
//! Usage: `bili-eval <request.json>` or `bili-eval -` to read stdin.
//! Prints the evaluation as pretty JSON. Demonstration mode only; output is
//! not for patient care.

use std::io::Read;
use std::process::ExitCode;

use bili_mate_cli::{run, CliError};

fn main() -> ExitCode {
    let arg = match std::env::args().nth(1) {
        Some(arg) => arg,
        None => {
            eprintln!("usage: bili-eval <request.json | ->");
            return ExitCode::from(64);
        }
    };

    let request = if arg == "-" {
        let mut buffer = String::new();
        if let Err(e) = std::io::stdin().read_to_string(&mut buffer) {
            eprintln!("failed to read stdin: {e}");
            return ExitCode::from(66);
        }
        buffer
    } else {
        match std::fs::read_to_string(&arg) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("failed to read {arg}: {e}");
                return ExitCode::from(66);
            }
        }
    };

    match run(&request) {
        Ok(response) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&response).expect("response serialises")
            );
            ExitCode::SUCCESS
        }
        Err(CliError::Schema(message)) => {
            eprintln!("schema validation failed: {message}");
            ExitCode::from(2)
        }
        Err(CliError::Domain(errors)) => {
            eprintln!("domain validation failed:");
            for e in errors {
                eprintln!(
                    "  {} [{}]: {}",
                    e.pointer,
                    serde_json::to_string(&e.code).unwrap(),
                    e.message
                );
            }
            ExitCode::from(2)
        }
        Err(CliError::RulePackMismatch {
            requested,
            available,
        }) => {
            eprintln!("rule pack conflict: requested {requested}, active pack is {available}; no clinical result produced");
            ExitCode::from(3)
        }
        Err(CliError::Safety(message)) => {
            eprintln!("engine safety check failed: {message}; no clinical result produced");
            ExitCode::from(4)
        }
        Err(CliError::PackIntegrity(message)) => {
            eprintln!("rule pack integrity failure: {message}; refusing to evaluate");
            ExitCode::from(5)
        }
    }
}
