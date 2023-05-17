use dlopen2::wrapper::{Container, WrapperApi};
use std::ops::Deref;

const LIB_DIR: &str = "/Users/mag1cian/dev/mr/applibs";

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}

pub trait MapReduce {
    fn map(filename: &str, contents: &str) -> Vec<KeyValue>;
    fn reduce(key: &str, values: Vec<String>) -> String;
}

#[derive(WrapperApi)]
pub struct Api {
    map: fn(filename: &str, contents: &str) -> Vec<KeyValue>,
    reduce: fn(key: &str, values: Vec<String>) -> String,
}

pub struct App {
    pub app_name: String,
    cont: Container<Api>,
}
impl App {
    pub fn load(app_name: &str) -> anyhow::Result<Self> {
        let lib_path = format!("{LIB_DIR}/lib{app_name}.dylib");
        let cont: Container<Api> = unsafe { Container::load(lib_path) }?;
        Ok(Self {
            app_name: app_name.to_string(),
            cont,
        })
    }
}

impl Deref for App {
    type Target = Container<Api>;

    fn deref(&self) -> &Self::Target {
        &self.cont
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_function() {
        let wc = App::load("wc").unwrap();
        assert_eq!(wc.map("", "").len(), 0);
        assert_eq!("1".to_string(), wc.reduce("", vec!["hhhlo".to_string()]));
    }
}
