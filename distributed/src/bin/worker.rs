use anyhow::Result;
use clap::Parser;
use common::{App, KeyValue};
use distributed::{
    init_logger,
    service::{map_reduce_client::*, task::Inner, *},
    temp_file, ADDR,
};
use futures::{future::try_join_all, FutureExt};
use itertools::Itertools;
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    env,
    fmt::Debug,
    hash::{Hash, Hasher},
    io::Write,
    process,
    time::Duration,
};
use tokio::{
    fs::{self, read_to_string, File},
    io::AsyncWriteExt,
    time,
};
use tracing::{info, instrument};
use uuid::Uuid;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(short, long, default_value = ADDR)]
    connect: String,
    #[arg(short, long)]
    app_name: String,
}

#[allow(dead_code)]
struct Worker {
    cli: Cli,
    id: String,
    app: App,
    client: MapReduceClient<tonic::transport::Channel>,
}

impl Debug for Worker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Worker")
            .field("id", &self.id)
            .field("app", &self.app.app_name)
            .finish()
    }
}

macro_rules! write_kv {
    ($file:expr, $k:expr, $v:expr) => {
        $file.write_all(format!("{} {}\n", $k, $v).as_bytes())
    };
}

impl Worker {
    pub fn new(cli: Cli, client: MapReduceClient<tonic::transport::Channel>) -> Result<Self> {
        let app = App::load(&cli.app_name)?;
        let id = process::id().to_string();
        Ok(Self {
            cli,
            id,
            app,
            client,
        })
    }
    #[instrument]
    pub async fn run(mut self) -> Result<()> {
        loop {
            let PollTaskReply { task, shutdown } = self
                .client
                .poll_task(PollTaskRequest {})
                .await?
                .into_inner();

            if shutdown {
                info!("shutdown");
                return Ok(());
            }

            match task {
                Some(task) => {
                    info!("task: {:?}", task);
                    let complete: CompleteTaskRequest = match &task.inner {
                        Some(Inner::MapTask(map)) => {
                            let reduce_files = self.run_map(map.clone()).await?;
                            CompleteTaskRequest {
                                task: Some(task),
                                reduce_files,
                            }
                        }
                        Some(Inner::ReduceTask(reduce)) => {
                            self.run_reduce(reduce.clone()).await?;
                            CompleteTaskRequest {
                                task: Some(task),
                                reduce_files: Default::default(),
                            }
                        }
                        None => unreachable!(),
                    };
                    self.client.complete_task(complete).await?;
                    info!("task completed");
                }
                None => {
                    time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }

    #[instrument]
    pub async fn run_map(&self, task: MapTask) -> Result<HashMap<u64, String>> {
        info!("???");
        let MapTask {
            index,
            files,
            n_reduce,
        } = task;

        let mut k1v1s = vec![];
        for name in files {
            info!("filename is :{}", name);
            let s = std::fs::read_to_string(&name)?;
            info!("s len {}", s.len());
            k1v1s.push((name, s));
        }

        let k2v2s = k1v1s.into_iter().flat_map(|(k, s)| self.app.map(&k, &s));

        let intermediate_filenames = (0..n_reduce)
            .map(|j| format!("mr-{}-{}-{}", index, j, Uuid::new_v4()))
            .collect_vec();
        let mut intermediate_files = vec![];
        for f in intermediate_filenames.iter() {
            let ff = std::fs::File::create(f)?;
            intermediate_files.push(ff);
        }

        for KeyValue { key, value } in k2v2s {
            let file_index = {
                let mut hasher = DefaultHasher::new();
                key.hash(&mut hasher);
                (hasher.finish() % n_reduce) as usize
            };
            let file = intermediate_files.get_mut(file_index).unwrap();
            file.write_all(format!("{} {}\n", key, value).as_bytes())?;
        }

        Ok(intermediate_filenames
            .into_iter()
            .enumerate()
            .map(|(i, f)| (i as u64, f))
            .collect())
    }

    #[instrument]
    pub async fn do_map(&self, task: MapTask) -> Result<HashMap<u64, String>> {
        // async write to file
        let MapTask {
            index,
            files,
            n_reduce,
        } = task;

        let k1v1s = {
            let kv_futures = files.into_iter().map(|file| {
                read_to_string(file.clone()).map(|result| result.map(|content| (file, content)))
            });
            try_join_all(kv_futures).await?
        };

        let k2v2s = k1v1s.into_iter().flat_map(|(k, v)| self.app.map(&k, &v));

        let intermediate_filenames = (0..n_reduce)
            .map(|j| format!("mr-{}-{}-{}", index, j, Uuid::new_v4()))
            .collect_vec();
        let mut intermediate_files =
            try_join_all(intermediate_filenames.iter().map(File::create)).await?;

        for KeyValue { key: k2, value: v2 } in k2v2s {
            let file_index = {
                let mut hasher = DefaultHasher::new();
                k2.hash(&mut hasher);
                (hasher.finish() % n_reduce) as usize
            };
            let file = intermediate_files.get_mut(file_index).unwrap();
            write_kv!(file, k2, v2).await?;
        }

        // sync
        try_join_all(intermediate_files.iter().map(|f| f.sync_all())).await?;

        Ok(intermediate_filenames
            .into_iter()
            .enumerate()
            .map(|(i, f)| (i as u64, f))
            .collect())
    }

    #[instrument]
    pub async fn run_reduce(&self, task: ReduceTask) -> Result<()> {
        let ReduceTask { index, files } = task;
        let mut kvs = vec![];
        for f in files.iter() {
            let s = std::fs::read_to_string(f).expect(&format!("can not read file: {}", f));
            for line in s.lines() {
                let mut tokens = line.split_whitespace();
                kvs.push(KeyValue {
                    key: tokens.next().unwrap().to_owned(),
                    value: tokens.next().unwrap().to_owned(),
                })
            }
        }
        kvs.sort();
        let (temp_path, output_path) = (temp_file(), format!("mr-out-{}", index));
        info!("tmp_path: {} ",temp_path);
        let mut temp_file = std::fs::File::create(&temp_path)?;
        for (k, ks) in kvs.into_iter().group_by(|kv| kv.key.clone()).into_iter() {
            let output = self.app.reduce(&k, ks.map(|kv| kv.value).collect_vec());
            temp_file.write_all(output.as_bytes())?;
        }
        // rename
        std::fs::rename(temp_path, output_path)?;
        Ok(())
    }
    #[instrument]
    pub async fn do_reduce(&self, task: ReduceTask) -> Result<()> {
        // async write
        let ReduceTask { index, files } = task;

        let k2v2s = {
            let kv_futures = files.into_iter().map(|file| {
                info!("current dir is {}", env::current_dir().unwrap().display());
                read_to_string(file).map(|result| {
                    result.map(|content| {
                        content
                            .lines()
                            .map(|line| {
                                let mut tokens = line.split_whitespace();
                                KeyValue {
                                    key: tokens.next().unwrap().to_owned(),
                                    value: tokens.next().unwrap().to_owned(),
                                }
                            })
                            .collect_vec()
                    })
                })
            });

            let mut k2v2s = try_join_all(kv_futures)
                .await?
                .into_iter()
                .flatten()
                .collect_vec();

            k2v2s.sort();
            k2v2s
        };

        let (temp_path, output_path) = (temp_file(), format!("mr-out-{}", index));
        let mut temp_file = File::create(&temp_path).await?;
        for (k, kvs) in k2v2s.into_iter().group_by(|kv| kv.key.clone()).into_iter() {
            let output = self.app.reduce(&k, kvs.map(|kv| kv.value).collect_vec());
            write_kv!(temp_file, k, output).await?;
        }

        // sync
        temp_file.sync_all().await?;
        // rename
        fs::rename(temp_path, output_path).await?;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger();

    let cli = Cli::parse();
    let addr = format!("http://{}", cli.connect);
    let client = MapReduceClient::connect(addr).await?;

    let worker = Worker::new(cli, client)?;
    worker.run().await
}
