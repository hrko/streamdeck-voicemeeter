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
    channels: BTreeMap<Target, ChannelSnapshot>,
    macros: BTreeMap<u8, bool>,
    writes: Vec<String>,
}
struct Fake(Arc<Mutex<State>>);
impl Backend for Fake {
    fn connect(&mut self) -> anyhow::Result<Option<Edition>> {
        Ok(Some(Edition::Potato))
    }
    fn poll(&mut self) -> anyhow::Result<()> {
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
