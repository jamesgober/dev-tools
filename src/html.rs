//! HTML meta-report rendering.
//!
//! [`multi_report_to_html`] takes a [`MultiReport`] and produces a
//! single self-contained HTML document: inline CSS, inline SVG charts,
//! no external assets, no JavaScript dependencies. The page is fully
//! readable with JavaScript disabled; collapse/expand uses native
//! HTML5 `<details>` elements.
//!
//! Colors are CSS custom properties at the top of the document, sourced
//! from [`crate::brand`]. The brand kit lands later; replacing those
//! constants automatically re-themes every report.
//!
//! Output is deterministic for a given `MultiReport`: no clock reads,
//! no random IDs, iteration order matches the input.
//!
//! Use via the [`MultiReportHtmlExt`](crate::MultiReportHtmlExt) trait
//! to call `multi.to_html()` directly, or call this function on a
//! borrowed `MultiReport`.

// HTML emission deliberately uses `write!(..., "...\n", ...)` so the
// runs of literal HTML/CSS read top-to-bottom without mixing `writeln!`
// and `push_str` styles.
#![allow(clippy::write_with_newline)]

use std::fmt::Write;

use dev_report::{CheckResult, MultiReport, Report, Severity, Verdict};

use crate::brand;

/// Render `multi` as a self-contained HTML document.
///
/// See [`crate::html`] for output guarantees.
///
/// # Example
///
/// ```
/// use dev_report::{CheckResult, MultiReport, Report};
///
/// let mut bench = Report::new("crate", "0.1.0").with_producer("dev-bench");
/// bench.push(CheckResult::pass("hot"));
/// let mut multi = MultiReport::new("crate", "0.1.0");
/// multi.push(bench);
///
/// let html = dev_tools::html::multi_report_to_html(&multi);
/// assert!(html.starts_with("<!DOCTYPE html>"));
/// assert!(html.contains("</html>"));
/// ```
pub fn multi_report_to_html(multi: &MultiReport) -> String {
    let mut out = String::with_capacity(16 * 1024);
    write_doc(&mut out, multi);
    out
}

fn write_doc(out: &mut String, multi: &MultiReport) {
    let overall = multi.overall_verdict();
    let (pass, fail, warn, skip) = multi.verdict_counts();
    let total = pass + fail + warn + skip;

    out.push_str("<!DOCTYPE html>\n");
    out.push_str("<html lang=\"en\">\n<head>\n");
    out.push_str("<meta charset=\"UTF-8\">\n");
    out.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    out.push_str("<title>");
    write_html_text(out, &multi.subject);
    out.push_str(" v");
    write_html_text(out, &multi.subject_version);
    out.push_str(" — dev-tools meta-report</title>\n");
    write_inline_css(out);
    out.push_str("</head>\n<body>\n");

    write_header(out, multi, overall, total);
    write_summary_section(out, pass, fail, warn, skip);
    write_duration_section(out, multi);
    write_producers_section(out, multi);
    write_footer(out);

    out.push_str("</body>\n</html>\n");
}

fn write_inline_css(out: &mut String) {
    out.push_str("<style>\n:root {\n");
    write!(out, "    --color-accent: {};\n", brand::COLOR_ACCENT).unwrap();
    write!(out, "    --color-pass: {};\n", brand::COLOR_PASS).unwrap();
    write!(out, "    --color-fail: {};\n", brand::COLOR_FAIL).unwrap();
    write!(out, "    --color-warn: {};\n", brand::COLOR_WARN).unwrap();
    write!(out, "    --color-lint: {};\n", brand::COLOR_LINT).unwrap();
    write!(out, "    --color-bg: {};\n", brand::COLOR_BG).unwrap();
    write!(out, "    --color-fg: {};\n", brand::COLOR_FG).unwrap();
    out.push_str("    --color-muted: #888;\n");
    out.push_str("    --color-surface: #1a1f26;\n");
    out.push_str("    --color-border: #2a3038;\n");
    out.push_str("}\n");
    out.push_str(r#"
* { box-sizing: border-box; }
html, body {
    margin: 0;
    padding: 0;
    background: var(--color-bg);
    color: var(--color-fg);
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", system-ui, sans-serif;
    line-height: 1.5;
}
main {
    max-width: 1100px;
    margin: 0 auto;
    padding: 2rem 1.5rem 4rem;
}
header.page {
    border-bottom: 1px solid var(--color-border);
    padding-bottom: 1.5rem;
    margin-bottom: 2rem;
}
header.page h1 {
    margin: 0 0 .25rem;
    font-size: 1.6rem;
    font-weight: 600;
}
header.page .subtitle {
    color: var(--color-muted);
    font-size: .95rem;
}
.verdict-badge {
    display: inline-block;
    padding: .35rem .9rem;
    border-radius: 4px;
    font-size: .8rem;
    font-weight: 700;
    letter-spacing: .08em;
    text-transform: uppercase;
    margin-top: .85rem;
    color: #fff;
}
.verdict-pass  { background: var(--color-pass); }
.verdict-fail  { background: var(--color-fail); }
.verdict-warn  { background: var(--color-warn); color: #1a1a1a; }
.verdict-skip  { background: var(--color-muted); }
section { margin: 2rem 0; }
section > h2 {
    font-size: 1.1rem;
    font-weight: 600;
    margin: 0 0 .9rem;
    padding-bottom: .35rem;
    border-bottom: 1px solid var(--color-border);
}
.counts {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
    margin: .5rem 0 1.2rem;
}
.count {
    flex: 1 1 6rem;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-left: 4px solid var(--color-muted);
    padding: .7rem 1rem;
    border-radius: 3px;
}
.count.pass { border-left-color: var(--color-pass); }
.count.fail { border-left-color: var(--color-fail); }
.count.warn { border-left-color: var(--color-warn); }
.count.skip { border-left-color: var(--color-muted); }
.count .value { font-size: 1.4rem; font-weight: 700; display: block; }
.count .label { font-size: .8rem; color: var(--color-muted); text-transform: uppercase; letter-spacing: .06em; }
svg.bar-chart, svg.histogram {
    width: 100%;
    height: auto;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 3px;
    padding: .25rem;
}
details.producer {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 3px;
    margin: .5rem 0;
    padding: 0;
}
details.producer > summary {
    cursor: pointer;
    padding: .75rem 1rem;
    font-weight: 600;
    user-select: none;
    list-style: none;
}
details.producer > summary::-webkit-details-marker { display: none; }
details.producer > summary::before {
    content: "▸ ";
    color: var(--color-muted);
    font-weight: 400;
    transition: transform .15s ease;
    display: inline-block;
    width: 1em;
}
details.producer[open] > summary::before { content: "▾ "; }
details.producer summary .producer-meta {
    color: var(--color-muted);
    font-weight: 400;
    margin-left: .5rem;
    font-size: .9rem;
}
table.checks {
    width: 100%;
    border-collapse: collapse;
    font-size: .9rem;
}
table.checks th, table.checks td {
    text-align: left;
    padding: .45rem .75rem;
    border-top: 1px solid var(--color-border);
    vertical-align: top;
}
table.checks th {
    background: var(--color-bg);
    color: var(--color-muted);
    text-transform: uppercase;
    font-size: .72rem;
    letter-spacing: .06em;
    font-weight: 600;
}
table.checks td.verdict {
    font-weight: 600;
    white-space: nowrap;
    width: 6ch;
}
table.checks td.verdict.pass { color: var(--color-pass); }
table.checks td.verdict.fail { color: var(--color-fail); }
table.checks td.verdict.warn { color: var(--color-warn); }
table.checks td.verdict.skip { color: var(--color-muted); }
table.checks td.duration { color: var(--color-muted); white-space: nowrap; width: 8ch; text-align: right; }
table.checks td.severity { color: var(--color-muted); white-space: nowrap; }
table.checks td.detail { color: var(--color-fg); }
table.checks td.detail .empty { color: var(--color-muted); }
footer.page {
    margin-top: 3rem;
    padding-top: 1rem;
    border-top: 1px solid var(--color-border);
    color: var(--color-muted);
    font-size: .85rem;
    text-align: center;
}
@media print {
    body { background: #fff; color: #000; }
    main { max-width: none; padding: 0; }
    details.producer { border: 1px solid #ccc; }
    details.producer[open] > summary::before, details.producer > summary::before { content: ""; }
    details.producer > div.producer-body { display: block !important; }
}
"#);
    out.push_str("</style>\n");
}

fn write_header(out: &mut String, multi: &MultiReport, overall: Verdict, total: usize) {
    out.push_str("<main>\n<header class=\"page\">\n");
    out.push_str("    <h1>");
    write_html_text(out, &multi.subject);
    out.push_str(" <span class=\"version\">v");
    write_html_text(out, &multi.subject_version);
    out.push_str("</span></h1>\n");

    out.push_str("    <div class=\"subtitle\">started ");
    write_html_text(out, &multi.started_at.to_rfc3339());
    if let Some(end) = multi.finished_at {
        out.push_str(" · finished ");
        write_html_text(out, &end.to_rfc3339());
    }
    write!(
        out,
        " · {} check{} across {} producer{}",
        total,
        if total == 1 { "" } else { "s" },
        multi.reports.len(),
        if multi.reports.len() == 1 { "" } else { "s" }
    )
    .unwrap();
    out.push_str("</div>\n");

    let verdict_class = match overall {
        Verdict::Pass => "verdict-pass",
        Verdict::Fail => "verdict-fail",
        Verdict::Warn => "verdict-warn",
        Verdict::Skip => "verdict-skip",
    };
    write!(out, "    <div class=\"verdict-badge {}\">", verdict_class).unwrap();
    out.push_str(verdict_label(overall));
    out.push_str("</div>\n</header>\n");
}

fn verdict_label(v: Verdict) -> &'static str {
    match v {
        Verdict::Pass => "Pass",
        Verdict::Fail => "Fail",
        Verdict::Warn => "Warn",
        Verdict::Skip => "Skip",
    }
}

fn write_summary_section(out: &mut String, pass: usize, fail: usize, warn: usize, skip: usize) {
    out.push_str("<section>\n    <h2>Summary</h2>\n    <div class=\"counts\">\n");
    for (label, count, class) in [
        ("Pass", pass, "pass"),
        ("Fail", fail, "fail"),
        ("Warn", warn, "warn"),
        ("Skip", skip, "skip"),
    ] {
        write!(
            out,
            "        <div class=\"count {}\"><span class=\"value\">{}</span><span class=\"label\">{}</span></div>\n",
            class, count, label
        )
        .unwrap();
    }
    out.push_str("    </div>\n");
    write_bar_chart(out, pass, fail, warn, skip);
    out.push_str("</section>\n");
}

fn write_bar_chart(out: &mut String, pass: usize, fail: usize, warn: usize, skip: usize) {
    let total = pass + fail + warn + skip;
    if total == 0 {
        return;
    }
    // Stacked horizontal bar: pass | fail | warn | skip
    let width = 1000u32;
    let height = 40u32;
    let mut x = 0u32;
    let mk_seg = |count: usize, color_var: &str| -> Option<(u32, String)> {
        if count == 0 {
            return None;
        }
        let w = ((count as f64 / total as f64) * width as f64).round() as u32;
        if w == 0 {
            return None;
        }
        Some((w, color_var.into()))
    };
    let segments: Vec<(usize, &'static str, &'static str)> = vec![
        (pass, "var(--color-pass)", "Pass"),
        (fail, "var(--color-fail)", "Fail"),
        (warn, "var(--color-warn)", "Warn"),
        (skip, "var(--color-muted)", "Skip"),
    ];

    write!(
        out,
        "    <svg class=\"bar-chart\" viewBox=\"0 0 {} {}\" preserveAspectRatio=\"none\" aria-label=\"Verdict distribution: {} pass, {} fail, {} warn, {} skip\">\n",
        width, height, pass, fail, warn, skip
    )
    .unwrap();
    for (count, color, label) in segments {
        if let Some((w, _)) = mk_seg(count, color) {
            write!(
                out,
                "        <rect x=\"{}\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"{}\"><title>{} — {}</title></rect>\n",
                x, w, height, color, label, count
            )
            .unwrap();
            x = x.saturating_add(w);
        }
    }
    // Fill any rounding gap on the right with the last non-zero color.
    if x < width {
        // pick fallback color: muted
        write!(
            out,
            "        <rect x=\"{}\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"var(--color-muted)\"/>\n",
            x,
            width - x,
            height
        )
        .unwrap();
    }
    out.push_str("    </svg>\n");
}

fn write_duration_section(out: &mut String, multi: &MultiReport) {
    let mut durations: Vec<u64> = multi
        .reports
        .iter()
        .flat_map(|r| r.checks.iter().filter_map(|c| c.duration_ms))
        .collect();
    if durations.is_empty() {
        return;
    }
    durations.sort_unstable();
    let min = *durations.first().unwrap();
    let max = *durations.last().unwrap();
    let count = durations.len();

    out.push_str("<section>\n    <h2>Duration distribution</h2>\n");
    write!(
        out,
        "    <div class=\"subtitle\" style=\"color:var(--color-muted);font-size:.85rem;margin-bottom:.5rem\">{} sample{}, {} ms min, {} ms max</div>\n",
        count,
        if count == 1 { "" } else { "s" },
        min,
        max
    )
    .unwrap();
    write_histogram(out, &durations, min, max);
    out.push_str("</section>\n");
}

fn write_histogram(out: &mut String, sorted_durations: &[u64], min: u64, max: u64) {
    let buckets: usize = 10;
    let mut bins = vec![0usize; buckets];
    if min == max {
        bins[0] = sorted_durations.len();
    } else {
        let range = max - min;
        for &d in sorted_durations {
            let idx = ((d - min) as f64 / range as f64 * buckets as f64) as usize;
            let idx = idx.min(buckets - 1);
            bins[idx] += 1;
        }
    }
    let max_bin = bins.iter().copied().max().unwrap_or(1).max(1);
    let width = 1000u32;
    let height = 160u32;
    let bar_w = width / buckets as u32;

    write!(
        out,
        "    <svg class=\"histogram\" viewBox=\"0 0 {} {}\" preserveAspectRatio=\"none\" aria-label=\"Histogram of check durations\">\n",
        width, height
    )
    .unwrap();

    for (i, &bin) in bins.iter().enumerate() {
        let x = i as u32 * bar_w;
        let h = ((bin as f64 / max_bin as f64) * (height - 10) as f64).round() as u32;
        let y = height - h;
        let bucket_lo = if max == min {
            min
        } else {
            min + (max - min) * i as u64 / buckets as u64
        };
        let bucket_hi = if max == min {
            max
        } else {
            min + (max - min) * (i as u64 + 1) / buckets as u64
        };
        write!(
            out,
            "        <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"var(--color-accent)\"><title>{}\u{2013}{} ms — {} sample{}</title></rect>\n",
            x + 1,
            y,
            bar_w.saturating_sub(2),
            h,
            bucket_lo,
            bucket_hi,
            bin,
            if bin == 1 { "" } else { "s" }
        )
        .unwrap();
    }
    out.push_str("    </svg>\n");
}

fn write_producers_section(out: &mut String, multi: &MultiReport) {
    out.push_str("<section>\n    <h2>Per-producer reports</h2>\n");
    if multi.reports.is_empty() {
        out.push_str("    <p style=\"color:var(--color-muted)\">No reports.</p>\n");
        out.push_str("</section>\n");
        return;
    }
    for report in &multi.reports {
        write_producer(out, report);
    }
    out.push_str("</section>\n");
}

fn write_producer(out: &mut String, report: &Report) {
    let (pass, fail, warn, skip) = report.verdict_counts();
    let total = pass + fail + warn + skip;
    let producer_name = report.producer.as_deref().unwrap_or("(unnamed producer)");
    // Open <details> by default if the producer has failures or warnings.
    let open_attr = if fail > 0 || warn > 0 { " open" } else { "" };
    write!(out, "    <details class=\"producer\"{}>\n", open_attr).unwrap();
    out.push_str("        <summary>");
    write_html_text(out, producer_name);
    write!(
        out,
        "<span class=\"producer-meta\">— {} pass · {} fail · {} warn · {} skip · {} total</span>",
        pass, fail, warn, skip, total
    )
    .unwrap();
    out.push_str("</summary>\n");
    out.push_str("        <div class=\"producer-body\">\n");
    if report.checks.is_empty() {
        out.push_str("            <p style=\"padding:0 1rem 1rem;color:var(--color-muted)\">No checks.</p>\n");
    } else {
        write_check_table(out, &report.checks);
    }
    out.push_str("        </div>\n    </details>\n");
}

fn write_check_table(out: &mut String, checks: &[CheckResult]) {
    out.push_str("            <table class=\"checks\">\n");
    out.push_str("                <thead><tr><th>Check</th><th>Verdict</th><th>Severity</th><th>Duration</th><th>Detail</th></tr></thead>\n");
    out.push_str("                <tbody>\n");
    for c in checks {
        write_check_row(out, c);
    }
    out.push_str("                </tbody>\n            </table>\n");
}

fn write_check_row(out: &mut String, c: &CheckResult) {
    let verdict_class = match c.verdict {
        Verdict::Pass => "pass",
        Verdict::Fail => "fail",
        Verdict::Warn => "warn",
        Verdict::Skip => "skip",
    };
    out.push_str("                    <tr>\n");
    out.push_str("                        <td>");
    write_html_text(out, &c.name);
    out.push_str("</td>\n");
    write!(
        out,
        "                        <td class=\"verdict {}\">{}</td>\n",
        verdict_class,
        verdict_label(c.verdict)
    )
    .unwrap();
    out.push_str("                        <td class=\"severity\">");
    match c.severity {
        Some(s) => out.push_str(severity_label(s)),
        None => out.push('—'),
    }
    out.push_str("</td>\n                        <td class=\"duration\">");
    match c.duration_ms {
        Some(ms) => {
            write!(out, "{} ms", ms).unwrap();
        }
        None => out.push('—'),
    }
    out.push_str("</td>\n                        <td class=\"detail\">");
    match &c.detail {
        Some(d) => write_html_text(out, d),
        None => out.push_str("<span class=\"empty\">—</span>"),
    }
    out.push_str("</td>\n                    </tr>\n");
}

fn severity_label(s: Severity) -> &'static str {
    match s {
        Severity::Info => "Info",
        Severity::Warning => "Warning",
        Severity::Error => "Error",
        Severity::Critical => "Critical",
    }
}

fn write_footer(out: &mut String) {
    out.push_str("<footer class=\"page\">");
    write_html_text(out, brand::FOOTER);
    out.push_str("</footer>\n</main>\n");
}

fn write_html_text(out: &mut String, s: &str) {
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use dev_report::Evidence;

    fn frozen_multi() -> MultiReport {
        let t0 = chrono::Utc.with_ymd_and_hms(2026, 5, 11, 12, 0, 0).unwrap();
        let mut bench = Report::new("crate", "0.1.0").with_producer("dev-bench");
        bench.set_started_at(t0);
        let mut c1 = CheckResult::pass("hot_path")
            .with_duration_ms(120)
            .with_evidence(Evidence::numeric("mean_ns", 1234.5));
        c1.at = t0;
        let mut c2 = CheckResult::pass("cold_path").with_duration_ms(45);
        c2.at = t0;
        bench.push(c1);
        bench.push(c2);
        bench.set_finished_at(Some(t0));

        let mut chaos = Report::new("crate", "0.1.0").with_producer("dev-chaos");
        chaos.set_started_at(t0);
        let mut c3 = CheckResult::fail("recover", Severity::Error)
            .with_detail("service did not recover within 5s")
            .with_duration_ms(5_001);
        c3.at = t0;
        let mut c4 = CheckResult::warn("flaky::retry", Severity::Warning)
            .with_detail("3rd retry of 5 was slow");
        c4.at = t0;
        let mut c5 = CheckResult::skip("network::flaky").with_detail("no net in sandbox");
        c5.at = t0;
        chaos.push(c3);
        chaos.push(c4);
        chaos.push(c5);
        chaos.set_finished_at(Some(t0));

        let mut multi = MultiReport::new("crate", "0.1.0");
        multi.started_at = t0;
        multi.push(bench);
        multi.push(chaos);
        multi.finished_at = Some(t0);
        multi
    }

    #[test]
    fn output_is_doctype_html() {
        let html = multi_report_to_html(&frozen_multi());
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("<html lang=\"en\""));
        assert!(html.contains("</html>"));
    }

    #[test]
    fn output_contains_subject_and_version() {
        let html = multi_report_to_html(&frozen_multi());
        assert!(html.contains(">crate"));
        assert!(html.contains("v0.1.0"));
    }

    #[test]
    fn output_contains_verdict_badge_for_overall_fail() {
        let html = multi_report_to_html(&frozen_multi());
        assert!(html.contains("verdict-badge verdict-fail"));
        assert!(html.contains(">Fail<"));
    }

    #[test]
    fn output_contains_one_details_per_producer() {
        let html = multi_report_to_html(&frozen_multi());
        let n = html.matches("<details class=\"producer\"").count();
        assert_eq!(n, 2);
        assert!(html.contains("dev-bench"));
        assert!(html.contains("dev-chaos"));
    }

    #[test]
    fn output_is_deterministic() {
        let multi = frozen_multi();
        let a = multi_report_to_html(&multi);
        let b = multi_report_to_html(&multi);
        assert_eq!(a, b);
    }

    #[test]
    fn output_escapes_special_chars() {
        let mut r = Report::new("crate<&>", "0.1.0").with_producer("dev<bad>");
        r.push(CheckResult::fail("name<&>\"'", Severity::Error).with_detail("oh <bad> & \"x\""));
        let mut m = MultiReport::new("crate<&>", "0.1.0");
        m.push(r);
        let html = multi_report_to_html(&m);
        assert!(html.contains("crate&lt;&amp;&gt;"));
        assert!(html.contains("name&lt;&amp;&gt;&quot;&#39;"));
        assert!(html.contains("oh &lt;bad&gt; &amp; &quot;x&quot;"));
        assert!(!html.contains("crate<&>"));
    }

    #[test]
    fn empty_multi_renders_without_panic() {
        let m = MultiReport::new("empty", "0.0.0");
        let html = multi_report_to_html(&m);
        assert!(html.contains("No reports."));
        assert!(html.contains("verdict-skip"));
    }

    #[test]
    fn duration_section_present_when_any_duration() {
        let html = multi_report_to_html(&frozen_multi());
        assert!(html.contains("Duration distribution"));
        assert!(html.contains("<svg class=\"histogram\""));
    }

    #[test]
    fn duration_section_absent_when_no_durations() {
        let mut r = Report::new("c", "0.1.0").with_producer("p");
        r.push(CheckResult::pass("a"));
        let mut m = MultiReport::new("c", "0.1.0");
        m.push(r);
        let html = multi_report_to_html(&m);
        assert!(!html.contains("Duration distribution"));
        assert!(!html.contains("<svg class=\"histogram\""));
    }

    #[test]
    fn producer_with_failures_is_open_by_default() {
        let html = multi_report_to_html(&frozen_multi());
        // dev-chaos has fail+warn; should be open. dev-bench is all pass; closed.
        assert!(html.contains("<details class=\"producer\" open>"));
        let open_count = html.matches("<details class=\"producer\" open>").count();
        let closed_count = html.matches("<details class=\"producer\">").count();
        assert_eq!(open_count, 1);
        assert_eq!(closed_count, 1);
    }

    #[test]
    fn no_external_resources() {
        let html = multi_report_to_html(&frozen_multi());
        assert!(!html.contains("http://"));
        // Allow https://www.w3.org for SVG namespace if present; we don't use it explicitly.
        // Disallow external <script>, <link>, <img>.
        assert!(!html.contains("<script src="));
        assert!(!html.contains("<link rel="));
        assert!(!html.contains("<img "));
    }

    #[test]
    fn uses_css_custom_properties_for_brand_colors() {
        let html = multi_report_to_html(&frozen_multi());
        assert!(html.contains("--color-accent:"));
        assert!(html.contains("--color-pass:"));
        assert!(html.contains("--color-fail:"));
        assert!(html.contains("--color-warn:"));
        assert!(html.contains("var(--color-accent)"));
    }

    #[test]
    fn footer_contains_brand_footer_constant() {
        let html = multi_report_to_html(&frozen_multi());
        assert!(html.contains(brand::FOOTER));
    }

    #[test]
    fn output_size_is_reasonable() {
        // Smoke check: shouldn't be larger than ~64 KiB for a small report.
        let html = multi_report_to_html(&frozen_multi());
        assert!(html.len() < 64 * 1024, "got {} bytes", html.len());
        assert!(html.len() > 1_000, "got {} bytes (likely truncated)", html.len());
    }
}
