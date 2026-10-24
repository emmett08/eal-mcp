//! Command selection for the isolated research tools.
use eal_context_experiment::{planning, probe, rehearse};
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("rehearse") => rehearse::run(&args[1..]),
        Some("probe") => probe::run(&args[1..]),
        Some("planning") => planning::run(&args[1..]),
        Some("--help" | "-h") | None => {
            println!(
                "eal-context rehearse [--output NEW_FILE]\neal-context probe --reference-root DIR --reference-revision SHA40 --candidate-root DIR --candidate-revision SHA40 --output NEW_FILE [--git PATH] [--build] [--cargo PATH]\neal-context planning allocation|sensitivity [--replications N] [--seed N] [--output NEW_FILE]"
            );
            return;
        }
        _ => Err("Unknown command. Use eal-context --help".into()),
    };
    match result {
        Ok(report) => println!("{}", serde_json::to_string(&report).expect("valid report")),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
