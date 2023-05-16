use anyhow::Result;
use common::{load_function, Mapf, Reducef};
use distributed::{
    init_log,
    service::{
        map_reduce_client::MapReduceClient, HeartbeatArgs, HeartbeatReply, ReportArgs, TaskType,
    },
};
use log::{error, info};
use tokio::{
    fs::read_to_string,
    time::{sleep, Duration},
};
use tonic::transport::Channel;

struct Worker {
    client: MapReduceClient<Channel>,
    mapf: Mapf,
    reducef: Reducef,
}

impl Worker {
    fn new(app_name: &str, client: MapReduceClient<Channel>) -> Self {
        let (mapf, reducef) = load_function(app_name);
        Self {
            client,
            mapf,
            reducef,
        }
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
                Some(TaskType::Map) => {
                    self.do_map(rly).await;
                }
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

    async fn do_map(&self, rly: HeartbeatReply) -> Result<()> {
        info!("do map task_id{}", rly.task_id);
        let filename = rly.filename;
        let content = read_to_string(&filename).await?;
        let mapf = self.mapf;
        let kva = mapf(&filename, &content);
        //todo 
        Ok(())
    }

    async fn do_reduce() {}
}

#[tokio::main]
async fn main() -> Result<()> {
    init_log();
    info!("hello world");
    let addr = "http://[::1]:56789";
    let client = MapReduceClient::connect(addr).await?;
    let mut worker = Worker::new("wc", client);
    worker.run().await?;
    Ok(())
}
