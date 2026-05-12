//! Compose four producers from across the suite into a single
//! `MultiReport`, then render it as a self-contained HTML document.
//!
//! Requires the `coverage`, `security`, `flaky`, and `mutate` features:
//!
//! ```text
//! cargo run --example full_pipeline --features "coverage,security,flaky,mutate"
//! ```
//!
//! Each producer is wired through `dev_tools::<feature>::*Producer`,
//! but the underlying subprocess invocations are not actually run —
//! the example instead constructs synthetic `*Result` values and
//! converts them to `Report`s, then aggregates them. This keeps the
//! example deterministic and fast in CI.
//!
//! For the real pipeline (one that spawns cargo subcommands), wrap
//! each `*Run` in its corresponding `*Producer` and pass them to the
//! `full_run!` macro. See `examples/cargo_test_producer.rs` for the
//! producer-construction pattern used across the suite.

use dev_tools::coverage::{CoverageResult, CoverageThreshold};
use dev_tools::flaky::{FlakyResult, TestReliability};
use dev_tools::mutate::{MutateResult, MutateThreshold};
use dev_tools::report::MultiReport;
use dev_tools::report::Severity;
use dev_tools::security::{AuditResult, AuditScope, Finding, FindingSource};
use dev_tools::MultiReportHtmlExt;

fn main() {
    // ---- coverage producer (synthetic result) ----
    let cov = CoverageResult {
        name: "my-crate".into(),
        version: "0.1.0".into(),
        line_pct: 82.5,
        function_pct: 91.0,
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
    let cov_report = {
        let mut r =
            dev_tools::report::Report::new("my-crate", "0.1.0").with_producer("dev-coverage");
        r.push(cov.into_check_result(CoverageThreshold::min_line_pct(80.0)));
        r.finish();
        r
    };

    // ---- security producer (synthetic result) ----
    let sec_report = AuditResult {
        name: "my-crate".into(),
        version: "0.1.0".into(),
        scope: AuditScope::All,
        findings: vec![Finding {
            id: "RUSTSEC-2024-0001".into(),
            title: "Use after free".into(),
            severity: Severity::Critical,
            affected_crate: "foo".into(),
            affected_version: Some("1.2.3".into()),
            url: None,
            description: None,
            source: FindingSource::Audit,
        }],
    }
    .into_report();

    // ---- flaky producer (synthetic result) ----
    let flk_report = FlakyResult {
        name: "my-crate".into(),
        version: "0.1.0".into(),
        iterations: 20,
        tests: vec![
            TestReliability {
                name: "math::add".into(),
                passes: 20,
                failures: 0,
            },
            TestReliability {
                name: "net::flaky_endpoint".into(),
                passes: 17,
                failures: 3,
            },
        ],
        reliability_threshold_pct: None,
    }
    .into_report();

    // ---- mutate producer (synthetic result) ----
    let mut_result = MutateResult {
        name: "my-crate".into(),
        version: "0.1.0".into(),
        mutants_total: 100,
        mutants_killed: 88,
        mutants_survived: 12,
        mutants_timeout: 0,
        survivors: Vec::new(),
        files: Vec::new(),
    };
    let mut_report = {
        let mut r = dev_tools::report::Report::new("my-crate", "0.1.0").with_producer("dev-mutate");
        r.push(mut_result.into_check_result(MutateThreshold::min_kill_pct(85.0)));
        r.finish();
        r
    };

    // ---- compose into a MultiReport and render to HTML ----
    let mut multi = MultiReport::new("my-crate", "0.1.0");
    multi.push(cov_report);
    multi.push(sec_report);
    multi.push(flk_report);
    multi.push(mut_report);
    multi.finish();

    let (pass, fail, warn, skip) = multi.verdict_counts();
    println!(
        "aggregate: {} pass, {} fail, {} warn, {} skip ({:?} overall)",
        pass,
        fail,
        warn,
        skip,
        multi.overall_verdict()
    );

    let html = multi.to_html();
    let out = std::env::temp_dir().join("dev-tools-full-pipeline.html");
    std::fs::write(&out, html).expect("write html");
    println!(
        "\nwrote {} ({} bytes)",
        out.display(),
        std::fs::metadata(&out).unwrap().len()
    );
}
