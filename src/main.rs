use clap::{Parser, ValueEnum};

use comm::ModbusWatcher;
use model::PolledConnection;

use std::io;
use tokio::sync::mpsc;
use tracing::{error, info, Level};
use tracing_appender::rolling;
use tracing_subscriber::{fmt, EnvFilter};

mod comm;
mod data;
mod model;
mod api;

#[derive(Debug, Clone, ValueEnum, PartialEq)]
enum LogLevel {
    No,
    Debug,
    Info,
    Warning,
    Error,
}

impl LogLevel {
    pub fn to_tracing_level(&self) -> Level {
        match self {
            LogLevel::Debug => Level::DEBUG,
            LogLevel::No => panic!("Shouldn't be called"),
            LogLevel::Info => Level::INFO,
            LogLevel::Warning => Level::WARN,
            LogLevel::Error => Level::ERROR,
        }
    }
}

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
    api_port: u16
}

fn init_logger(
    log_level: LogLevel,
    log_file: String,
) -> Option<tracing_appender::non_blocking::WorkerGuard> {
    let env_filter = EnvFilter::from_default_env()
        .add_directive(log_level.to_tracing_level().as_str().parse().unwrap());

    if !log_file.is_empty() {
        let file_appender = rolling::daily(".", log_file);
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        let subscriber = fmt()
            .with_writer(non_blocking)
            .with_ansi(false)
            .with_env_filter(env_filter)
            .with_file(false)
            .with_target(false)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("No se pudo establecer subscriber para archivo");

        //We need to keep the worker guard alive
        Some(guard)
    } else {
        let subscriber = fmt()
            .with_writer(io::stdout)
            .with_env_filter(env_filter)
            .with_file(false)
            .with_target(false)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("No se pudo establecer subscriber para stdout");
        None
    }
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

    let (tx, rx) = mpsc::channel::<data::InsertValueMessage>(1024);

    let mut db = data::DbManager::new(args.db_file, &config, rx).unwrap_or_else(|e| {
        error!("Couldn't init db: {}", e);
        std::process::exit(1);
    });

    let api_db_access = db.get_db();

    tokio::spawn(async move {
        db.listen().await;
    });

    let mut modbus_watcher = ModbusWatcher::new(config.clone(), tx);

    modbus_watcher.watch().await.unwrap_or_else(|e| {
        error!("Couldn't init polling tasks: {}", e);
        std::process::exit(1);
    });

    api::serve_api(config, api_db_access, args.api_port).await;

    tokio::signal::ctrl_c().await.unwrap();

    info!("polling interrupted by user, stopping process");
}
