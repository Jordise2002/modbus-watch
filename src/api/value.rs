use crate::model::PolledValue;
use crate::{api::ApiState, data::ModbusPoll};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use std::sync::Arc;

pub async fn get_value(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<String>,
) -> Result<Json<ModbusPoll>, Response> {
    let db_conn = state.db.get().or_else(|_| {
        Err((StatusCode::INTERNAL_SERVER_ERROR, "Access to db failed").into_response())
    })?;

    let poll = crate::data::read::get_last_poll(&db_conn, id);

    if let Ok(poll) = poll {
        Ok(Json(poll))
    } else {
        Err((StatusCode::NOT_FOUND, "Value not found").into_response())
    }
}
