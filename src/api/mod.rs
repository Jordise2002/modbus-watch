use axum::{routing::get, Router};
use std::net::SocketAddr;

use crate::model::PolledConnection;
use std::sync::Arc;

mod config;

pub struct ApiState {
    pub config: Vec<PolledConnection>,
}

pub async fn serve_api(config: Vec<PolledConnection>, port: u16) {
    let state = Arc::new(ApiState { config });
    let api_v1 = Router::new()
        .route("/config/{id}", get(config::get_config))
        .route("/config", get(config::list_config))
        .with_state(state);

    let api = Router::new().nest("/api/v1", api_v1);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    tokio::spawn(async move {
        axum::serve(listener, api).await.unwrap();
    });
}
