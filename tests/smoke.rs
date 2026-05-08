use dev_tools::report::{Report, Verdict};

#[test]
fn smoke_report_always_available() {
    let r = Report::new("self", "0.1.0");
    assert_eq!(r.subject, "self");
    assert!(matches!(r.overall_verdict(), Verdict::Skip));
}

#[cfg(feature = "fixtures")]
#[test]
fn smoke_fixtures_available() {
    use dev_tools::fixtures::TempProject;
    let p = TempProject::new()
        .with_file("a.txt", "hi")
        .build()
        .unwrap();
    assert!(p.path().join("a.txt").exists());
}

#[cfg(feature = "bench")]
#[test]
fn smoke_bench_available() {
    use dev_tools::bench::Benchmark;
    let mut b = Benchmark::new("noop");
    b.iter(|| std::hint::black_box(1));
    let r = b.finish();
    assert_eq!(r.samples.len(), 1);
}
