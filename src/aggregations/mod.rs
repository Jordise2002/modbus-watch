use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tracing::{info, warn};
use std::collections::HashMap;
use std::hash::Hash;
use std::{sync::Arc, time::UNIX_EPOCH};

use crate::model::{PolledConnection, Value};

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Period {
    Minute,
    Hour,
    Day,
}

pub struct Aggregation {
    value_id: String,
    period: Period,
    start_time: std::time::SystemTime,
    finish_time: std::time::SystemTime,
    average: Value,
    median: Value,
    min: Value,
    max: Value,
    ammount: u64,
}

pub struct OnGoingAggregationInfo {
    last_min_aggregated: std::time::SystemTime,
    last_hour_aggregated: std::time::SystemTime,
    last_day_aggregated: std::time::SystemTime,

    max_polls: Option<u64>,
    max_min_aggregations: Option<u64>,
    max_hour_aggregations: Option<u64>,
    max_day_aggregations: Option<u64>,
}

impl OnGoingAggregationInfo {
    pub fn new(
        max_polls: Option<u64>,
        max_min_aggregations: Option<u64>,
        max_hour_aggregations: Option<u64>,
        max_day_aggregations: Option<u64>,
    ) -> Self {
        OnGoingAggregationInfo {
            last_min_aggregated: UNIX_EPOCH,
            last_hour_aggregated: UNIX_EPOCH,
            last_day_aggregated: UNIX_EPOCH,
            max_polls,
            max_min_aggregations,
            max_hour_aggregations,
            max_day_aggregations,
        }
    }
}

async fn aggregation_periodic_task(
    mut aggregation_info: HashMap<String, OnGoingAggregationInfo>,
    db_access: Arc<Pool<SqliteConnectionManager>>
) {
    let duration = std::time::Duration::from_secs(30);

    let mut interval = tokio::time::interval(duration);

    loop {
        interval.tick().await;

        let now = std::time::SystemTime::now();

        for (id, info) in &aggregation_info {
            //TODO: Build the actual aggregations before deleting the excess

            if let Some(max_polls) = info.max_polls {
                let conn = db_access.get().unwrap();
                if let Err(err) =
                    crate::data::write::delete_exceeding_polls(&conn, id.clone(), max_polls)
                {
                    tracing::error!("Error deleting exceeding polls: {}", err);
                }
            }

            if let Some(max_aggregations) = info.max_min_aggregations {
                let conn = db_access.get().unwrap();
                if let Err(err) = crate::data::write::delete_exceeding_aggregations(
                    &conn,
                    id.clone(),
                    Period::Minute,
                    max_aggregations,
                ) {
                    tracing::error!("Error deleting exceeding minute aggregations: {}", err);
                }
            }

            if let Some(max_aggregations) = info.max_hour_aggregations {
                let conn = db_access.get().unwrap();
                if let Err(err) = crate::data::write::delete_exceeding_aggregations(
                    &conn,
                    id.clone(),
                    Period::Hour,
                    max_aggregations,
                ) {
                    tracing::error!("Error deleting exceeding hour aggregations: {}", err);
                }
            }

            if let Some(max_aggregations) = info.max_day_aggregations {
                let conn = db_access.get().unwrap();
                if let Err(err) = crate::data::write::delete_exceeding_aggregations(
                    &conn,
                    id.clone(),
                    Period::Day,
                    max_aggregations,
                ) {
                    tracing::error!("Error deleting exceeding day aggregations: {}", err);
                }
            }
        }
    }
}

pub async fn start_aggregation_building(
    db_access: Arc<Pool<SqliteConnectionManager>>,
    config: Vec<PolledConnection>,
) {
    let mut aggregation_info = HashMap::new();

    for connection in &config {
        for slave in &connection.slaves {
            for value in &slave.values {
                aggregation_info.insert(
                    value.id.clone(),
                    OnGoingAggregationInfo::new(
                        value.max_polls_to_keep,
                        value.max_minute_aggregations_to_keep,
                        value.max_hour_aggregations_to_keep,
                        value.max_day_aggregations_to_keep,
                    ),
                );
            }
        }
    }

    tokio::spawn( async move {
        aggregation_periodic_task(aggregation_info, db_access)
    }.await);
}
