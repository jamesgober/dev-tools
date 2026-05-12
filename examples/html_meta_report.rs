//! Build a synthetic `MultiReport`, render it to HTML, and write the
//! result to disk.
//!
//! Demonstrates the HTML meta-report end-to-end: a passing producer
//! (`dev-bench`), a failing producer (`dev-chaos`), and a producer with
//! warnings (`dev-fixtures`). The output is a self-contained HTML file
//! you can open in any browser — no external assets, no JavaScript
//! dependencies.
//!
//! ```text
//! cargo run --example html_meta_report
//! ```
//!
//! By default the output goes to
//! `target/dev-tools-examples/meta-report.html`. Override with an
//! argument:
//!
//! ```text
//! cargo run --example html_meta_report -- /tmp/meta.html
//! ```

use std::fs;
use std::path::PathBuf;

use dev_tools::prelude::*;

fn build_synthetic_multi() -> MultiReport {
    let mut bench = Report::new("sample-crate", "0.9.3").with_producer("dev-bench");
    bench.push(CheckResult::pass("parse::hot").with_duration_ms(120));
    bench.push(CheckResult::pass("parse::cold").with_duration_ms(560));
    bench.push(
        CheckResult::pass("encode::small")
            .with_duration_ms(35)
            .with_evidence(Evidence::numeric("mean_ns", 35_000.0))
            .with_evidence(Evidence::numeric_int("iterations", 100_000)),
    );
    bench.finish();

    let mut chaos = Report::new("sample-crate", "0.9.3").with_producer("dev-chaos");
    chaos.push(
        CheckResult::fail("recover::after_io_error", Severity::Error)
            .with_detail("service did not recover within 5s")
            .with_duration_ms(5_001),
    );
    chaos.push(
        CheckResult::fail("recover::after_oom", Severity::Critical)
            .with_detail("process exited 137 (OOM) without restart"),
    );
    chaos.push(CheckResult::pass("recover::after_disk_full").with_duration_ms(420));
    chaos.finish();

    let mut fixtures = Report::new("sample-crate", "0.9.3").with_producer("dev-fixtures");
    fixtures.push(CheckResult::pass("temp_project::lifecycle").with_duration_ms(8));
    fixtures.push(
        CheckResult::warn("golden::out_of_date", Severity::Warning)
            .with_detail("baseline last refreshed 47 days ago"),
    );
    fixtures.push(CheckResult::skip("integration::network").with_detail("no net in sandbox"));
    fixtures.finish();

    let mut multi = MultiReport::new("sample-crate", "0.9.3");
    multi.push(bench);
    multi.push(chaos);
    multi.push(fixtures);
    multi.finish();
    multi
}

fn main() {
    let multi = build_synthetic_multi();
    let html = multi.to_html();

    let target_path: PathBuf = match std::env::args().nth(1) {
        Some(p) => PathBuf::from(p),
        None => {
            let mut p = PathBuf::from("target");
            p.push("dev-tools-examples");
            p.push("meta-report.html");
            p
        }
    };
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).expect("create output directory");
    }
    fs::write(&target_path, &html).expect("write HTML output");
    println!("wrote {} bytes to {}", html.len(), target_path.display());
}
