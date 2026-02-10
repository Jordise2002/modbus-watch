use crate::server::model::ServedValue;
use crate::server::state::AppState;
use axum::{
    extract::{Path, State},
    response::{Response, IntoResponse},
    http::StatusCode,
    Json,
};

pub async fn get_config(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ServedValue>, Response> {
    let state = state.lock().await;

    if !state.contains_key(&id) {
        return Err((StatusCode::NOT_FOUND, "Value not defined").into_response());
    }

    let value = state.get(&id).unwrap();

    Ok(Json(value.config.clone()))
}
