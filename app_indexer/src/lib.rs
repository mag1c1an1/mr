#![allow(unused)]
use std::collections::HashMap;

use itertools::Itertools;
struct KeyValue {
    key: String,
    value: String,
}

#[no_mangle]
fn map(filename: &str, contents: &str) -> Vec<KeyValue> {
    contents
        .split(|c: char| !c.is_alphabetic())
        .filter(|w| !w.is_empty())
        .unique()
        .map(|w| KeyValue {
            key: w.to_string(),
            value: filename.to_string(),
        })
        .collect()
    // let mut occurs = HashMap::new();
    // for word in words {
    //     occurs
    //         .entry(word.to_string())
    //         .or_insert(filename.to_string());
    // }
    // occurs
    //     .into_iter()
    //     .map(|(k, v)| KeyValue { key: k, value: v })
    //     .collect()
}
#[no_mangle]
fn reduce(key: &str, values: Vec<String>) -> String {
    format!("{} {}", values.len(), values.join(","))
}
