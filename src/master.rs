use clap::{Parser};

use modbus_watch::client::comm::ModbusWatcher;
use modbus_watch::client::model::PolledConnection;
use modbus_watch::common::logging::{init_logger, LogLevel};

use tokio::sync::mpsc;
use tracing::{error, info};

#[derive(Parser, Debug)]
struct Args {
    config_file: std::path::PathBuf,
    #[arg(long = "db", default_value = "modbus-watch.db3")]
    db_file: std::path::PathBuf,
    #[arg(long = "log-level", value_enum, default_value_t = LogLevel::Info)]
    log_level: LogLevel,
    #[arg(long = "log-file", default_value = "")]
    log_file: String,
    #[arg(long = "api-port", default_value = "8000")]
    api_port: u16,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    //We have to keep the worker_guard alive
    let _worker_guard = if args.log_level != LogLevel::No {
        init_logger(args.log_level, args.log_file)
    } else {
        None
    };

    let config = std::fs::read_to_string(&args.config_file).unwrap_or_else(|e| {
        error!("Couldn't read config file: {}", e);
        std::process::exit(1);
    });

    let config: Vec<PolledConnection> = serde_json::from_str(&config).unwrap_or_else(|e| {
        error!("Couldn't parse config file: {}", e);
        std::process::exit(1);
    });

    for connection in &config {
        if let Err(err) = connection.validate() {
            error!("Wrong config:\n{}", err);
            std::process::exit(1);
        }
    }

    let (tx, rx) = mpsc::channel::<modbus_watch::client::data::InsertValueMessage>(1024);

    let mut db = modbus_watch::client::data::DbManager::new(args.db_file, &config, rx)
        .unwrap_or_else(|e| {
            error!("Couldn't init db: {}", e);
            std::process::exit(1);
        });

    let api_db_access = db.get_db();
    let aggregation_db_access = db.get_db();

    tokio::spawn(async move {
        db.listen().await;
    });

    let mut modbus_watcher = ModbusWatcher::new(config.clone(), tx);

    modbus_watcher.watch().await.unwrap_or_else(|e| {
        error!("Couldn't init polling tasks: {}", e);
        std::process::exit(1);
    });

    modbus_watch::client::api::serve_api(config.clone(), api_db_access, args.api_port).await;

    modbus_watch::client::aggregations::start_aggregation_building(aggregation_db_access, config)
        .await;

    tokio::signal::ctrl_c().await.unwrap();

    info!("polling interrupted by user, stopping process");

    std::process::exit(0);
}
