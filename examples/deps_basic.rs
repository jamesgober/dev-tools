//! Use the `deps` feature through `dev_tools::deps` to assemble a
//! dependency-health result with both unused and outdated findings,
//! then produce a `Report`.
//!
//! Requires the `deps` feature:
//!
//! ```text
//! cargo run --example deps_basic --features deps
//! ```
//!
//! Constructs the result directly. To run a real dependency scan, see
//! `dev_tools::deps::DepCheck::execute()` (requires
//! `cargo install cargo-udeps cargo-outdated` + nightly toolchain).

use dev_tools::deps::{DepKind, DepResult, DepScope, OutdatedDep, UnusedDep};

fn main() {
    let result = DepResult {
        name: "demo".into(),
        version: "0.1.0".into(),
        scope: DepScope::All,
        unused: vec![UnusedDep {
            crate_name: "legacy-shim".into(),
            kind: DepKind::Development,
        }],
        outdated: vec![OutdatedDep {
            crate_name: "serde".into(),
            current: "1.0.0".into(),
            latest: "2.0.0".into(),
            major_behind: 1,
            kind: Some(DepKind::Normal),
        }],
        escalate_at_majors: Some(3),
    };

    println!(
        "unused: {}, outdated: {}, total: {}",
        result.unused_count(),
        result.outdated_count(),
        result.total_findings()
    );
    println!("worst severity: {:?}", result.worst_severity());

    let report = result.into_report();
    println!("\n{}", report.to_json().expect("serialize report"));
}
