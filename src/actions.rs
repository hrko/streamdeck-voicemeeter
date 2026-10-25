use crate::{
    domain::{self, ActionKind, Command},
    engine::{Snapshot, Subscription},
    render::{self, IconCache},
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
    repeating: Option<JoinHandle<()>>,
    active: Arc<AtomicBool>,
    invalid_settings: bool,
}

type Context<'a> = ActionContext<'a, Settings, WorkerClient>;

impl<const KIND: u8> DeckAction<KIND> {
    fn kind() -> ActionKind {
        match KIND {
            0 => ActionKind::Gain,
            1 => ActionKind::Combo,
            3 => ActionKind::GainButton,
            _ => ActionKind::Macro,
        }
    }
    fn send(&self, request: Request) -> Result<()> {
        self.client
            .send(request)
            .map_err(|error| streamdeck_plugin::Error::State(error.to_string()))
    }
    fn stop_repeating(&mut self) {
        if let Some(task) = self.repeating.take() {
            task.abort();
        }
    }
    fn stop_rendering(&mut self) {
        self.stop_repeating();
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
        if self.invalid_settings {
            self.send(Request::Remove(context))?;
            ctx.show_alert()?;
            let mut status =
                inspector_status(&self.client.snapshots.borrow(), &ctx.identity().context);
            status["error"] = json!({"code":"invalidSettings", "field":""});
            ctx.send_to_property_inspector(&status)?;
            return Ok(());
        }
        self.send(Request::Subscribe {
            context: context.clone(),
            subscription: Box::new(Subscription {
                kind: Self::kind(),
                settings: settings.clone(),
            }),
        })?;
        if let Err(error) = settings.validate() {
            ctx.show_alert()?;
            ctx.send_to_property_inspector(&json!({"event":"status", "error":error}))?;
            return Ok(());
        }
        let sender = ctx.sender().clone();
        if matches!(Self::kind(), ActionKind::Gain | ActionKind::Combo) {
            ctx.set_feedback_layout(crate::dial::layout(Self::kind()))?;
        }
        let mut snapshots = self.client.snapshots.clone();
        let language = self.client.language.clone();
        let active = Arc::new(AtomicBool::new(true));
        self.active = active.clone();
        self.rendering = Some(tokio::spawn(async move {
            let mut previous = Value::Null;
            let mut previous_image = String::new();
            let mut previous_feedback = Value::Null;
            let mut previous_state = None;
            let mut previous_title: Option<Option<String>> = None;
            let mut cache = IconCache::default();
            loop {
                let snapshot = snapshots.borrow_and_update().clone();
                if !active.load(Ordering::Acquire) {
                    break;
                }
                let (frame, feedback, missing_glyph) = if Self::kind() == ActionKind::Macro {
                    // A newly visible subscription has not necessarily been read yet.
                    // Keep the host's existing image until data or an actual error arrives.
                    if snapshot.edition.is_some()
                        && !snapshot
                            .macros
                            .contains_key(&settings.macro_button.logical_id)
                        && !snapshot.errors.contains_key(&context)
                    {
                        (None, Value::Null, false)
                    } else {
                        let frame = render::macro_frame(&snapshot, &settings, &mut cache);
                        let missing = frame.missing_glyph;
                        (Some(frame), Value::Null, missing)
                    }
                } else if Self::kind() == ActionKind::GainButton {
                    let frame =
                        crate::gain_button::frame(&snapshot, &settings, &mut cache, &language);
                    let missing = frame.missing_glyph;
                    (Some(frame), Value::Null, missing)
                } else {
                    let (feedback, missing) = crate::dial::feedback(
                        &snapshot,
                        &settings,
                        Self::kind(),
                        &mut cache,
                        &language,
                    );
                    (None, feedback, missing)
                };
                let mut message = inspector_status(&snapshot, &context);
                if missing_glyph && message["error"].is_null() {
                    message["error"] = json!({"code":"missingGlyph", "field":""});
                }
                if message != previous {
                    let _ = sender.send_to_property_inspector(&context, &message);
                    previous = message;
                }
                if Self::kind() == ActionKind::Macro
                    && let Some(on) = snapshot.macros.get(&settings.macro_button.logical_id)
                    && Some(*on) != previous_state
                {
                    let _ = sender.set_state(&context, if *on { 0 } else { 1 });
                    previous_state = Some(*on);
                }
                if Self::kind() == ActionKind::GainButton {
                    let title = crate::gain_button::title(&snapshot, &settings);
                    if previous_title.as_ref() != Some(&title) {
                        let _ = sender.set_title(
                            &context,
                            title.as_deref(),
                            streamdeck_plugin::Target::HardwareAndSoftware,
                            None,
                        );
                        previous_title = Some(title);
                    }
                }
                if let Some(frame) = frame
                    && frame.svg != previous_image
                {
                    let image = render::data_url(&frame.svg);
                    let _ = sender.set_image(
                        &context,
                        Some(&image),
                        streamdeck_plugin::Target::HardwareAndSoftware,
                        None,
                    );
                    previous_image = frame.svg;
                }
                if !feedback.is_null() && feedback != previous_feedback {
                    if let Err(error) = sender.set_feedback(&context, &feedback) {
                        tracing::warn!(%error, "Could not send dial feedback");
                    }
                    previous_feedback = feedback;
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
        3 => "jp.hrko.streamdeck.voicemeeter.rust.gain-button",
        _ => "jp.hrko.streamdeck.voicemeeter.rust.macro",
    };
    fn new(state: Arc<Self::State>) -> Self {
        Self {
            client: state,
            context: None,
            rendering: None,
            repeating: None,
            active: Arc::new(AtomicBool::new(false)),
            invalid_settings: false,
        }
    }
    async fn on_will_appear(&mut self, payload: &ActionPayload, ctx: &Context<'_>) -> Result<()> {
        self.invalid_settings =
            serde_json::from_value::<Settings>(payload.settings.clone()).is_err();
        if KIND == 3 && !payload.is_in_multi_action && !self.invalid_settings {
            self.client
                .pairs
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .appear(ctx.identity().clone(), ctx.settings());
        }
        self.update(ctx)
    }
    async fn on_did_receive_settings(
        &mut self,
        payload: &ActionPayload,
        _: &Context<'_>,
    ) -> Result<()> {
        // SDK 0.1.0 falls back to Settings::default() on deserialization errors.
        // Inspect the raw payload so malformed targets can never act on Strip 0.
        self.invalid_settings =
            serde_json::from_value::<Settings>(payload.settings.clone()).is_err();
        Ok(())
    }
    async fn on_settings_changed(&mut self, _: &Settings, ctx: &Context<'_>) -> Result<()> {
        if KIND == 3 {
            let mut pairs = self.client.pairs.lock().unwrap_or_else(|e| e.into_inner());
            if self.invalid_settings {
                pairs.remove(&ctx.identity().context);
            } else {
                if let Some(latest) =
                    pairs.changed(&ctx.identity().context, ctx.settings(), ctx.sender())?
                {
                    ctx.update_settings(|settings| *settings = latest)?;
                }
            }
        }
        self.update(ctx)
    }
    async fn on_will_disappear(&mut self, _: &ActionPayload, ctx: &Context<'_>) -> Result<()> {
        self.stop_rendering();
        self.context = None;
        self.client
            .pairs
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&ctx.identity().context);
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
        if let Some(command) = domain::touched(
            Self::kind(),
            &ctx.settings(),
            payload.tap_x as u32,
            payload.hold,
        ) {
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
        if Self::kind() == ActionKind::GainButton {
            if self.repeating.is_some() {
                return Ok(());
            }
            let settings = ctx.settings();
            let command = domain::gain_button_command(&settings);
            let generation = self.client.generation();
            self.send(Request::Command {
                context: ctx.identity().context.clone(),
                generation,
                command: command.clone(),
            })?;
            if settings.gain_button.operation == crate::settings::GainOperation::Adjust
                && !self.invalid_settings
                && settings.validate().is_ok()
                && self.client.snapshots.borrow().edition.is_some()
            {
                let client = self.client.clone();
                let context = ctx.identity().context.clone();
                self.repeating = Some(tokio::spawn(async move {
                    let period = std::time::Duration::from_millis(250);
                    let mut timer =
                        tokio::time::interval_at(tokio::time::Instant::now() + period, period);
                    timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                    loop {
                        timer.tick().await;
                        if client.generation() != generation
                            || client.snapshots.borrow().edition.is_none()
                        {
                            break;
                        }
                        if client
                            .send(Request::Command {
                                context: context.clone(),
                                generation,
                                command: command.clone(),
                            })
                            .is_err()
                        {
                            break;
                        }
                    }
                }));
            }
        }
        if Self::kind() == ActionKind::Macro {
            self.send(Request::Press {
                context: ctx.identity().context.clone(),
                generation: self.client.generation(),
            })?;
        }
        Ok(())
    }
    async fn on_key_up(&mut self, _: &ActionPayload, ctx: &Context<'_>) -> Result<()> {
        self.stop_repeating();
        if Self::kind() == ActionKind::Macro {
            self.send(Request::Release(ctx.identity().context.clone()))?;
        }
        Ok(())
    }
    async fn on_property_inspector_did_appear(&mut self, ctx: &Context<'_>) -> Result<()> {
        let mut status = inspector_status(&self.client.snapshots.borrow(), &ctx.identity().context);
        if self.invalid_settings {
            status["error"] = json!({"code":"invalidSettings", "field":""});
        }
        ctx.send_to_property_inspector(&status)
    }
    async fn on_property_inspector_message(&mut self, _: &Value, ctx: &Context<'_>) -> Result<()> {
        self.on_property_inspector_did_appear(ctx).await
    }
}

impl<const KIND: u8> Drop for DeckAction<KIND> {
    fn drop(&mut self) {
        self.stop_rendering();
        if let Some(context) = self.context.take() {
            self.client
                .pairs
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .remove(&context);
            let _ = self.client.send(Request::Remove(context));
        }
    }
}
