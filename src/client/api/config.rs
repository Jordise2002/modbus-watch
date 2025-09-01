use crate::client::model::PolledValue;
use crate::client::api::ApiState;

use axum::{extract::{Path, State}, http::StatusCode, response::{IntoResponse, Response}, Json};
use std::sync::Arc;

pub async fn get_config(State(state): State<Arc<ApiState>>, Path(id): Path<String>) -> Result<Json<PolledValue>, Response> 
{
    for connection in &state.config {
        for slave in &connection.slaves {
            for value in &slave.values {
                if value.id == id {
                    return Ok(Json(value.clone()));
                }
            }
        }
    }

    Err((StatusCode::NOT_FOUND, "Value not found").into_response())

}

