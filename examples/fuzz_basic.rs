//! Use the `fuzz` feature through `dev_tools::fuzz` to assemble a fuzz
//! result with a critical-severity crash finding, then produce a
//! `Report`.
//!
//! Requires the `fuzz` feature:
//!
//! ```text
//! cargo run --example fuzz_basic --features fuzz
//! ```
//!
//! Constructs `FuzzResult` directly so the example doesn't require
//! `cargo-fuzz` + nightly to be installed. To run a real fuzz session,
//! see `dev_tools::fuzz::FuzzRun::execute()`.

use dev_tools::fuzz::{FuzzFinding, FuzzFindingKind, FuzzResult};

fn main() {
    let result = FuzzResult {
        target: "parse_input".into(),
        version: "0.1.0".into(),
        executions: 1_234_567,
        findings: vec![FuzzFinding {
            kind: FuzzFindingKind::Crash,
            reproducer_path: "fuzz/artifacts/parse_input/crash-deadbeef".into(),
            summary: "thread '<unnamed>' panicked at 'index out of bounds'".into(),
        }],
    };

    println!(
        "executions: {}, findings: {}",
        result.executions,
        result.total_findings()
    );
    println!("crashes: {}", result.count_of(FuzzFindingKind::Crash));
    println!("worst severity: {:?}", result.worst_severity());

    let report = result.into_report();
    println!("\n{}", report.to_json().expect("serialize report"));
}
