//! Use the `ci` feature through `dev_tools::ci` to generate a GitHub
//! Actions workflow that runs the full `dev-*` suite. Prints the YAML
//! to stdout.
//!
//! Requires the `ci` feature:
//!
//! ```text
//! cargo run --example ci_generate --features ci > .github/workflows/ci.yml
//! ```
//!
//! The output uses `actions/checkout@v5`, `Swatinem/rust-cache@v2`, and
//! the patterns the dev-* suite uses for its own CI.

use dev_tools::ci::{Generator, PathDep, Target};

fn main() {
    let yaml = Generator::new()
        .target(Target::GitHubActions)
        .workflow_name("CI")
        .branches(["main"])
        .matrix_os(["ubuntu-latest", "macos-latest", "windows-latest"])
        .with_workspace()
        .with_all_features_build()
        .with_clippy()
        .with_fmt()
        .with_docs()
        .with_msrv("1.85")
        .with_path_dep(PathDep::new(
            "dev-report",
            "https://github.com/jamesgober/dev-report.git",
        ))
        .with_path_dep(PathDep::new(
            "dev-tools",
            "https://github.com/jamesgober/dev-tools.git",
        ))
        .generate();

    print!("{yaml}");
}
