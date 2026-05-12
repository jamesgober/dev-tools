//! Demonstrate `use dev_tools::prelude::*;`.
//!
//! ```text
//! cargo run --example prelude_demo
//! ```
//!
//! Pulls in the schema types (`Report`, `CheckResult`, `Verdict`,
//! `Severity`, `Evidence`, ...) and the `MultiReportHtmlExt` trait in
//! one import. No optional features required.

use dev_tools::prelude::*;

fn main() {
    let mut r = Report::new("my-crate", "0.1.0").with_producer("prelude-demo");
    r.push(CheckResult::pass("compile"));
    r.push(
        CheckResult::warn("style::trailing_ws", Severity::Warning)
            .with_detail("3 trailing-whitespace warnings"),
    );
    r.push(
        CheckResult::fail("test::round_trip", Severity::Error)
            .with_detail("expected 42, got 41")
            .with_evidence(Evidence::file_ref_lines("site", "tests/smoke.rs", 42, 47)),
    );
    r.finish();

    println!("overall verdict: {:?}", r.overall_verdict());
    println!(
        "passed: {}, failed: {}, warned: {}",
        r.passed(),
        r.failed(),
        r.warned()
    );

    let mut multi = MultiReport::new("my-crate", "0.1.0");
    multi.push(r);
    multi.finish();

    // MultiReportHtmlExt is in the prelude; .to_html() is one call away.
    let html = multi.to_html();
    println!("\nHTML meta-report: {} bytes", html.len());
}
