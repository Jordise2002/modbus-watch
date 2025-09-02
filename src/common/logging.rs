use clap::ValueEnum;
use std::io;
use tracing::Level;
use tracing_appender::rolling;
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum LogLevel {
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

pub fn init_logger(
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
