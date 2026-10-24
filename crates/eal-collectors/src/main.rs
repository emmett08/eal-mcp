//! One JSON request and one successful observation envelope.
use std::io::{Read, Write};

#[tokio::main]
async fn main() {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    if arguments
        .iter()
        .any(|value| value == "--help" || value == "-h")
    {
        println!("{}", eal_collectors::config::HELP);
        return;
    }
    if let Err(error) = run(arguments).await {
        eprintln!("eal-collector: {error}");
        std::process::exit(2);
    }
}

async fn run(arguments: Vec<String>) -> Result<(), String> {
    if arguments.first().map(String::as_str) == Some("describe") {
        let version = match arguments.as_slice() {
            [_] => "1",
            [_, flag, version] if flag == "--catalogue-version" => version,
            _ => return Err("describe accepts only --catalogue-version 1|2".into()),
        };
        let catalogue = eal_collectors::catalogue::native_version(version)?;
        let bytes = serde_json::to_vec_pretty(catalogue)
            .map_err(|_| "Collector catalogue encoding failed")?;
        std::io::stdout()
            .write_all(&bytes)
            .map_err(|_| "Cannot write collector catalogue")?;
        return Ok(());
    }
    let config = eal_collectors::Config::parse(arguments)?;
    let mut input = Vec::new();
    std::io::stdin()
        .take(1_048_577)
        .read_to_end(&mut input)
        .map_err(|_| "Cannot read the collector request")?;
    if input.len() > 1_048_576 {
        return Err("Collector request exceeds 1 MiB".into());
    }
    let request = eal_collectors::Request::parse(&input)?;
    let envelope = eal_collectors::collect(&config, &request).await?;
    let bytes = serde_json::to_vec(&envelope).map_err(|_| "Observation encoding failed")?;
    std::io::stdout()
        .write_all(&bytes)
        .map_err(|_| "Cannot write observation")?;
    Ok(())
}
