<h1 align="center">
    <img width="99" alt="Rust logo" src="https://raw.githubusercontent.com/jamesgober/rust-collection/72baabd71f00e14aa9184efcb16fa3deddda3a0a/assets/rust-logo.svg">
    <br>
    <strong>dev-tools</strong>
    <br>
    <sup><sub>RUST VERIFICATION TOOLKIT &mdash; TESTS &middot; BENCHES &middot; COVERAGE &middot; FUZZ &middot; AUDIT</sub></sup>
</h1>
<p align="center">
    <a href="https://crates.io/crates/dev-tools"><img alt="crates.io" src="https://img.shields.io/crates/v/dev-tools.svg"></a>
    <a href="https://crates.io/crates/dev-tools"><img alt="downloads" src="https://img.shields.io/crates/d/dev-tools.svg"></a>
    <a href="https://github.com/jamesgober/dev-tools/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/jamesgober/dev-tools/actions/workflows/ci.yml/badge.svg"></a>
    <img alt="MSRV" src="https://img.shields.io/badge/MSRV-1.85%2B-blue.svg?style=flat-square" title="Rust Version">
    <a href="https://docs.rs/dev-tools"><img alt="docs.rs" src="https://docs.rs/dev-tools/badge.svg"></a>
</p>

<p align="center">
    <strong>Verify a Rust crate from every angle.</strong> One dependency, one feature flag per check.<br>
    Tests, benches, coverage, fuzz, audit, mutation, chaos, async, stress, dep hygiene, CI generation — all in one toolkit.
</p>

<br>

## What it is

`dev-tools` is the one-import entry point for the `dev-*` verification
suite. Instead of pulling in a dozen separate crates and wiring them
together, you add one dependency and turn on the parts you need with
feature flags.

The suite answers, with machine-readable evidence, the questions
a Rust crate maintainer actually cares about:

- Did it compile? Did tests pass?
- Did performance regress against the baseline?
- Did async code hang? Task leaks? Hung shutdown?
- Did the system collapse under sustained load?
- Did failure recovery actually work?
- What % of the code is exercised by tests?
- Any known CVEs, banned licenses, or policy violations in the dep tree?
- Any unused or many-major-versions-behind dependencies?
- Does the CI workflow match the project's enabled features?
- Did the fuzz harness catch the crashes the budget should have caught?
- Which tests are flaky vs. reliably broken?
- Does mutation testing reveal tests that pass without asserting anything?

Every check flows through [`dev-report`](https://crates.io/crates/dev-report)
— a stable, versioned JSON schema. No log scraping, no colored
checkmarks. Output is consumable by a CI gate, a release pipeline,
a dashboard, a `jq` one-liner, or an AI assistant trying to validate
its own work.

## Feature flags

Pick features by what you want to verify:

| Feature | Default | What it brings in | Use case |
|---|:---:|---|---|
| `report` | _always on_ | [`dev-report`](https://crates.io/crates/dev-report) — JSON schema, `Report`/`MultiReport`/`Diff` types | Required by everything else; never disabled. |
| `fixtures` | ✅ | [`dev-fixtures`](https://crates.io/crates/dev-fixtures) — `TempProject`, file-tree builders, golden snapshots, mock data | Repeatable test environments and inputs. |
| `bench` | ✅ | [`dev-bench`](https://crates.io/crates/dev-bench) — `Benchmark`, percentile stats, regression thresholds, baseline storage | Performance verification. |
| `async` | — | [`dev-async`](https://crates.io/crates/dev-async) — `run_with_timeout`, deadlock detection, task tracking, shutdown probes | Hung futures, task leaks, shutdown verification. |
| `stress` | — | [`dev-stress`](https://crates.io/crates/dev-stress) — `StressRun`, `SoakRun`, latency percentiles | High-load and sustained-load testing. |
| `chaos` | — | [`dev-chaos`](https://crates.io/crates/dev-chaos) — failure injection, latency injection, crash points, IO wrappers | Failure-recovery testing. |
| `coverage` | — | [`dev-coverage`](https://crates.io/crates/dev-coverage) — wraps `cargo-llvm-cov`; line / function / region kill rates; baseline diff | "Did this PR drop coverage below 80%?" |
| `security` | — | [`dev-security`](https://crates.io/crates/dev-security) — wraps `cargo-audit` + `cargo-deny`; CVE scan + license + banned-crate policy | "Any vulnerable dependencies?" |
| `deps` | — | [`dev-deps`](https://crates.io/crates/dev-deps) — wraps `cargo-udeps` + `cargo-outdated`; unused + outdated + major-lag findings | "Are we behind on the dep tree?" |
| `ci` | — | [`dev-ci`](https://crates.io/crates/dev-ci) — `Generator` builder for GitHub Actions workflow YAML | "Keep the CI workflow in sync with the feature set." |
| `fuzz` | — | [`dev-fuzz`](https://crates.io/crates/dev-fuzz) — wraps `cargo-fuzz`; crash / timeout / OOM with reproducer paths | "Did this fuzz target survive the budget?" |
| `flaky` | — | [`dev-flaky`](https://crates.io/crates/dev-flaky) — N-iteration `cargo test`; stable / flaky / broken classification | "Which tests fail intermittently?" |
| `mutate` | — | [`dev-mutate`](https://crates.io/crates/dev-mutate) — wraps `cargo-mutants`; kill rate + surviving-mutant evidence | "Does the test suite actually assert enough?" |
| `full` | — | Every feature above. | Kitchen-sink verification rigs. |

**Default** = `fixtures` + `bench` + `report`. Sensible for most projects.

## Quick start

### Defaults (most common)

```toml
[dependencies]
dev-tools = "0.9.5"
```

You get: `report` (always), `fixtures`, `bench`.

### Just the schema (lightest)

```toml
[dependencies]
dev-tools = { version = "0.9.5", default-features = false }
```

You get: `report` only. No `fixtures`, no `bench`. Ideal when you
just need to consume reports produced elsewhere.

### Specific features

Pick exactly what you need. Examples:

```toml
# Async-heavy service: schema + async helpers, no fixtures/bench.
dev-tools = { version = "0.9.5", default-features = false, features = ["async"] }
```

```toml
# Defaults plus async (additive).
dev-tools = { version = "0.9.5", features = ["async"] }
```

```toml
# Defaults plus chaos and stress.
dev-tools = { version = "0.9.5", features = ["chaos", "stress"] }
```

```toml
# Library shipping to production: coverage + security + deps + flaky.
dev-tools = { version = "0.9.5", features = ["coverage", "security", "deps", "flaky"] }
```

```toml
# Mutation-testing + fuzz harness with default test environments.
dev-tools = { version = "0.9.5", features = ["mutate", "fuzz"] }
```

```toml
# Everything (CI verification rigs, AI agents that drive the whole suite).
dev-tools = { version = "0.9.5", features = ["full"] }
```

### Toggle features off

`fixtures` and `bench` are on by default. Disable them with
`default-features = false` and re-add only what you want:

```toml
# Async + chaos, NO fixtures/bench.
dev-tools = { version = "0.9.5", default-features = false, features = ["async", "chaos"] }
```

## API map

Each feature exposes the underlying sub-crate at a fixed path:

| Feature | Module path | Top-level types |
|---|---|---|
| _always_ | `dev_tools::report` | `Report`, `CheckResult`, `Verdict`, `Severity`, `Evidence`, `Producer`, `MultiReport`, `Diff`, `DiffOptions` |
| _always_ | `dev_tools::producers` | `cargo_test_producer`, `clippy_producer`, `cargo_check_producer` |
| _always_ | `dev_tools::brand` | `COLOR_ACCENT`, `COLOR_PASS`, `COLOR_FAIL`, `COLOR_WARN`, `COLOR_LINT`, `COLOR_BG`, `COLOR_FG`, `FOOTER` |
| `fixtures` | `dev_tools::fixtures` | `TempProject`, `FixtureProducer`, `Golden`, `BinaryGolden`, `tree::*`, `adversarial::*`, `mock::*` |
| `bench` | `dev_tools::bench` | `Benchmark`, `BenchmarkResult`, `BenchProducer`, `Threshold`, `CompareOptions`, `Baseline`, `BaselineStore`, `JsonFileBaselineStore` |
| `async` | `dev_tools::r#async` | `run_with_timeout`, `join_all_with_timeout`, `AsyncProducer`, `BlockingAsyncProducer`, `deadlock::*`, `tasks::*`, `shutdown::*` |
| `stress` | `dev_tools::stress` | `StressRun`, `StressResult`, `SoakRun`, `Workload`, `LatencyTracker`, `LatencyStats`, `StressProducer` |
| `chaos` | `dev_tools::chaos` | `FailureSchedule`, `FailureMode`, `assert_recovered`, `ChaosProducer`, `io::*`, `latency::*`, `crash::*` |
| `coverage` | `dev_tools::coverage` | `CoverageRun`, `CoverageResult`, `CoverageThreshold`, `Baseline`, `JsonFileBaselineStore`, `CoverageProducer`, `FileCoverage` |
| `security` | `dev_tools::security` | `AuditRun`, `AuditScope`, `AuditResult`, `Finding`, `FindingSource`, `AuditProducer` |
| `deps` | `dev_tools::deps` | `DepCheck`, `DepScope`, `DepKind`, `DepResult`, `UnusedDep`, `OutdatedDep`, `DepProducer` |
| `ci` | `dev_tools::ci` | `Generator`, `Target`, `PathDep` |
| `fuzz` | `dev_tools::fuzz` | `FuzzRun`, `FuzzBudget`, `FuzzFindingKind`, `FuzzResult`, `Sanitizer`, `FuzzProducer` |
| `flaky` | `dev_tools::flaky` | `FlakyRun`, `FlakyResult`, `Classification`, `TestReliability`, `FlakyProducer` |
| `mutate` | `dev_tools::mutate` | `MutateRun`, `MutateResult`, `MutateThreshold`, `SurvivingMutant`, `FileBreakdown`, `MutateProducer` |

### Built-in producers

The `producers` module ships three reusable `Producer` implementations
that wrap common `cargo` subcommands. Each one spawns a subprocess,
parses its output, and emits one `CheckResult` per result. Subprocess
failures map to a `CheckResult::fail` named `subprocess::spawn` — no
panics.

```rust,no_run
use dev_tools::producers::{cargo_test_producer, clippy_producer};
use dev_tools::report::Producer;

let report = cargo_test_producer("my-crate", "0.1.0").produce();
let lints = clippy_producer("my-crate", "0.1.0").produce();
```

`cargo_test_producer` maps libtest output: pass → `Pass`, FAILED →
`Fail` + `Severity::Error`, ignored → `Skip`. `clippy_producer` and
`cargo_check_producer` parse `--message-format=json`: warnings →
`Warn` + `Severity::Warning`, errors → `Fail` + `Severity::Error`.
Source locations propagate as `Evidence::FileRef`; the rendered
diagnostic propagates as `Evidence::Snippet`.

`CARGO_TARGET_DIR`, `CARGO`, and the rest of the parent environment
are inherited by the subprocess. Set a working directory with
`.in_dir(path)` on any of the producer types.

### HTML meta-report

`MultiReport::to_html()` (via the `MultiReportHtmlExt` trait, which the
prelude exports) renders a `MultiReport` as a self-contained HTML
document — inline CSS, inline SVG charts, no JavaScript dependencies,
no external assets.

```rust
use dev_tools::prelude::*;

let mut bench = Report::new("my-crate", "0.1.0").with_producer("dev-bench");
bench.push(CheckResult::pass("hot_path"));
let mut multi = MultiReport::new("my-crate", "0.1.0");
multi.push(bench);

let html = multi.to_html();
std::fs::write("report.html", html).unwrap();
```

The output is byte-deterministic for a given input. Sections: header
with the overall verdict badge, summary counts + stacked verdict bar,
duration histogram (only when at least one check has a duration), and
one collapsible `<details>` per producer (auto-opened when that
producer has failures or warnings). Colors come from CSS custom
properties at the top of the document, sourced from the `brand`
module — replacing those constants re-themes every future report.

## Prelude

`use dev_tools::prelude::*;` pulls in the schema types in one line:

```rust
use dev_tools::prelude::*;

let mut r = Report::new("my-crate", "0.1.0");
r.push(CheckResult::pass("compile"));
r.finish();
assert!(r.passed());
```

The prelude includes: `Report`, `CheckResult`, `Verdict`, `Severity`,
`Evidence`, `EvidenceData`, `EvidenceKind`, `FileRef`, `MultiReport`,
`Diff`, `DiffOptions`, `DurationRegression`, `SeverityChange`,
`Producer`.

When the `async` feature is on, `dev_tools::prelude::async_prelude::*`
additionally includes `run_with_timeout`, `join_all_with_timeout`,
`AsyncCheck`, `AsyncProducer`, and `BlockingAsyncProducer`.

## Putting it together

### Build a report by hand

```rust
use dev_tools::prelude::*;

let mut r = Report::new("my-crate", "0.1.0").with_producer("ci-harness");
r.push(CheckResult::pass("compile"));
r.push(
    CheckResult::fail("test::round_trip", Severity::Error)
        .with_detail("expected 42, got 41")
);
r.finish();

println!("{}", r.to_json().unwrap());
```

### Compose multiple producers — sync

The `full_run!` macro takes any number of `Producer` values and emits
one [`MultiReport`](https://docs.rs/dev-report/latest/dev_report/struct.MultiReport.html)
sharing one subject/version:

```rust
use dev_tools::{full_run, fixtures, bench};

let fixture = fixtures::FixtureProducer::new("temp_lifecycle", "0.1.0", || {
    let _p = fixtures::TempProject::new()
        .with_file("README.md", "hi")
        .build()?;
    Ok(())
});

let benchmark = bench::BenchProducer::new(
    || {
        let mut b = bench::Benchmark::new("noop");
        for _ in 0..100 { b.iter(|| std::hint::black_box(1 + 1)); }
        b.finish()
    },
    "0.1.0",
    None,
    bench::Threshold::regression_pct(20.0),
);

let multi = full_run!("my-crate", "0.1.0"; fixture, benchmark);
println!("{}", multi.to_json().unwrap());
```

### Cross-dimension pipeline (sketch)

With coverage / security / mutation / flake features enabled, the same
`full_run!` macro combines producers from every verification dimension:

```rust,no_run
use dev_tools::{full_run, coverage, security, mutate, flaky};
use dev_tools::report::Producer;

let cov = coverage::CoverageProducer::new(
    coverage::CoverageRun::new("my-crate", "0.1.0"),
    coverage::CoverageThreshold::min_line_pct(80.0),
);
let sec = security::AuditProducer::new(
    security::AuditRun::new("my-crate", "0.1.0").scope(security::AuditScope::All),
);
let mut_ = mutate::MutateProducer::new(
    mutate::MutateRun::new("my-crate", "0.1.0"),
    mutate::MutateThreshold::min_kill_pct(70.0),
);
let flk = flaky::FlakyProducer::new(flaky::FlakyRun::new("my-crate", "0.1.0").iterations(20));

let multi = full_run!("my-crate", "0.1.0"; cov, sec, mut_, flk);
let html = dev_tools::html::multi_report_to_html(&multi);
std::fs::write("report.html", html).unwrap();
```

### Compose multiple producers — async

With the `async` feature, the [`async_full_run!`] macro does the same
for async producers (futures returning `Report`):

```rust,ignore
use dev_tools::async_full_run;
use dev_tools::report::{CheckResult, Report};

async fn produce_a() -> Report {
    let mut r = Report::new("crate", "0.1.0").with_producer("a");
    r.push(CheckResult::pass("ok"));
    r.finish();
    r
}

async fn produce_b() -> Report {
    let mut r = Report::new("crate", "0.1.0").with_producer("b");
    r.push(CheckResult::pass("ok"));
    r.finish();
    r
}

# async fn ex() {
let multi = async_full_run!("crate", "0.1.0"; produce_a(), produce_b()).await;
println!("{}", multi.to_json().unwrap());
# }
```

`full_run!` expects sync `Producer` values; `async_full_run!` expects
futures. Mix and match — use `BlockingAsyncProducer` to wrap an async
producer into a sync `Producer` if you need to combine them in
`full_run!`.

### Compute a diff between runs

`Report::diff(&baseline) -> Diff` and `Report::diff_with(&baseline, &opts) -> Diff`
report regressions:

```rust
use dev_tools::prelude::*;

let prev: Report = serde_json::from_str(&std::fs::read_to_string("baseline.json").unwrap()).unwrap();
let curr: Report = produce_current_report();

let diff = curr.diff(&prev);
if !diff.is_clean() {
    eprintln!("regressions detected!");
    eprintln!("{}", diff.to_terminal()); // requires `report` feature gated `terminal`
}
# fn produce_current_report() -> Report { Report::new("c","0.1.0") }
```

## Why a verification suite

`cargo test` is necessary, not sufficient. Code that compiles and
passes the test suite can still:

- Behave correctly in unit tests but fail under sustained load
- Introduce silent performance regressions against the baseline
- Break async shutdown or leak tasks
- Hide race conditions and memory leaks
- Look clean while being one specific input away from a crash
- Have tests that pass without asserting anything meaningful
- Drag in unused, outdated, or vulnerable dependencies
- Pass on the developer's machine and fail intermittently in CI

The `dev-*` suite produces machine-readable evidence at every one of
those layers. Use it from `cargo test`, from a CI gate, from a release
pipeline — or to give an AI assistant something concrete to validate
its own work against. The output is the same.

## Command-line tools

Most of the suite is library-only — you drive it from a `tests/` file
or a custom binary, and consume the resulting JSON. One sub-crate ships
an actual CLI:

### `dev-ci` — generate a calibrated CI workflow

[`dev-ci`](https://crates.io/crates/dev-ci) emits a GitHub Actions
workflow YAML tuned to the dev-* features you enable. Install once:

```bash
cargo install dev-ci
```

Quick examples:

```bash
# Default: test job on ubuntu-latest, written to .github/workflows/ci.yml
dev-ci generate

# Multi-OS matrix + lint/fmt/docs/MSRV jobs
dev-ci generate \
    --matrix ubuntu-latest,macos-latest,windows-latest \
    --with clippy,fmt,docs,msrv \
    --msrv 1.85

# Project that uses path-deps to sibling repos
dev-ci generate \
    --features fixtures,bench,coverage \
    --path-dep dev-report=https://github.com/jamesgober/dev-report.git \
    --path-dep dev-fixtures=https://github.com/jamesgober/dev-fixtures.git

# Preview without writing
dev-ci generate --print
```

The generator stays in sync with the dev-* feature set — turning on
`coverage` adds an `llvm-cov` job, `security` adds a `cargo-audit`
job, `mutate` adds a `cargo-mutants` job, and so on. See
[`dev-ci`'s README](https://github.com/jamesgober/dev-ci#readme) for
the full reference.

## Status

`v0.9.x` is the pre-1.0 stabilization line across all sub-crates.
APIs are expected to be near-final; minor adjustments may still
happen ahead of `1.0`. The schema (`dev-report`) stays at
`schema_version = 1` through this line.

Sub-crate dependency constraints are pinned at `^0.9` (any 0.9.x).
The umbrella crate does not require a coordinated patch release of
the sibling crates; you can safely use `dev-tools 0.9.5` alongside
sibling crates at any 0.9.x version.

## Roadmap

The collection is iterative. The current 14 crates are listed in the
[feature table](#feature-flags) above; the table below tracks
libraries planned for upcoming slices.

| Crate | Purpose | Status |
|---|---|---|
| `dev-property` | Property-based testing wrapper (proptest / quickcheck) | 📋 Planned |
| `dev-sanitizer` | ASAN / MSAN / TSAN integration | 📋 Planned |
| `dev-build` | Build-time and binary-size regression tracking | 📋 Planned |
| `dev-doc` | Doc-test orchestration and doc-coverage gates | 📋 Planned |
| `dev-msrv` | MSRV verification across the dep tree | 📋 Planned |

Legend: ✅ Released &middot; 🧪 Testing &middot; 🚧 In development &middot; 📋 Planned.

Got a suggestion? Open an issue on
[`dev-tools`](https://github.com/jamesgober/dev-tools/issues) — the
collection is shaped by what real Rust crates actually need at
release time.

## Minimum supported Rust version

`1.85` — pinned in `Cargo.toml` via `rust-version` and verified by
the MSRV job in CI. (Bumped from 1.75 to match sibling sub-crate
MSRVs after their transitive deps required Rust 1.81+ and
`edition2024`.)

## License

Apache-2.0. See [LICENSE](LICENSE).





<!-- COPYRIGHT
---------------------------------->
<div align="center">
    <br>
    <h2></h2>
    Copyright &copy; 2026 James Gober.
</div>
