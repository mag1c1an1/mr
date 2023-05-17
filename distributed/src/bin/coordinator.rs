use anyhow::Result;
use clap::Parser;
use crossbeam_queue::ArrayQueue;
use dashmap::DashMap;
use distributed::{
    init_logger,
    service::{map_reduce_server::*, *},
    ADDR,
};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    sync::{oneshot, Mutex},
    task::JoinHandle,
    time,
};
use tonic::{transport::Server, Request, Response, Status};
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Parser, Debug)]
pub struct Cli {
    #[arg(short,long,default_value = ADDR)]
    listen: String,
    #[arg(short, long, default_value_t = 10)]
    n_reduce: u64,
    input_files: Vec<PathBuf>,
}

type TaskId = String;
type TaskMap = DashMap<TaskId, Task>;

#[derive(Debug)]
pub struct Coordinator {
    cli: Cli,
    shutdown: Mutex<Option<oneshot::Sender<()>>>,
    reduce_stage: AtomicBool,
    pending_tasks: Arc<ArrayQueue<Task>>,
    running_tasks: Arc<TaskMap>,
    reduce_files: DashMap<u64, Vec<String>>,
    retry_handlers: DashMap<TaskId, JoinHandle<()>>,
}

impl Coordinator {
    fn init_map(&self) {
        let map_tasks = self
            .cli
            .input_files
            .iter()
            .enumerate()
            .map(|(i, path)| MapTask {
                index: i as u64,
                files: vec![path.to_string_lossy().into_owned()],
                n_reduce: self.cli.n_reduce,
            });

        for task in map_tasks {
            let id = Uuid::new_v4().to_string();
            let task = Task {
                id,
                inner: Some(task::Inner::MapTask(task)),
            };
            self.pending_tasks.push(task).unwrap();
        }
    }

    fn init_reduce(&self) {
        assert!(self.pending_tasks.is_empty() && self.running_tasks.is_empty());

        let reduce_tasks = self.reduce_files.iter().map(|pair| ReduceTask {
            index: *pair.key(),
            files: pair.value().clone(),
        });

        for task in reduce_tasks {
            let id = Uuid::new_v4().to_string();
            let task = Task {
                id,
                inner: Some(task::Inner::ReduceTask(task)),
            };
            self.pending_tasks.push(task).unwrap();
        }
    }

    pub fn new(cli: Cli, shutdown: oneshot::Sender<()>) -> Self {
        let task_capacity = cli.input_files.len() * (cli.n_reduce as usize) + 5;

        let this = Self {
            cli,
            shutdown: Mutex::new(Some(shutdown)),
            reduce_stage: AtomicBool::new(false),
            pending_tasks: Arc::new(ArrayQueue::new(task_capacity)),
            running_tasks: Arc::new(TaskMap::new()),
            reduce_files: DashMap::new(),
            retry_handlers: DashMap::new(),
        };
        this.init_map();
        this
    }
}

impl Coordinator {
    fn spawn_retry(&self, task: &Task) {
        let id = task.id.clone();

        let pending = Arc::clone(&self.pending_tasks);
        let running = Arc::clone(&self.running_tasks);

        let handler = tokio::spawn(async move {
            use dashmap::mapref::entry::Entry;

            time::sleep(Duration::from_secs(5)).await;
            if let Entry::Occupied(o) = running.entry(id) {
                let mut task = o.remove_entry().1;
                warn!("task timeout: {:?}", task);
                task.id = Uuid::new_v4().to_string();
                pending.push(task).unwrap();
            }
        });

        self.retry_handlers.insert(task.id.clone(), handler);
    }

    fn reduce_stage(&self) -> bool {
        self.reduce_stage.load(Ordering::Acquire)
    }

    async fn shutdown(&self) {
        let mut inner = self.shutdown.lock().await;
        if let Some(sender) = inner.take() {
            let _ = sender.send(());
        }
    }
}

#[tonic::async_trait]
impl MapReduce for Coordinator {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        let name = request.into_inner().name;
        Ok(Response::new(HelloReply {
            message: format!("Hello, {}!", name),
        }))
    }

    async fn poll_task(
        &self,
        _request: Request<PollTaskRequest>,
    ) -> Result<Response<PollTaskReply>, tonic::Status> {
        let reply = match self.pending_tasks.pop() {
            Some(task) => {
                assert!(self.reduce_stage() ^ matches!(task.inner, Some(task::Inner::MapTask(_))));

                self.running_tasks.insert(task.id.clone(), task.clone());
                self.spawn_retry(&task);
                PollTaskReply {
                    task: Some(task),
                    shutdown: false,
                }
            }
            None => PollTaskReply {
                task: None,
                shutdown: self.reduce_stage() && self.running_tasks.is_empty(),
            },
        };

        info!("poll task reply: {:?}", reply);
        Ok(Response::new(reply))
    }

    async fn complete_task(
        &self,
        request: Request<CompleteTaskRequest>,
    ) -> Result<Response<CompleteTaskReply>, tonic::Status> {
        let CompleteTaskRequest { task, reduce_files } = request.into_inner();
        let task = task.ok_or(tonic::Status::invalid_argument(""))?;

        if self.running_tasks.remove(&task.id).is_some() {
            info!("task done: {:?}", task);
            if let Some((_, handler)) = self.retry_handlers.remove(&task.id) {
                handler.abort()
            }

            // only update `reduce_files` if the task is previously in `running_tasks`,
            // that is, not treated as "dead worker"
            for (reducer_index, file) in reduce_files {
                self.reduce_files
                    .entry(reducer_index)
                    .or_default()
                    .push(file);
            }
        }

        if self.pending_tasks.is_empty() && self.running_tasks.is_empty() {
            let previous_reduce = self.reduce_stage.fetch_or(true, Ordering::SeqCst);
            if !previous_reduce {
                info!("map done, init reduce");
                info!("reduce_files: {:?}", self.reduce_files);
                self.init_reduce();
            } else {
                info!("all done");
                self.shutdown().await;
            }
        }

        Ok(Response::new(CompleteTaskReply {}))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger();

    let cli = Cli::parse();
    let (tx, rx) = oneshot::channel::<()>();
    let addr = cli.listen.parse()?;
    let coordinator = Coordinator::new(cli, tx);

    let server = MapReduceServer::new(coordinator);
    Server::builder()
        .add_service(server)
        .serve_with_shutdown(addr, async move {
            rx.await.ok();
            time::sleep(Duration::from_millis(1000)).await;
        })
        .await?;
    Ok(())
}
