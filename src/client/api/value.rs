use crate::client::model::DataType;
use crate::client::{api::ApiState, data::ModbusPoll};

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
    let mut found = false;
    let mut data_type = DataType::Boolean;

    'search_loop: for connection in &state.config
    {
        for slave in &connection.slaves {
            for value in &slave.values {
                if value.id == id {
                    found = true;
                    data_type = value.data_type.clone();
                    break 'search_loop;  
                }
            }
        }
    }

    if ! found {
        return Err((StatusCode::NOT_FOUND, "Value was not configured").into_response());
    }

    let db_conn = state.db.get().or_else(|_| {
        Err((StatusCode::INTERNAL_SERVER_ERROR, "Access to db failed").into_response())
    })?;

    let poll = crate::client::data::read::get_last_poll(&db_conn, id, data_type);

    if let Ok(poll) = poll {
        Ok(Json(poll))
    } else {
        Err((StatusCode::NOT_FOUND, "Value not found").into_response())
    }
}
