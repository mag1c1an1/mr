#![allow(unused)]

use std::{thread, time, vec};
struct KeyValue {
    key: String,
    value: String,
}

#[no_mangle]
fn map(filename: &str, contents: &str) -> Vec<KeyValue> {
    vec![KeyValue {
        key: filename.to_string(),
        value: "1".to_string(),
    }]
}
#[no_mangle]
fn reduce(key: &str, values: Vec<String>) -> String {
    if key.contains("sherlock") || key.contains("tom") {
        thread::sleep(time::Duration::from_secs(3));
    }
    values.len().to_string()
}
