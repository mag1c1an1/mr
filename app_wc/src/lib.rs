#![allow(unused)]
struct KeyValue {
    key: String,
    value: String,
}

#[no_mangle]
fn map(filename: &str, contents: &str) -> Vec<KeyValue> {
    contents
        .split(|c: char| !c.is_alphabetic())
        .filter(|w| !w.is_empty())
        .map(|w| KeyValue {
            key: w.to_string(),
            value: "1".to_string(),
        })
        .collect()
}
#[no_mangle]
fn reduce(key: &str, values: Vec<String>) -> String {
    values.len().to_string()
}
