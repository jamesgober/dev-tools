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
//! - [`mod@report`]: structured machine-readable verdicts (always enabled).
//! - [`mod@fixtures`]: deterministic test environments.
//! - [`mod@bench`]: performance measurement and regression detection.
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
//! dev-tools = "0.9.2"
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

/// Convenience re-exports for the most common items across the suite.
///
/// `use dev_tools::prelude::*;` to pull in the schema types
/// ([`Report`], [`CheckResult`], [`Verdict`], [`Severity`], [`Evidence`],
/// the [`Producer`] trait) plus `MultiReport` and `Diff`. Optional
/// per-feature items (`fixtures::TempProject`, `bench::Benchmark`,
/// etc.) are NOT in the prelude — pull them in directly via the
/// re-exported sub-crate modules.
///
/// # Example
///
/// ```
/// use dev_tools::prelude::*;
///
/// let mut r = Report::new("my-crate", "0.1.0");
/// r.push(CheckResult::pass("compile"));
/// r.finish();
/// assert!(r.passed());
/// ```
///
/// [`Report`]: dev_report::Report
/// [`CheckResult`]: dev_report::CheckResult
/// [`Verdict`]: dev_report::Verdict
/// [`Severity`]: dev_report::Severity
/// [`Evidence`]: dev_report::Evidence
/// [`Producer`]: dev_report::Producer
pub mod prelude {
    pub use dev_report::{
        CheckResult, Diff, DiffOptions, DurationRegression, Evidence, EvidenceData, EvidenceKind,
        FileRef, MultiReport, Producer, Report, Severity, SeverityChange, Verdict,
    };

    /// Async-flavored prelude. Available with the `async` feature.
    ///
    /// Pulls in the standard prelude plus `dev_async`'s
    /// `AsyncCheck`, `AsyncProducer`, and `BlockingAsyncProducer`
    /// types so callers driving async producers don't have to
    /// import them individually.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use dev_tools::prelude::async_prelude::*;
    ///
    /// // run_with_timeout, BlockingAsyncProducer, etc. all in scope
    /// ```
    #[cfg(feature = "async")]
    #[cfg_attr(docsrs, doc(cfg(feature = "async")))]
    pub mod async_prelude {
        pub use super::*;
        pub use dev_async::{
            join_all_with_timeout, run_with_timeout, AsyncCheck, AsyncProducer,
            BlockingAsyncProducer,
        };
    }
}

/// Combine multiple `dev_report::Producer` results into a single
/// `MultiReport` keyed by `subject`/`version`.
///
/// Pure composition: no new types, no new logic. Each producer is
/// invoked once via `Producer::produce()` and pushed into the
/// returned [`dev_report::MultiReport`].
///
/// # Example
///
/// ```
/// use dev_tools::full_run;
/// use dev_tools::report::{CheckResult, Producer, Report, Verdict};
///
/// struct A;
/// impl Producer for A {
///     fn produce(&self) -> Report {
///         let mut r = Report::new("crate", "0.1.0").with_producer("a");
///         r.push(CheckResult::pass("ok"));
///         r.finish();
///         r
///     }
/// }
/// struct B;
/// impl Producer for B {
///     fn produce(&self) -> Report {
///         let mut r = Report::new("crate", "0.1.0").with_producer("b");
///         r.push(CheckResult::pass("ok"));
///         r.finish();
///         r
///     }
/// }
///
/// let multi = full_run!("crate", "0.1.0"; A, B);
/// assert_eq!(multi.reports.len(), 2);
/// assert_eq!(multi.overall_verdict(), Verdict::Pass);
/// ```
#[macro_export]
macro_rules! full_run {
    ($subject:expr, $version:expr; $($producer:expr),* $(,)?) => {{
        let mut multi = $crate::report::MultiReport::new($subject, $version);
        $(
            multi.push(<_ as $crate::report::Producer>::produce(&$producer));
        )*
        multi.finish();
        multi
    }};
}

/// Combine multiple `Future<Output = Report>` values into a single
/// `MultiReport` keyed by `subject`/`version`.
///
/// Async equivalent of [`full_run!`] for callers already inside an
/// async context. Each future is awaited in sequence (use a
/// futures-runtime helper if you need concurrency); the resulting
/// reports are pushed into the returned [`dev_report::MultiReport`].
///
/// Available with the `async` feature.
///
/// # Example
///
/// ```ignore
/// use dev_tools::async_full_run;
/// use dev_tools::report::{CheckResult, Report, Verdict};
///
/// async fn produce_a() -> Report {
///     let mut r = Report::new("crate", "0.1.0").with_producer("a");
///     r.push(CheckResult::pass("ok"));
///     r.finish();
///     r
/// }
///
/// async fn produce_b() -> Report {
///     let mut r = Report::new("crate", "0.1.0").with_producer("b");
///     r.push(CheckResult::pass("ok"));
///     r.finish();
///     r
/// }
///
/// # async fn ex() {
/// let multi = async_full_run!("crate", "0.1.0"; produce_a(), produce_b()).await;
/// assert_eq!(multi.reports.len(), 2);
/// assert_eq!(multi.overall_verdict(), Verdict::Pass);
/// # }
/// ```
#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
#[macro_export]
macro_rules! async_full_run {
    ($subject:expr, $version:expr; $($fut:expr),* $(,)?) => {{
        async {
            let mut multi = $crate::report::MultiReport::new($subject, $version);
            $(
                multi.push($fut.await);
            )*
            multi.finish();
            multi
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_module_is_always_available() {
        let r = report::Report::new("self", "0.1.0");
        assert_eq!(r.subject, "self");
    }

    #[test]
    fn prelude_pulls_core_types() {
        // The prelude should make these immediately accessible
        // without further imports.
        use crate::prelude::*;

        let mut r = Report::new("c", "0.1.0");
        r.push(CheckResult::pass("ok"));
        r.finish();
        assert_eq!(r.overall_verdict(), Verdict::Pass);
        assert!(r.passed());

        let _ev = Evidence::numeric_int("count", 42);
        let _opts = DiffOptions::default();
        let _multi = MultiReport::new("c", "0.1.0");

        // 0.9.2: also includes DurationRegression and SeverityChange.
        let _dr: Option<DurationRegression> = None;
        let _sc: Option<SeverityChange> = None;

        // Sanity-check that Severity and Producer/Diff/etc. are in scope.
        let _sev = Severity::Error;
        fn _takes_producer(_p: &dyn Producer) {}
        fn _takes_diff(_d: &Diff) {}
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

    #[test]
    fn full_run_combines_zero_producers() {
        let multi = full_run!("crate", "0.1.0";);
        assert_eq!(multi.reports.len(), 0);
        assert_eq!(multi.overall_verdict(), report::Verdict::Skip);
    }

    #[test]
    fn full_run_combines_two_producers() {
        struct OkProducer(&'static str);
        impl report::Producer for OkProducer {
            fn produce(&self) -> report::Report {
                let mut r = report::Report::new("c", "0.1.0").with_producer(self.0);
                r.push(report::CheckResult::pass("x"));
                r.finish();
                r
            }
        }
        let multi = full_run!("c", "0.1.0"; OkProducer("a"), OkProducer("b"));
        assert_eq!(multi.reports.len(), 2);
        assert_eq!(multi.overall_verdict(), report::Verdict::Pass);
    }

    #[test]
    fn full_run_propagates_failures() {
        struct OkProducer;
        impl report::Producer for OkProducer {
            fn produce(&self) -> report::Report {
                let mut r = report::Report::new("c", "0.1.0").with_producer("ok");
                r.push(report::CheckResult::pass("x"));
                r.finish();
                r
            }
        }
        struct FailProducer;
        impl report::Producer for FailProducer {
            fn produce(&self) -> report::Report {
                let mut r = report::Report::new("c", "0.1.0").with_producer("fail");
                r.push(report::CheckResult::fail("y", report::Severity::Error));
                r.finish();
                r
            }
        }
        let multi = full_run!("c", "0.1.0"; OkProducer, FailProducer);
        assert_eq!(multi.overall_verdict(), report::Verdict::Fail);
    }

    #[cfg(all(feature = "fixtures", feature = "bench"))]
    #[test]
    fn full_run_with_real_producers() {
        // fixtures: a self-test of TempProject lifecycle.
        let fixture_producer =
            fixtures::FixtureProducer::new("temp_project_lifecycle", "0.1.0", || {
                let _p = fixtures::TempProject::new()
                    .with_file("README.md", "hi")
                    .build()?;
                Ok(())
            });
        // bench: a tiny benchmark with no baseline.
        let bench_producer = bench::BenchProducer::new(
            || {
                let mut b = bench::Benchmark::new("hot");
                for _ in 0..5 {
                    b.iter(|| std::hint::black_box(1 + 1));
                }
                b.finish()
            },
            "0.1.0",
            None,
            bench::Threshold::regression_pct(20.0),
        );
        let multi = full_run!("crate", "0.1.0"; fixture_producer, bench_producer);
        assert_eq!(multi.reports.len(), 2);
    }

    #[cfg(feature = "async")]
    #[test]
    fn async_full_run_compiles() {
        // Compile-time check that async_full_run! expands cleanly.
        // We don't drive the future here (no runtime in dev-deps), but
        // compilation alone is meaningful: it catches macro-hygiene bugs.
        async fn produce_a() -> report::Report {
            let mut r = report::Report::new("c", "0.1.0").with_producer("a");
            r.push(report::CheckResult::pass("x"));
            r.finish();
            r
        }
        let _fut = async_full_run!("c", "0.1.0"; produce_a(), produce_a());
        // Drop the future without polling; compiles cleanly.
    }
}
