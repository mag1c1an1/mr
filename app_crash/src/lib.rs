
#![allow(unused)]
use std::{env,process,thread,time};
use rand::Rng;
struct KeyValue {
    key: String,
    value: String,
}
fn maybe_crash() {
    if env::var("CRASH").unwrap_or_default() != "1" {
        return;
    }
    let rr = rand::thread_rng().gen_range(0..1000);
    if rr < 330 {
        process::exit(1);
    } else if rr < 660 {
        let ms = rand::thread_rng().gen_range(0..10000);
        thread::sleep(time::Duration::from_millis(ms)); // bad idea
    }
}
#[no_mangle]
fn map(filename: &str, contents: &str) -> Vec<KeyValue> {
     maybe_crash();
        vec![
            KeyValue{key:"a".to_owned(),value: filename.to_string()},
            KeyValue{key:"b".to_owned(),value: filename.len().to_string()},
            KeyValue{key:"c".to_owned(),value: contents.len().to_string()},
            KeyValue{key:"d".to_owned(),value: "xyzzy".to_string()},
        ]
}
#[no_mangle]
fn reduce(key: &str, values: Vec<String>) -> String {
    maybe_crash();
    let mut v = values;
    v.sort();
    v.join(" ")
}
