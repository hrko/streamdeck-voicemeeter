use crate::{settings::Settings, worker::WorkerClient};
use std::sync::Arc;
use streamdeck_plugin::{Action, ActionContext, ActionPayload, Result, Target, async_trait};
use tokio::task::JoinHandle;

type Context<'a> = ActionContext<'a, Settings, WorkerClient>;

pub struct FrameTest<const DIAL: bool> {
    client: Arc<WorkerClient>,
    task: Option<JoinHandle<()>>,
}

fn svg(sequence: u64, elapsed_ms: u64, dial: bool) -> String {
    let (width, height, font, baseline) = if dial {
        (200, 100, 48, 59)
    } else {
        (144, 144, 38, 82)
    };
    let center = width / 2;
    let phase = if sequence.is_multiple_of(2) {
        "#ffffff"
    } else {
        "#303030"
    };
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}"><rect width="{width}" height="{height}" fill="#000000"/><rect x="6" y="6" width="10" height="10" fill="{phase}"/><text x="{center}" y="{baseline}" text-anchor="middle" font-family="Consolas,monospace" font-size="{font}" font-weight="bold" fill="#ffffff">{:05}</text><text x="{center}" y="{}" text-anchor="middle" font-family="Consolas,monospace" font-size="12" fill="#aaaaaa">{} ms</text></svg>"##,
        sequence % 100_000,
        baseline + 24,
        elapsed_ms
    )
}

#[async_trait]
impl<const DIAL: bool> Action for FrameTest<DIAL> {
    type Settings = Settings;
    type State = WorkerClient;
    const UUID: &'static str = if DIAL {
        "jp.hrko.streamdeck.voicemeeter.rust.frame-test-dial"
    } else {
        "jp.hrko.streamdeck.voicemeeter.rust.frame-test-key"
    };
    fn new(client: Arc<Self::State>) -> Self {
        Self { client, task: None }
    }
    async fn on_will_appear(&mut self, _: &ActionPayload, ctx: &Context<'_>) -> Result<()> {
        if let Some(task) = self.task.take() {
            task.abort();
        }
        if DIAL {
            ctx.set_feedback_layout("layouts/frame-test.json")?;
        }
        let sender = ctx.sender().clone();
        let context = ctx.identity().context.clone();
        let mut frames = self.client.snapshots.clone();
        self.task = Some(tokio::spawn(async move {
            let mut previous = None;
            loop {
                let (sequence, elapsed) = {
                    let snapshot = frames.borrow_and_update();
                    (snapshot.frame_sequence, snapshot.frame_elapsed_ms)
                };
                if previous != Some(sequence) {
                    let image = crate::render::data_url(&svg(sequence, elapsed, DIAL));
                    let result = if DIAL {
                        sender.set_feedback(&context, &serde_json::json!({"frame":image}))
                    } else {
                        sender.set_image(&context, Some(&image), Target::HardwareAndSoftware, None)
                    };
                    if let Err(error) = result {
                        tracing::warn!(%error, sequence, "Frame test send failed");
                    }
                    previous = Some(sequence);
                }
                if frames.changed().await.is_err() {
                    break;
                }
            }
        }));
        Ok(())
    }
    async fn on_will_disappear(&mut self, _: &ActionPayload, _: &Context<'_>) -> Result<()> {
        if let Some(task) = self.task.take() {
            task.abort();
        }
        Ok(())
    }
}
impl<const DIAL: bool> Drop for FrameTest<DIAL> {
    fn drop(&mut self) {
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn counter_and_phase_advance_and_wrap() {
        for dial in [false, true] {
            let a = super::svg(99999, 3333300, dial);
            let b = super::svg(100000, 3333333, dial);
            assert!(a.contains(">99999</text>"));
            assert!(b.contains(">00000</text>"));
            assert!(b.contains(">3333333 ms</text>"));
            assert!(a.contains("height=\"10\" fill=\"#303030\""));
            assert!(b.contains("height=\"10\" fill=\"#ffffff\""));
        }
    }
}
