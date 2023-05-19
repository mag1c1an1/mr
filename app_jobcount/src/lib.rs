#![allow(unused)]
use rand::Rng;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{fs, process, thread};
struct KeyValue {
    key: String,
    value: String,
}
static COUNT: AtomicUsize = AtomicUsize::new(0);

#[no_mangle]
fn map(filename: &str, contents: &str) -> Vec<KeyValue> {
    let file = format!(
        "mr-worker-jobcount-{}-{}",
        process::id(),
        COUNT.fetch_add(1, Ordering::SeqCst)
    );
    fs::File::create(file).unwrap().write_all(b"x").unwrap();
    thread::sleep(std::time::Duration::from_millis(
        2000 + rand::thread_rng().gen_range(0..3000),
    ));

    vec![KeyValue {
        key: "a".to_string(),
        value: "x".to_string(),
    }]
}
#[no_mangle]
fn reduce(key: &str, values: Vec<String>) -> String {
    let invocations = std::fs::read_dir(".")
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .path()
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .starts_with("mr-worker-jobcount")
        })
        .count();

    invocations.to_string()
}
