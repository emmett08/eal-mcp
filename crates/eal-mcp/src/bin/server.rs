//! Native MCP server entry point; stdout is reserved for stdio protocol frames.
#[tokio::main]
async fn main() -> std::process::ExitCode {
    let result=async {
        let arguments=std::env::args_os().skip(1).map(|v|v.into_string().map_err(|_|"Arguments must be UTF-8".to_owned())).collect::<Result<Vec<_>,_>>()?;
        if arguments.iter().any(|value|matches!(value.as_str(),"--help"|"-h")) {
            println!("usage: eal-mcp [--config TOML] [--workspace DIR] [--registry TOML] [--database SQLITE]\n  [--methods builtin|engineering|aspic|all] [--limits TOML]\n  [--transport stdio|http] [--exposure operator|registered] [--known-entry ID ...]\n  [--max-in-flight N] [--host ADDRESS] [--port N] [--path /mcp]\n  [--token-env NAME] [--allowed-host HOST ...] [--allowed-origin ORIGIN ...]");
            return Ok(());
        }
        let environment=eal_mcp::application::environment()?;
        let settings=eal_mcp::settings::parse_server_settings(&arguments,&environment,&std::env::current_dir().map_err(|e|e.to_string())?)?;
        let application=eal_mcp::application::build(settings,&environment)?;
        eal_mcp::application::run(application).await
    }.await;
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::from(2)
        }
    }
}
