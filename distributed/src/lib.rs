use std::path::PathBuf;
use uuid::Uuid;
pub mod service {
    tonic::include_proto!("service");
}

pub const ADDR: &str = "[::1]:56789";

const TMP_PATH: &str = "/Users/mag1cian/dev/mr/tmp";

// TODO: add file appender
pub fn init_logger() {
    tracing_subscriber::fmt::init();
}

pub fn temp_file() -> String {
    let mut path = PathBuf::from(TMP_PATH);
    path.push(Uuid::new_v4().to_string());
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_temp_file() {
        println!("{}", temp_file());
    }
}
