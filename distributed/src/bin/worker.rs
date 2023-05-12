use anyhow::Result;
use distributed::{
    init_log,
    service::{
        map_reduce_client::MapReduceClient, HeartbeatArgs, HeartbeatReply, ReportArgs, TaskType,
    },
};
use log::{error, info};
use tokio::time::{sleep, Duration};
use tonic::transport::Channel;

struct Worker {
    client: MapReduceClient<Channel>,
}

impl Worker {
    fn new(_app_name: &str, client: MapReduceClient<Channel>) -> Self {
        Self { client }
    }

    async fn heartbeat(&mut self) -> Result<HeartbeatReply> {
        Ok(self.client.heartbeat(HeartbeatArgs {}).await?.into_inner())
    }

    async fn report(&mut self, task_id: i32, task_type: i32) {
        if self
            .client
            .report(ReportArgs { task_id, task_type })
            .await
            .is_err()
        {
            error!("REPORT ERROR task_id: {}", task_id)
        }
    }

    async fn run(&mut self) -> Result<()> {
        loop {
            let rly = self.heartbeat().await?;
            match TaskType::from_i32(rly.task_type) {
                Some(TaskType::Map) => {}
                Some(TaskType::Reduce) => {}
                Some(TaskType::Exit) => {
                    info!("Worker Exit");
                    return Ok(());
                }
                Some(TaskType::Wait) => {
                    sleep(Duration::from_secs(1)).await;
                }
                None => {
                    error!("WRONG TASK TYPE {}", rly.task_type);
                }
            }
        }
    }

    async fn do_map(&self, rly: HeartbeatReply) {
        todo!()
    }

    async fn do_reduce() {}
}

#[tokio::main]
async fn main() -> Result<()> {
    init_log();
    info!("hello world");
    let addr = "http://[::1]:56789";
    let client = MapReduceClient::connect(addr).await?;
    let mut worker = Worker::new("..", client);
    worker.run().await?;
    Ok(())
}
