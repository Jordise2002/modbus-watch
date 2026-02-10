use axum::{extract::State, Json};
use crate::server::state::AppState;

pub async fn list_values(State(state): State<AppState>) -> Json<Vec<String>> {
    let state = state.lock().await;
    let keys: Vec<String> = state.keys().cloned().collect();

    Json(keys)
}