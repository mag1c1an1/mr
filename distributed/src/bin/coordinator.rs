use ::time::macros::format_description;
use anyhow::Result;
use clap::Parser;
use crossbeam_queue::ArrayQueue;
use dashmap::DashMap;
use distributed::{
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
use tonic::{transport::Server, Request, Response};
use tracing::{info, instrument, warn};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::time::LocalTime;
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
        // each file is a map task
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

        for t in map_tasks {
            let id = Uuid::new_v4().to_string();
            let task = Task {
                id,
                inner: Some(task::Inner::MapTask(t)),
            };
            self.pending_tasks.push(task).unwrap();
        }
        self.retry_handlers.iter().for_each(|r| r.abort());
        self.retry_handlers.clear();
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
        self.retry_handlers.iter().for_each(|r| r.abort());
        self.retry_handlers.clear();
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

impl std::fmt::Debug for Coordinator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Coordinator")
            .field("pending_tasks_len", &self.pending_tasks.len())
            .field("running_tasks_len", &self.running_tasks.len())
            .field("reduce_files", &self.reduce_files)
            .field("retry_handlers_len", &self.retry_handlers.len())
            .finish()
    }
}

impl Coordinator {
    // retry
    fn spawn_retry(&self, task: &Task) {
        let id = task.id.clone();

        let pending = Arc::clone(&self.pending_tasks);
        let running = Arc::clone(&self.running_tasks);
        // new async work
        let handler = tokio::spawn(async move {
            use dashmap::mapref::entry::Entry;
            // wait 10 secs for job_count
            time::sleep(Duration::from_secs(10)).await;
            if let Entry::Occupied(o) = running.entry(id) {
                let mut task = o.remove_entry().1;
                warn!("task timeout: {:?}", task);
                // reset task id
                task.id = Uuid::new_v4().to_string();
                // TODO: queue task overflow?
                pending.push(task).unwrap();
            }
        });

        self.retry_handlers.insert(task.id.clone(), handler);
    }

    fn reduce_stage(&self) -> bool {
        self.reduce_stage.load(Ordering::Acquire)
    }

    async fn shutdown(&self) {
        // only once
        let mut inner = self.shutdown.lock().await;
        if let Some(sender) = inner.take() {
            let _ = sender.send(());
        }
    }
}

#[tonic::async_trait]
impl MapReduce for Coordinator {
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

        info!("{}", reply);
        Ok(Response::new(reply))
    }
    #[instrument]
    async fn complete_task(
        &self,
        request: Request<CompleteTaskRequest>,
    ) -> Result<Response<CompleteTaskReply>, tonic::Status> {
        let CompleteTaskRequest { task, reduce_files } = request.into_inner();
        let task = task.ok_or(tonic::Status::invalid_argument(""))?;

        if self.running_tasks.remove(&task.id).is_some() {
            info!("Task Done id: {}", task.id);
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

const LOG_PATH: &str = "/Users/mag1cian/dev/mr/log";
fn init_logger() -> WorkerGuard {
    let format = tracing_subscriber::fmt::format().compact();
    let appender = tracing_appender::rolling::never(LOG_PATH, "coordinator.log");
    let (non_blockking_appender, guard) = tracing_appender::non_blocking(appender);
    let lt = LocalTime::new(format_description!("[hour]:[minute]:[second]"));
    tracing_subscriber::fmt()
        .event_format(format)
        .with_ansi(false)
        .with_timer(lt)
        .with_writer(non_blockking_appender)
        .init();
    guard
}

#[tokio::main]
async fn main() -> Result<()> {
    let _log_guard = init_logger();

    let cli = Cli::parse();
    let (tx, rx) = oneshot::channel::<()>();
    let addr = cli.listen.parse()?;
    let coordinator = Coordinator::new(cli, tx);

    let server = MapReduceServer::new(coordinator);
    Server::builder()
        .add_service(server)
        .serve_with_shutdown(addr, async move {
            rx.await.ok();
            // sleep 1s
            time::sleep(Duration::from_millis(1000)).await;
        })
        .await?;
    Ok(())
}
