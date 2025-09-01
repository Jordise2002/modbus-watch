use std::collections::HashMap;

use crate::client::{aggregations::Aggregation};
use crate::common::model::Value;

pub fn build_integer_aggregates(mut values: Vec<i128>) -> Aggregation {
    let sum: i128 = values.iter().copied().sum();
    let average = sum / values.len() as i128;

    values.sort();

    let min = values.first().unwrap().clone();
    let max = values.last().unwrap().clone();

    let median = if values.len() % 2 == 0 {
        let mid = values.len() / 2;
        (values[mid - 1] + values[mid]) / 2
    } else {
        values[values.len() / 2]
    };

    let mut frequency = HashMap::new();
    for &value in &values {
        *frequency.entry(value).or_insert(0) += 1;
    }

    let moda = frequency
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(val, _)| val)
        .unwrap();

    let average = Value::Integer(average);
    let median = Value::Integer(median);
    let moda = Value::Integer(moda);
    let min = Value::Integer(min);
    let max = Value::Integer(max);

    let ammount = values.len() as u64;

    Aggregation {
        average,
        median,
        moda,
        min,
        max,
        ammount,
    }
}

pub fn build_floatin_point_aggregates(mut values: Vec<f64>) -> Aggregation {
    let sum: f64 = values.iter().copied().sum();
    let average = sum / values.len() as f64;

    values.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let min = values.first().unwrap().clone();
    let max = values.last().unwrap().clone();

    let median = if values.len() % 2 == 0 {
        let mid = values.len() / 2;
        (values[mid - 1] + values[mid]) / 2 as f64
    } else {
        values[values.len() / 2]
    };

    let mut frequency = HashMap::new();
    for &value in &values {
        let key_value = value as i128;
        *frequency.entry(key_value).or_insert(0) += 1;
    }

    let moda = frequency
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(val, _)| val)
        .unwrap();

    let average = Value::FloatingPoint(average);
    let median = Value::FloatingPoint(median);
    let moda = Value::Integer(moda);
    let min = Value::FloatingPoint(min);
    let max = Value::FloatingPoint(max);

    let ammount = values.len() as u64;

    Aggregation {
        average,
        median,
        moda,
        min,
        max,
        ammount,
    }
}

pub fn build_boolean_aggregates(values: Vec<bool>) -> Aggregation {
    let mut max = false;
    let mut min = true;

    let mut false_counter = 0;
    let mut true_counter = 0;

    for value in &values {
        if *value {
            max = true;
            true_counter += 1;
        } else {
            min = false;
            false_counter += 1;
        }
    }

    let (average, median, moda) = if true_counter >= false_counter {
        (true, true, true)
    } else {
        (false, false, false)
    };

    let max = Value::Boolean(max);
    let min = Value::Boolean(min);

    let average = Value::Boolean(average);
    let median = Value::Boolean(median);
    let moda = Value::Boolean(moda);

    let ammount = values.len() as u64;

    Aggregation {
        average,
        median,
        moda,
        min,
        max,
        ammount,
    }
}
