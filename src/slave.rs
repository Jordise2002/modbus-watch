use clap::Parser;
use tracing::error;

use modbus_watch::common::logging::{init_logger, LogLevel};
use modbus_watch::server::model::ServedConnection;
use modbus_watch::server::state;

#[derive(Parser)]
struct Args {
    config_file: std::path::PathBuf,
    #[arg(long = "log-level", value_enum, default_value_t = LogLevel::Info)]
    log_level: LogLevel,
    #[arg(long = "log-file", default_value = "")]
    log_file: String,
}
fn main() {
    let args = Args::parse();

    //We must keep the worker guard alive
    let _worker_guard = if args.log_level != LogLevel::No {
        init_logger(args.log_level, args.log_file)
    } else {
        None
    };

    let config = std::fs::read_to_string(&args.config_file).unwrap_or_else(|e| {
        error!("Couldn't read config file: {}", e);
        std::process::exit(1);
    });

    let config: Vec<ServedConnection> = serde_json::from_str(&config).unwrap_or_else(|e| {
        error!("Couldn't parse config file: {}", e);
        std::process::exit(1);
    });

    for connection in &config {
        if let Err(err) = connection.validate() {
            error!("Wrong config:\n{}", err);
            std::process::exit(1);
        }
    }

    let app_state = state::build_app_state(&config);

    
}
