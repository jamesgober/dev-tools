# Changelog

## [Unreleased]

### Added

- New `dev_tools::producers` module shipping three reusable `Producer` implementations driven by `cargo` subprocesses:
  - `cargo_test_producer(subject, version)`: runs `cargo test --no-fail-fast`, parses libtest's per-test output, emits one `CheckResult` per test (pass / fail / skip for ignored).
  - `clippy_producer(subject, version)`: runs `cargo clippy --message-format=json`, maps each diagnostic to a `CheckResult` (warning → warn + `Severity::Warning`; error → fail + `Severity::Error`). Source location propagates as `Evidence::FileRef`; rendered diagnostic text propagates as `Evidence::Snippet`.
  - `cargo_check_producer(subject, version)`: same diagnostic-to-CheckResult mapping as the clippy producer, but for `cargo check`.
  - Subprocess failures become a `CheckResult::fail` with `Severity::Critical` named `subprocess::spawn` inside the produced report. No panics. `CARGO_TARGET_DIR`, `CARGO`, and the rest of the parent environment are inherited.
  - All three producers carry a builder method `in_dir(path)` to set the working directory.
- New `dev_tools::brand` module exposing color and footer constants for the HTML meta-report (`COLOR_ACCENT`, `COLOR_PASS`, `COLOR_FAIL`, `COLOR_WARN`, `COLOR_LINT`, `COLOR_BG`, `COLOR_FG`, `FOOTER`). Values are placeholders; the real palette lands in a later release when the brand kit is finalized.
- New `dev_tools::html` module and `MultiReportHtmlExt` trait. Calling `multi.to_html()` produces a self-contained HTML document: inline CSS, inline SVG charts (stacked verdict bar + duration histogram), no JavaScript dependencies, no external assets. Collapse/expand uses native HTML5 `<details>`; producers with failures or warnings open by default. Colors come from CSS custom properties set from the `brand` module so theming is a one-line change. Output is byte-deterministic for a given input (no clock reads, no random IDs).
- `examples/cargo_test_producer.rs`, `examples/clippy_producer.rs`, `examples/cargo_check_producer.rs` demonstrate each producer's API surface. The example runs are gated by the `DEV_TOOLS_EXAMPLE_RUN` env var so CI does not pay for a recursive cargo invocation on every example build.
- `examples/html_meta_report.rs` builds a synthetic three-producer `MultiReport`, renders it via `to_html()`, and writes the result to `target/dev-tools-examples/meta-report.html` (or a path of the user's choosing).
- `MultiReportHtmlExt` is exported through `dev_tools::prelude` so `use dev_tools::prelude::*;` brings `.to_html()` into scope alongside the other report types.

### Changed

- `serde` and `serde_json` are now direct dependencies of `dev-tools` (previously transitive via `dev-report`). Used by the `producers` module to parse `cargo --message-format=json` output.
- `chrono` added as a `[dev-dependencies]` entry (also previously transitive) for deterministic timestamps in the `html` module's tests.
- CI: `actions/checkout` bumped to `v5` (was `v4`); removes Node 20 deprecation warnings.

## [0.9.2] - 2026-05-10

### Added

- New `async_full_run!` macro (gated by the `async` feature): async equivalent of `full_run!` for futures returning `Report`. Awaits each future in sequence and pushes its result into a `MultiReport`.
- `prelude` expanded with `DurationRegression` and `SeverityChange` types from `dev-report`.
- New `prelude::async_prelude` module (gated by `async`) re-exporting `dev-async`'s key items: `run_with_timeout`, `join_all_with_timeout`, `AsyncCheck`, `AsyncProducer`, `BlockingAsyncProducer`.

### Documentation

- README rewrite:
  - Feature flag table prominently at the top with use-case column.
  - Quick-start section now shows install snippets for defaults, no-default-features (schema only), specific feature combinations, and feature-toggling.
  - New API map table mapping each feature to its module path and top-level types.
  - New "Composing producers" section covering both `full_run!` and `async_full_run!` with full examples.
  - "Diff between runs" example added.

[0.9.2]: https://github.com/jamesgober/dev-tools/releases/tag/v0.9.2

## [0.9.1] - 2026-05-09

### Added

- `dev_tools::prelude` module re-exporting the most common items across the suite: `CheckResult`, `Diff`, `DiffOptions`, `Evidence`, `EvidenceData`, `EvidenceKind`, `FileRef`, `MultiReport`, `Producer`, `Report`, `Severity`, `Verdict`. Per-feature items (e.g. `fixtures::TempProject`) are NOT in the prelude — pull them in via the re-exported sub-crate modules.

### Notes

- Sub-crate dependency constraints remain `^0.9` (matches any 0.9.x). This keeps `dev-tools` compatible with both the prior 0.9.0 patch line and the new 0.9.1 features, and decouples `dev-tools` from coordinated patch releases of the sibling crates.

[0.9.1]: https://github.com/jamesgober/dev-tools/releases/tag/v0.9.1

## [0.9.0] - 2026-05-08

### Changed

- Bumped all sibling-crate dependencies to `0.9`:
  - `dev-report = "0.9"`
  - `dev-fixtures = "0.9"`
  - `dev-bench = "0.9"`
  - `dev-async = "0.9"`
  - `dev-stress = "0.9"`
  - `dev-chaos = "0.9"`

### Added

- `full_run!` macro for combining multiple `dev_report::Producer` results into a single `MultiReport`. Pure composition; defines no new types.
- Smoke tests covering each feature flag and a fixtures+bench `full_run!` integration.

### Note

The umbrella crate stays thin per DIRECTIVES § 2: no new types, no
external dependencies. The `full_run!` macro is sugar over
`MultiReport::push` and `Producer::produce`, both of which live in
sibling crates.

[0.9.0]: https://github.com/jamesgober/dev-tools/releases/tag/v0.9.0

## [0.1.0] - 2026-05-07

### Added

- Initial umbrella crate.
- Always-on re-export of `dev_report` as `dev_tools::report`.
- Optional re-exports behind feature flags:
  - `fixtures` -> `dev_tools::fixtures` (default)
  - `bench` -> `dev_tools::bench` (default)
  - `async` -> `dev_tools::async` (opt-in)
  - `stress` -> `dev_tools::stress` (opt-in)
  - `chaos` -> `dev_tools::chaos` (opt-in)
- `full` feature for all sub-crates.
- Smoke tests covering report-only and feature-enabled paths.

### Note

Name-claim release. Coordinated v0.1.0 of all sibling crates:
`dev-report`, `dev-fixtures`, `dev-bench`, `dev-async`,
`dev-stress`, `dev-chaos`.

[Unreleased]: https://github.com/jamesgober/dev-tools/compare/v0.9.2...HEAD
[0.1.0]: https://github.com/jamesgober/dev-tools/releases/tag/v0.1.0
