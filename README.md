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
verification suite. Pick the features you need; pull them in with a
single dependency.

The suite gives an AI agent (or a CI gate) machine-readable evidence
about a Rust project:

- Did it compile?
- Did tests pass?
- Did performance regress?
- Did async code hang?
- Did the system collapse under load?
- Did failure recovery work?

## Quick start

```toml
[dependencies]
dev-tools = "0.9.1"
```

Default features include `fixtures`, `bench`, and the always-on
`report` schema.

```rust
use dev_tools::{report, fixtures, bench};

// Build a report.
let mut r = report::Report::new("my-crate", "0.1.0")
    .with_producer("ci-harness");

// Spin up a deterministic temp environment.
let project = fixtures::TempProject::new()
    .with_file("input.txt", "hello")
    .build()
    .unwrap();

// Run a benchmark and add the verdict to the report.
let mut b = bench::Benchmark::new("hot_path");
for _ in 0..1000 {
    b.iter(|| std::hint::black_box(40 + 2));
}
let result = b.finish();
r.push(result.compare_against_baseline(None, bench::Threshold::regression_pct(10.0)));

r.finish();
println!("{}", r.to_json().unwrap());
```

## Combining producers with `full_run!`

The `full_run!` macro turns a list of `dev_report::Producer` values
into a `MultiReport` — one entry per producer, all sharing the same
subject and version.

```rust
use dev_tools::{full_run, fixtures, bench, report};

let fixture = fixtures::FixtureProducer::new(
    "temp_project_lifecycle",
    "0.1.0",
    || {
        let _p = fixtures::TempProject::new()
            .with_file("README.md", "hi")
            .build()?;
        Ok(())
    },
);

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

The macro is pure composition: zero new types, zero external
dependencies. It just calls `Producer::produce()` on each argument
and pushes into a `dev_report::MultiReport`.

## Features

| Feature | Default | What it brings in |
|---|:---:|---|
| `report` | always | [`dev-report`](https://crates.io/crates/dev-report) — schema |
| `fixtures` | yes | [`dev-fixtures`](https://crates.io/crates/dev-fixtures) — test environments |
| `bench` | yes | [`dev-bench`](https://crates.io/crates/dev-bench) — performance |
| `async` | no | [`dev-async`](https://crates.io/crates/dev-async) — async validation |
| `stress` | no | [`dev-stress`](https://crates.io/crates/dev-stress) — load testing |
| `chaos` | no | [`dev-chaos`](https://crates.io/crates/dev-chaos) — failure injection |
| `full` | no | all of the above |

### Common feature combinations

Async-heavy project:

```toml
dev-tools = { version = "0.9.1", features = ["async"] }
```

Kitchen sink (CI verification rigs, AI agents):

```toml
dev-tools = { version = "0.9.1", features = ["full"] }
```

Schema-only (lightest possible):

```toml
dev-tools = { version = "0.9.1", default-features = false }
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

## Minimum supported Rust version

`1.85` — pinned in `Cargo.toml` via `rust-version` and verified by
the MSRV job in CI. (Bumped from 1.75 to match sibling sub-crate
MSRVs after their transitive deps required Rust 1.81+ and
`edition2024`.)

## License

Apache-2.0. See [LICENSE](LICENSE).
