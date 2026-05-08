//! # dev-tools
//!
//! Modular verification toolkit for AI-assisted Rust development.
//! Umbrella crate over the `dev-*` suite.
//!
//! `dev-tools` is the convenient one-import entry point. Pick the
//! features you need and pull them in with one line.
//!
//! ## Default features
//!
//! By default, you get:
//!
//! - [`report`]: structured machine-readable verdicts (always enabled).
//! - [`fixtures`]: deterministic test environments.
//! - [`bench`]: performance measurement and regression detection.
//!
//! ## Opt-in features
//!
//! Enable with `features = ["..."]`:
//!
//! - `async`: async-specific validation (deadlocks, hung futures, leaks).
//! - `stress`: high-load stress testing (concurrency, volume).
//! - `chaos`: failure injection and recovery testing.
//! - `full`: all of the above.
//!
//! ## Quick example
//!
//! ```toml
//! [dependencies]
//! dev-tools = "0.1"
//! ```
//!
//! ```rust
//! use dev_tools::report::{Report, Verdict};
//!
//! let mut r = Report::new("my-crate", "0.1.0");
//! // ... use r ...
//! ```
//!
//! ## See also
//!
//! - [`dev-report`](https://crates.io/crates/dev-report) - schema only
//! - [`dev-fixtures`](https://crates.io/crates/dev-fixtures) - test environments
//! - [`dev-bench`](https://crates.io/crates/dev-bench) - performance
//! - [`dev-async`](https://crates.io/crates/dev-async) - async validation
//! - [`dev-stress`](https://crates.io/crates/dev-stress) - load testing
//! - [`dev-chaos`](https://crates.io/crates/dev-chaos) - failure injection

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

/// Re-export of [`dev_report`]. Always available.
pub use dev_report as report;

/// Re-export of [`dev_fixtures`]. Available with the `fixtures` feature.
#[cfg(feature = "fixtures")]
#[cfg_attr(docsrs, doc(cfg(feature = "fixtures")))]
pub use dev_fixtures as fixtures;

/// Re-export of [`dev_bench`]. Available with the `bench` feature.
#[cfg(feature = "bench")]
#[cfg_attr(docsrs, doc(cfg(feature = "bench")))]
pub use dev_bench as bench;

/// Re-export of [`dev_async`]. Available with the `async` feature.
#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub use dev_async as r#async;

/// Re-export of [`dev_stress`]. Available with the `stress` feature.
#[cfg(feature = "stress")]
#[cfg_attr(docsrs, doc(cfg(feature = "stress")))]
pub use dev_stress as stress;

/// Re-export of [`dev_chaos`]. Available with the `chaos` feature.
#[cfg(feature = "chaos")]
#[cfg_attr(docsrs, doc(cfg(feature = "chaos")))]
pub use dev_chaos as chaos;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_module_is_always_available() {
        let r = report::Report::new("self", "0.1.0");
        assert_eq!(r.subject, "self");
    }

    #[cfg(feature = "fixtures")]
    #[test]
    fn fixtures_module_is_available_with_feature() {
        let _ = fixtures::TempProject::new();
    }

    #[cfg(feature = "bench")]
    #[test]
    fn bench_module_is_available_with_feature() {
        let _ = bench::Benchmark::new("x");
    }
}
