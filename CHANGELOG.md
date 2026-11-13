# Changelog

## [Unreleased]

## [0.11.0] - 2026-11-12

Moves to the latest sub-crate releases, all from crates.io, and fixes
how `dev test` reads test results.

### Breaking

- **`dev-security` bumped from `0.9` to `0.10`, and `dev-fuzz` from
  `0.9` to `0.10`.** Both are re-exported (`dev_tools::security`,
  `dev_tools::fuzz`). Their check names now identify each finding
  (`security::<source>::<id>::<crate>@<version>`,
  `fuzz::<target>::<kind>::<id>`), so baselines that match on the old
  names need updating once.

### Fixed

- 0.10.0 did not build against the current `dev-security`: its
  requirement `0.9.3` rejected 0.10. The floors now match the released
  sub-crates.
- The `cargo test` producer dropped tests ignored with a reason
  (`test x ... ignored, needs network`), and read result-shaped lines
  anywhere in the output, so a failing test that printed
  `test y ... ok` added a phantom passing check. Results are now read
  only from the list after each `running N tests` header, up to
  `failures:` or `test result:`, and `ignored, <reason>` counts as
  ignored.

### Changed

- Every sub-crate, and `dev-report`, now comes from crates.io instead
  of a sibling checkout, so a plain clone builds and CI tests what
  users get. Floors: `dev-report` 0.9.9, `dev-fixtures` 0.9.6,
  `dev-bench` 0.10.2, `dev-async` 0.9.6, `dev-stress` 0.9.6,
  `dev-chaos` 0.9.6, `dev-coverage` 0.9.3, `dev-security` 0.10.1,
  `dev-deps` 0.9.3, `dev-ci` 0.9.4, `dev-fuzz` 0.10.0, `dev-flaky`
  0.9.3, `dev-mutate` 0.9.4.

[0.11.0]: https://github.com/jamesgober/dev-tools/releases/tag/v0.11.0

## [0.10.0] - 2026-10-09

Moves to `dev-bench` 0.10 and the reviewed sub-crate releases, adds
time limits and `dev doctor`, fixes the `dev` CLI's version table and
build-failure handling, and rolls the library MSRV back to Rust 1.75.
This replaces the unreleased 0.9.10.

### Breaking

- **`dev-bench` dependency bumped from `0.9` to `0.10`.** `dev_tools::bench`
  re-exports `dev-bench`, so its major version is part of this crate's
  API. `dev-bench` 0.10 moved to `mod-alloc` 1.0 (see its changelog);
  code that only goes through `dev_tools::bench` needs no changes.
  Projects that also depend on `mod-alloc` 0.9 directly should move to
  `mod-alloc` 1.

### Added

- **`dev doctor`**: checks `rustc`, `cargo`, nightly availability, and
  every cargo plugin the CLI shells out to (`cargo-llvm-cov`,
  `cargo-audit`, `cargo-deny`, `cargo-udeps`, `cargo-outdated`,
  `cargo-fuzz`, `cargo-mutants`), with install commands for the missing
  ones. `--json` emits the same data for CI gates.
- **`dev async`**, **`dev stress`**, **`dev chaos`**: these crates are
  library helpers that need user code, so the commands explain that and
  show a library example instead of failing with "unrecognized
  subcommand". `--json` is supported.
- Time limits. Each producer in `producers` gained a `timeout(Duration)`
  builder: when it expires the whole cargo process tree, test binaries
  included, is killed, finished results are kept, and a
  `cargo::<subcommand>::timeout` failure is added. The CLI exposes
  `--timeout` on `dev test`, `dev clippy`, `dev check` and
  `dev coverage`, `--run-timeout` on `dev fuzz` and
  `--iteration-timeout` on `dev flaky`.

### Fixed

- **A build failure passed.** When the crate did not compile, `cargo
  test` printed no test lines, the report had no checks, its verdict was
  Skip, and `dev test` exited 0. The same happened to `dev clippy` /
  `dev check` when cargo failed without a JSON diagnostic (broken
  manifest, missing clippy). A non-zero cargo exit with no failing item
  now adds a `cargo::<subcommand>` failure carrying the tail of stderr.
- `dev version` showed the versions in the `Cargo.lock` packaged when
  dev-tools was published, not the versions actually linked. After
  `cargo install`, a newer sibling patch (for example `dev-report`
  0.9.7) was reported as the old one. Every sibling crate now exports
  `VERSION`, and the table reads those. Crates whose feature is off show
  as "not built in", and the JSON form gains a `linked` field. The
  `build.rs` that parsed `Cargo.lock` is gone.
- `dev doctor` reported cargo-llvm-cov, cargo-udeps, cargo-outdated and
  cargo-mutants as missing when they were installed: it ran the
  `cargo-<name>` binary directly, which most plugins reject. It now
  probes `cargo <name> --version`, the way the CLI runs them.
- Subcommand `--version` output (`dev coverage --version` and the rest)
  was hard-coded to old numbers; it now shows the linked crate version.
- A diagnostic without a code and a first line over 80 bytes could
  panic when a multi-byte character straddled the cut. The name is now
  cut on a character boundary.
- A compiler message without a `spans` field was dropped.
- The subject name and version read from `Cargo.toml` were wrong for
  `version.workspace = true` (it became `.workspace = true`) and for
  values followed by a comment. Workspace-inherited versions are now
  read from the workspace root's `[workspace.package]`.
- `dev html` on a `MultiReport` that failed to parse reported the error
  from trying it as a single `Report`; it now reports the `MultiReport`
  error.
- The HTML duration histogram overflowed on very large durations.
- `dev test --full` help claimed it also runs coverage, audit and deps;
  it runs test, clippy and check.

### Changed

- **Library MSRV rolled back from `1.85` to `1.75`.** The `dev`
  binary still needs 1.85 because of clap. CI now checks both: the
  library (`--features full`) on 1.75 against an MSRV-compatible
  lockfile, and the binary on 1.85. Before, only the binary was built.
- **`dev-ci` is pulled with `default-features = false`**, so the `ci`
  feature does not bring in clap or its 1.85 requirement.
- Sibling dependency floors raised to the releases that export
  `VERSION`: dev-report 0.9.8, dev-fixtures / dev-async / dev-stress /
  dev-chaos 0.9.5, dev-coverage / dev-deps / dev-fuzz / dev-flaky 0.9.2,
  dev-security / dev-ci / dev-mutate 0.9.3, dev-bench 0.10.
- The local `mod-tempdir` / `mod-alloc` clones in CI moved to
  `../../mod-collection/`, matching the sibling crates' path
  dependencies.

### Documentation

- README: command list includes `version`, `doctor`, `async`, `stress`
  and `chaos`; new "Time limits" section; the dev-coverage row said
  "kill rates"; the Status and MSRV sections are current; install
  snippets name 0.10.

[0.10.0]: https://github.com/jamesgober/dev-tools/releases/tag/v0.10.0

## [0.9.9] - 2026-05-12

Pre-release audit follow-up. Documentation-only patch — no code or
behavior changes.

### Fixed

- `build.rs` doc comment no longer contains the word `robust`. The word slipped into the build-script's module-level documentation during the v0.9.8 work and was caught by the post-publish banned-words audit. Pure docs-only change to an internal file; the build script's behavior, output, and the resulting `siblings.rs` are byte-identical to v0.9.8.

### Notes

- No runtime change. Library and CLI users get the same compiled behavior as v0.9.8.
- No new dependencies, no MSRV change.
- The next dev-tools release after this one will fold in any further pre-RC polish surfaced by ongoing audits. This release is intentionally minimal.

[0.9.9]: https://github.com/jamesgober/dev-tools/releases/tag/v0.9.9

## [0.9.8] - 2026-05-12

Final defer-list clearout ahead of `1.0`-RC.

### Added

- **`build.rs`-generated component table**. `src/bin/dev.rs` no longer carries a hand-maintained `SIBLINGS` const. A new `build.rs` parses `Cargo.lock` at compile time and emits `siblings.rs` into `$OUT_DIR`, which the binary `include!`s. Bumping a sibling crate now updates `dev version`'s output automatically on the next build — drift is structurally impossible. Degraded mode: if `Cargo.lock` is missing or unparseable, the generated table uses `"0.0.0"` placeholders rather than aborting the build.
- **`dev version --json`** emits the component table as a structured JSON object: `{ "binary": { "name", "version" }, "components": [{ "alias", "name", "version" }, ...] }`. The flag composes with the filtered form (`dev version coverage --json` returns a one-element `components` array). Useful for dashboards, CI comparators, and anything that wants to parse the version surface programmatically.

### Changed

- `dev version`'s footer tip line now mentions `--json` alongside the `<name>` filter form.

### Internal

- New `normalize_component()`, `lookup_sibling()`, and `unknown_component_msg()` helpers in `src/bin/dev.rs`. The two component-resolution paths (table-and-styled vs. JSON) now share normalization, lookup, and error-formatting code instead of duplicating it.

### Notes

- No new runtime dependencies. `serde_json` was already a direct dep of `dev-tools`; the `--json` flag uses its existing `json!` macro.
- No sub-crate version bumps. Every sibling stays at its current 0.9.x patch version.
- MSRV unchanged at Rust 1.85.

[0.9.8]: https://github.com/jamesgober/dev-tools/releases/tag/v0.9.8

## [0.9.7] - 2026-05-12

CLI version surface.

### Added

- New `dev version` subcommand prints a stylized component table — the dev binary version on top, every sub-crate listed by name with its compiled-in version, plus a tip pointing at the `dev version <name>` filtered form.
- `dev version <name>` prints just one component (e.g. `dev version coverage` → `dev-coverage 0.9.1`). Accepts either the short alias (`coverage`) or the full crate name (`dev-coverage`); both resolve to the same row. Unknown names error with a list of valid options.
- Each subprocess-wrapping subcommand now reports its underlying sub-crate's version through `--version`: `dev coverage --version` → `dev-coverage 0.9.1`, `dev audit --version` → `dev-audit 0.9.2`, `dev ci --version` → `dev-ci 0.9.2`, etc. Implemented via per-subcommand `#[command(version = "...")]` overrides on the args structs.
- `--version` / `-V` is now propagated to every subcommand via clap's `propagate_version = true`, so the flag is uniformly available everywhere instead of root-only.

### Internal

- New `SIBLINGS` const-table in `src/bin/dev.rs` keeps the sub-crate version map in one place. Hand-maintained; mirrors the `^0.9` pins in `Cargo.toml`. Doc-comment flags the keep-in-sync requirement.

[0.9.7]: https://github.com/jamesgober/dev-tools/releases/tag/v0.9.7

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
