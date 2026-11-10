//! Audits a dataset and prints the JSON report (exit 1 on any problem). Usage:
//! `cargo run --example audit [-- path/to/dataset.json]` (default: the bundled tables).

use mc_mappings::Dataset;
use std::process::ExitCode;

fn main() -> ExitCode {
    let dataset = match std::env::args().nth(1) {
        Some(path) => std::fs::read_to_string(&path)
            .map_err(|e| format!("{path}: {e}"))
            .and_then(|text| Dataset::from_json(&text).map_err(|e| format!("{path}: {e}"))),
        None => Dataset::minecraft_v1().map_err(|e| e.to_string()),
    };
    match dataset {
        Err(e) => {
            eprintln!("invalid dataset: {e}");
            ExitCode::FAILURE
        }
        Ok(dataset) => {
            let report = dataset.audit();
            println!(
                "{}",
                serde_json::to_string_pretty(&report).expect("serializable report")
            );
            if report.is_ok() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
    }
}
