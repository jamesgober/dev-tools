//! Demonstrate `clippy_producer`: build a `Producer` that runs
//! `cargo clippy --message-format=json` and emits one `CheckResult` per
//! diagnostic (warning → warn, error → fail).
//!
//! The example only constructs the producer by default. Set the env var
//! `DEV_TOOLS_EXAMPLE_RUN=1` to actually spawn `cargo clippy` and print
//! the resulting JSON report. Opt-in so CI doesn't pay for it on every
//! run.
//!
//! ```text
//! cargo run --example clippy_producer
//! DEV_TOOLS_EXAMPLE_RUN=1 cargo run --example clippy_producer
//! ```

use dev_tools::producers::clippy_producer;
use dev_tools::report::Producer;

fn main() {
    let producer = clippy_producer("my-crate", "0.1.0");
    println!("Constructed clippy_producer for 'my-crate' v0.1.0.");

    if std::env::var("DEV_TOOLS_EXAMPLE_RUN").is_ok() {
        let report = producer.produce();
        println!("{}", report.to_json().expect("serialize"));
    } else {
        println!("Set DEV_TOOLS_EXAMPLE_RUN=1 to spawn `cargo clippy --message-format=json`");
        println!("in the current directory and print the resulting JSON report.");
    }
}
