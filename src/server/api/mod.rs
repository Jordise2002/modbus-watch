use std::net::SocketAddr;

use axum::{routing::get, Router};

use crate::server::state::AppState;

mod value;

pub async fn serve_api(app_state: AppState, port: u16) {
    let api_v1 = Router::new()
        .route("/value", get(value::list_values))
        .route("/value/{id}", get(value::get_value).post(value::set_value))
        .with_state(app_state.clone());

    let api = Router::new().nest("/api/v1", api_v1);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    tokio::spawn(async move {
        axum::serve(listener, api).await.unwrap();
    });
}
