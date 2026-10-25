use crate::{
    backend::Backend,
    domain::Command,
    engine::{Engine, Snapshot, Subscription},
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc::{self, Sender},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use tokio::sync::watch;

pub enum Request {
    RefreshRate(u32),
    Subscribe {
        context: String,
        subscription: Box<Subscription>,
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
    pub language: String,
    pub(crate) pairs: Arc<std::sync::Mutex<crate::pairs::Pairs>>,
    requests: Sender<QueuedRequest>,
    pending_input: Arc<AtomicUsize>,
    pub snapshots: watch::Receiver<Snapshot>,
}

struct QueuedRequest {
    request: Request,
    charged: bool,
}

impl WorkerClient {
    pub fn send(&self, request: Request) -> anyhow::Result<()> {
        let charged = matches!(request, Request::Command { .. } | Request::Press { .. });
        if charged {
            self.pending_input
                .try_update(Ordering::AcqRel, Ordering::Acquire, |pending| {
                    (pending < 256).then_some(pending + 1)
                })
                .map_err(|_| anyhow::anyhow!("Worker input queue full"))?;
        }
        // Lifecycle messages share FIFO ordering with input but cannot be rejected
        // by a saturated input quota (especially keyUp and willDisappear).
        if self
            .requests
            .send(QueuedRequest { request, charged })
            .is_err()
        {
            if charged {
                self.pending_input.fetch_sub(1, Ordering::AcqRel);
            }
            anyhow::bail!("Worker stopped");
        }
        Ok(())
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
        let (requests, receiver) = mpsc::channel::<QueuedRequest>();
        let pending_input = Arc::new(AtomicUsize::new(0));
        let pending_worker = pending_input.clone();
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
                let started = Instant::now();
                let mut frame_at = started;
                let mut frame_period = Duration::from_secs_f64(1.0 / 15.0);
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
                    if now >= frame_at {
                        engine.refresh();
                        engine.record_frame(started.elapsed().as_millis() as u64);
                        snapshots.send_replace(engine.snapshot().clone());
                        frame_at = now + frame_period;
                    }
                    let request = receiver
                        .recv_timeout(Duration::from_millis(2))
                        .map(|queued| {
                            if queued.charged {
                                pending_worker.fetch_sub(1, Ordering::AcqRel);
                            }
                            queued.request
                        });
                    match request {
                        Ok(Request::RefreshRate(fps)) => {
                            if matches!(fps, 15 | 20 | 25 | 30) {
                                frame_period = Duration::from_secs_f64(1.0 / f64::from(fps));
                                frame_at = Instant::now() + frame_period;
                            }
                        }
                        Ok(Request::Subscribe {
                            context,
                            subscription,
                        }) => engine.subscribe(context, *subscription),
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
                language: "en".into(),
                pairs: Default::default(),
                requests,
                pending_input,
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn input_saturation_preserves_release_and_lifecycle_order() {
        let (requests, receiver) = mpsc::channel();
        let (_, snapshots) = watch::channel(Snapshot::default());
        let client = WorkerClient {
            language: "en".into(),
            pairs: Default::default(),
            requests,
            snapshots,
            pending_input: Arc::new(AtomicUsize::new(0)),
        };
        for _ in 0..256 {
            client
                .send(Request::Press {
                    context: "key".into(),
                    generation: 0,
                })
                .unwrap();
        }
        assert!(
            client
                .send(Request::Press {
                    context: "key".into(),
                    generation: 0
                })
                .is_err()
        );
        client.send(Request::Release("key".into())).unwrap();
        client.send(Request::Remove("key".into())).unwrap();
        for _ in 0..256 {
            assert!(matches!(
                receiver.recv().unwrap().request,
                Request::Press { .. }
            ));
        }
        assert!(matches!(
            receiver.recv().unwrap().request,
            Request::Release(_)
        ));
        assert!(matches!(
            receiver.recv().unwrap().request,
            Request::Remove(_)
        ));
    }
}
