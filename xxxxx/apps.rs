// TODO move to dynamic loading
use crate::KeyValue;
/// word_count
pub fn wc_m(_filename: &str, contents: &str) -> Vec<KeyValue> {
    let words: Vec<&str> = contents
        .split(|r| !char::is_alphabetic(r))
        .filter(|&s| s != "")
        .collect();
    let mut kva: Vec<KeyValue> = vec![];
    for w in words {
        kva.push(KeyValue {
            key: w.into(),
            value: "1".into(),
        })
    }
    return kva;
}

pub fn wc_r(_key: String, values: Vec<String>) -> String {
    values.len().to_string()
}

/// crash test
fn maybe_crash() {}

/// crash
fn mc_m() {}

fn mc_r() {
    maybe_crash();

    // sort
}

/// early exit
/// TODO
fn ee_m() {
    todo!()
}
fn ee_r() {
    todo!()
}

/// indexer
/// TODO
/// jobcount
/// TODO
///mtiming
///TODO

/// rtiming
/// TODO

/// no crash
/// TODO
fn nocrash() {}

fn nc_m() {}
fn nc_r() {}
