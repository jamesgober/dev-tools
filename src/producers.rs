//! Reusable [`Producer`] implementations for common `cargo` subcommands.
//!
//! Each producer spawns a `cargo` subprocess, captures its output, and
//! converts the result into a [`Report`]. Subprocess failures (missing
//! `cargo`, non-zero exit, parse errors) become a [`CheckResult::fail`]
//! inside the produced report rather than panicking.
//!
//! ### Available producers
//!
//! | Function                | Subcommand        | Mapping                                                                 |
//! |-------------------------|-------------------|-------------------------------------------------------------------------|
//! | [`cargo_test_producer`] | `cargo test`      | Each test → one `CheckResult` (pass / fail+Error / skip for ignored).   |
//! | [`clippy_producer`]     | `cargo clippy`    | Each diagnostic → one `CheckResult` (warning → warn, error → fail).     |
//! | [`cargo_check_producer`]| `cargo check`     | Each diagnostic → one `CheckResult` (same mapping as clippy).           |
//!
//! Both `clippy` and `cargo check` parse `--message-format=json`. The
//! producers do NOT escalate warnings to errors (no `-D warnings`); the
//! distinction is preserved in the produced `CheckResult` verdicts.
//!
//! ### Environment
//!
//! `CARGO_TARGET_DIR`, `CARGO`, and the rest of the parent environment
//! are inherited by the subprocess. Callers that need a clean environment
//! should configure one before constructing the producer.
//!
//! [`Producer`]: dev_report::Producer
//! [`Report`]: dev_report::Report
//! [`CheckResult::fail`]: dev_report::CheckResult::fail

use std::path::PathBuf;
use std::process::Command;

use dev_report::{CheckResult, Evidence, Producer, Report, Severity};
use serde::Deserialize;

// ---------------------------------------------------------------------------
// cargo test
// ---------------------------------------------------------------------------

/// Producer that runs `cargo test --no-fail-fast` and maps libtest's
/// human-readable output to one [`CheckResult`] per test.
///
/// Constructed via [`cargo_test_producer`].
pub struct CargoTestProducer {
    subject: String,
    subject_version: String,
    workdir: Option<PathBuf>,
}

impl CargoTestProducer {
    /// Set the working directory to invoke `cargo` in. Defaults to the
    /// current process CWD.
    pub fn in_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.workdir = Some(dir.into());
        self
    }
}

impl Producer for CargoTestProducer {
    fn produce(&self) -> Report {
        let mut report =
            Report::new(&self.subject, &self.subject_version).with_producer("cargo-test");

        let output = match run_cargo(&self.workdir, &["test", "--no-fail-fast"]) {
            Ok(o) => o,
            Err(c) => {
                report.push(*c);
                report.finish();
                return report;
            }
        };

        for c in parse_cargo_test_output(&output.combined) {
            report.push(c);
        }
        report.finish();
        report
    }
}

/// Build a producer that runs `cargo test --no-fail-fast` and emits one
/// [`CheckResult`] per test.
///
/// # Example
///
/// ```no_run
/// use dev_tools::producers::cargo_test_producer;
/// use dev_tools::report::Producer;
///
/// let producer = cargo_test_producer("my-crate", "0.1.0");
/// let report = producer.produce();
/// println!("{}", report.to_json().unwrap());
/// ```
pub fn cargo_test_producer(
    subject: impl Into<String>,
    subject_version: impl Into<String>,
) -> CargoTestProducer {
    CargoTestProducer {
        subject: subject.into(),
        subject_version: subject_version.into(),
        workdir: None,
    }
}

fn parse_cargo_test_output(text: &str) -> Vec<CheckResult> {
    let mut results = Vec::new();
    for line in text.lines() {
        // libtest emits one line per test, of the shape:
        //   test some::name ... ok
        //   test some::name ... FAILED
        //   test some::name ... ignored
        let rest = match line.strip_prefix("test ") {
            Some(r) => r,
            None => continue,
        };
        let (name, outcome) = match rest.rsplit_once(" ... ") {
            Some(pair) => pair,
            None => continue,
        };
        let trimmed_outcome = outcome.split_whitespace().next().unwrap_or("");
        let check = match trimmed_outcome {
            "ok" => CheckResult::pass(name),
            "FAILED" => CheckResult::fail(name, Severity::Error),
            "ignored" => CheckResult::skip(name),
            _ => continue,
        };
        results.push(check);
    }
    results
}

// ---------------------------------------------------------------------------
// cargo clippy / cargo check
// ---------------------------------------------------------------------------

/// Producer that runs `cargo clippy --message-format=json` and maps each
/// compiler diagnostic to one [`CheckResult`].
///
/// Constructed via [`clippy_producer`].
pub struct ClippyProducer {
    subject: String,
    subject_version: String,
    workdir: Option<PathBuf>,
}

impl ClippyProducer {
    /// Set the working directory to invoke `cargo` in.
    pub fn in_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.workdir = Some(dir.into());
        self
    }
}

impl Producer for ClippyProducer {
    fn produce(&self) -> Report {
        let mut report = Report::new(&self.subject, &self.subject_version)
            .with_producer("cargo-clippy");
        run_message_format_json(&self.workdir, "clippy", &mut report);
        report
    }
}

/// Build a producer that runs `cargo clippy --message-format=json`.
///
/// Each diagnostic emitted by clippy becomes a `CheckResult`. Warnings
/// map to `Verdict::Warn` + `Severity::Warning`; errors map to
/// `Verdict::Fail` + `Severity::Error`. Source locations propagate as
/// `Evidence::FileRef`; the rendered diagnostic text propagates as
/// `Evidence::Snippet`.
///
/// # Example
///
/// ```no_run
/// use dev_tools::producers::clippy_producer;
/// use dev_tools::report::Producer;
///
/// let producer = clippy_producer("my-crate", "0.1.0");
/// let report = producer.produce();
/// println!("{}", report.to_json().unwrap());
/// ```
pub fn clippy_producer(
    subject: impl Into<String>,
    subject_version: impl Into<String>,
) -> ClippyProducer {
    ClippyProducer {
        subject: subject.into(),
        subject_version: subject_version.into(),
        workdir: None,
    }
}

/// Producer that runs `cargo check --message-format=json` and maps each
/// compiler diagnostic to one [`CheckResult`].
///
/// Constructed via [`cargo_check_producer`].
pub struct CargoCheckProducer {
    subject: String,
    subject_version: String,
    workdir: Option<PathBuf>,
}

impl CargoCheckProducer {
    /// Set the working directory to invoke `cargo` in.
    pub fn in_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.workdir = Some(dir.into());
        self
    }
}

impl Producer for CargoCheckProducer {
    fn produce(&self) -> Report {
        let mut report =
            Report::new(&self.subject, &self.subject_version).with_producer("cargo-check");
        run_message_format_json(&self.workdir, "check", &mut report);
        report
    }
}

/// Build a producer that runs `cargo check --message-format=json`.
///
/// Same diagnostic-to-CheckResult mapping as [`clippy_producer`].
///
/// # Example
///
/// ```no_run
/// use dev_tools::producers::cargo_check_producer;
/// use dev_tools::report::Producer;
///
/// let producer = cargo_check_producer("my-crate", "0.1.0");
/// let report = producer.produce();
/// println!("{}", report.to_json().unwrap());
/// ```
pub fn cargo_check_producer(
    subject: impl Into<String>,
    subject_version: impl Into<String>,
) -> CargoCheckProducer {
    CargoCheckProducer {
        subject: subject.into(),
        subject_version: subject_version.into(),
        workdir: None,
    }
}

fn run_message_format_json(
    workdir: &Option<PathBuf>,
    subcommand: &str,
    report: &mut Report,
) {
    let output = match run_cargo(workdir, &[subcommand, "--message-format=json"]) {
        Ok(o) => o,
        Err(c) => {
            report.push(*c);
            report.finish();
            return;
        }
    };
    for line in output.stdout.lines() {
        if let Some(c) = parse_cargo_message_line(line) {
            report.push(c);
        }
    }
    report.finish();
}

#[derive(Deserialize)]
struct CargoMessage {
    reason: String,
    message: Option<CompilerMessage>,
}

#[derive(Deserialize)]
struct CompilerMessage {
    level: String,
    message: String,
    spans: Vec<CompilerSpan>,
    code: Option<DiagnosticCode>,
    rendered: Option<String>,
}

#[derive(Deserialize)]
struct CompilerSpan {
    file_name: String,
    line_start: u32,
    line_end: u32,
    is_primary: bool,
}

#[derive(Deserialize)]
struct DiagnosticCode {
    code: String,
}

fn parse_cargo_message_line(line: &str) -> Option<CheckResult> {
    let msg: CargoMessage = serde_json::from_str(line).ok()?;
    if msg.reason != "compiler-message" {
        return None;
    }
    let compiler_msg = msg.message?;
    let (verdict_kind, severity) = match compiler_msg.level.as_str() {
        "warning" => (Verdict::Warn, Severity::Warning),
        "error" | "error: internal compiler error" => (Verdict::Fail, Severity::Error),
        _ => return None, // ignore notes, helps, ICE notes, etc.
    };

    let name = compiler_msg
        .code
        .as_ref()
        .map(|c| c.code.clone())
        .unwrap_or_else(|| short_name_from_message(&compiler_msg.message));

    let mut check = match verdict_kind {
        Verdict::Warn => CheckResult::warn(name, severity),
        Verdict::Fail => CheckResult::fail(name, severity),
        _ => return None,
    };
    check = check.with_detail(compiler_msg.message.clone());

    let primary_span = compiler_msg
        .spans
        .iter()
        .find(|s| s.is_primary)
        .or_else(|| compiler_msg.spans.first());
    if let Some(span) = primary_span {
        check = check.with_evidence(Evidence::file_ref_lines(
            "site",
            span.file_name.clone(),
            span.line_start,
            span.line_end,
        ));
    }
    if let Some(rendered) = compiler_msg.rendered {
        check = check.with_evidence(Evidence::snippet("rendered", rendered));
    }
    Some(check)
}

fn short_name_from_message(msg: &str) -> String {
    // Diagnostics without a code (e.g. some internal ones) still need a
    // stable-ish name. Take the first line, trim, and cap at 80 chars.
    let first_line = msg.lines().next().unwrap_or("diagnostic").trim();
    if first_line.len() <= 80 {
        first_line.to_string()
    } else {
        format!("{}...", &first_line[..77])
    }
}

// Local alias so we don't need to `use dev_report::Verdict` only for matching.
use dev_report::Verdict;

// ---------------------------------------------------------------------------
// Subprocess plumbing
// ---------------------------------------------------------------------------

struct CapturedOutput {
    stdout: String,
    #[allow(dead_code)]
    stderr: String,
    combined: String,
}

fn run_cargo(
    workdir: &Option<PathBuf>,
    args: &[&str],
) -> Result<CapturedOutput, Box<CheckResult>> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut cmd = Command::new(&cargo);
    cmd.args(args);
    if let Some(dir) = workdir.as_ref() {
        cmd.current_dir(dir);
    }

    let output = match cmd.output() {
        Ok(o) => o,
        Err(e) => {
            return Err(Box::new(
                CheckResult::fail("subprocess::spawn", Severity::Critical)
                    .with_detail(format!("failed to spawn cargo: {}", e)),
            ));
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let combined = format!("{}\n{}", stdout, stderr);
    Ok(CapturedOutput {
        stdout,
        stderr,
        combined,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cargo_test_output_recognizes_ok_failed_ignored() {
        let stdout = "\
running 4 tests
test foo::bar ... ok
test foo::baz ... FAILED
test foo::qux ... ignored
test foo::quux ... ok (0.01s)

failures:
foo::baz
";
        let results = parse_cargo_test_output(stdout);
        let names: Vec<&str> = results.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["foo::bar", "foo::baz", "foo::qux", "foo::quux"]);
        assert_eq!(results[0].verdict, Verdict::Pass);
        assert_eq!(results[1].verdict, Verdict::Fail);
        assert_eq!(results[1].severity, Some(Severity::Error));
        assert_eq!(results[2].verdict, Verdict::Skip);
        assert_eq!(results[3].verdict, Verdict::Pass);
    }

    #[test]
    fn parse_cargo_test_output_ignores_unrelated_lines() {
        let stdout = "\
   Compiling foo v0.1.0
running 1 test
test test_a ... ok

test result: ok. 1 passed; 0 failed; 0 ignored
";
        let results = parse_cargo_test_output(stdout);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "test_a");
    }

    #[test]
    fn parse_cargo_message_line_maps_warning() {
        let line = r#"{"reason":"compiler-message","package_id":"x","manifest_path":"x","target":{},"message":{"level":"warning","message":"unused variable: `x`","spans":[{"file_name":"src/lib.rs","line_start":10,"line_end":10,"is_primary":true,"byte_start":0,"byte_end":1,"column_start":1,"column_end":2,"text":[]}],"code":{"code":"unused_variables"},"rendered":"warning: unused variable: `x`\n  --> src/lib.rs:10:1"}}"#;
        let check = parse_cargo_message_line(line).expect("should parse");
        assert_eq!(check.name, "unused_variables");
        assert_eq!(check.verdict, Verdict::Warn);
        assert_eq!(check.severity, Some(Severity::Warning));
        assert_eq!(check.detail.as_deref(), Some("unused variable: `x`"));
        assert_eq!(check.evidence.len(), 2);
    }

    #[test]
    fn parse_cargo_message_line_maps_error() {
        let line = r#"{"reason":"compiler-message","message":{"level":"error","message":"cannot find type `Foo`","spans":[{"file_name":"src/main.rs","line_start":3,"line_end":3,"is_primary":true,"byte_start":0,"byte_end":1,"column_start":1,"column_end":2,"text":[]}],"code":{"code":"E0412"},"rendered":"error[E0412]: cannot find type `Foo`"}}"#;
        let check = parse_cargo_message_line(line).expect("should parse");
        assert_eq!(check.name, "E0412");
        assert_eq!(check.verdict, Verdict::Fail);
        assert_eq!(check.severity, Some(Severity::Error));
    }

    #[test]
    fn parse_cargo_message_line_ignores_non_diagnostic_reasons() {
        for line in [
            r#"{"reason":"compiler-artifact","package_id":"x"}"#,
            r#"{"reason":"build-finished","success":true}"#,
            r#"{"reason":"build-script-executed","package_id":"x"}"#,
        ] {
            assert!(parse_cargo_message_line(line).is_none());
        }
    }

    #[test]
    fn parse_cargo_message_line_handles_diagnostic_without_code() {
        let line = r#"{"reason":"compiler-message","message":{"level":"warning","message":"this is a long warning that has no diagnostic code attached","spans":[],"code":null,"rendered":""}}"#;
        let check = parse_cargo_message_line(line).expect("should parse");
        assert_eq!(
            check.name,
            "this is a long warning that has no diagnostic code attached"
        );
    }

    #[test]
    fn parse_cargo_message_line_truncates_very_long_message_for_name() {
        let long = "a".repeat(200);
        let line = format!(
            r#"{{"reason":"compiler-message","message":{{"level":"warning","message":"{}","spans":[],"code":null,"rendered":""}}}}"#,
            long
        );
        let check = parse_cargo_message_line(&line).expect("should parse");
        assert!(check.name.ends_with("..."));
        assert!(check.name.len() <= 80);
    }

    #[test]
    fn parse_cargo_message_line_skips_unrecognized_levels() {
        for level in ["note", "help", "failure-note"] {
            let line = format!(
                r#"{{"reason":"compiler-message","message":{{"level":"{}","message":"x","spans":[],"code":null,"rendered":""}}}}"#,
                level
            );
            assert!(parse_cargo_message_line(&line).is_none(), "level {}", level);
        }
    }

    #[test]
    fn parse_cargo_message_line_ignores_malformed_json() {
        assert!(parse_cargo_message_line("not json").is_none());
        assert!(parse_cargo_message_line("").is_none());
    }
}
