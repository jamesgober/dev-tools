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
    let p = TempProject::new().with_file("a.txt", "hi").build().unwrap();
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

#[cfg(feature = "stress")]
#[test]
fn smoke_stress_available() {
    use dev_tools::stress::{StressRun, Workload};
    #[derive(Clone)]
    struct Noop;
    impl Workload for Noop {
        fn run_once(&self) {
            std::hint::black_box(1 + 1);
        }
    }
    let r = StressRun::new("x").iterations(50).threads(1).execute(&Noop);
    assert!(r.ops_per_sec() > 0.0);
}

#[cfg(feature = "chaos")]
#[test]
fn smoke_chaos_available() {
    use dev_tools::chaos::{assert_recovered, FailureMode, FailureSchedule};
    let s = FailureSchedule::on_attempts(&[1], FailureMode::IoError);
    assert!(s.maybe_fail(1).is_err());
    let c = assert_recovered("op", 1, 1, true);
    assert!(matches!(c.verdict, dev_tools::report::Verdict::Pass));
}

#[cfg(feature = "async")]
#[test]
fn smoke_async_re_export_compiles() {
    // No tokio runtime in dev-tools dev-deps (DIRECTIVES § 7); just
    // verify the re-export and a key item are accessible at compile time.
    fn _assert_trait_visible<T: dev_tools::r#async::AsyncCheck>() {}
    // Type-system-level check; never called.
    let _f: fn() = || {};
    let _ = _f;
}

#[cfg(all(feature = "fixtures", feature = "bench"))]
#[test]
fn smoke_full_run_combines_fixtures_and_bench() {
    use dev_tools::full_run;
    use dev_tools::report::Verdict;

    let fix = dev_tools::fixtures::FixtureProducer::new("temp_lifecycle", "0.1.0", || {
        let _p = dev_tools::fixtures::TempProject::new()
            .with_file("README.md", "hi")
            .build()?;
        Ok(())
    });

    let benchp = dev_tools::bench::BenchProducer::new(
        || {
            let mut b = dev_tools::bench::Benchmark::new("noop");
            for _ in 0..3 {
                b.iter(|| std::hint::black_box(1 + 1));
            }
            b.finish()
        },
        "0.1.0",
        None,
        dev_tools::bench::Threshold::regression_pct(20.0),
    );

    let multi = full_run!("crate", "0.1.0"; fix, benchp);
    assert_eq!(multi.reports.len(), 2);
    assert!(matches!(
        multi.overall_verdict(),
        Verdict::Pass | Verdict::Skip
    ));
}
