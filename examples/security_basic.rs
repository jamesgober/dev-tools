//! Use the `security` feature through `dev_tools::security` to assemble
//! an audit result, classify each finding, and produce a multi-check
//! `Report`.
//!
//! Requires the `security` feature:
//!
//! ```text
//! cargo run --example security_basic --features security
//! ```
//!
//! Constructs `AuditResult` + `Finding` values directly so the example
//! doesn't depend on `cargo-audit` / `cargo-deny` being installed.

use dev_tools::report::Severity;
use dev_tools::security::{AuditResult, AuditScope, Finding, FindingSource};

fn main() {
    let result = AuditResult {
        name: "demo".into(),
        version: "0.1.0".into(),
        scope: AuditScope::All,
        findings: vec![
            Finding {
                id: "RUSTSEC-2024-0001".into(),
                title: "Use after free in foo".into(),
                severity: Severity::Critical,
                affected_crate: "foo".into(),
                affected_version: Some("1.2.3".into()),
                url: Some("https://rustsec.org/advisories/RUSTSEC-2024-0001".into()),
                description: None,
                source: FindingSource::Audit,
            },
            Finding {
                id: "L001".into(),
                title: "license `GPL-3.0` not allowed".into(),
                severity: Severity::Error,
                affected_crate: "bar".into(),
                affected_version: Some("0.5.0".into()),
                url: None,
                description: None,
                source: FindingSource::Deny,
            },
        ],
    };

    println!(
        "{} findings; worst severity: {:?}",
        result.findings.len(),
        result.worst_severity()
    );
    println!(
        "audit findings:  {}",
        result.count_from(FindingSource::Audit)
    );
    println!(
        "policy findings: {}",
        result.count_from(FindingSource::Deny)
    );

    let report = result.into_report();
    println!("\n{}", report.to_json().expect("serialize report"));
}
