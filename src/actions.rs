use crate::{
    domain::{self, ActionKind, Command},
    engine::{Snapshot, Subscription},
    settings::Settings,
    worker::{Request, WorkerClient},
};
use serde_json::{Value, json};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use streamdeck_plugin::{
    Action, ActionContext, ActionPayload, DialRotatePayload, Result, TouchTapPayload, async_trait,
};
use tokio::task::JoinHandle;

pub const PLUGIN_ID: &str = "jp.hrko.streamdeck.voicemeeter.rust";

pub struct DeckAction<const KIND: u8> {
    client: Arc<WorkerClient>,
    context: Option<String>,
    rendering: Option<JoinHandle<()>>,
    active: Arc<AtomicBool>,
}

type Context<'a> = ActionContext<'a, Settings, WorkerClient>;

impl<const KIND: u8> DeckAction<KIND> {
    fn kind() -> ActionKind {
        match KIND {
            0 => ActionKind::Gain,
            1 => ActionKind::Combo,
            _ => ActionKind::Macro,
        }
    }
    fn send(&self, request: Request) -> Result<()> {
        self.client
            .send(request)
            .map_err(|error| streamdeck_plugin::Error::State(error.to_string()))
    }
    fn stop_rendering(&mut self) {
        self.active.store(false, Ordering::Release);
        if let Some(task) = self.rendering.take() {
            task.abort();
        }
    }
    fn update(&mut self, ctx: &Context<'_>) -> Result<()> {
        self.stop_rendering();
        let settings = ctx.settings();
        let context = ctx.identity().context.clone();
        self.context = Some(context.clone());
        self.send(Request::Subscribe {
            context: context.clone(),
            subscription: Subscription {
                kind: Self::kind(),
                settings: settings.clone(),
            },
        })?;
        if let Err(error) = settings.validate() {
            ctx.show_alert()?;
            ctx.send_to_property_inspector(&json!({"event":"status", "error":error}))?;
            return Ok(());
        }
        let sender = ctx.sender().clone();
        let mut snapshots = self.client.snapshots.clone();
        let active = Arc::new(AtomicBool::new(true));
        self.active = active.clone();
        self.rendering = Some(tokio::spawn(async move {
            let mut previous = Value::Null;
            loop {
                let snapshot = snapshots.borrow_and_update().clone();
                if !active.load(Ordering::Acquire) {
                    break;
                }
                let message = inspector_status(&snapshot, &context);
                if message != previous {
                    let _ = sender.send_to_property_inspector(&context, &message);
                    previous = message;
                }
                if Self::kind() == ActionKind::Macro {
                    if let Some(on) = snapshot.macros.get(&settings.macro_button.logical_id) {
                        let _ = sender.set_state(&context, if *on { 0 } else { 1 });
                    }
                } else {
                    let feedback = gain_feedback(&snapshot, &settings, Self::kind());
                    let _ = sender.set_feedback(&context, &feedback);
                }
                if snapshots.changed().await.is_err() {
                    break;
                }
            }
        }));
        Ok(())
    }
    fn command(&self, ctx: &Context<'_>, command: Command) -> Result<()> {
        self.send(Request::Command {
            context: ctx.identity().context.clone(),
            generation: self.client.generation(),
            command,
        })
    }
}

pub fn inspector_status(snapshot: &Snapshot, context: &str) -> Value {
    json!({"event":"status", "edition":snapshot.edition, "connected":snapshot.edition.is_some(), "error":snapshot.errors.get(context).map(|code| json!({"code":code,"field":""}))})
}

/// Text feedback is extended with SVG meter/icon fields by the renderer.
pub fn gain_feedback(snapshot: &Snapshot, settings: &Settings, kind: ActionKind) -> Value {
    let mut feedback = serde_json::Map::new();
    let count = if kind == ActionKind::Combo { 2 } else { 1 };
    for index in 0..count {
        let suffix = if index == 0 { "" } else { "1" };
        let target = settings.channels[index].target;
        let channel = snapshot
            .channels
            .iter()
            .find(|(key, _)| *key == target)
            .map(|(_, channel)| channel);
        let title = channel
            .map(|c| c.label.clone())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("{:?} {}", target.kind, target.index));
        feedback.insert(format!("title{suffix}"), json!(title));
        feedback.insert(
            format!("gainValue{suffix}"),
            json!(
                channel
                    .map(|c| format!("{:.1} dB", c.gain))
                    .unwrap_or_else(|| "—".into())
            ),
        );
    }
    Value::Object(feedback)
}

#[async_trait]
impl<const KIND: u8> Action for DeckAction<KIND> {
    type Settings = Settings;
    type State = WorkerClient;
    const UUID: &'static str = match KIND {
        0 => "jp.hrko.streamdeck.voicemeeter.rust.gain-control",
        1 => "jp.hrko.streamdeck.voicemeeter.rust.gain-control-combo",
        _ => "jp.hrko.streamdeck.voicemeeter.rust.macro",
    };
    fn new(state: Arc<Self::State>) -> Self {
        Self {
            client: state,
            context: None,
            rendering: None,
            active: Arc::new(AtomicBool::new(false)),
        }
    }
    async fn on_will_appear(&mut self, _: &ActionPayload, ctx: &Context<'_>) -> Result<()> {
        self.update(ctx)
    }
    async fn on_settings_changed(&mut self, _: &Settings, ctx: &Context<'_>) -> Result<()> {
        self.update(ctx)
    }
    async fn on_will_disappear(&mut self, _: &ActionPayload, ctx: &Context<'_>) -> Result<()> {
        self.stop_rendering();
        self.context = None;
        self.send(Request::Remove(ctx.identity().context.clone()))
    }
    async fn on_dial_rotate(
        &mut self,
        payload: &DialRotatePayload,
        ctx: &Context<'_>,
    ) -> Result<()> {
        if let Some(command) = domain::rotated(
            Self::kind(),
            &ctx.settings(),
            payload.ticks,
            payload.pressed,
        ) {
            self.command(ctx, command)?;
        }
        Ok(())
    }
    async fn on_touch_tap(&mut self, payload: &TouchTapPayload, ctx: &Context<'_>) -> Result<()> {
        if !(0..200).contains(&payload.tap_x) {
            return Ok(());
        }
        if let Some(command) = domain::tapped(Self::kind(), &ctx.settings(), payload.tap_x as u32) {
            self.command(ctx, command)?;
        }
        Ok(())
    }
    async fn on_dial_down(&mut self, _: &ActionPayload, ctx: &Context<'_>) -> Result<()> {
        if Self::kind() == ActionKind::Gain {
            self.command(ctx, Command::ToggleMute(ctx.settings().channels[0].target))?;
        }
        Ok(())
    }
    async fn on_key_down(&mut self, _: &ActionPayload, ctx: &Context<'_>) -> Result<()> {
        if Self::kind() == ActionKind::Macro {
            self.send(Request::Press {
                context: ctx.identity().context.clone(),
                generation: self.client.generation(),
            })?;
        }
        Ok(())
    }
    async fn on_key_up(&mut self, _: &ActionPayload, ctx: &Context<'_>) -> Result<()> {
        if Self::kind() == ActionKind::Macro {
            self.send(Request::Release(ctx.identity().context.clone()))?;
        }
        Ok(())
    }
    async fn on_property_inspector_did_appear(&mut self, ctx: &Context<'_>) -> Result<()> {
        ctx.send_to_property_inspector(&inspector_status(
            &self.client.snapshots.borrow(),
            &ctx.identity().context,
        ))
    }
    async fn on_property_inspector_message(&mut self, _: &Value, ctx: &Context<'_>) -> Result<()> {
        self.on_property_inspector_did_appear(ctx).await
    }
}

impl<const KIND: u8> Drop for DeckAction<KIND> {
    fn drop(&mut self) {
        self.stop_rendering();
        if let Some(context) = self.context.take() {
            let _ = self.client.send(Request::Remove(context));
        }
    }
}
