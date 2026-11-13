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
//! ### Time limit
//!
//! By default a producer waits for cargo to finish, however long that
//! takes. Each producer has a `timeout` builder: when it expires, the
//! whole cargo process tree (including test binaries) is killed, results
//! already printed are kept, and a `Fail` check named
//! `cargo::<subcommand>::timeout` is added.
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
use std::time::Duration;

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
    timeout: Option<Duration>,
}

impl CargoTestProducer {
    /// Set the working directory to invoke `cargo` in. Defaults to the
    /// current process CWD.
    pub fn in_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.workdir = Some(dir.into());
        self
    }

    /// Stop cargo once it has run for `limit`. The whole process tree
    /// is killed, results printed so far are kept, and a `Fail` check
    /// named `cargo::<subcommand>::timeout` is added. Off by default.
    pub fn timeout(mut self, limit: Duration) -> Self {
        self.timeout = Some(limit);
        self
    }
}

impl Producer for CargoTestProducer {
    fn produce(&self) -> Report {
        let mut report =
            Report::new(&self.subject, &self.subject_version).with_producer("cargo-test");

        let output = match run_cargo(&self.workdir, &["test", "--no-fail-fast"], self.timeout) {
            Ok(o) => o,
            Err(c) => {
                report.push(*c);
                report.finish();
                return report;
            }
        };

        let checks = parse_cargo_test_output(&output.combined);
        let any_failed = checks.iter().any(|c| c.verdict == Verdict::Fail);
        for c in checks {
            report.push(c);
        }
        // A build error, a crashed test binary, or a bad flag makes
        // `cargo test` exit non-zero without printing a single FAILED
        // test line. Without this check the report would be empty (an
        // overall Skip) and read as success.
        if output.timed_out {
            report.push(timeout_failure("test", self.timeout));
        } else if !output.success && !any_failed {
            report.push(exit_failure("test", &output));
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
        timeout: None,
    }
}

fn parse_cargo_test_output(text: &str) -> Vec<CheckResult> {
    // libtest prints one result line per test, of the shape
    //   test some::name ... ok
    //   test some::name ... FAILED
    //   test some::name ... ignored[, reason]
    // in the list that follows each `running N tests` header. The list
    // ends at `failures:` (where the captured output of failing tests is
    // replayed) or at `test result:`. Lines outside a list are never read
    // as results, so a test that prints `test x ... ok` cannot add a
    // check of its own.
    let mut results = Vec::new();
    let mut in_list = false;
    // Inside a failures block, until `test result:`. A `running N tests`
    // line there is captured output, not a new list.
    let mut in_failures = false;
    for raw in text.lines() {
        let line = raw.trim_end();
        if line.starts_with("test result: ") {
            in_list = false;
            in_failures = false;
            continue;
        }
        if in_failures {
            continue;
        }
        if is_running_header(line) {
            in_list = true;
            continue;
        }
        if line == "failures:" {
            in_list = false;
            in_failures = true;
            continue;
        }
        if !in_list {
            continue;
        }
        let rest = match line.strip_prefix("test ") {
            Some(r) => r,
            None => continue,
        };
        let (name, outcome) = match rest.rsplit_once(" ... ") {
            Some(pair) => pair,
            None => continue,
        };
        let word = outcome.split_whitespace().next().unwrap_or("");
        let check = match word.trim_end_matches(',') {
            "ok" => CheckResult::pass(name),
            "FAILED" => CheckResult::fail(name, Severity::Error),
            "ignored" => CheckResult::skip(name),
            _ => continue,
        };
        results.push(check);
    }
    results
}

/// `running 3 tests` / `running 1 test`.
fn is_running_header(line: &str) -> bool {
    let Some(rest) = line.strip_prefix("running ") else {
        return false;
    };
    let Some((count, word)) = rest.split_once(' ') else {
        return false;
    };
    !count.is_empty()
        && count.bytes().all(|b| b.is_ascii_digit())
        && matches!(word, "test" | "tests")
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
    timeout: Option<Duration>,
}

impl ClippyProducer {
    /// Set the working directory to invoke `cargo` in.
    pub fn in_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.workdir = Some(dir.into());
        self
    }

    /// Stop cargo once it has run for `limit`. The whole process tree
    /// is killed, results printed so far are kept, and a `Fail` check
    /// named `cargo::<subcommand>::timeout` is added. Off by default.
    pub fn timeout(mut self, limit: Duration) -> Self {
        self.timeout = Some(limit);
        self
    }
}

impl Producer for ClippyProducer {
    fn produce(&self) -> Report {
        let mut report =
            Report::new(&self.subject, &self.subject_version).with_producer("cargo-clippy");
        run_message_format_json(&self.workdir, "clippy", self.timeout, &mut report);
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
        timeout: None,
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
    timeout: Option<Duration>,
}

impl CargoCheckProducer {
    /// Set the working directory to invoke `cargo` in.
    pub fn in_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.workdir = Some(dir.into());
        self
    }

    /// Stop cargo once it has run for `limit`. The whole process tree
    /// is killed, results printed so far are kept, and a `Fail` check
    /// named `cargo::<subcommand>::timeout` is added. Off by default.
    pub fn timeout(mut self, limit: Duration) -> Self {
        self.timeout = Some(limit);
        self
    }
}

impl Producer for CargoCheckProducer {
    fn produce(&self) -> Report {
        let mut report =
            Report::new(&self.subject, &self.subject_version).with_producer("cargo-check");
        run_message_format_json(&self.workdir, "check", self.timeout, &mut report);
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
        timeout: None,
    }
}

fn run_message_format_json(
    workdir: &Option<PathBuf>,
    subcommand: &str,
    timeout: Option<Duration>,
    report: &mut Report,
) {
    let output = match run_cargo(workdir, &[subcommand, "--message-format=json"], timeout) {
        Ok(o) => o,
        Err(c) => {
            report.push(*c);
            report.finish();
            return;
        }
    };
    let mut any_failed = false;
    for line in output.stdout.lines() {
        if let Some(c) = parse_cargo_message_line(line) {
            any_failed |= c.verdict == Verdict::Fail;
            report.push(c);
        }
    }
    // A broken manifest, an unknown flag, or a missing clippy component
    // makes cargo exit non-zero without any JSON error diagnostic.
    if output.timed_out {
        report.push(timeout_failure(subcommand, timeout));
    } else if !output.success && !any_failed {
        report.push(exit_failure(subcommand, &output));
    }
    report.finish();
}

/// `Fail` check for a cargo run that was killed at its time limit.
fn timeout_failure(subcommand: &str, limit: Option<Duration>) -> CheckResult {
    let limit = limit.unwrap_or_default();
    CheckResult::fail(format!("cargo::{}::timeout", subcommand), Severity::Error)
        .with_detail(format!(
            "`cargo {}` did not finish within {:?} and was killed; results printed before that are kept",
            subcommand, limit
        ))
        .with_evidence(Evidence::numeric_int(
            "timeout_ms",
            i64::try_from(limit.as_millis()).unwrap_or(i64::MAX),
        ))
}

/// `Fail` check for a cargo run that exited non-zero without reporting
/// the failure in its structured output. Carries the tail of stderr.
fn exit_failure(subcommand: &str, output: &CapturedOutput) -> CheckResult {
    let code = output
        .code
        .map(|c| c.to_string())
        .unwrap_or_else(|| "a signal".to_string());
    let tail = last_lines(&output.stderr, 40);
    let mut check = CheckResult::fail(format!("cargo::{}", subcommand), Severity::Error)
        .with_detail(format!(
        "`cargo {}` exited with {} without reporting a failing item (build error or tool failure)",
        subcommand, code
    ));
    if !tail.is_empty() {
        check = check.with_evidence(Evidence::snippet("stderr", tail));
    }
    check
}

fn last_lines(s: &str, n: usize) -> String {
    let lines: Vec<&str> = s.lines().collect();
    let start = lines.len().saturating_sub(n);
    lines[start..].join("\n")
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
    #[serde(default)]
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
    // Cut on a char boundary: byte-slicing at 77 panics when a
    // multi-byte character straddles that offset.
    let first_line = msg.lines().next().unwrap_or("diagnostic").trim();
    if first_line.len() <= 80 {
        return first_line.to_string();
    }
    let mut cut = 77;
    while !first_line.is_char_boundary(cut) {
        cut -= 1;
    }
    format!("{}...", &first_line[..cut])
}

// Local alias so we don't need to `use dev_report::Verdict` only for matching.
use dev_report::Verdict;

// ---------------------------------------------------------------------------
// Subprocess plumbing
// ---------------------------------------------------------------------------

struct CapturedOutput {
    stdout: String,
    stderr: String,
    combined: String,
    success: bool,
    code: Option<i32>,
    timed_out: bool,
}

fn run_cargo(
    workdir: &Option<PathBuf>,
    args: &[&str],
    timeout: Option<Duration>,
) -> Result<CapturedOutput, Box<CheckResult>> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut cmd = Command::new(cargo);
    cmd.args(args);
    if let Some(dir) = workdir.as_ref() {
        cmd.current_dir(dir);
    }

    let output = match crate::process::output(&mut cmd, timeout) {
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
        success: output.status.is_some_and(|s| s.success()),
        code: output.status.and_then(|s| s.code()),
        timed_out: output.status.is_none(),
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
    fn ignored_with_reason_is_kept() {
        let stdout = "running 2 tests\ntest needs_net ... ignored, needs network\ntest plain ... ignored\n\ntest result: ok. 0 passed; 0 failed; 2 ignored\n";
        let results = parse_cargo_test_output(stdout);
        let names: Vec<&str> = results.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["needs_net", "plain"]);
        assert!(results.iter().all(|c| c.verdict == Verdict::Skip));
    }

    #[test]
    fn captured_output_is_not_read_as_results() {
        let stdout = "\
running 2 tests
test real_fail ... FAILED
test real_pass ... ok

failures:

---- real_fail stdout ----
test phantom_pass ... ok
running 9 tests
test phantom_fail ... FAILED

failures:
    real_fail

test result: FAILED. 1 passed; 1 failed; 0 ignored
";
        let results = parse_cargo_test_output(stdout);
        let names: Vec<&str> = results.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["real_fail", "real_pass"]);
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
    fn long_multibyte_message_is_cut_on_a_char_boundary() {
        // 76 ASCII bytes then a 3-byte char spanning bytes 76..79: a
        // byte slice at 77 would land inside it and panic.
        let msg = format!("{}\u{2192}{}", "a".repeat(76), "b".repeat(20));
        let name = short_name_from_message(&msg);
        assert_eq!(name, format!("{}...", "a".repeat(76)));
    }

    #[test]
    fn spans_field_is_optional() {
        let line = r#"{"reason":"compiler-message","message":{"level":"warning","message":"w","code":null,"rendered":null}}"#;
        let check = parse_cargo_message_line(line).expect("should parse");
        assert!(check.evidence.is_empty());
    }

    fn captured(success: bool, code: Option<i32>, stderr: &str) -> CapturedOutput {
        CapturedOutput {
            stdout: String::new(),
            stderr: stderr.to_string(),
            combined: stderr.to_string(),
            success,
            code,
            timed_out: false,
        }
    }

    #[test]
    fn exit_failure_names_the_subcommand_and_keeps_the_stderr_tail() {
        let mut stderr = String::new();
        for i in 1..=60 {
            stderr.push_str(&format!("line {i}\n"));
        }
        let check = exit_failure("test", &captured(false, Some(101), &stderr));
        assert_eq!(check.name, "cargo::test");
        assert_eq!(check.verdict, Verdict::Fail);
        assert!(check.detail.as_deref().unwrap().contains("exited with 101"));
        match &check.evidence[0].data {
            dev_report::EvidenceData::Snippet(s) => {
                assert!(s.starts_with("line 21\n"), "{s}");
                assert!(s.ends_with("line 60"), "{s}");
            }
            other => panic!("expected snippet, got {other:?}"),
        }
        let no_code = exit_failure("check", &captured(false, None, ""));
        assert!(no_code.detail.as_deref().unwrap().contains("a signal"));
        assert!(no_code.evidence.is_empty());
    }

    #[test]
    fn producers_report_a_build_failure_instead_of_an_empty_pass() {
        // A crate that does not compile: `cargo test` prints no test
        // lines and `cargo check` exits non-zero.
        let dir = std::env::temp_dir().join(format!("dev-tools-producer-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(
            dir.join("Cargo.toml"),
            "[package]\nname = \"broken\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n",
        )
        .unwrap();
        std::fs::write(dir.join("src/lib.rs"), "pub fn f() -> u32 { \"x\" }\n").unwrap();

        let test = cargo_test_producer("broken", "0.1.0")
            .in_dir(&dir)
            .produce();
        assert_eq!(test.overall_verdict(), Verdict::Fail, "{test:?}");
        assert!(test.checks.iter().any(|c| c.name == "cargo::test"));

        let check = cargo_check_producer("broken", "0.1.0")
            .in_dir(&dir)
            .produce();
        assert_eq!(check.overall_verdict(), Verdict::Fail);
        assert!(check.checks.iter().any(|c| c.name == "E0308"));
        // The diagnostic itself is the failure; no extra exit check.
        assert!(!check.checks.iter().any(|c| c.name == "cargo::check"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn timeout_kills_a_hanging_test_run_and_keeps_finished_results() {
        let dir = std::env::temp_dir().join(format!("dev-tools-timeout-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(
            dir.join("Cargo.toml"),
            "[package]\nname = \"hangs\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("src/lib.rs"),
            "#[test]\nfn quick() {}\n\n#[test]\nfn hangs() {\n    std::thread::sleep(std::time::Duration::from_secs(600));\n}\n",
        )
        .unwrap();
        // Build first so the time limit only covers running the tests.
        let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
        let built = Command::new(cargo)
            .args(["test", "--no-run"])
            .current_dir(&dir)
            .output()
            .unwrap();
        assert!(built.status.success(), "{built:?}");

        let started = std::time::Instant::now();
        let report = cargo_test_producer("hangs", "0.1.0")
            .in_dir(&dir)
            .timeout(Duration::from_secs(5))
            .produce();
        assert!(
            started.elapsed() < Duration::from_secs(60),
            "took {:?}",
            started.elapsed()
        );
        assert_eq!(report.overall_verdict(), Verdict::Fail);
        assert!(report
            .checks
            .iter()
            .any(|c| c.name == "cargo::test::timeout"));
        assert!(report
            .checks
            .iter()
            .any(|c| c.name == "quick" && c.verdict == Verdict::Pass));
        let _ = std::fs::remove_dir_all(&dir);
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
