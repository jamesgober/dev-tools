//! Demonstrate `cargo_test_producer`: build a `Producer` that runs
//! `cargo test --no-fail-fast` and emits one `CheckResult` per test.
//!
//! The example only constructs the producer by default. Set the env var
//! `DEV_TOOLS_EXAMPLE_RUN=1` to actually spawn `cargo test` and print the
//! resulting JSON report — this is opt-in so the example stays fast in CI
//! and does not recursively invoke cargo on every run.
//!
//! ```text
//! cargo run --example cargo_test_producer
//! DEV_TOOLS_EXAMPLE_RUN=1 cargo run --example cargo_test_producer
//! ```

use dev_tools::producers::cargo_test_producer;
use dev_tools::report::Producer;

fn main() {
    let producer = cargo_test_producer("my-crate", "0.1.0");
    println!("Constructed cargo_test_producer for 'my-crate' v0.1.0.");

    if std::env::var("DEV_TOOLS_EXAMPLE_RUN").is_ok() {
        let report = producer.produce();
        println!("{}", report.to_json().expect("serialize"));
    } else {
        println!("Set DEV_TOOLS_EXAMPLE_RUN=1 to spawn `cargo test --no-fail-fast`");
        println!("in the current directory and print the resulting JSON report.");
    }
}
