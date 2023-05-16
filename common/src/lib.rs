use dlopen2::wrapper::{Container, WrapperApi};

const LIB_DIR: &str = "/Users/mag1cian/dev/mr/applibs";

pub struct KeyValue {
    key: String,
    value: String,
}

pub type Mapf = fn(filename: &str, contents: &str) -> Vec<KeyValue>;
pub type Reducef = fn(key: &str, values: Vec<String>) -> String;

#[derive(WrapperApi)]
struct Api {
    map: fn(filename: &str, contents: &str) -> Vec<KeyValue>,
    reduce: fn(key: &str, values: Vec<String>) -> String,
}

pub fn load_function(app_name: &str) -> (Mapf, Reducef) {
    let lib_path = format!("{LIB_DIR}/lib{app_name}.dylib");
    let cont: Container<Api> = unsafe { Container::load(lib_path) }.expect("load app lib failed");
    (cont.map, cont.reduce)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_function() {
        let (map, reduce) = load_function("wc");
        map("", "");
        let v = vec!["??".to_string()];
        reduce("", v);
    }
}
