//! Demonstrate `cargo_check_producer`: build a `Producer` that runs
//! `cargo check --message-format=json` and emits one `CheckResult` per
//! diagnostic.
//!
//! The example only constructs the producer by default. Set the env var
//! `DEV_TOOLS_EXAMPLE_RUN=1` to actually spawn `cargo check` and print
//! the resulting JSON report. Opt-in so CI stays fast.
//!
//! ```text
//! cargo run --example cargo_check_producer
//! DEV_TOOLS_EXAMPLE_RUN=1 cargo run --example cargo_check_producer
//! ```

use dev_tools::producers::cargo_check_producer;
use dev_tools::report::Producer;

fn main() {
    let producer = cargo_check_producer("my-crate", "0.1.0");
    println!("Constructed cargo_check_producer for 'my-crate' v0.1.0.");

    if std::env::var("DEV_TOOLS_EXAMPLE_RUN").is_ok() {
        let report = producer.produce();
        println!("{}", report.to_json().expect("serialize"));
    } else {
        println!("Set DEV_TOOLS_EXAMPLE_RUN=1 to spawn `cargo check --message-format=json`");
        println!("in the current directory and print the resulting JSON report.");
    }
}
