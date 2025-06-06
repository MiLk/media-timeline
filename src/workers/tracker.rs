use async_trait::async_trait;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tokio_util::task::task_tracker::TaskTrackerWaitFuture;

#[async_trait]
pub trait Worker {
    async fn run(&self, cancellation_token: CancellationToken) -> ();
}

pub struct WorkerTracker {
    tracker: TaskTracker,
    cancellation_token: CancellationToken,
    workers: Vec<Arc<dyn Worker + Send + Sync + 'static>>,
}

impl WorkerTracker {
    pub fn new() -> Self {
        let tracker = TaskTracker::new();
        let cancellation_token = CancellationToken::new();
        Self {
            tracker,
            cancellation_token,
            workers: Vec::new(),
        }
    }

    pub fn register_worker<T>(&mut self, worker: T) -> ()
    where
        T: Worker + Send + Sync + 'static,
    {
        self.workers.push(Arc::new(worker));
    }

    pub fn start(&self) -> () {
        log::info!("starting the workers");
        for worker in &self.workers {
            let token = self.cancellation_token.clone();
            let worker_ = worker.clone();
            self.tracker.spawn(async move {
                worker_.run(token).await;
            });
        }
        self.tracker.close();
    }

    pub fn stop(&self) -> () {
        self.cancellation_token.cancel();
    }

    pub fn wait(&self) -> TaskTrackerWaitFuture<'_> {
        self.tracker.wait()
    }
}
