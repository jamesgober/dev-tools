//! `dev` — the unified CLI for the `dev-*` verification collection.
//!
//! One binary, one subcommand per verification dimension. Every
//! subcommand produces a structured [`Report`] which is rendered to
//! the terminal (default) or to a file in JSON, markdown, SARIF, or
//! JUnit XML. Run `dev --help` for the full surface.

use std::fs;
use std::io::{self, IsTerminal as _, Write as _};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand, ValueEnum};

use dev_tools::report::{Diff, MultiReport, Report, Verdict};

// =============================================================================
// CLI shell
// =============================================================================

#[derive(Debug, Parser)]
#[command(
    name = "dev",
    version,
    propagate_version = true,
    about = "Rust verification toolkit. Tests, benches, coverage, fuzz, audit, mutation, more.",
    long_about = "The `dev` CLI is the umbrella entry point for the dev-* verification \
                  collection. Every subcommand drives one verification dimension and produces \
                  a structured Report. Output is rendered to the terminal by default; --format \
                  and --out control the wire format and destination."
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// Run `cargo test`, parse output, emit a Report.
    ///
    /// `--full` runs the entire available verification stack (test +
    /// clippy + check, plus coverage / audit / deps when enabled).
    Test(TestArgs),
    /// Run `cargo clippy --message-format=json` and emit a Report.
    Clippy(SimpleProducerArgs),
    /// Run `cargo check --message-format=json` and emit a Report.
    Check(SimpleProducerArgs),
    /// Run `cargo bench` and emit a Report.
    Bench(BenchArgs),
    /// Compute coverage via `cargo-llvm-cov` with optional gating.
    Coverage(CoverageArgs),
    /// Run security audit (`cargo-audit` + `cargo-deny`).
    Audit(AuditArgs),
    /// Check dependency hygiene (`cargo-udeps` + `cargo-outdated`).
    Deps(DepsArgs),
    /// Run a fuzz target via `cargo-fuzz` with a budget.
    Fuzz(FuzzArgs),
    /// Run mutation testing via `cargo-mutants` with kill-rate gate.
    Mutate(MutateArgs),
    /// Detect flaky tests via N-iteration cargo test.
    Flaky(FlakyArgs),
    /// Generate a GitHub Actions workflow YAML.
    Ci(CiArgs),
    /// Pretty-print a Report from disk.
    Report(ReportArgs),
    /// Diff two reports and show what changed.
    Diff(DiffArgs),
    /// Render a Report or MultiReport to a self-contained HTML file.
    Html(HtmlArgs),
    /// Print the version of `dev` plus every sub-crate it wraps.
    ///
    /// `dev version` lists every component. `dev version <name>` prints
    /// just one (e.g. `dev version coverage` → `dev-coverage 0.9.1`).
    Version(VersionArgs),
}

// =============================================================================
// Shared output handling
// =============================================================================

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    /// Terminal-friendly rendering with ANSI color when stdout is a TTY.
    Terminal,
    /// Pretty-printed JSON (the canonical wire format).
    Json,
    /// CommonMark markdown.
    Markdown,
    /// SARIF 2.1.0 (defect-report format; failures and warnings only).
    Sarif,
    /// Jenkins / Surefire JUnit XML.
    Junit,
}

impl OutputFormat {
    fn default_for_destination(is_file: bool) -> Self {
        if is_file {
            Self::Json
        } else {
            Self::Terminal
        }
    }
}

/// Flags that every Report-producing subcommand shares.
#[derive(Debug, Args, Clone)]
struct CommonOutputArgs {
    /// Write output to a file instead of stdout. Default format is JSON;
    /// override with `--format`.
    #[arg(long, short = 'o', global = true)]
    out: Option<PathBuf>,

    /// Output format. Default: `terminal` when writing to stdout, `json`
    /// when writing to a file.
    #[arg(long, short = 'f', value_enum, global = true)]
    format: Option<OutputFormat>,

    /// Subject name to attach to the Report. Defaults to the crate name
    /// from `Cargo.toml` (or `unknown` if not found).
    #[arg(long, global = true)]
    subject: Option<String>,

    /// Subject version. Defaults to the version from `Cargo.toml`.
    #[arg(long = "subject-version", global = true)]
    subject_version: Option<String>,

    /// Suppress terminal output entirely; rely on exit code only.
    #[arg(long, short = 'q', global = true)]
    quiet: bool,

    /// Working directory for the underlying tool. Defaults to cwd.
    #[arg(long = "in", value_name = "DIR", global = true)]
    workdir: Option<PathBuf>,
}

impl CommonOutputArgs {
    fn resolved_subject(&self) -> String {
        if let Some(s) = &self.subject {
            return s.clone();
        }
        cargo_metadata(self.workdir.as_deref())
            .map(|m| m.name)
            .unwrap_or_else(|| "unknown".to_string())
    }

    fn resolved_version(&self) -> String {
        if let Some(v) = &self.subject_version {
            return v.clone();
        }
        cargo_metadata(self.workdir.as_deref())
            .map(|m| m.version)
            .unwrap_or_else(|| "0.0.0".to_string())
    }
}

// =============================================================================
// Subcommand arg structs
// =============================================================================

#[derive(Debug, Args)]
struct TestArgs {
    /// Run the full available verification stack (test + clippy + check,
    /// plus coverage / audit / deps when their underlying tools are
    /// installed). Without `--full`, only `cargo test` runs.
    #[arg(long)]
    full: bool,

    #[command(flatten)]
    common: CommonOutputArgs,
}

#[derive(Debug, Args)]
struct SimpleProducerArgs {
    #[command(flatten)]
    common: CommonOutputArgs,
}

#[derive(Debug, Args)]
struct BenchArgs {
    /// Pass `--workspace` to cargo bench.
    #[arg(long)]
    workspace: bool,

    /// Comma-separated cargo features.
    #[arg(long)]
    features: Option<String>,

    #[command(flatten)]
    common: CommonOutputArgs,
}

#[derive(Debug, Args)]
#[command(version = "0.9.1")]
struct CoverageArgs {
    /// Minimum line coverage % required to pass (e.g. `--threshold 80`).
    /// Below this, the produced CheckResult is `Fail`.
    #[arg(long, short = 't')]
    threshold: Option<f64>,

    /// Pass `--workspace` to cargo-llvm-cov.
    #[arg(long)]
    workspace: bool,

    /// Comma-separated cargo features.
    #[arg(long)]
    features: Option<String>,

    #[command(flatten)]
    common: CommonOutputArgs,
}

#[derive(Debug, Args)]
#[command(version = "0.9.2")]
struct AuditArgs {
    /// What to check: `all` (audit + deny), `audit` only, or `deny` only.
    #[arg(long, short = 's', value_enum, default_value_t = AuditScopeArg::All)]
    scope: AuditScopeArg,

    /// Optional path to a `deny.toml` config (overrides project default).
    #[arg(long = "deny-config")]
    deny_config: Option<PathBuf>,

    #[command(flatten)]
    common: CommonOutputArgs,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum AuditScopeArg {
    /// Both vulnerability scan + policy (cargo-audit + cargo-deny).
    All,
    /// Vulnerability scan only (cargo-audit).
    Vulnerabilities,
    /// License + banned-crate policy only (cargo-deny).
    Policy,
}

#[derive(Debug, Args)]
#[command(version = "0.9.1")]
struct DepsArgs {
    /// Only check udeps (unused). Skip cargo-outdated.
    #[arg(long)]
    unused_only: bool,

    /// Only check outdated. Skip cargo-udeps.
    #[arg(long)]
    outdated_only: bool,

    #[command(flatten)]
    common: CommonOutputArgs,
}

#[derive(Debug, Args)]
#[command(version = "0.9.1")]
struct FuzzArgs {
    /// Name of the fuzz target to run.
    target: String,

    /// Fuzzing time budget in seconds.
    #[arg(long, short = 'b', default_value_t = 60)]
    budget: u64,

    /// Sanitizer: `address`, `none`, `thread`, `memory`, `leak`.
    #[arg(long, default_value = "address")]
    sanitizer: String,

    #[command(flatten)]
    common: CommonOutputArgs,
}

#[derive(Debug, Args)]
#[command(version = "0.9.2")]
struct MutateArgs {
    /// Minimum kill rate % required to pass (default 70).
    #[arg(long, short = 't', default_value_t = 70.0)]
    threshold: f64,

    /// Pass `--workspace` to cargo-mutants.
    #[arg(long)]
    workspace: bool,

    /// Per-mutant timeout in seconds.
    #[arg(long)]
    timeout: Option<u64>,

    #[command(flatten)]
    common: CommonOutputArgs,
}

#[derive(Debug, Args)]
#[command(version = "0.9.1")]
struct FlakyArgs {
    /// Number of test iterations to run.
    #[arg(long, short = 'n', default_value_t = 10)]
    iterations: u32,

    /// Minimum reliability % required for a test to be classified stable.
    #[arg(long, short = 'r', default_value_t = 90.0)]
    reliability_threshold: f64,

    /// Pass `--workspace` to each cargo test invocation.
    #[arg(long)]
    workspace: bool,

    /// Comma-separated cargo features.
    #[arg(long)]
    features: Option<String>,

    #[command(flatten)]
    common: CommonOutputArgs,
}

#[derive(Debug, Args)]
#[command(version = "0.9.2")]
struct CiArgs {
    /// Where to write the workflow file. `-` writes to stdout.
    #[arg(long, short = 'o', default_value = ".github/workflows/ci.yml")]
    output: PathBuf,

    /// Workflow `name:` field.
    #[arg(long, default_value = "CI")]
    workflow_name: String,

    /// Comma-separated branches for `push` + `pull_request`.
    #[arg(long, value_delimiter = ',', default_value = "main")]
    branches: Vec<String>,

    /// Comma-separated OS matrix for the `test` job.
    #[arg(long, value_delimiter = ',', default_value = "ubuntu-latest")]
    matrix: Vec<String>,

    /// Comma-separated extra jobs: `clippy`, `fmt`, `docs`, `msrv`.
    #[arg(long, value_delimiter = ',')]
    with: Vec<String>,

    /// MSRV version for the `msrv` job.
    #[arg(long)]
    msrv: Option<String>,

    /// Sibling path-dep declaration as `name=repo-url`. Repeatable.
    #[arg(long = "path-dep", value_name = "NAME=URL")]
    path_deps: Vec<String>,
}

#[derive(Debug, Args)]
struct ReportArgs {
    /// Path to a Report JSON document.
    path: PathBuf,

    #[command(flatten)]
    common: CommonOutputArgs,
}

#[derive(Debug, Args)]
struct DiffArgs {
    /// Path to the baseline Report JSON.
    baseline: PathBuf,
    /// Path to the current Report JSON.
    current: PathBuf,

    /// Flag duration regressions above this percentage.
    #[arg(long)]
    duration_regression_pct: Option<f64>,

    /// Flag duration regressions above this absolute ms delta.
    #[arg(long)]
    duration_regression_abs_ms: Option<u64>,

    #[command(flatten)]
    common: CommonOutputArgs,
}

#[derive(Debug, Args)]
struct HtmlArgs {
    /// Path to a Report or MultiReport JSON document.
    path: PathBuf,

    /// Write HTML to this file. Defaults to `<input>.html`.
    #[arg(long, short = 'o')]
    out: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct VersionArgs {
    /// Optional component name: `report`, `tools`, `fixtures`, `bench`,
    /// `async`, `stress`, `chaos`, `coverage`, `security`, `deps`, `ci`,
    /// `fuzz`, `flaky`, `mutate`. With no argument, prints the full
    /// component table.
    component: Option<String>,

    /// Emit the component table as JSON instead of the styled human
    /// view. Useful for dashboards, CI comparators, and anything that
    /// wants to parse the version surface programmatically.
    #[arg(long)]
    json: bool,
}

// =============================================================================
// Entry point
// =============================================================================

fn main() -> ExitCode {
    let cli = Cli::parse();
    let res = match cli.cmd {
        Cmd::Test(a) => run_test(a),
        Cmd::Clippy(a) => run_clippy(a),
        Cmd::Check(a) => run_check(a),
        Cmd::Bench(a) => run_bench(a),
        Cmd::Coverage(a) => run_coverage(a),
        Cmd::Audit(a) => run_audit(a),
        Cmd::Deps(a) => run_deps(a),
        Cmd::Fuzz(a) => run_fuzz(a),
        Cmd::Mutate(a) => run_mutate(a),
        Cmd::Flaky(a) => run_flaky(a),
        Cmd::Ci(a) => run_ci(a),
        Cmd::Report(a) => run_report(a),
        Cmd::Diff(a) => run_diff(a),
        Cmd::Html(a) => run_html(a),
        Cmd::Version(a) => run_version(a),
    };
    match res {
        Ok(code) => code,
        Err(e) => {
            let _ = writeln!(io::stderr(), "dev: {e}");
            ExitCode::from(2)
        }
    }
}

// =============================================================================
// `dev report` — pretty-print a report on disk
// =============================================================================

fn run_report(args: ReportArgs) -> CliResult {
    let text =
        fs::read_to_string(&args.path).map_err(|e| format!("read {}: {e}", args.path.display()))?;
    let report =
        Report::from_json(&text).map_err(|e| format!("parse {}: {e}", args.path.display()))?;
    render_report(&report, &args.common)?;
    Ok(exit_for_verdict(report.overall_verdict()))
}

// =============================================================================
// `dev diff` — diff two reports
// =============================================================================

fn run_diff(args: DiffArgs) -> CliResult {
    let baseline_text = fs::read_to_string(&args.baseline)
        .map_err(|e| format!("read {}: {e}", args.baseline.display()))?;
    let baseline = Report::from_json(&baseline_text)
        .map_err(|e| format!("parse {}: {e}", args.baseline.display()))?;
    let current_text = fs::read_to_string(&args.current)
        .map_err(|e| format!("read {}: {e}", args.current.display()))?;
    let current = Report::from_json(&current_text)
        .map_err(|e| format!("parse {}: {e}", args.current.display()))?;

    let opts = dev_tools::report::DiffOptions {
        duration_regression_pct: args.duration_regression_pct,
        duration_regression_abs_ms: args.duration_regression_abs_ms,
    };
    let diff = current.diff_with(&baseline, &opts);
    render_diff(&diff, &args.common)?;
    if diff.is_clean() {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::from(1))
    }
}

// =============================================================================
// `dev html` — render a report to a self-contained HTML document
// =============================================================================

fn run_html(args: HtmlArgs) -> CliResult {
    use dev_tools::prelude::MultiReportHtmlExt as _;
    let text =
        fs::read_to_string(&args.path).map_err(|e| format!("read {}: {e}", args.path.display()))?;
    // Accept either a single Report or a MultiReport — wrap a singleton.
    let multi: MultiReport = match MultiReport::from_json(&text) {
        Ok(m) => m,
        Err(_) => {
            let report = Report::from_json(&text)
                .map_err(|e| format!("not a Report or MultiReport JSON: {e}"))?;
            let mut m = MultiReport::new(&report.subject, &report.subject_version);
            m.push(report);
            m.finish();
            m
        }
    };

    let html = multi.to_html();
    let out_path = args.out.unwrap_or_else(|| {
        let mut p = args.path.clone();
        p.set_extension("html");
        p
    });
    fs::write(&out_path, &html).map_err(|e| format!("write {}: {e}", out_path.display()))?;
    eprintln!("wrote {} ({} bytes)", out_path.display(), html.len());
    Ok(ExitCode::SUCCESS)
}

// =============================================================================
// `dev test` / `dev clippy` / `dev check` — cargo subprocess producers
// =============================================================================

fn run_test(args: TestArgs) -> CliResult {
    use dev_tools::report::Producer;
    let subject = args.common.resolved_subject();
    let version = args.common.resolved_version();

    if args.full {
        // Full stack: test + clippy + check, aggregated into a MultiReport.
        let mut multi = MultiReport::new(&subject, &version);
        for (label, report) in run_full_stack(&subject, &version, args.common.workdir.as_deref()) {
            if !args.common.quiet {
                eprintln!("=> {label} ({} checks)", report.checks.len());
            }
            multi.push(report);
        }
        multi.finish();
        render_multi(&multi, &args.common)?;
        Ok(exit_for_multi(&multi))
    } else {
        let mut p = dev_tools::producers::cargo_test_producer(&subject, &version);
        if let Some(d) = args.common.workdir.as_deref() {
            p = p.in_dir(d);
        }
        let report = p.produce();
        render_report(&report, &args.common)?;
        Ok(exit_for_verdict(report.overall_verdict()))
    }
}

fn run_clippy(args: SimpleProducerArgs) -> CliResult {
    use dev_tools::report::Producer;
    let subject = args.common.resolved_subject();
    let version = args.common.resolved_version();
    let mut p = dev_tools::producers::clippy_producer(&subject, &version);
    if let Some(d) = args.common.workdir.as_deref() {
        p = p.in_dir(d);
    }
    let report = p.produce();
    render_report(&report, &args.common)?;
    Ok(exit_for_verdict(report.overall_verdict()))
}

fn run_check(args: SimpleProducerArgs) -> CliResult {
    use dev_tools::report::Producer;
    let subject = args.common.resolved_subject();
    let version = args.common.resolved_version();
    let mut p = dev_tools::producers::cargo_check_producer(&subject, &version);
    if let Some(d) = args.common.workdir.as_deref() {
        p = p.in_dir(d);
    }
    let report = p.produce();
    render_report(&report, &args.common)?;
    Ok(exit_for_verdict(report.overall_verdict()))
}

/// Run the full available verification stack and yield one labeled
/// `Report` per dimension that succeeded in spawning its underlying
/// tool. Subprocess failures become `Fail` `CheckResult`s within
/// each Report — they don't short-circuit the stack.
fn run_full_stack(
    subject: &str,
    version: &str,
    workdir: Option<&std::path::Path>,
) -> Vec<(&'static str, Report)> {
    use dev_tools::report::Producer;

    let mut out = Vec::new();

    // 1. cargo test
    let mut p = dev_tools::producers::cargo_test_producer(subject, version);
    if let Some(d) = workdir {
        p = p.in_dir(d);
    }
    out.push(("cargo test", p.produce()));

    // 2. clippy
    let mut p = dev_tools::producers::clippy_producer(subject, version);
    if let Some(d) = workdir {
        p = p.in_dir(d);
    }
    out.push(("cargo clippy", p.produce()));

    // 3. cargo check (separate from clippy to surface non-lint errors)
    let mut p = dev_tools::producers::cargo_check_producer(subject, version);
    if let Some(d) = workdir {
        p = p.in_dir(d);
    }
    out.push(("cargo check", p.produce()));

    out
}

// =============================================================================
// `dev bench` — cargo bench wrapper
// =============================================================================

fn run_bench(args: BenchArgs) -> CliResult {
    use dev_tools::report::{CheckResult, Severity};
    use std::process::Command;

    let subject = args.common.resolved_subject();
    let version = args.common.resolved_version();

    let mut cmd = Command::new("cargo");
    cmd.arg("bench");
    if args.workspace {
        cmd.arg("--workspace");
    }
    if let Some(f) = &args.features {
        cmd.args(["--features", f]);
    }
    if let Some(d) = args.common.workdir.as_deref() {
        cmd.current_dir(d);
    }

    let out = cmd
        .output()
        .map_err(|e| format!("spawn cargo bench: {e}"))?;

    let mut r = Report::new(&subject, &version).with_producer("dev bench");
    let check = if out.status.success() {
        CheckResult::pass("cargo::bench")
    } else {
        let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
        CheckResult::fail("cargo::bench", Severity::Error).with_detail(first_lines(&stderr, 20))
    };
    r.push(check);
    r.finish();

    render_report(&r, &args.common)?;
    Ok(exit_for_verdict(r.overall_verdict()))
}

// =============================================================================
// `dev coverage` — dev-coverage wrapper
// =============================================================================

fn run_coverage(args: CoverageArgs) -> CliResult {
    use dev_coverage::{CoverageRun, CoverageThreshold};
    let subject = args.common.resolved_subject();
    let version = args.common.resolved_version();

    let mut run = CoverageRun::new(&subject, &version);
    if args.workspace {
        run = run.workspace();
    }
    if let Some(f) = &args.features {
        for name in f.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            run = run.feature(name);
        }
    }
    if let Some(d) = args.common.workdir.as_deref() {
        run = run.in_dir(d);
    }

    let result = run.execute().map_err(|e| format!("coverage failed: {e}"))?;

    // `--threshold` is optional; 0.0 disables the gate (every coverage
    // value passes a >= 0.0 check) while still producing the Report.
    let threshold = CoverageThreshold::min_line_pct(args.threshold.unwrap_or(0.0));
    let check = result.into_check_result(threshold);

    // Coverage emits a single CheckResult; wrap it in a Report for
    // consistent rendering with the rest of the CLI surface.
    let mut r = Report::new(&subject, &version).with_producer("dev coverage");
    r.push(check);
    r.finish();
    render_report(&r, &args.common)?;
    Ok(exit_for_verdict(r.overall_verdict()))
}

// =============================================================================
// `dev audit` — dev-security wrapper
// =============================================================================

fn run_audit(args: AuditArgs) -> CliResult {
    use dev_security::{AuditRun, AuditScope};
    let subject = args.common.resolved_subject();
    let version = args.common.resolved_version();

    let scope = match args.scope {
        AuditScopeArg::All => AuditScope::All,
        AuditScopeArg::Vulnerabilities => AuditScope::Vulnerabilities,
        AuditScopeArg::Policy => AuditScope::Policy,
    };
    let mut run = AuditRun::new(&subject, &version).scope(scope);
    if let Some(d) = args.common.workdir.as_deref() {
        run = run.in_dir(d);
    }
    if let Some(cfg) = args.deny_config.as_deref() {
        run = run.deny_config(cfg);
    }

    let result = run.execute().map_err(|e| format!("audit failed: {e}"))?;
    let report = result.into_report();
    render_report(&report, &args.common)?;
    Ok(exit_for_verdict(report.overall_verdict()))
}

// =============================================================================
// `dev deps` — dev-deps wrapper
// =============================================================================

fn run_deps(args: DepsArgs) -> CliResult {
    use dev_deps::{DepCheck, DepScope};
    let subject = args.common.resolved_subject();
    let version = args.common.resolved_version();

    let scope = if args.unused_only {
        DepScope::Unused
    } else if args.outdated_only {
        DepScope::Outdated
    } else {
        DepScope::All
    };
    let mut check = DepCheck::new(&subject, &version).scope(scope);
    if let Some(d) = args.common.workdir.as_deref() {
        check = check.in_dir(d);
    }

    let result = check
        .execute()
        .map_err(|e| format!("deps check failed: {e}"))?;
    let report = result.into_report();
    render_report(&report, &args.common)?;
    Ok(exit_for_verdict(report.overall_verdict()))
}

// =============================================================================
// `dev fuzz` — dev-fuzz wrapper
// =============================================================================

fn run_fuzz(args: FuzzArgs) -> CliResult {
    use dev_fuzz::{FuzzBudget, FuzzRun, Sanitizer};
    let version = args.common.resolved_version();

    let sanitizer = match args.sanitizer.as_str() {
        "address" => Sanitizer::Address,
        "thread" => Sanitizer::Thread,
        "memory" => Sanitizer::Memory,
        "leak" => Sanitizer::Leak,
        "none" => Sanitizer::None,
        other => return Err(format!("unknown sanitizer: {other}")),
    };
    let budget = FuzzBudget::time(std::time::Duration::from_secs(args.budget));
    // FuzzRun is keyed on (target, version); the subject metadata is
    // attached to the Report by FuzzResult::into_report().
    let mut run = FuzzRun::new(&args.target, &version)
        .budget(budget)
        .sanitizer(sanitizer);
    if let Some(d) = args.common.workdir.as_deref() {
        run = run.in_dir(d);
    }

    let result = run.execute().map_err(|e| format!("fuzz failed: {e}"))?;
    let report = result.into_report();
    render_report(&report, &args.common)?;
    Ok(exit_for_verdict(report.overall_verdict()))
}

// =============================================================================
// `dev mutate` — dev-mutate wrapper
// =============================================================================

fn run_mutate(args: MutateArgs) -> CliResult {
    use dev_mutate::{MutateRun, MutateThreshold};
    let subject = args.common.resolved_subject();
    let version = args.common.resolved_version();

    let mut run = MutateRun::new(&subject, &version);
    if args.workspace {
        run = run.workspace();
    }
    if let Some(t) = args.timeout {
        run = run.timeout(std::time::Duration::from_secs(t));
    }
    if let Some(d) = args.common.workdir.as_deref() {
        run = run.in_dir(d);
    }
    let threshold = MutateThreshold::min_kill_pct(args.threshold);

    let result = run.execute().map_err(|e| format!("mutate failed: {e}"))?;
    let check = result.into_check_result(threshold);
    let mut r = Report::new(&subject, &version).with_producer("dev mutate");
    r.push(check);
    r.finish();
    render_report(&r, &args.common)?;
    Ok(exit_for_verdict(r.overall_verdict()))
}

// =============================================================================
// `dev flaky` — dev-flaky wrapper
// =============================================================================

fn run_flaky(args: FlakyArgs) -> CliResult {
    use dev_flaky::FlakyRun;
    let subject = args.common.resolved_subject();
    let version = args.common.resolved_version();

    let mut run = FlakyRun::new(&subject, &version)
        .iterations(args.iterations)
        .reliability_threshold(args.reliability_threshold);
    if args.workspace {
        run = run.workspace();
    }
    if let Some(f) = &args.features {
        run = run.features(f.clone());
    }
    if let Some(d) = args.common.workdir.as_deref() {
        run = run.in_dir(d);
    }

    let result = run
        .execute()
        .map_err(|e| format!("flaky run failed: {e}"))?;
    let report = result.into_report();
    render_report(&report, &args.common)?;
    Ok(exit_for_verdict(report.overall_verdict()))
}

// =============================================================================
// `dev version` — print component versions
// =============================================================================
//
// The `SIBLINGS` table is generated at compile-time by `build.rs` from
// the actual resolved versions in `Cargo.lock`. This means `dev version`
// reports exactly what's linked into the binary — there's no
// hand-maintained table to drift away from reality.

include!(concat!(env!("OUT_DIR"), "/siblings.rs"));

fn run_version(args: VersionArgs) -> CliResult {
    let dev_version = env!("CARGO_PKG_VERSION");

    // --json takes precedence over both the table and the filtered form.
    // The same JSON shape is emitted whether or not `<component>` is
    // supplied; the `components` array is filtered to one entry when
    // a name is given.
    if args.json {
        let resolved = if let Some(name) = args.component.as_deref() {
            let normalized = normalize_component(name);
            let row = lookup_sibling(normalized).ok_or_else(|| unknown_component_msg(name))?;
            vec![*row]
        } else {
            SIBLINGS.to_vec()
        };

        let value = serde_json::json!({
            "binary": {
                "name": "dev",
                "version": dev_version,
            },
            "components": resolved
                .iter()
                .map(|(alias, name, version)| {
                    serde_json::json!({
                        "alias": alias,
                        "name": name,
                        "version": version,
                    })
                })
                .collect::<Vec<_>>(),
        });
        let text = serde_json::to_string_pretty(&value)
            .map_err(|e| format!("serialize version JSON: {e}"))?;
        println!("{text}");
        return Ok(ExitCode::SUCCESS);
    }

    if let Some(name) = args.component.as_deref() {
        let normalized = normalize_component(name);
        let row = lookup_sibling(normalized).ok_or_else(|| unknown_component_msg(name))?;
        println!("{} {}", row.1, row.2);
        return Ok(ExitCode::SUCCESS);
    }

    let color = io::stdout().is_terminal();
    println!();
    println!(
        "  {}",
        paint(
            &format!("dev {dev_version}"),
            &format!("{C_BOLD}{C_CYAN}"),
            color
        )
    );
    println!(
        "  {}",
        paint("the Rust verification toolkit CLI", C_DIM, color)
    );
    println!();
    println!("  {}", paint("components", C_BOLD, color));
    let crate_w = SIBLINGS.iter().map(|(_, c, _)| c.len()).max().unwrap_or(12);
    for (_, crate_name, version) in SIBLINGS {
        println!(
            "    {:<width$}   {}",
            crate_name,
            paint(version, C_GREEN, color),
            width = crate_w,
        );
    }
    println!();
    println!(
        "  {}",
        paint(
            "tip: `dev version <name>` prints just one component; --json emits machine-readable output",
            C_DIM,
            color
        )
    );
    println!();
    Ok(ExitCode::SUCCESS)
}

/// Normalize a user-supplied component name into the short-alias form.
/// Accepts either `coverage` or `dev-coverage`; both resolve to the
/// same row in the `SIBLINGS` table.
fn normalize_component(name: &str) -> String {
    let lower = name.trim().to_ascii_lowercase();
    lower.strip_prefix("dev-").unwrap_or(&lower).to_string()
}

fn lookup_sibling(
    normalized_alias: String,
) -> Option<&'static (&'static str, &'static str, &'static str)> {
    SIBLINGS
        .iter()
        .find(|(alias, _, _)| *alias == normalized_alias)
}

fn unknown_component_msg(name: &str) -> String {
    format!(
        "unknown component: {name:?}. Known: {}",
        SIBLINGS
            .iter()
            .map(|(a, _, _)| *a)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

// =============================================================================
// `dev ci` — proxy to dev-ci Generator
// =============================================================================

fn run_ci(args: CiArgs) -> CliResult {
    use dev_ci::{Generator, PathDep, Target};

    let mut gen = Generator::new()
        .target(Target::GitHubActions)
        .workflow_name(args.workflow_name)
        .branches(args.branches)
        .matrix_os(args.matrix);

    for job in &args.with {
        match job.trim().to_ascii_lowercase().as_str() {
            "" => {}
            "clippy" => gen = gen.with_clippy(),
            "fmt" => gen = gen.with_fmt(),
            "docs" => gen = gen.with_docs(),
            "msrv" => {
                let v = args
                    .msrv
                    .as_deref()
                    .ok_or_else(|| "--with msrv requires --msrv <VERSION>".to_string())?;
                gen = gen.with_msrv(v);
            }
            other => return Err(format!("unknown job in --with: {other:?}")),
        }
    }
    if let Some(v) = &args.msrv {
        if !args.with.iter().any(|j| j.eq_ignore_ascii_case("msrv")) {
            gen = gen.with_msrv(v.clone());
        }
    }
    for raw in &args.path_deps {
        let (name, url) = raw
            .split_once('=')
            .ok_or_else(|| format!("--path-dep must be name=url; got {raw:?}"))?;
        gen = gen.with_path_dep(PathDep::new(name, url));
    }

    let yaml = gen.generate();
    if args.output.as_os_str() == "-" {
        io::stdout()
            .write_all(yaml.as_bytes())
            .map_err(|e| format!("stdout: {e}"))?;
        return Ok(ExitCode::SUCCESS);
    }
    if let Some(parent) = args.output.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("create_dir_all({}): {e}", parent.display()))?;
        }
    }
    fs::write(&args.output, yaml).map_err(|e| format!("write {}: {e}", args.output.display()))?;
    eprintln!("wrote {}", args.output.display());
    Ok(ExitCode::SUCCESS)
}

// =============================================================================
// Rendering helpers
// =============================================================================

type CliResult = Result<ExitCode, String>;

fn render_report(report: &Report, common: &CommonOutputArgs) -> Result<(), String> {
    if common.quiet && common.out.is_none() {
        return Ok(());
    }
    let fmt = common
        .format
        .unwrap_or_else(|| OutputFormat::default_for_destination(common.out.is_some()));
    let text = match fmt {
        OutputFormat::Terminal => {
            let color = common.out.is_none() && io::stdout().is_terminal();
            pretty_report(report, color)
        }
        OutputFormat::Json => report
            .to_json()
            .map_err(|e| format!("serialize Report: {e}"))?,
        OutputFormat::Markdown => dev_tools::report::markdown::to_markdown(report),
        OutputFormat::Sarif => dev_tools::report::sarif::to_sarif(report),
        OutputFormat::Junit => dev_tools::report::junit::to_junit_xml(report),
    };
    write_text(&text, common.out.as_deref())
}

fn render_multi(multi: &MultiReport, common: &CommonOutputArgs) -> Result<(), String> {
    if common.quiet && common.out.is_none() {
        return Ok(());
    }
    let fmt = common
        .format
        .unwrap_or_else(|| OutputFormat::default_for_destination(common.out.is_some()));
    let text = match fmt {
        OutputFormat::Terminal => {
            let color = common.out.is_none() && io::stdout().is_terminal();
            pretty_multi(multi, color)
        }
        OutputFormat::Json => multi
            .to_json()
            .map_err(|e| format!("serialize MultiReport: {e}"))?,
        OutputFormat::Markdown => dev_tools::report::markdown::multi_to_markdown(multi),
        OutputFormat::Sarif => dev_tools::report::sarif::multi_to_sarif(multi),
        OutputFormat::Junit => dev_tools::report::junit::multi_to_junit_xml(multi),
    };
    write_text(&text, common.out.as_deref())
}

fn render_diff(diff: &Diff, common: &CommonOutputArgs) -> Result<(), String> {
    if common.quiet && common.out.is_none() {
        return Ok(());
    }
    let fmt = common
        .format
        .unwrap_or_else(|| OutputFormat::default_for_destination(common.out.is_some()));
    let text = match fmt {
        OutputFormat::Terminal => {
            if common.out.is_some() || !io::stdout().is_terminal() {
                dev_tools::report::terminal::diff_to_terminal(diff)
            } else {
                dev_tools::report::terminal::diff_to_terminal_color(diff)
            }
        }
        OutputFormat::Markdown => diff.to_markdown(),
        OutputFormat::Json | OutputFormat::Sarif | OutputFormat::Junit => {
            // Diff is not a wire-format object; fall back to JSON of the diff struct.
            serde_json::to_string_pretty(diff).map_err(|e| format!("serialize Diff: {e}"))?
        }
    };
    write_text(&text, common.out.as_deref())
}

fn write_text(text: &str, out: Option<&std::path::Path>) -> Result<(), String> {
    match out {
        Some(p) => fs::write(p, text).map_err(|e| format!("write {}: {e}", p.display())),
        None => {
            let mut stdout = io::stdout().lock();
            stdout
                .write_all(text.as_bytes())
                .map_err(|e| format!("stdout: {e}"))?;
            if !text.ends_with('\n') {
                let _ = writeln!(stdout);
            }
            Ok(())
        }
    }
}

fn exit_for_verdict(v: Verdict) -> ExitCode {
    match v {
        Verdict::Pass | Verdict::Skip => ExitCode::SUCCESS,
        Verdict::Warn => ExitCode::from(1),
        Verdict::Fail => ExitCode::from(1),
    }
}

fn exit_for_multi(m: &MultiReport) -> ExitCode {
    let mut overall = Verdict::Skip;
    for r in &m.reports {
        let v = r.overall_verdict();
        overall = worst(overall, v);
    }
    exit_for_verdict(overall)
}

fn worst(a: Verdict, b: Verdict) -> Verdict {
    // Fail > Warn > Pass > Skip
    fn rank(v: Verdict) -> u8 {
        match v {
            Verdict::Fail => 3,
            Verdict::Warn => 2,
            Verdict::Pass => 1,
            Verdict::Skip => 0,
        }
    }
    if rank(a) >= rank(b) {
        a
    } else {
        b
    }
}

fn first_lines(s: &str, n: usize) -> String {
    s.lines().take(n).collect::<Vec<_>>().join("\n")
}

// =============================================================================
// Cargo metadata discovery
// =============================================================================

struct CargoMeta {
    name: String,
    version: String,
}

/// Best-effort parse of `<workdir>/Cargo.toml` for the `[package]`
/// `name` and `version` fields. Returns `None` if anything goes wrong;
/// callers fall back to `unknown` / `0.0.0`.
///
/// Intentionally hand-rolled (no `cargo metadata` subprocess) so the
/// CLI can answer `--help` and produce a Report even outside a cargo
/// workspace.
fn cargo_metadata(workdir: Option<&std::path::Path>) -> Option<CargoMeta> {
    let base = workdir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().ok().unwrap_or_default());
    let manifest = base.join("Cargo.toml");
    let text = fs::read_to_string(manifest).ok()?;

    let mut name = None;
    let mut version = None;
    let mut in_package = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            continue;
        }
        if !in_package {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("name") {
            name = extract_quoted(rest);
        } else if let Some(rest) = trimmed.strip_prefix("version") {
            version = extract_quoted(rest);
        }
    }
    Some(CargoMeta {
        name: name?,
        version: version?,
    })
}

fn extract_quoted(s: &str) -> Option<String> {
    let s = s.trim_start_matches(|c: char| c == '=' || c.is_whitespace());
    let s = s.trim_end_matches(|c: char| c.is_whitespace() || c == ',');
    let s = s.trim_matches('"');
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

// =============================================================================
// Pretty-printer — the polished terminal renderer for `dev` output
// =============================================================================
//
// Independent of dev-report's own `terminal` module so we can iterate on
// the CLI look without forcing a wire-format change. ANSI color is opt-in
// (caller checks IsTerminal + --format).

const C_RESET: &str = "\x1b[0m";
const C_BOLD: &str = "\x1b[1m";
const C_DIM: &str = "\x1b[2m";
const C_GREEN: &str = "\x1b[32m";
const C_RED: &str = "\x1b[31m";
const C_YELLOW: &str = "\x1b[33m";
const C_CYAN: &str = "\x1b[36m";
const C_GRAY: &str = "\x1b[90m";

fn paint(s: &str, color: &str, enabled: bool) -> String {
    if enabled {
        format!("{color}{s}{C_RESET}")
    } else {
        s.to_string()
    }
}

fn verdict_glyph_color(v: Verdict) -> (&'static str, &'static str, &'static str) {
    match v {
        Verdict::Pass => ("✓", C_GREEN, "pass"),
        Verdict::Fail => ("✗", C_RED, "fail"),
        Verdict::Warn => ("⚠", C_YELLOW, "warn"),
        Verdict::Skip => ("⊘", C_GRAY, "skip"),
    }
}

fn pretty_report(report: &Report, color: bool) -> String {
    let mut out = String::with_capacity(512);

    // Header.
    let title = format!("dev · {} {}", report.subject, report.subject_version);
    out.push('\n');
    out.push_str(&paint(
        &format!("  {}", title),
        &format!("{C_BOLD}{C_CYAN}"),
        color,
    ));
    out.push('\n');
    if let Some(p) = &report.producer {
        out.push_str(&paint(&format!("  via {p}"), C_DIM, color));
        out.push('\n');
    }
    out.push('\n');

    // Per-check lines.
    let name_width = report
        .checks
        .iter()
        .map(|c| c.name.len())
        .max()
        .unwrap_or(0)
        .clamp(20, 60);
    for c in &report.checks {
        let (glyph, glyph_color, _label) = verdict_glyph_color(c.verdict);
        let dur = match c.duration_ms {
            Some(ms) => paint(&format!("{:>7}ms", ms), C_DIM, color),
            None => "         ".to_string(),
        };
        out.push_str("  ");
        out.push_str(&paint(glyph, glyph_color, color));
        out.push(' ');
        out.push_str(&format!("{:<width$}", c.name, width = name_width));
        out.push_str("  ");
        out.push_str(&dur);
        out.push('\n');
        if let Some(detail) = &c.detail {
            for line in detail.lines() {
                out.push_str("      ");
                out.push_str(&paint(line, C_DIM, color));
                out.push('\n');
            }
        }
    }

    // Summary.
    let (mut p, mut f, mut w, mut s) = (0usize, 0usize, 0usize, 0usize);
    for c in &report.checks {
        match c.verdict {
            Verdict::Pass => p += 1,
            Verdict::Fail => f += 1,
            Verdict::Warn => w += 1,
            Verdict::Skip => s += 1,
        }
    }

    out.push('\n');
    out.push_str(&paint(
        "  ─────────────────────────────────────────────────────────",
        C_DIM,
        color,
    ));
    out.push('\n');
    out.push_str("  ");
    let total = report.checks.len();
    out.push_str(&format!("{total} checks "));
    out.push_str(&paint(&format!("· {p} pass"), C_GREEN, color));
    out.push(' ');
    out.push_str(&paint(&format!("· {f} fail"), C_RED, color));
    out.push(' ');
    out.push_str(&paint(&format!("· {w} warn"), C_YELLOW, color));
    out.push(' ');
    out.push_str(&paint(&format!("· {s} skip"), C_GRAY, color));
    out.push('\n');

    // Overall.
    let overall = report.overall_verdict();
    let (glyph, glyph_color, label) = verdict_glyph_color(overall);
    let label_upper = label.to_uppercase();
    out.push_str("  ");
    out.push_str(&paint(
        &format!("{} Overall: {}", glyph, label_upper),
        &format!("{C_BOLD}{glyph_color}"),
        color,
    ));
    if let (Some(end), start) = (report.finished_at, report.started_at) {
        let ms = (end - start).num_milliseconds();
        out.push_str(&paint(&format!("   · {ms}ms total"), C_DIM, color));
    }
    out.push('\n');
    out.push('\n');
    out
}

fn pretty_multi(multi: &MultiReport, color: bool) -> String {
    let mut out = String::with_capacity(1024);
    out.push('\n');
    out.push_str(&paint(
        &format!("  dev · {} {}", multi.subject, multi.subject_version),
        &format!("{C_BOLD}{C_CYAN}"),
        color,
    ));
    out.push('\n');
    out.push_str(&paint(
        &format!(
            "  {} producer{}",
            multi.reports.len(),
            if multi.reports.len() == 1 { "" } else { "s" }
        ),
        C_DIM,
        color,
    ));
    out.push('\n');

    let mut total_p = 0;
    let mut total_f = 0;
    let mut total_w = 0;
    let mut total_s = 0;
    for r in &multi.reports {
        let (glyph, glyph_color, _) = verdict_glyph_color(r.overall_verdict());
        let label = r.producer.as_deref().unwrap_or("(unnamed)");
        out.push('\n');
        out.push_str("  ");
        out.push_str(&paint(glyph, glyph_color, color));
        out.push(' ');
        out.push_str(&paint(label, C_BOLD, color));
        let (p, f, w, s) = r.verdict_counts();
        total_p += p;
        total_f += f;
        total_w += w;
        total_s += s;
        out.push_str(&paint(
            &format!("   {} pass · {} fail · {} warn · {} skip", p, f, w, s),
            C_DIM,
            color,
        ));
        out.push('\n');
        // Show only failures + warnings inline; the rest is on demand.
        for c in &r.checks {
            if matches!(c.verdict, Verdict::Fail | Verdict::Warn) {
                let (g, gc, _) = verdict_glyph_color(c.verdict);
                out.push_str("    ");
                out.push_str(&paint(g, gc, color));
                out.push(' ');
                out.push_str(&c.name);
                if let Some(d) = &c.detail {
                    out.push_str(&paint(
                        &format!(" — {}", d.lines().next().unwrap_or(d)),
                        C_DIM,
                        color,
                    ));
                }
                out.push('\n');
            }
        }
    }

    out.push('\n');
    out.push_str(&paint(
        "  ─────────────────────────────────────────────────────────",
        C_DIM,
        color,
    ));
    out.push('\n');
    out.push_str(&format!(
        "  totals: {} pass · {} fail · {} warn · {} skip\n",
        total_p, total_f, total_w, total_s,
    ));
    let overall_v = if total_f > 0 {
        Verdict::Fail
    } else if total_w > 0 {
        Verdict::Warn
    } else if total_p > 0 {
        Verdict::Pass
    } else {
        Verdict::Skip
    };
    let (glyph, glyph_color, label) = verdict_glyph_color(overall_v);
    out.push_str("  ");
    out.push_str(&paint(
        &format!("{} Overall: {}", glyph, label.to_uppercase()),
        &format!("{C_BOLD}{glyph_color}"),
        color,
    ));
    out.push_str("\n\n");
    out
}
