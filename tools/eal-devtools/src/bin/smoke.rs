//! Dispatch the native installed acceptance checks.
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.as_slice() == ["--help"] || args.as_slice() == ["-h"] {
        println!(
            "eal-smoke applications|coordination --prefix DIR [--workspace DIR] [--worker FILE]\neal-smoke consumer --consumer FILE --binary FILE --node FILE [--views FILE] [--save-views FILE]"
        );
        return;
    }
    match eal_devtools::smoke::Options::parse(args).and_then(eal_devtools::smoke::run) {
        Ok(value) => println!("{value}"),
        Err(error) => {
            eprintln!("{}", eal_devtools::smoke::result_error(&error));
            std::process::exit(1);
        }
    }
}
