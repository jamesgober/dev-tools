<h1 align="center">
    <strong>dev-tools</strong>
    <br>
    <sup><sub>VERIFICATION TOOLKIT FOR AI-ASSISTED RUST DEVELOPMENT</sub></sup>
</h1>

<p align="center">
    <a href="https://crates.io/crates/dev-tools"><img alt="crates.io" src="https://img.shields.io/crates/v/dev-tools.svg"></a>
    <a href="https://docs.rs/dev-tools"><img alt="docs.rs" src="https://docs.rs/dev-tools/badge.svg"></a>
    <a href="https://github.com/jamesgober/dev-tools/blob/main/LICENSE"><img alt="License" src="https://img.shields.io/badge/license-Apache--2.0-blue.svg"></a>
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
dev-tools = "0.1"
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
dev-tools = { version = "0.1", features = ["async"] }
```

Kitchen sink (CI verification rigs, AI agents):

```toml
dev-tools = { version = "0.1", features = ["full"] }
```

Schema-only (lightest possible):

```toml
dev-tools = { version = "0.1", default-features = false }
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

`v0.1.x` is a name-claim release across all sub-crates. APIs WILL
expand significantly in `0.2.x` and beyond. Production use is
discouraged until `1.0`.

## License

Apache-2.0. See [LICENSE](LICENSE).
