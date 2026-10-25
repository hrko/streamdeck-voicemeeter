use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use streamdeck_plugin::{
    Action, ActionEvent, ActionInstance, CommandSender, IncomingMessage, TypedInstance,
};
use streamdeck_voicemeeter::{
    actions::DeckAction,
    backend::{Backend, ChannelSnapshot},
    domain::Edition,
    settings::{Settings, Target},
    worker::Worker,
};

#[derive(Default)]
struct State {
    unavailable: bool,
    channels: BTreeMap<Target, ChannelSnapshot>,
    macros: BTreeMap<u8, bool>,
    writes: Vec<String>,
    writes_at_drop: Option<Vec<String>>,
    writes_at_flush: Option<Vec<String>>,
}
struct Fake(Arc<Mutex<State>>);
impl Drop for Fake {
    fn drop(&mut self) {
        let mut state = self.0.lock().unwrap();
        state.writes_at_drop = Some(state.writes.clone());
    }
}
impl Backend for Fake {
    fn flush(&mut self) -> anyhow::Result<()> {
        let mut state = self.0.lock().unwrap();
        assert!(state.writes_at_drop.is_none());
        state.writes_at_flush = Some(state.writes.clone());
        Ok(())
    }
    fn connect(&mut self) -> anyhow::Result<Option<Edition>> {
        Ok((!self.0.lock().unwrap().unavailable).then_some(Edition::Potato))
    }
    fn poll(&mut self) -> anyhow::Result<()> {
        anyhow::ensure!(!self.0.lock().unwrap().unavailable, "simulated outage");
        Ok(())
    }
    fn channel(&mut self, target: Target) -> anyhow::Result<ChannelSnapshot> {
        Ok(self
            .0
            .lock()
            .unwrap()
            .channels
            .entry(target)
            .or_default()
            .clone())
    }
    fn set_gain(&mut self, target: Target, value: f32) -> anyhow::Result<()> {
        let mut state = self.0.lock().unwrap();
        state.channels.entry(target).or_default().gain = value;
        state.writes.push(format!("gain:{target:?}:{value}"));
        Ok(())
    }
    fn set_mute(&mut self, target: Target, value: bool) -> anyhow::Result<()> {
        let mut state = self.0.lock().unwrap();
        state.channels.entry(target).or_default().mute = value;
        state.writes.push(format!("mute:{target:?}:{value}"));
        Ok(())
    }
    fn macro_state(&mut self, id: u8) -> anyhow::Result<bool> {
        Ok(*self.0.lock().unwrap().macros.entry(id).or_default())
    }
    fn set_macro(&mut self, id: u8, value: bool) -> anyhow::Result<()> {
        let mut state = self.0.lock().unwrap();
        state.macros.insert(id, value);
        state.writes.push(format!("macro:{id}:{value}"));
        Ok(())
    }
}

async fn wait_connected(worker: &Worker, connected: bool) {
    let mut snapshots = worker.client.snapshots.clone();
    tokio::time::timeout(Duration::from_secs(4), async {
        loop {
            if snapshots.borrow_and_update().edition.is_some() == connected {
                break;
            }
            snapshots.changed().await.unwrap();
        }
    })
    .await
    .expect("worker did not reach expected connection state");
}

#[tokio::test]
async fn outage_discards_offline_input_and_original_push_release() {
    use streamdeck_voicemeeter::worker::Request;
    let (worker, mut action, sender, _output, state) = setup::<2>(json!({})).await;
    action
        .handle(event("keyDown", json!({})), sender.clone())
        .await
        .unwrap();
    settle(&worker).await;
    assert_eq!(state.lock().unwrap().writes, vec!["macro:0:true"]);
    state.lock().unwrap().unavailable = true;
    wait_connected(&worker, false).await;
    let offline_generation = worker.client.generation();
    action
        .handle(event("keyUp", json!({})), sender.clone())
        .await
        .unwrap();
    action
        .handle(event("keyDown", json!({})), sender.clone())
        .await
        .unwrap();
    state.lock().unwrap().unavailable = false;
    wait_connected(&worker, true).await;
    // Even a delayed offline request delivered after recovery must be discarded.
    worker
        .client
        .send(Request::Press {
            context: "ctx".into(),
            generation: offline_generation,
        })
        .unwrap();
    action
        .handle(event("keyUp", json!({})), sender.clone())
        .await
        .unwrap();
    settle(&worker).await;
    assert_eq!(state.lock().unwrap().writes, vec!["macro:0:true"]);
    action
        .handle(event("keyDown", json!({})), sender.clone())
        .await
        .unwrap();
    action
        .handle(event("keyUp", json!({})), sender)
        .await
        .unwrap();
    settle(&worker).await;
    assert_eq!(
        state.lock().unwrap().writes,
        vec!["macro:0:true", "macro:0:true", "macro:0:false"]
    );
}
async fn settle(worker: &Worker) {
    let mut receiver = worker.client.snapshots.clone();
    tokio::time::timeout(Duration::from_secs(3), async {
        for _ in 0..3 {
            receiver.changed().await.unwrap();
        }
    })
    .await
    .unwrap();
}
fn event(name: &str, payload: Value) -> ActionEvent {
    let message: IncomingMessage =
        serde_json::from_value(json!({"event":name,"payload":payload})).unwrap();
    ActionEvent::from_message(&message).unwrap()
}
async fn setup<const KIND: u8>(
    settings: Value,
) -> (
    Worker,
    TypedInstance<DeckAction<KIND>>,
    CommandSender,
    tokio::sync::mpsc::UnboundedReceiver<String>,
    Arc<Mutex<State>>,
) {
    let state = Arc::new(Mutex::new(State::default()));
    let backend = state.clone();
    let worker = Worker::spawn(move || Fake(backend)).unwrap();
    settle(&worker).await;
    let mut action = TypedInstance::<DeckAction<KIND>>::new(Arc::new(worker.client.clone()));
    action.attach(
        "ctx".into(),
        DeckAction::<KIND>::UUID.into(),
        "device".into(),
        None,
    );
    action.apply_settings(&settings);
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let sender = CommandSender::new(tx, "plugin");
    action
        .handle(
            event("willAppear", json!({"settings":settings})),
            sender.clone(),
        )
        .await
        .unwrap();
    settle(&worker).await;
    (worker, action, sender, rx, state)
}

#[tokio::test]
async fn sdk_combo_rotation_uses_both_steps_and_disappearance_stops_rendering() {
    let mut settings = Settings::default();
    settings.channels[1].gain_step = 0.5;
    let (worker, mut action, sender, mut output, state) =
        setup::<1>(serde_json::to_value(&settings).unwrap()).await;
    action
        .handle(
            event("dialRotate", json!({"ticks":2,"pressed":false})),
            sender.clone(),
        )
        .await
        .unwrap();
    action
        .handle(
            event("dialRotate", json!({"ticks":3,"pressed":true})),
            sender.clone(),
        )
        .await
        .unwrap();
    settle(&worker).await;
    {
        let state = state.lock().unwrap();
        assert_eq!(state.channels[&settings.channels[0].target].gain, 6.0);
        assert_eq!(state.channels[&settings.channels[1].target].gain, 1.5);
    }
    action
        .handle(event("willDisappear", json!({})), sender)
        .await
        .unwrap();
    while output.try_recv().is_ok() {}
    settle(&worker).await;
    assert!(
        output.try_recv().is_err(),
        "disappeared action emitted commands"
    );
}

#[tokio::test]
async fn gain_keys_adjust_set_mute_sync_and_reject_offline_input() {
    use streamdeck_voicemeeter::settings::{GainAppearance, GainOperation};
    let mut settings = Settings::default();
    settings.gain_button.appearance = GainAppearance::Horizontal;
    settings.gain_button.delta = 72.0;
    let (worker, mut action, sender, mut output, state) =
        setup::<3>(serde_json::to_value(&settings).unwrap()).await;
    action
        .handle(event("keyDown", json!({})), sender.clone())
        .await
        .unwrap();
    action
        .handle(event("keyUp", json!({})), sender.clone())
        .await
        .unwrap();
    settle(&worker).await;
    assert_eq!(
        state.lock().unwrap().channels[&settings.channels[0].target].gain,
        12.0
    );
    let writes = state.lock().unwrap().writes.len();
    settle(&worker).await;
    assert_eq!(
        state.lock().unwrap().writes.len(),
        writes,
        "holding must not repeat"
    );
    settings.gain_button.delta = -72.0;
    action
        .handle(
            event("didReceiveSettings", json!({"settings": settings})),
            sender.clone(),
        )
        .await
        .unwrap();
    action
        .handle(event("keyDown", json!({})), sender.clone())
        .await
        .unwrap();
    settle(&worker).await;
    assert_eq!(
        state.lock().unwrap().channels[&settings.channels[0].target].gain,
        -60.0
    );
    settings.gain_button.operation = GainOperation::Set;
    settings.gain_button.gain = -4.5;
    action
        .handle(
            event("didReceiveSettings", json!({"settings": settings})),
            sender.clone(),
        )
        .await
        .unwrap();
    action
        .handle(event("keyDown", json!({})), sender.clone())
        .await
        .unwrap();
    settle(&worker).await;
    assert_eq!(
        state.lock().unwrap().channels[&settings.channels[0].target].gain,
        -4.5
    );
    settings.gain_button.operation = GainOperation::Mute;
    action
        .handle(
            event("didReceiveSettings", json!({"settings": settings})),
            sender.clone(),
        )
        .await
        .unwrap();
    action
        .handle(event("keyDown", json!({})), sender.clone())
        .await
        .unwrap();
    settle(&worker).await;
    assert!(state.lock().unwrap().channels[&settings.channels[0].target].mute);
    while output.try_recv().is_ok() {}
    state
        .lock()
        .unwrap()
        .channels
        .get_mut(&settings.channels[0].target)
        .unwrap()
        .mute = false;
    settle(&worker).await;
    let mut images = 0;
    while let Ok(message) = output.try_recv() {
        let message: Value = serde_json::from_str(&message).unwrap();
        if message["event"] == "setImage" {
            images += 1;
        }
        assert_ne!(message["event"], "setFeedbackLayout");
    }
    assert!(images > 0, "external changes must update the key image");
    state.lock().unwrap().unavailable = true;
    wait_connected(&worker, false).await;
    let writes = state.lock().unwrap().writes.len();
    action
        .handle(event("keyDown", json!({})), sender.clone())
        .await
        .unwrap();
    settle(&worker).await;
    state.lock().unwrap().unavailable = false;
    wait_connected(&worker, true).await;
    settle(&worker).await;
    assert_eq!(state.lock().unwrap().writes.len(), writes);
    action
        .handle(event("willDisappear", json!({})), sender.clone())
        .await
        .unwrap();
    while output.try_recv().is_ok() {}
    action
        .handle(event("keyDown", json!({})), sender)
        .await
        .unwrap();
    settle(&worker).await;
    assert_eq!(state.lock().unwrap().writes.len(), writes);
    assert!(output.try_recv().is_err());
}

#[tokio::test]
async fn sdk_invalid_settings_never_fall_back_to_strip_zero() {
    let (worker, mut action, sender, mut output, state) =
        setup::<0>(json!({"channels":"invalid"})).await;
    action
        .handle(event("dialDown", json!({})), sender.clone())
        .await
        .unwrap();
    settle(&worker).await;
    assert!(state.lock().unwrap().writes.is_empty());
    let messages: Vec<Value> = std::iter::from_fn(|| output.try_recv().ok())
        .map(|s| serde_json::from_str(&s).unwrap())
        .collect();
    assert!(
        messages
            .iter()
            .any(|m| m["payload"]["error"]["code"] == "invalidSettings")
    );
    action
        .handle(
            event(
                "didReceiveSettings",
                json!({"settings":Settings::default()}),
            ),
            sender.clone(),
        )
        .await
        .unwrap();
    action
        .handle(event("dialDown", json!({})), sender.clone())
        .await
        .unwrap();
    settle(&worker).await;
    assert_eq!(state.lock().unwrap().writes.len(), 1);
    action
        .handle(
            event(
                "didReceiveSettings",
                json!({"settings":{"channels":"invalid"}}),
            ),
            sender.clone(),
        )
        .await
        .unwrap();
    action
        .handle(event("dialDown", json!({})), sender)
        .await
        .unwrap();
    settle(&worker).await;
    assert_eq!(state.lock().unwrap().writes.len(), 1);
}

#[tokio::test]
async fn sdk_macro_settings_change_and_drop_release_original_push() {
    let (worker, mut action, sender, _output, state) = setup::<2>(json!({})).await;
    action
        .handle(event("keyDown", json!({})), sender.clone())
        .await
        .unwrap();
    settle(&worker).await;
    let mut next = Settings::default();
    next.macro_button.logical_id = 4;
    action
        .handle(
            event("didReceiveSettings", json!({"settings":next})),
            sender.clone(),
        )
        .await
        .unwrap();
    action
        .handle(event("keyDown", json!({})), sender)
        .await
        .unwrap();
    settle(&worker).await;
    drop(action);
    settle(&worker).await;
    assert_eq!(
        state.lock().unwrap().writes,
        vec![
            "macro:0:true",
            "macro:0:false",
            "macro:4:true",
            "macro:4:false"
        ]
    );
}

#[tokio::test]
async fn sdk_shared_push_remains_on_until_last_action_disappears() {
    let (worker, mut first, sender, _output, state) = setup::<2>(json!({})).await;
    let mut second = TypedInstance::<DeckAction<2>>::new(Arc::new(worker.client.clone()));
    second.attach(
        "second".into(),
        DeckAction::<2>::UUID.into(),
        "device".into(),
        None,
    );
    second.apply_settings(&json!({}));
    second
        .handle(event("willAppear", json!({"settings":{}})), sender.clone())
        .await
        .unwrap();
    settle(&worker).await;
    first
        .handle(event("keyDown", json!({})), sender.clone())
        .await
        .unwrap();
    second
        .handle(event("keyDown", json!({})), sender.clone())
        .await
        .unwrap();
    first
        .handle(event("keyUp", json!({})), sender.clone())
        .await
        .unwrap();
    settle(&worker).await;
    assert_eq!(state.lock().unwrap().writes, vec!["macro:0:true"]);
    second
        .handle(event("willDisappear", json!({})), sender)
        .await
        .unwrap();
    settle(&worker).await;
    assert_eq!(
        state.lock().unwrap().writes,
        vec!["macro:0:true", "macro:0:false"]
    );
}

#[tokio::test]
async fn worker_shutdown_releases_held_push_before_backend_drop() {
    let (worker, mut action, sender, _output, state) = setup::<2>(json!({})).await;
    action
        .handle(event("keyDown", json!({})), sender)
        .await
        .unwrap();
    settle(&worker).await;
    assert_eq!(state.lock().unwrap().writes, vec!["macro:0:true"]);
    // Keep the SDK action/client alive: shutdown must not depend on their drop.
    drop(worker);
    let expected = vec!["macro:0:true".to_owned(), "macro:0:false".to_owned()];
    {
        let state = state.lock().unwrap();
        assert_eq!(state.writes, expected);
        assert_eq!(state.writes_at_drop.as_ref(), Some(&expected));
        assert_eq!(state.writes_at_flush.as_ref(), Some(&expected));
    }
    drop(action);
    assert_eq!(state.lock().unwrap().writes, expected);
}

#[tokio::test]
async fn sdk_rapid_gain_events_accumulate_and_clamp_before_reversing() {
    let settings = Settings::default();
    let target = settings.channels[0].target;
    let (worker, mut action, sender, _output, state) = setup::<0>(json!({})).await;
    for ticks in [-1, -1, -1, -1, 1, 1] {
        action
            .handle(
                event("dialRotate", json!({"ticks":ticks,"pressed":false})),
                sender.clone(),
            )
            .await
            .unwrap();
    }
    settle(&worker).await;
    assert_eq!(state.lock().unwrap().channels[&target].gain, -6.0);
    for (ticks, expected) in [(100, 12.0), (-1, 9.0), (-100, -60.0), (1, -57.0)] {
        action
            .handle(
                event("dialRotate", json!({"ticks":ticks,"pressed":false})),
                sender.clone(),
            )
            .await
            .unwrap();
        settle(&worker).await;
        assert_eq!(state.lock().unwrap().channels[&target].gain, expected);
    }
}

#[tokio::test]
async fn macro_page_reentry_does_not_flash_offline_but_real_outage_does() {
    use base64::Engine as _;
    let (worker, mut action, sender, mut output, state) = setup::<2>(json!({})).await;
    let images = |output: &mut tokio::sync::mpsc::UnboundedReceiver<String>| {
        std::iter::from_fn(|| output.try_recv().ok())
            .filter_map(|text| {
                let message: Value = serde_json::from_str(&text).unwrap();
                if message["event"] != "setImage" {
                    return None;
                }
                let image = message["payload"]["image"].as_str().unwrap();
                Some(
                    String::from_utf8(
                        base64::engine::general_purpose::STANDARD
                            .decode(image.split_once(',').unwrap().1)
                            .unwrap(),
                    )
                    .unwrap(),
                )
            })
            .collect::<Vec<_>>()
    };
    let first = images(&mut output);
    assert!(!first.is_empty());
    assert!(first.iter().all(|svg| !svg.contains("M8 136L136 8")));
    action
        .handle(event("willDisappear", json!({})), sender.clone())
        .await
        .unwrap();
    settle(&worker).await;
    assert!(worker.client.snapshots.borrow().macros.is_empty());
    action
        .handle(event("willAppear", json!({"settings":{}})), sender)
        .await
        .unwrap();
    settle(&worker).await;
    let reentered = images(&mut output);
    assert!(!reentered.is_empty());
    assert!(reentered.iter().all(|svg| !svg.contains("M8 136L136 8")));
    state.lock().unwrap().unavailable = true;
    wait_connected(&worker, false).await;
    settle(&worker).await;
    assert!(
        images(&mut output)
            .iter()
            .any(|svg| svg.contains("M8 136L136 8"))
    );
}
