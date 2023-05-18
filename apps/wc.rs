#![crate_type = "dylib"]
#![allow(unused)]
struct KeyValue {
    key: String,
    value: String,
}

#[no_mangle]
fn map(filename:&str, contents:&str) -> Vec<KeyValue> {
    let words: Vec<&str> =  contents
        .split(|r| !char::is_alphabetic(r))
        .filter(|&s| s != "")
        .collect();
    let mut kva : Vec<KeyValue> = vec![];
    for w in words {
        kva.push(KeyValue {
            key:w.into(),
            value:"1".into(),
        })
    }
    kva
}
#[no_mangle]
fn reduce(key: &str, values:Vec<String>) -> String {
    values.len().to_string()
}