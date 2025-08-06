use crate::model::PolledValue;
use crate::api::ApiState;

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

pub async fn list_config(State(state): State<Arc<ApiState>>) -> Json<Vec<String>>
{
    let mut values = vec![];

    for connection in &state.config
    {
        for slave in &connection.slaves {
            for value in &slave.values
            {
                values.push(value.id.clone());
            }
        }
    }

    Json(values)
}
