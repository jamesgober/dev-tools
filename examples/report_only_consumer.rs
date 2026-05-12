//! Show that `dev-tools` is useful even with NO opt-in features —
//! just the `report` schema is enough to consume reports produced
//! elsewhere.
//!
//! ```text
//! cargo run --example report_only_consumer
//! ```
//!
//! Construct a sample `Report` JSON in memory, parse it back through
//! `dev_tools::report::Report::from_json`, and summarize. This is the
//! pattern for any tool that reads dev-* reports from disk, a queue,
//! or stdin without driving the producers itself.

use dev_tools::report::Report;

fn main() {
    let sample = r#"{
        "schema_version": 1,
        "subject": "my-crate",
        "subject_version": "0.1.0",
        "producer": "external-producer",
        "started_at": "2026-05-12T12:00:00Z",
        "finished_at": "2026-05-12T12:00:05Z",
        "checks": [
            { "name": "compile",       "verdict": "pass", "severity": null,        "detail": null, "at": "2026-05-12T12:00:00Z", "duration_ms": 1200 },
            { "name": "test::add",     "verdict": "pass", "severity": null,        "detail": null, "at": "2026-05-12T12:00:01Z", "duration_ms": 5 },
            { "name": "test::round",   "verdict": "fail", "severity": "error",     "detail": "expected 42, got 41", "at": "2026-05-12T12:00:02Z", "duration_ms": 13 },
            { "name": "style::wsp",    "verdict": "warn", "severity": "warning",   "detail": "3 trailing-whitespace warnings", "at": "2026-05-12T12:00:03Z", "duration_ms": null }
        ]
    }"#;

    let report = Report::from_json(sample).expect("parse report");
    let (pass, fail, warn, skip) = report.verdict_counts();
    println!(
        "subject:        {} v{}",
        report.subject, report.subject_version
    );
    println!(
        "producer:       {}",
        report.producer.as_deref().unwrap_or("(unknown)")
    );
    println!("checks:         {}", report.checks.len());
    println!(
        "counts:         {} pass, {} fail, {} warn, {} skip",
        pass, fail, warn, skip
    );
    println!("overall:        {:?}", report.overall_verdict());

    if report.failed() {
        println!("\nfailing checks:");
        for c in report
            .checks
            .iter()
            .filter(|c| matches!(c.verdict, dev_tools::report::Verdict::Fail))
        {
            println!(
                "  - {} [{:?}] {}",
                c.name,
                c.severity,
                c.detail.as_deref().unwrap_or("")
            );
        }
    }
}
