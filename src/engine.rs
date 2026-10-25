use crate::{
    backend::{Backend, ChannelSnapshot},
    domain::{ActionKind, Command, Edition, Presses, adjusted_gain},
    settings::{Settings, Target},
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

#[derive(Clone)]
pub struct Subscription {
    pub kind: ActionKind,
    pub settings: Settings,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub frame_sequence: u64,
    pub frame_elapsed_ms: u64,
    pub generation: u64,
    pub edition: Option<Edition>,
    pub channels: Vec<(Target, ChannelSnapshot)>,
    pub meter_peaks: Vec<(Target, [f32; 2])>,
    pub macros: BTreeMap<u8, bool>,
    pub errors: BTreeMap<String, String>,
}

pub struct Engine<B> {
    backend: B,
    subscriptions: BTreeMap<String, Subscription>,
    presses: Presses,
    snapshot: Snapshot,
    meters: BTreeMap<Target, crate::meter::PeakMeter>,
}

impl<B: Backend> Engine<B> {
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            subscriptions: BTreeMap::new(),
            presses: Presses::default(),
            snapshot: Snapshot::default(),
            meters: BTreeMap::new(),
        }
    }

    pub fn snapshot(&self) -> &Snapshot {
        &self.snapshot
    }

    pub fn connect(&mut self) -> anyhow::Result<bool> {
        let edition = self.backend.connect()?;
        if edition != self.snapshot.edition {
            self.disconnect();
            self.snapshot.edition = edition;
        }
        Ok(edition.is_some())
    }

    pub fn disconnect(&mut self) {
        self.snapshot.generation = self.snapshot.generation.wrapping_add(1);
        self.snapshot.edition = None;
        self.snapshot.channels.clear();
        self.snapshot.meter_peaks.clear();
        self.meters.clear();
        self.snapshot.macros.clear();
        self.presses.clear();
    }

    pub fn subscribe(&mut self, context: String, subscription: Subscription) {
        self.release(&context);
        if let Err(error) = subscription.settings.validate() {
            self.subscriptions.remove(&context);
            self.snapshot.errors.insert(context, error.code);
        } else {
            self.snapshot.errors.remove(&context);
            self.subscriptions.insert(context, subscription);
        }
    }

    pub fn remove(&mut self, context: &str) {
        self.release(context);
        self.subscriptions.remove(context);
        self.snapshot.errors.remove(context);
    }

    fn apply(&mut self, command: Command) -> anyhow::Result<()> {
        let edition = self
            .snapshot
            .edition
            .ok_or_else(|| anyhow::anyhow!("disconnected"))?;
        match command {
            Command::Script(script) => {
                anyhow::ensure!(
                    !script.trim().is_empty()
                        && !script.contains('\0')
                        && script.encode_utf16().count() <= 4096,
                    "invalid script"
                );
                self.backend.execute_script(&script)
            }
            Command::SetGain { target, gain } => {
                anyhow::ensure!(edition.contains(target), "targetOutOfRange");
                anyhow::ensure!(
                    gain.is_finite() && (-60.0..=12.0).contains(&gain),
                    "invalidButtonGain"
                );
                self.backend.set_gain(target, gain)
            }
            Command::AdjustGain { target, delta } => {
                anyhow::ensure!(edition.contains(target), "targetOutOfRange");
                let current = self.backend.channel(target)?.gain;
                let gain = adjusted_gain(current, delta)
                    .ok_or_else(|| anyhow::anyhow!("invalidGainStep"))?;
                self.backend.set_gain(target, gain)
            }
            Command::ToggleMute(target) => {
                anyhow::ensure!(edition.contains(target), "targetOutOfRange");
                let mute = self.backend.channel(target)?.mute;
                self.backend.set_mute(target, !mute)
            }
            Command::ToggleMacro(id) => {
                anyhow::ensure!(id < 80, "macroOutOfRange");
                let state = self.backend.macro_state(id)?;
                self.backend.set_macro(id, !state)
            }
            Command::SetMacro { id, on } => {
                anyhow::ensure!(id < 80, "macroOutOfRange");
                self.backend.set_macro(id, on)
            }
        }
    }

    /// Reject events captured in another connection session, including queued input.
    pub fn command(&mut self, context: &str, generation: u64, command: Command) -> bool {
        if self.snapshot.edition.is_none()
            || generation != self.snapshot.generation
            || !self.subscriptions.contains_key(context)
        {
            return false;
        }
        let script = matches!(&command, Command::Script(_));
        match self.apply(command) {
            Ok(()) => {
                self.snapshot.errors.remove(context);
                true
            }
            Err(error) => {
                tracing::warn!(context, %error, "VoiceMeeter command failed");
                self.snapshot.errors.insert(
                    context.into(),
                    if script { "scriptError" } else { "apiError" }.into(),
                );
                false
            }
        }
    }

    pub fn press(&mut self, context: &str, generation: u64) {
        if self.snapshot.edition.is_none() || generation != self.snapshot.generation {
            return;
        }
        let Some(subscription) = self.subscriptions.get(context) else {
            return;
        };
        if subscription.kind != ActionKind::Macro {
            return;
        }
        if let Some(command) = self.presses.down(context, &subscription.settings)
            && !self.command(context, generation, command)
        {
            self.presses.release(context);
        }
    }

    pub fn release(&mut self, context: &str) {
        if let Some(command) = self.presses.release(context)
            && let Err(error) = self.apply(command)
        {
            tracing::warn!(context, %error, "Macro release failed");
            self.snapshot
                .errors
                .insert(context.into(), "apiError".into());
        }
    }

    pub fn poll(&mut self) -> anyhow::Result<()> {
        if let Err(error) = self.backend.poll() {
            self.disconnect();
            return Err(error);
        }
        Ok(())
    }

    /// Read each visible target once even when multiple actions refer to it.
    pub fn record_frame(&mut self, elapsed_ms: u64) {
        self.snapshot.frame_sequence += 1;
        self.snapshot.frame_elapsed_ms = elapsed_ms;
    }

    pub fn refresh(&mut self) {
        self.refresh_at(Instant::now());
    }

    fn refresh_at(&mut self, now: Instant) {
        let Some(edition) = self.snapshot.edition else {
            return;
        };
        let mut targets = BTreeSet::new();
        let mut macros = BTreeSet::new();
        for (context, sub) in &self.subscriptions {
            // Preserve command failures long enough for the PI to display them.
            // A successful command, settings update or removal clears the error.
            if self
                .snapshot
                .errors
                .get(context)
                .is_none_or(|code| code != "scriptError")
            {
                self.snapshot.errors.remove(context);
            }
            if sub.kind == ActionKind::Macro {
                macros.insert(sub.settings.macro_button.logical_id);
                continue;
            }
            let count = if sub.kind == ActionKind::Combo { 2 } else { 1 };
            for channel in &sub.settings.channels[..count] {
                if edition.contains(channel.target) {
                    targets.insert(channel.target);
                } else {
                    self.snapshot
                        .errors
                        .insert(context.clone(), "targetOutOfRange".into());
                }
            }
        }
        self.snapshot.channels.clear();
        self.snapshot.meter_peaks.clear();
        self.meters.retain(|target, _| targets.contains(target));
        self.snapshot.macros.clear();
        for target in targets {
            match self.backend.channel(target) {
                Ok(channel) => {
                    let meter = self
                        .meters
                        .entry(target)
                        .or_insert_with(|| crate::meter::PeakMeter::new(now, channel.levels));
                    self.snapshot
                        .meter_peaks
                        .push((target, meter.update(now, channel.levels)));
                    self.snapshot.channels.push((target, channel));
                }
                Err(error) => {
                    self.meters.remove(&target);
                    tracing::warn!(?target, %error, "Channel read failed");
                    for (context, sub) in &self.subscriptions {
                        let count = match sub.kind {
                            ActionKind::Gain | ActionKind::GainButton => 1,
                            ActionKind::Combo => 2,
                            ActionKind::Macro => 0,
                        };
                        if sub.settings.channels[..count]
                            .iter()
                            .any(|c| c.target == target)
                        {
                            self.snapshot
                                .errors
                                .entry(context.clone())
                                .or_insert_with(|| "apiError".into());
                        }
                    }
                }
            }
        }
        for id in macros {
            match self.backend.macro_state(id) {
                Ok(state) => {
                    self.snapshot.macros.insert(id, state);
                }
                Err(error) => {
                    tracing::warn!(id, %error, "Macro read failed");
                    for (context, sub) in &self.subscriptions {
                        if sub.kind == ActionKind::Macro
                            && sub.settings.macro_button.logical_id == id
                        {
                            self.snapshot
                                .errors
                                .insert(context.clone(), "apiError".into());
                        }
                    }
                }
            }
        }
    }

    pub fn shutdown(&mut self) {
        for command in self.presses.release_all() {
            if let Err(error) = self.apply(command) {
                tracing::warn!(%error, "Shutdown release failed");
            }
        }
        if self.snapshot.edition.is_some()
            && let Err(error) = self.backend.flush()
        {
            tracing::warn!(%error, "Shutdown write confirmation failed");
        }
        self.disconnect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Default)]
    struct Fake {
        reads: usize,
        gain: f32,
        levels: [f32; 2],
        writes: Vec<Command>,
        fail: bool,
    }
    impl Backend for Fake {
        fn connect(&mut self) -> anyhow::Result<Option<Edition>> {
            Ok(Some(Edition::Basic))
        }
        fn poll(&mut self) -> anyhow::Result<()> {
            anyhow::ensure!(!self.fail, "lost connection");
            Ok(())
        }
        fn channel(&mut self, _: Target) -> anyhow::Result<ChannelSnapshot> {
            self.reads += 1;
            anyhow::ensure!(!self.fail, "channel unavailable");
            Ok(ChannelSnapshot {
                gain: self.gain,
                levels: self.levels,
                ..Default::default()
            })
        }
        fn set_gain(&mut self, target: Target, value: f32) -> anyhow::Result<()> {
            self.gain = value;
            self.writes.push(Command::AdjustGain {
                target,
                delta: value,
            });
            Ok(())
        }
        fn set_mute(&mut self, target: Target, _: bool) -> anyhow::Result<()> {
            self.writes.push(Command::ToggleMute(target));
            Ok(())
        }
        fn macro_state(&mut self, _: u8) -> anyhow::Result<bool> {
            Ok(false)
        }
        fn set_macro(&mut self, id: u8, on: bool) -> anyhow::Result<()> {
            self.writes.push(Command::SetMacro { id, on });
            Ok(())
        }
    }
    fn setup(kind: ActionKind) -> Engine<Fake> {
        let mut e = Engine::new(Fake::default());
        e.connect().unwrap();
        for context in ["a", "b"] {
            e.subscribe(
                context.into(),
                Subscription {
                    kind,
                    settings: Settings::default(),
                },
            );
        }
        e
    }
    #[test]
    fn shared_targets_are_read_once_and_removed_actions_cannot_act() {
        let mut e = setup(ActionKind::Gain);
        e.refresh();
        assert_eq!(e.backend.reads, 1);
        e.remove("a");
        assert!(!e.command(
            "a",
            e.snapshot.generation,
            Command::ToggleMute(Target::default())
        ));
        assert!(e.backend.writes.is_empty());
    }

    #[test]
    fn absolute_gain_checks_edition_range_and_connection_generation() {
        let mut e = setup(ActionKind::GainButton);
        let target = Target::default();
        let generation = e.snapshot.generation;
        assert!(e.command(
            "a",
            generation,
            Command::SetGain {
                target,
                gain: -60.0
            }
        ));
        assert_eq!(e.backend.gain, -60.0);
        assert!(e.command("a", generation, Command::SetGain { target, gain: 12.0 }));
        for gain in [-61.0, 13.0, f32::NAN] {
            assert!(!e.command("a", generation, Command::SetGain { target, gain }));
        }
        assert!(!e.command(
            "a",
            generation,
            Command::SetGain {
                target: Target { index: 7, ..target },
                gain: 0.0
            }
        ));
        e.disconnect();
        e.connect().unwrap();
        assert!(!e.command("a", generation, Command::SetGain { target, gain: 0.0 }));
        assert_eq!(e.backend.gain, 12.0);
    }
    #[test]
    fn shared_meter_decays_once_and_resets_after_disconnect_or_removal() {
        use std::time::Duration;
        let mut e = setup(ActionKind::Gain);
        let now = Instant::now();
        e.refresh_at(now);
        e.backend.levels = [-60.0; 2];
        e.refresh_at(now + Duration::from_secs(1));
        assert_eq!(
            e.snapshot.meter_peaks,
            vec![(Target::default(), [-12.0; 2])]
        );
        assert_eq!(e.backend.reads, 2);
        assert_eq!(e.snapshot.channels[0].1.levels, [-60.0; 2]);
        e.disconnect();
        assert!(e.snapshot.meter_peaks.is_empty());
        e.connect().unwrap();
        e.refresh_at(now + Duration::from_secs(2));
        assert_eq!(e.snapshot.meter_peaks[0].1, [-60.0; 2]);
        e.remove("a");
        e.remove("b");
        e.refresh_at(now + Duration::from_secs(3));
        assert!(e.meters.is_empty());
        assert!(e.snapshot.meter_peaks.is_empty());
    }
    #[test]
    fn disconnected_and_previous_session_commands_are_discarded() {
        let mut e = setup(ActionKind::Gain);
        let generation = e.snapshot.generation;
        e.backend.fail = true;
        assert!(e.poll().is_err());
        assert!(!e.command("a", generation, Command::ToggleMute(Target::default())));
        e.backend.fail = false;
        e.connect().unwrap();
        assert!(!e.command("a", generation, Command::ToggleMute(Target::default())));
        assert!(e.backend.writes.is_empty());
    }
    #[test]
    fn settings_change_releases_original_macro_and_shutdown_releases_others() {
        let mut e = setup(ActionKind::Macro);
        let generation = e.snapshot.generation;
        e.press("a", generation);
        let mut settings = Settings::default();
        settings.macro_button.logical_id = 3;
        e.subscribe(
            "a".into(),
            Subscription {
                kind: ActionKind::Macro,
                settings,
            },
        );
        e.press("a", generation);
        e.shutdown();
        assert_eq!(
            e.backend.writes,
            vec![
                Command::SetMacro { id: 0, on: true },
                Command::SetMacro { id: 0, on: false },
                Command::SetMacro { id: 3, on: true },
                Command::SetMacro { id: 3, on: false }
            ]
        );
    }
    #[test]
    fn read_errors_reach_all_subscribers_and_clear_after_recovery() {
        let mut e = setup(ActionKind::Gain);
        e.backend.fail = true;
        e.refresh();
        assert!(e.snapshot.channels.is_empty());
        for context in ["a", "b"] {
            assert_eq!(
                e.snapshot.errors.get(context).map(String::as_str),
                Some("apiError")
            );
        }
        e.backend.fail = false;
        e.refresh();
        assert!(e.snapshot.errors.is_empty());
        assert_eq!(e.snapshot.channels.len(), 1);
    }
    #[test]
    fn rapid_gain_commands_accumulate_and_clamp() {
        let mut e = setup(ActionKind::Gain);
        for _ in 0..6 {
            assert!(e.command(
                "a",
                e.snapshot.generation,
                Command::AdjustGain {
                    target: Target::default(),
                    delta: 3.0
                }
            ));
        }
        assert_eq!(e.backend.gain, 12.0);
    }
}
