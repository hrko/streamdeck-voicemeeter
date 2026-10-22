use crate::{
    backend::Backend,
    domain::Command,
    engine::{Engine, Snapshot, Subscription},
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, SyncSender},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use tokio::sync::watch;

pub enum Request {
    Subscribe {
        context: String,
        subscription: Subscription,
    },
    Remove(String),
    Command {
        context: String,
        generation: u64,
        command: Command,
    },
    Press {
        context: String,
        generation: u64,
    },
    Release(String),
}

#[derive(Clone)]
pub struct WorkerClient {
    requests: SyncSender<Request>,
    pub snapshots: watch::Receiver<Snapshot>,
}

impl WorkerClient {
    pub fn send(&self, request: Request) -> anyhow::Result<()> {
        self.requests
            .try_send(request)
            .map_err(|error| anyhow::anyhow!("Worker request rejected: {error}"))
    }
    pub fn generation(&self) -> u64 {
        self.snapshots.borrow().generation
    }
}

pub struct Worker {
    pub client: WorkerClient,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    /// Construct the backend inside the worker to keep even initialization on one thread.
    pub fn spawn<B: Backend + 'static>(
        factory: impl FnOnce() -> B + Send + 'static,
    ) -> anyhow::Result<Self> {
        let (requests, receiver) = mpsc::sync_channel(256);
        let (snapshots, snapshot_receiver) = watch::channel(Snapshot::default());
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = stop.clone();
        let thread = thread::Builder::new()
            .name("voicemeeter".into())
            .spawn(move || {
                let mut engine = Engine::new(factory());
                let mut reconnect_at = Instant::now();
                let mut backoff = Duration::from_secs(1);
                let mut poll_at = Instant::now();
                let mut frame_at = Instant::now();
                while !stopped.load(Ordering::Acquire) {
                    let now = Instant::now();
                    if engine.snapshot().edition.is_none() && now >= reconnect_at {
                        match engine.connect() {
                            Ok(true) => {
                                backoff = Duration::from_secs(1);
                                frame_at = now;
                            }
                            result => {
                                if let Err(error) = result {
                                    tracing::warn!(%error, "VoiceMeeter connection failed");
                                }
                                reconnect_at = now + backoff;
                                backoff = (backoff * 2).min(Duration::from_secs(10));
                            }
                        }
                        snapshots.send_replace(engine.snapshot().clone());
                    }
                    if engine.snapshot().edition.is_some() && now >= poll_at {
                        if let Err(error) = engine.poll() {
                            tracing::warn!(%error, "VoiceMeeter disconnected");
                            reconnect_at = now + Duration::from_secs(1);
                            snapshots.send_replace(engine.snapshot().clone());
                        }
                        poll_at = now + Duration::from_millis(10);
                    }
                    if engine.snapshot().edition.is_some() && now >= frame_at {
                        engine.refresh();
                        snapshots.send_replace(engine.snapshot().clone());
                        frame_at = now + Duration::from_micros(66_667);
                    }
                    match receiver.recv_timeout(Duration::from_millis(2)) {
                        Ok(Request::Subscribe {
                            context,
                            subscription,
                        }) => engine.subscribe(context, subscription),
                        Ok(Request::Remove(context)) => engine.remove(&context),
                        Ok(Request::Command {
                            context,
                            generation,
                            command,
                        }) => {
                            engine.command(&context, generation, command);
                        }
                        Ok(Request::Press {
                            context,
                            generation,
                        }) => engine.press(&context, generation),
                        Ok(Request::Release(context)) => engine.release(&context),
                        Err(mpsc::RecvTimeoutError::Timeout) => {}
                        Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    }
                }
                engine.shutdown();
                snapshots.send_replace(engine.snapshot().clone());
            })?;
        Ok(Self {
            client: WorkerClient {
                requests,
                snapshots: snapshot_receiver,
            },
            stop,
            thread: Some(thread),
        })
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
