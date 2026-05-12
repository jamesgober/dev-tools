//! Use the `coverage` feature through `dev_tools::coverage` to build
//! a coverage run, classify the result against a threshold, and emit a
//! `CheckResult`.
//!
//! Requires the `coverage` feature:
//!
//! ```text
//! cargo run --example coverage_basic --features coverage
//! ```
//!
//! Constructs a `CoverageResult` directly (no subprocess) so the
//! example is fast and stable in CI. To run a real coverage scan, see
//! `dev_tools::coverage::CoverageRun::execute()` (requires
//! `cargo install cargo-llvm-cov`).

use dev_tools::coverage::{CoverageResult, CoverageThreshold};

fn main() {
    let result = CoverageResult {
        name: "demo".into(),
        version: "0.1.0".into(),
        line_pct: 82.5,
        function_pct: 90.0,
        region_pct: 78.0,
        branch_pct: None,
        total_lines: 200,
        covered_lines: 165,
        total_functions: 40,
        covered_functions: 36,
        total_regions: 100,
        covered_regions: 78,
        files: Vec::new(),
    };

    println!(
        "line: {:.1}%, function: {:.1}%, region: {:.1}%",
        result.line_pct, result.function_pct, result.region_pct
    );

    let check = result.into_check_result(CoverageThreshold::min_line_pct(80.0));
    println!("verdict against >= 80% line: {:?}", check.verdict);
    println!("detail: {}", check.detail.as_deref().unwrap_or("(none)"));
}
