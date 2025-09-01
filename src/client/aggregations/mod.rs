use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{anyhow, Result};

use crate::client::data;
use crate::client::data::read::get_polls_between;
use crate::client::model::PolledConnection;
use crate::common::model::{DataType, Value};

mod build_aggregates;

#[derive(PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum Period {
    NoGrouping = 0,
    Minute = 1,
    Hour = 2,
    Day = 3,
}

impl Period {
    pub fn from_repr(raw_value: u8) -> Result<Self> {
        match raw_value {
            0 => Ok(Self::NoGrouping),
            1 => Ok(Self::Minute),
            2 => Ok(Self::Hour),
            3 => Ok(Self::Day),
            _ => Err(anyhow!("Value not supported for period!"))
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AggregationInfo {
    pub value_id: String,
    pub period: Period,
    pub start_time: std::time::SystemTime,
    pub finish_time: std::time::SystemTime,
    #[serde(flatten)]
    pub aggregation: Aggregation,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct Aggregation {
    pub average: Value,
    pub median: Value,
    pub moda: Value,
    pub min: Value,
    pub max: Value,
    pub ammount: u64,
}

pub struct OnGoingAggregationInfo {
    last_min_aggregated: std::time::SystemTime,
    last_hour_aggregated: std::time::SystemTime,
    last_day_aggregated: std::time::SystemTime,
    data_type: DataType,

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
        data_type: DataType,
    ) -> Self {
        OnGoingAggregationInfo {
            last_min_aggregated: std::time::SystemTime::now(),
            last_hour_aggregated: std::time::SystemTime::now(),
            last_day_aggregated: std::time::SystemTime::now(),
            max_polls,
            max_min_aggregations,
            max_hour_aggregations,
            max_day_aggregations,
            data_type,
        }
    }
}

fn delete_excess_aggregates(
    id: String,
    info: &OnGoingAggregationInfo,
    conn: r2d2::PooledConnection<SqliteConnectionManager>,
) {
    if let Some(max_polls) = info.max_polls {
        if let Err(err) = crate::client::data::write::delete_exceeding_polls(&conn, id.clone(), max_polls) {
            tracing::error!("Error deleting exceeding polls: {}", err);
        }
    }

    if let Some(max_aggregations) = info.max_min_aggregations {
        if let Err(err) = crate::client::data::write::delete_exceeding_aggregations(
            &conn,
            id.clone(),
            Period::Minute,
            max_aggregations,
        ) {
            tracing::error!("Error deleting exceeding minute aggregations: {}", err);
        }
    }

    if let Some(max_aggregations) = info.max_hour_aggregations {
        if let Err(err) = crate::client::data::write::delete_exceeding_aggregations(
            &conn,
            id.clone(),
            Period::Hour,
            max_aggregations,
        ) {
            tracing::error!("Error deleting exceeding hour aggregations: {}", err);
        }
    }

    if let Some(max_aggregations) = info.max_day_aggregations {
        if let Err(err) = crate::client::data::write::delete_exceeding_aggregations(
            &conn,
            id.clone(),
            Period::Day,
            max_aggregations,
        ) {
            tracing::error!("Error deleting exceeding day aggregations: {}", err);
        }
    }
}

fn create_single_aggregate(
    id: &String,
    start_time: std::time::SystemTime,
    finish_time: std::time::SystemTime,
    data_type: &DataType,
    period: Period,
    conn: &r2d2::PooledConnection<SqliteConnectionManager>,
) {
    let values = get_polls_between(conn, id, data_type, start_time, finish_time).unwrap();

    if values.is_empty() {
        return;
    }

    let aggregate = match values.first().unwrap().value {
        Value::Integer(_) => {
            let integers: Vec<i128> = values
                .into_iter()
                .filter_map(|n| {
                    if let Value::Integer(v) = n.value {
                        Some(v)
                    } else {
                        None
                    }
                })
                .collect();
            build_aggregates::build_integer_aggregates(integers)
        }
        Value::FloatingPoint(_) => {
            let floating_points: Vec<f64> = values
                .into_iter()
                .filter_map(|n| {
                    if let Value::FloatingPoint(v) = n.value {
                        Some(v)
                    } else {
                        None
                    }
                })
                .collect();
            build_aggregates::build_floatin_point_aggregates(floating_points)
        }
        Value::Boolean(_) => {
            let booleans: Vec<bool> = values
                .into_iter()
                .filter_map(|n| {
                    if let Value::Boolean(v) = n.value {
                        Some(v)
                    } else {
                        None
                    }
                })
                .collect();

            build_aggregates::build_boolean_aggregates(booleans)
        }
    };

    let aggregate_info = AggregationInfo {
        value_id: id.clone(),
        start_time,
        finish_time,
        period,
        aggregation: aggregate,
    };

    data::write::insert_modbus_aggregate(conn, aggregate_info).unwrap();
}

fn create_aggregates(
    id: &String,
    now: std::time::SystemTime,
    info: &mut OnGoingAggregationInfo,
    db_access: r2d2::PooledConnection<SqliteConnectionManager>,
) {
    if now
        .duration_since(info.last_min_aggregated)
        .unwrap()
        .as_secs()
        >= 60
    {
        let mut start_time = info.last_min_aggregated;
        let mut finish_time = start_time + std::time::Duration::from_secs(60);

        while finish_time < now {
            create_single_aggregate(
                id,
                start_time,
                finish_time,
                &info.data_type,
                Period::Minute,
                &db_access,
            );
            start_time = finish_time;
            finish_time = start_time + std::time::Duration::from_secs(60);
        }
        info.last_min_aggregated = start_time;
    }

    if now
        .duration_since(info.last_hour_aggregated)
        .unwrap()
        .as_secs()
        >= 60 * 60
    {
        let mut start_time = info.last_hour_aggregated;
        let mut finish_time = start_time + std::time::Duration::from_secs(60 * 60);

        while finish_time < now {
            create_single_aggregate(
                id,
                start_time,
                finish_time,
                &info.data_type,
                Period::Hour,
                &db_access,
            );
            start_time = finish_time;
            finish_time = start_time + std::time::Duration::from_secs(60 * 60);
        }
        info.last_hour_aggregated = start_time;
    }

    if now
        .duration_since(info.last_day_aggregated)
        .unwrap()
        .as_secs()
        >= 60 * 60 * 24
    {
        let mut start_time = info.last_day_aggregated;
        let mut finish_time = start_time + std::time::Duration::from_secs(60 * 60 * 24);

        while finish_time < now {
            create_single_aggregate(
                id,
                start_time,
                finish_time,
                &info.data_type,
                Period::Day,
                &db_access,
            );
            start_time = finish_time;
            finish_time = start_time + std::time::Duration::from_secs(60 * 60 * 24);
        }
        info.last_day_aggregated = start_time;
    }
}

async fn aggregation_periodic_task(
    mut aggregation_info: HashMap<String, OnGoingAggregationInfo>,
    db_access: Arc<Pool<SqliteConnectionManager>>,
) {
    let duration = std::time::Duration::from_secs(30);

    let mut interval = tokio::time::interval(duration);

    loop {
        interval.tick().await;

        let now: std::time::SystemTime = std::time::SystemTime::now();

        for (id, info) in &mut aggregation_info {
            create_aggregates(id, now, info, db_access.get().unwrap());
            delete_excess_aggregates(id.clone(), info, db_access.get().unwrap());
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
                        value.data_type.clone(),
                    ),
                );
            }
        }
    }

    tokio::spawn(async move { aggregation_periodic_task(aggregation_info, db_access) }.await);
}
