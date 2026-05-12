//! Use the `mutate` feature through `dev_tools::mutate` to assemble a
//! mutation-testing result and grade it against a kill-rate threshold.
//!
//! Requires the `mutate` feature:
//!
//! ```text
//! cargo run --example mutate_basic --features mutate
//! ```
//!
//! Constructs `MutateResult` directly. For a real mutation run, see
//! `dev_tools::mutate::MutateRun::execute()` (requires
//! `cargo install cargo-mutants`).

use dev_tools::mutate::{FileBreakdown, MutateResult, MutateThreshold, SurvivingMutant};

fn main() {
    let result = MutateResult {
        name: "demo".into(),
        version: "0.1.0".into(),
        mutants_total: 120,
        mutants_killed: 88,
        mutants_survived: 22,
        mutants_timeout: 10,
        survivors: vec![SurvivingMutant {
            file: "src/parser.rs".into(),
            line: 142,
            description: "replace `<` with `<=`".into(),
            function: Some("validate_range".into()),
        }],
        files: vec![
            FileBreakdown {
                file: "src/parser.rs".into(),
                killed: 30,
                survived: 10,
                timeout: 2,
            },
            FileBreakdown {
                file: "src/codec.rs".into(),
                killed: 58,
                survived: 12,
                timeout: 8,
            },
        ],
    };

    println!(
        "kill rate: {:.2}% (killed {} / surviving {} / timeouts {})",
        result.kill_pct(),
        result.mutants_killed,
        result.mutants_survived,
        result.mutants_timeout
    );

    println!("\nweakest files (ascending kill rate):");
    for f in result.weakest_files(5) {
        println!("  {:<20} {:.1}%", f.file, f.kill_pct());
    }

    let check = result.into_check_result(MutateThreshold::min_kill_pct(75.0));
    println!("\nverdict: {:?}", check.verdict);
}
