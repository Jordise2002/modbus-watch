use axum::{routing::get, Router};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::net::SocketAddr;

use crate::client::model::PolledConnection;
use std::sync::Arc;

mod common;
mod config;
mod history;
mod value;

pub struct ApiState {
    pub config: Vec<PolledConnection>,
    pub db: Arc<Pool<SqliteConnectionManager>>,
}

pub async fn serve_api(
    config: Vec<PolledConnection>,
    db: Arc<Pool<SqliteConnectionManager>>,
    port: u16,
) {
    let state = Arc::new(ApiState { config, db });
    let api = Router::new()
        .route("/values", get(common::list_values))
        .route("/values/{id}", get(value::get_value))
        .route("/values/{id}/config", get(config::get_config))
        .route("/values/{id}/history", get(history::get_history))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    tokio::spawn(async move {
        axum::serve(listener, api).await.unwrap();
    });
}
