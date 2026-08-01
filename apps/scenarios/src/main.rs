//! `bili-scenarios` — run the clinical scenario set against the engine.
//!
//! Usage:
//!   bili-scenarios                 # run and summarise; exit 1 on divergence
//!   bili-scenarios --export FILE   # also write the clinical review document

use bili_mate_scenarios::{load_scenarios, render_review, run_all, scenario_set_digest};

fn main() {
    let mut export: Option<String> = None;
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--export" => export = Some(args.next().expect("--export needs a path")),
            other => {
                eprintln!("unknown flag {other}");
                std::process::exit(64);
            }
        }
    }

    let file = match load_scenarios() {
        Ok(file) => file,
        Err(error) => {
            eprintln!("scenario set invalid: {error}");
            std::process::exit(65);
        }
    };
    let results = run_all(&file);

    let mut divergent = 0usize;
    for result in &results {
        if result.passed() {
            println!("PASS      {}  {}", result.id, result.title);
        } else {
            divergent += 1;
            println!("DIVERGENT {}  {}", result.id, result.title);
            for failure in &result.failures {
                println!("          - {failure}");
            }
        }
    }
    println!(
        "\n{} scenarios, {} divergent · scenario set digest {}",
        results.len(),
        divergent,
        scenario_set_digest()
    );

    if let Some(path) = export {
        std::fs::write(&path, render_review(&file, &results)).expect("write review document");
        println!("review document written to {path}");
    }

    std::process::exit(if divergent == 0 { 0 } else { 1 });
}
