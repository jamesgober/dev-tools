# Changelog

## [Unreleased]

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

[Unreleased]: https://github.com/jamesgober/dev-tools/compare/v0.9.0...HEAD
[0.1.0]: https://github.com/jamesgober/dev-tools/releases/tag/v0.1.0
