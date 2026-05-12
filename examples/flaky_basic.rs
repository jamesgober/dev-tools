//! Use the `flaky` feature through `dev_tools::flaky` to assemble a
//! flaky-test result and classify each test (stable / flaky / broken).
//!
//! Requires the `flaky` feature:
//!
//! ```text
//! cargo run --example flaky_basic --features flaky
//! ```
//!
//! Constructs `FlakyResult` + `TestReliability` directly. For a real
//! repeated-run flakiness scan, see `dev_tools::flaky::FlakyRun::execute()`.

use dev_tools::flaky::{Classification, FlakyResult, TestReliability};

fn main() {
    let result = FlakyResult {
        name: "demo".into(),
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
            TestReliability {
                name: "regress::off_by_one".into(),
                passes: 0,
                failures: 20,
            },
        ],
        reliability_threshold_pct: None,
    };

    println!(
        "stable: {}, flaky: {}, broken: {}, total: {}",
        result.stable_count(),
        result.flaky_count(),
        result.broken_count(),
        result.total_count()
    );

    for t in &result.tests {
        let class = t.classification(None);
        println!(
            "  {:<28} {:>5.1}%  {}",
            t.name,
            t.reliability_pct(),
            match class {
                Classification::Stable => "stable",
                Classification::Flaky => "flaky",
                Classification::Broken => "broken",
            }
        );
    }

    let report = result.into_report();
    println!("\noverall verdict: {:?}", report.overall_verdict());
}
