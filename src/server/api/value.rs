use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::{
    common::{model::Value, value_processing},
    server::state::AppState,
};

pub async fn get_value(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, Response> {
    let state = state.lock().await;

    if !state.contains_key(&id) {
        return Err((StatusCode::NOT_FOUND, "Value not defined").into_response());
    }

    let value_ref = state.get(&id).unwrap();

    let value = value_processing::format_value(
        value_processing::registers_to_bytes(
            value_ref.get_all_registers(),
            &value_ref.config.formatting_params,
        ),
        &value_ref.config.formatting_params.data_type,
    );

    if value.is_err() {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "Error formating value").into_response());
    }

    let value = value.unwrap();

    Ok(Json(value))
}

pub async fn set_value(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(value): Json<Value>,
) -> StatusCode {
    let mut state = state.lock().await;

    if !state.contains_key(&id) {
        return StatusCode::NOT_FOUND;
    }

    let value_ref = state.get_mut(&id).unwrap();

    let value_registers =
        value_processing::value_to_registers(value, &value_ref.config.formatting_params).unwrap();

    value_ref.set_all_registers(value_registers);

    StatusCode::NO_CONTENT
}
