<h1 align="center">
    <strong>dev-tools</strong>
    <br>
    <sup><sub>VERIFICATION TOOLKIT FOR AI-ASSISTED RUST DEVELOPMENT</sub></sup>
</h1>

<p align="center">
    <a href="https://crates.io/crates/dev-tools"><img alt="crates.io" src="https://img.shields.io/crates/v/dev-tools.svg"></a>
    <a href="https://crates.io/crates/dev-tools"><img alt="downloads" src="https://img.shields.io/crates/d/dev-tools.svg"></a>
    <a href="https://github.com/jamesgober/dev-tools/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/jamesgober/dev-tools/actions/workflows/ci.yml/badge.svg"></a>
    <a href="https://docs.rs/dev-tools"><img alt="docs.rs" src="https://docs.rs/dev-tools/badge.svg"></a>
</p>

<p align="center">
    Umbrella crate over the <code>dev-*</code> verification suite.
</p>

---

## What it is

`dev-tools` is the convenient one-import entry point for the `dev-*`
verification suite. Instead of pulling in seven separate crates and
wiring them together, you add one dependency and turn on the parts
you need with feature flags.

The suite gives an AI agent (or a CI gate) machine-readable evidence
about a Rust project:

- Did it compile?
- Did tests pass?
- Did performance regress?
- Did async code hang?
- Did the system collapse under load?
- Did failure recovery work?

Output flows through [`dev-report`](https://crates.io/crates/dev-report)
— a stable, versioned JSON schema. No log parsing.

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
| `full` | — | All of the above | Kitchen-sink verification rigs. |

**Default** = `fixtures` + `bench` + `report`. Sensible for most projects.

## Quick start

### Defaults (most common)

```toml
[dependencies]
dev-tools = "0.9.2"
```

You get: `report` (always), `fixtures`, `bench`.

### Just the schema (lightest)

```toml
[dependencies]
dev-tools = { version = "0.9.2", default-features = false }
```

You get: `report` only. No `fixtures`, no `bench`. Ideal when you
just need to consume reports produced elsewhere.

### Specific features

Pick exactly what you need. Examples:

```toml
# Async-heavy service: schema + async helpers, no fixtures/bench.
dev-tools = { version = "0.9.2", default-features = false, features = ["async"] }
```

```toml
# Defaults plus async (additive).
dev-tools = { version = "0.9.2", features = ["async"] }
```

```toml
# Defaults plus chaos and stress.
dev-tools = { version = "0.9.2", features = ["chaos", "stress"] }
```

```toml
# Everything (CI verification rigs, AI agents that drive the whole suite).
dev-tools = { version = "0.9.2", features = ["full"] }
```

### Toggle features off

`fixtures` and `bench` are on by default. Disable them with
`default-features = false` and re-add only what you want:

```toml
# Async + chaos, NO fixtures/bench.
dev-tools = { version = "0.9.2", default-features = false, features = ["async", "chaos"] }
```

## API map

Each feature exposes the underlying sub-crate at a fixed path:

| Feature | Module path | Top-level types |
|---|---|---|
| _always_ | `dev_tools::report` | `Report`, `CheckResult`, `Verdict`, `Severity`, `Evidence`, `Producer`, `MultiReport`, `Diff`, `DiffOptions` |
| `fixtures` | `dev_tools::fixtures` | `TempProject`, `FixtureProducer`, `Golden`, `BinaryGolden`, `tree::*`, `adversarial::*`, `mock::*` |
| `bench` | `dev_tools::bench` | `Benchmark`, `BenchmarkResult`, `BenchProducer`, `Threshold`, `CompareOptions`, `Baseline`, `BaselineStore`, `JsonFileBaselineStore` |
| `async` | `dev_tools::r#async` | `run_with_timeout`, `join_all_with_timeout`, `AsyncProducer`, `BlockingAsyncProducer`, `deadlock::*`, `tasks::*`, `shutdown::*` |
| `stress` | `dev_tools::stress` | `StressRun`, `StressResult`, `SoakRun`, `Workload`, `LatencyTracker`, `LatencyStats`, `StressProducer` |
| `chaos` | `dev_tools::chaos` | `FailureSchedule`, `FailureMode`, `assert_recovered`, `ChaosProducer`, `io::*`, `latency::*`, `crash::*` |

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

AI can generate code quickly. Without verification, AI-generated code can:

- Compile but behave incorrectly
- Pass simple tests but fail under load
- Introduce performance regressions
- Break async shutdown
- Leak memory
- Hide race conditions
- Look clean while being fragile

The `dev-*` suite gives an AI agent a structured way to validate its
own work before a human has to trust it.

## Status

`v0.9.x` is the pre-1.0 stabilization line across all sub-crates.
APIs are expected to be near-final; minor adjustments may still
happen ahead of `1.0`. The schema (`dev-report`) stays at
`schema_version = 1` through this line.

Sub-crate dependency constraints are pinned at `^0.9` (any 0.9.x).
The umbrella crate does not require a coordinated patch release of
the sibling crates; you can safely use `dev-tools 0.9.2` alongside
sibling crates at any 0.9.x version.

## Minimum supported Rust version

`1.85` — pinned in `Cargo.toml` via `rust-version` and verified by
the MSRV job in CI. (Bumped from 1.75 to match sibling sub-crate
MSRVs after their transitive deps required Rust 1.81+ and
`edition2024`.)

## License

Apache-2.0. See [LICENSE](LICENSE).
