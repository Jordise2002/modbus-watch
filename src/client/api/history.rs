use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::UNIX_EPOCH, u64};

use crate::client::{
    aggregations::{AggregationInfo, Period},
    api::ApiState,
    data::ModbusPoll,
    model::DataType,
};

#[derive(Debug, Deserialize)]
pub struct HistoryParams {
    start_date: Option<u64>,
    end_date: Option<u64>,
    max_group: Option<Period>,
    min_group: Option<Period>,
}

#[derive(PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HistoryResult {
    AggregationValue { aggregation_info: AggregationInfo },
    Value { value_info: ModbusPoll },
}

pub async fn get_history(
    Path(value_id): Path<String>,
    Query(params): Query<HistoryParams>,
    State(state): State<Arc<ApiState>>,
) -> Result<Json<Vec<HistoryResult>>, Response> {
    let start_date = if let Some(start_date) = params.start_date {
        UNIX_EPOCH + std::time::Duration::from_secs(start_date)
    } else {
        UNIX_EPOCH
    };

    let end_date = if let Some(end_date) = params.end_date {
        UNIX_EPOCH + std::time::Duration::from_secs(end_date)
    } else {
        UNIX_EPOCH + std::time::Duration::from_secs(i64::MAX as u64)
    };

    let max_group = params.max_group.unwrap_or(Period::Day);
    let min_group = params.min_group.unwrap_or(Period::NoGrouping);

    let mut found = false;
    let mut data_type = DataType::Boolean;

    'search_loop: for connection in &state.config {
        for slave in &connection.slaves {
            for value in &slave.values {
                if value.id == value_id {
                    found = true;
                    data_type = value.data_type.clone();
                    break 'search_loop;
                }
            }
        }
    }

    if !found {
        return Err((StatusCode::NOT_FOUND, "Value was not configured").into_response());
    }

    let conn = state.db.get().or_else(|_| {
        Err((StatusCode::INTERNAL_SERVER_ERROR, "Access to db failed").into_response())
    })?;

    let mut result = vec![];

    let aggregations = crate::client::data::read::get_aggregates_between(
        &conn, &value_id, &data_type, start_date, end_date, max_group, min_group,
    )
    .or_else(|_| Err((StatusCode::INTERNAL_SERVER_ERROR, "Access to db failed").into_response()))?;

    for aggregation_info in aggregations {
        result.push(HistoryResult::AggregationValue { aggregation_info });
    }

    if min_group == Period::NoGrouping {
        let polls = crate::client::data::read::get_polls_between(
            &conn, &value_id, &data_type, start_date, end_date,
        )
        .or_else(|_| {
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Access to db failed").into_response())
        })?;

        for value_info in polls {
            result.push(HistoryResult::Value { value_info });
        }
    }

    Ok(Json(result))
}
