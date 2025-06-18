use clap::Parser;

use model::PolledConnection;

mod model;

#[derive(Parser, Debug)]
struct Args {
    config_file: std::path::PathBuf
}
fn main() {
    let args = Args::parse();

    let config = std::fs::read_to_string(&args.config_file).unwrap_or_else(|e| {
        eprintln!("Couldn't read config file: {e}");
        std::process::exit(1);
    });

    let config: Vec<PolledConnection> = serde_json::from_str(&config).unwrap_or_else(|e| {
        eprintln!("Couldn't parse config file: {e}");
        std::process::exit(1);
    });

    for connection in &config {
        if let Err(err) = connection.validate()
        {
            eprintln!("Wrong config:\n{}", err);
            std::process::exit(1);
        }
    }

    
}
