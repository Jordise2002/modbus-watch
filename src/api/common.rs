use crate::api::ApiState;

use axum::{extract::State, Json};
use std::sync::Arc;

pub async fn list_values(State(state): State<Arc<ApiState>>) -> Json<Vec<String>> {
    let mut values = vec![];

    for connection in &state.config {
        for slave in &connection.slaves {
            for value in &slave.values {
                values.push(value.id.clone());
            }
        }
    }

    Json(values)
}
