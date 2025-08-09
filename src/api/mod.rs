use axum::{routing::get, Router};
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;
use std::net::SocketAddr;

use crate::model::PolledConnection;
use std::sync::Arc;

mod config;
mod value;
mod common;

pub struct ApiState {
    pub config: Vec<PolledConnection>,
    pub db: Arc<Pool<SqliteConnectionManager>>
}

pub async fn serve_api(config: Vec<PolledConnection>, db: Arc<Pool<SqliteConnectionManager>>, port: u16) {
    let state = Arc::new(ApiState { config, db });
    let api_v1 = Router::new()
        .route("/config/{id}", get(config::get_config))
        .route("/config", get(common::list_values))
        .route("/value/{id}", get(value::get_value))
        .route("/value", get(common::list_values))
        .with_state(state);

    let api = Router::new().nest("/api/v1", api_v1);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    tokio::spawn(async move {
        axum::serve(listener, api).await.unwrap();
    });
}

