pub mod service {
    tonic::include_proto!("service");
}

pub fn init_log() {
    log4rs::init_file("/Users/mag1cian/dev/mr/log4rs.yml", Default::default()).unwrap();
}
