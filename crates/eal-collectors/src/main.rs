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
    if arguments.as_slice() == ["capabilities"] {
        let bytes = serde_json::to_vec(&serde_json::json!({
            "schema":"eal.collector-capabilities/1",
            "coverage_proofs":["identifier_audit_v1"],
            "qualified_population_quantity":true
        }))
        .map_err(|_| "cannot encode collector capabilities")?;
        std::io::stdout()
            .write_all(&bytes)
            .map_err(|_| "cannot write collector capabilities")?;
        return Ok(());
    }
    if arguments.first().map(String::as_str) == Some("prepare-native-provider") {
        let prepared = eal_collectors::evidence::prepare_native_provider(&arguments[1..])?;
        let bytes =
            serde_json::to_vec(&prepared).map_err(|_| "cannot encode prepared native provider")?;
        std::io::stdout()
            .write_all(&bytes)
            .map_err(|_| "cannot write prepared native provider")?;
        return Ok(());
    }
    if arguments.first().map(String::as_str) == Some("evidence-reconcile") {
        let result = eal_collectors::evidence::reconcile(&arguments[1..])?;
        let bytes = serde_json::to_vec(&result)
            .map_err(|_| "Cannot encode source reconciliation result")?;
        std::io::stdout()
            .write_all(&bytes)
            .map_err(|_| "Cannot write source reconciliation result")?;
        return Ok(());
    }
    if arguments.first().map(String::as_str) == Some("evidence-discover") {
        let report = eal_collectors::evidence::discover(&arguments[1..]).await?;
        let bytes =
            serde_json::to_vec(&report).map_err(|_| "Evidence discovery encoding failed")?;
        std::io::stdout()
            .write_all(&bytes)
            .map_err(|_| "Cannot write evidence discovery report")?;
        return Ok(());
    }
    if arguments.first().map(String::as_str) == Some("support-graph") {
        let mut input = Vec::new();
        std::io::stdin()
            .take(1_048_577)
            .read_to_end(&mut input)
            .map_err(|_| "Cannot read support acquisition request")?;
        if input.len() > 1_048_576 {
            return Err("Collector request exceeds 1 MiB".into());
        }
        let envelope = eal_collectors::evidence::support_graph(&arguments[1..], &input).await?;
        let bytes =
            serde_json::to_vec(&envelope).map_err(|_| "Support observation encoding failed")?;
        std::io::stdout()
            .write_all(&bytes)
            .map_err(|_| "Cannot write support observation")?;
        return Ok(());
    }
    if arguments.first().map(String::as_str) == Some("describe") {
        let version = match arguments.as_slice() {
            [_] => "1",
            [_, flag, version] if flag == "--catalogue-version" => version,
            _ => return Err("describe accepts only --catalogue-version 1|2|3".into()),
        };
        let catalogue = eal_collectors::catalogue::native_version(version)?;
        let bytes = serde_json::to_vec_pretty(catalogue)
            .map_err(|_| "Collector catalogue encoding failed")?;
        std::io::stdout()
            .write_all(&bytes)
            .map_err(|_| "Cannot write collector catalogue")?;
        return Ok(());
    }
    if arguments.first().map(String::as_str) == Some("describe-api") {
        let config = eal_collectors::Config::parse(arguments.into_iter().skip(1).collect())?;
        let eal_collectors::config::ProviderConfig::Api(selected) = config.selected() else {
            return Err("describe-api requires an API provider".into());
        };
        let catalogue = eal_collectors::api::describe(selected)?;
        let bytes =
            serde_json::to_vec_pretty(&catalogue).map_err(|_| "Cannot encode API catalogue")?;
        std::io::stdout()
            .write_all(&bytes)
            .map_err(|_| "Cannot write API catalogue")?;
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
