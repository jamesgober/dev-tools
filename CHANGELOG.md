# Changelog

## [Unreleased]

## [0.9.6] - 2026-05-12

The `dev` CLI lands. One binary, one subcommand per verification dimension.

### Added

- New `dev` binary gated behind the `cli` feature flag. `cargo install dev-tools --features cli` installs the unified CLI; library consumers continue to pay zero compile-time cost for clap when they don't opt in.
- Fourteen flat subcommands, one verb per verification dimension: `dev test`, `dev clippy`, `dev check`, `dev bench`, `dev coverage`, `dev audit`, `dev deps`, `dev fuzz <target>`, `dev mutate`, `dev flaky`, `dev ci`, `dev report <path>`, `dev diff <a> <b>`, `dev html <path>`. No nested sub-subcommands — `dev test`, not `dev test run --foo`.
- `dev test --full` runs the entire available verification stack (cargo test + clippy + check) and produces a `MultiReport`. Failures in one dimension don't short-circuit the others.
- Polished terminal renderer with ANSI color, unicode verdict glyphs (`✓` / `✗` / `⚠` / `⊘`), aligned per-check timings, and a footer summary with the overall verdict. Color is opt-in based on `IsTerminal` detection — pipe to a file or non-TTY and you get plain text.
- Every Report-producing subcommand accepts shared flags: `--out PATH` (write to file), `--format terminal|json|markdown|sarif|junit`, `--subject NAME`, `--subject-version V`, `--in DIR`, `--quiet`. Cargo.toml is auto-parsed for subject + version when the flags are omitted.
- Exit codes: `0` for clean, `1` for any failed / warned check or non-clean diff, `2` for CLI / I/O errors. Errors print to stderr.
- README `## Command-line tools` section rewritten with the full `dev` CLI surface, command table, common flags, example session, and exit-code reference. The existing `dev-ci` standalone CLI documentation is preserved below for users who only want the workflow generator.
- New `cli` feature in `Cargo.toml` pulls in `clap` (4.x, derive feature) plus every sub-crate plus the renderer features of `dev-report` (`terminal`, `markdown`, `sarif`, `junit`). The feature is fully self-contained — toggling it on enables every dimension the CLI dispatches to.

### Internal

- `src/bin/dev.rs` is the binary entry point. ~600 lines total — clap derive shell, one `run_*` function per subcommand, a `pretty_report` / `pretty_multi` renderer, and a best-effort `cargo_metadata()` helper that scrapes the subject + version out of `Cargo.toml` without spawning `cargo metadata`.

[0.9.6]: https://github.com/jamesgober/dev-tools/releases/tag/v0.9.6

## [0.9.5] - 2026-05-12

Documentation and SEO pass. No code changes.

### Changed

- Header tagline rewritten to lead with developer value rather than the umbrella's internal role. The `AI-assisted` framing is demoted to one of several use cases (CI gates, release pipelines, AI assistants — all consume the same JSON).
- Subtitle now reads `RUST VERIFICATION TOOLKIT — TESTS · BENCHES · COVERAGE · FUZZ · AUDIT` (was `VERIFICATION TOOLKIT FOR AI-ASSISTED RUST DEVELOPMENT`). Target audience widened from AI-tooling to every Rust crate maintainer.
- "What it is" and "Why a verification suite" sections rewritten with a developer-pain-first framing. The list of failure modes now leads with regressions, async hangs, and silent fragility rather than AI code generation.
- `Cargo.toml` `description` rewritten for crates.io search: leads with `Rust verification toolkit`, enumerates the actual verification dimensions, drops the `AI-assisted` framing.
- `Cargo.toml` `keywords` retuned for crates.io search: `testing`, `benchmark`, `coverage`, `fuzz`, `audit` (was `testing`, `verification`, `benchmark`, `chaos`, `ai-tools`). Five slots, devs-find-this-via-search optimized.
- Cross-wave-pipeline example heading renamed to `Cross-dimension pipeline` (no more `second-wave` references).

### Added

- New `## Command-line tools` section documents the `dev-ci` CLI (the suite's only binary): install instructions, multiple invocation patterns, link to `dev-ci`'s own README for the full reference.
- New `## Roadmap` section with a planned-libraries status table covering `dev-property`, `dev-sanitizer`, `dev-build`, `dev-doc`, `dev-msrv`. Status legend included.

[0.9.5]: https://github.com/jamesgober/dev-tools/releases/tag/v0.9.5

## [0.9.4] - 2026-05-12

Suite expansion. Seven additional `dev-*` crates are now wired into
the umbrella through new optional features.

### Added

- Seven new optional dependencies, one per new sub-crate, all pinned at `^0.9` with path-deps to sibling repos:
  - `dev-coverage` — test coverage via `cargo-llvm-cov`.
  - `dev-security` — vulnerability + policy scanning via `cargo-audit` + `cargo-deny`.
  - `dev-deps` — dependency health via `cargo-udeps` + `cargo-outdated`.
  - `dev-ci` — GitHub Actions workflow generator (`Generator` builder + `PathDep` type; the CLI binary still installs via `cargo install dev-ci`).
  - `dev-fuzz` — libFuzzer integration via `cargo-fuzz`.
  - `dev-flaky` — repeated-run flaky-test detection.
  - `dev-mutate` — mutation testing via `cargo-mutants`.
- Seven new feature flags mirror the dependencies: `coverage`, `security`, `deps`, `ci`, `fuzz`, `flaky`, `mutate`. Each gates its corresponding sub-crate exactly the same way `fixtures`, `bench`, `async`, `stress`, `chaos` gate theirs.
- `full` feature now pulls in all twelve sub-crates (was five). Use `features = ["full"]` to get the entire suite.
- Seven new feature-gated re-exports in `dev_tools::*`: `coverage`, `security`, `deps`, `ci`, `fuzz`, `flaky`, `mutate`. Each mirrors its underlying crate one-to-one. Example: `dev_tools::coverage::CoverageRun` is the same type as `dev_coverage::CoverageRun`.

### Changed

- `default = ["fixtures", "bench"]` is unchanged. New features are opt-in to keep the default build footprint small.
- README rewritten to reflect the full 13-sub-crate map (one report schema + twelve verification dimensions). The feature flag table now lists every option in one place; the API map table maps each feature to its module path and top-level types.
- CI workflow now clones every sibling crate as a path-dep in every job that runs cargo.

### Note

This release is purely additive on top of `0.9.3`. Existing code that
uses `dev_tools::report`, `dev_tools::fixtures`, `dev_tools::bench`,
etc. continues to compile unchanged. Projects opting into the new
features should set `default-features = false` if they want a minimal
build and pick exactly the verification dimensions they need.

[0.9.4]: https://github.com/jamesgober/dev-tools/releases/tag/v0.9.4

## [0.9.3] - 2026-05-12

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

[0.9.3]: https://github.com/jamesgober/dev-tools/releases/tag/v0.9.3

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

[Unreleased]: https://github.com/jamesgober/dev-tools/compare/v0.9.3...HEAD
[0.1.0]: https://github.com/jamesgober/dev-tools/releases/tag/v0.1.0
