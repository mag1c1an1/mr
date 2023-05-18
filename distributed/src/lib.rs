use std::path::PathBuf;
use uuid::Uuid;
pub mod service {
    use std::fmt::Display;

    tonic::include_proto!("service");
    impl Display for PollTaskReply {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "PollTaskReply:{{ task: {:?}, shutdown: {} }}",
                self.task, self.shutdown
            )
        }
    }
}

pub const ADDR: &str = "[::1]:56789";

const TMP_PATH: &str = "/Users/mag1cian/dev/mr/mr-tmp";

// TODO: add file

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
