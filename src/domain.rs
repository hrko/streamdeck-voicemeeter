use crate::settings::{ButtonMode, Settings, Target, TargetKind};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum Edition {
    Basic,
    Banana,
    Potato,
}

impl Edition {
    pub fn counts(self) -> (u8, u8, u8, u8) {
        // physical inputs, total strips, physical buses, total buses
        match self {
            Self::Basic => (2, 3, 1, 2),
            Self::Banana => (3, 5, 3, 5),
            Self::Potato => (5, 8, 5, 8),
        }
    }
    pub fn contains(self, target: Target) -> bool {
        let (_, strips, _, buses) = self.counts();
        target.index
            < match target.kind {
                TargetKind::Strip => strips,
                TargetKind::Bus => buses,
            }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionKind {
    Gain,
    Combo,
    Macro,
    GainButton,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Script(String),
    AdjustGain { target: Target, delta: f32 },
    SetGain { target: Target, gain: f32 },
    ToggleMute(Target),
    ToggleMacro(u8),
    SetMacro { id: u8, on: bool },
}

pub fn rotated(
    kind: ActionKind,
    settings: &Settings,
    ticks: i32,
    pressed: bool,
) -> Option<Command> {
    if matches!(kind, ActionKind::Macro | ActionKind::GainButton) || ticks == 0 {
        return None;
    }
    let index = if kind == ActionKind::Combo && settings.dial.select_channel {
        usize::from(settings.dial.selected_channel.min(1))
    } else {
        usize::from(kind == ActionKind::Combo && pressed)
    };
    let channel = &settings.channels[index];
    Some(Command::AdjustGain {
        target: channel.target,
        delta: channel.gain_step * ticks as f32,
    })
}

pub fn tapped(kind: ActionKind, settings: &Settings, x: u32) -> Option<Command> {
    match kind {
        ActionKind::Gain => Some(Command::ToggleMute(settings.channels[0].target)),
        ActionKind::Combo => Some(Command::ToggleMute(
            settings.channels[usize::from(x >= 100)].target,
        )),
        ActionKind::Macro | ActionKind::GainButton => None,
    }
}

/// Single-channel touch actions distinguish a tap from a hold. Combo keeps
/// its existing left/right mute behavior regardless of the hold flag.
pub fn touched(kind: ActionKind, settings: &Settings, x: u32, hold: bool) -> Option<Command> {
    if kind != ActionKind::Gain {
        return tapped(kind, settings, x);
    }
    use crate::settings::TouchOperation;
    let operation = if hold {
        settings.dial.hold_operation
    } else {
        settings.dial.tap_operation
    };
    let target = settings.channels[0].target;
    match operation {
        TouchOperation::Script => Some(Command::Script(if hold {
            settings.dial.hold_script.clone()
        } else {
            settings.dial.tap_script.clone()
        })),
        TouchOperation::None => None,
        TouchOperation::ToggleMute => Some(Command::ToggleMute(target)),
        TouchOperation::ResetGain => Some(Command::SetGain { target, gain: 0.0 }),
    }
}

pub fn adjusted_gain(current: f32, delta: f32) -> Option<f32> {
    if !current.is_finite() || !delta.is_finite() {
        return None;
    }
    Some((current + delta).clamp(-60.0, 12.0))
}

pub fn gain_button_command(settings: &Settings) -> Command {
    use crate::settings::GainOperation;
    let target = settings.channels[0].target;
    match settings.gain_button.operation {
        GainOperation::Adjust => Command::AdjustGain {
            target,
            delta: settings.gain_button.delta,
        },
        GainOperation::Set => Command::SetGain {
            target,
            gain: settings.gain_button.gain,
        },
        GainOperation::Mute => Command::ToggleMute(target),
    }
}

/// Tracks the original target of every accepted Push, including overlapping keys.
/// Clear on disconnect: releases must never be replayed into a different session.
#[derive(Default)]
pub struct Presses {
    by_context: BTreeMap<String, u8>,
}

impl Presses {
    pub fn down(&mut self, context: &str, settings: &Settings) -> Option<Command> {
        let m = &settings.macro_button;
        if m.mode == ButtonMode::Toggle {
            return Some(Command::ToggleMacro(m.logical_id));
        }
        if self.by_context.contains_key(context) {
            return None;
        }
        let already_pressed = self.by_context.values().any(|id| *id == m.logical_id);
        self.by_context.insert(context.into(), m.logical_id);
        (!already_pressed).then_some(Command::SetMacro {
            id: m.logical_id,
            on: true,
        })
    }

    pub fn release(&mut self, context: &str) -> Option<Command> {
        let id = self.by_context.remove(context)?;
        (!self.by_context.values().any(|other| *other == id))
            .then_some(Command::SetMacro { id, on: false })
    }

    pub fn clear(&mut self) {
        self.by_context.clear();
    }

    pub fn release_all(&mut self) -> Vec<Command> {
        let ids: std::collections::BTreeSet<_> = self.by_context.values().copied().collect();
        self.clear();
        ids.into_iter()
            .map(|id| Command::SetMacro { id, on: false })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn combo_uses_independent_steps_and_tap_boundary() {
        let mut s = Settings::default();
        s.channels[0].gain_step = 1.0;
        s.channels[1].gain_step = 0.5;
        assert_eq!(
            rotated(ActionKind::Combo, &s, -3, true),
            Some(Command::AdjustGain {
                target: s.channels[1].target,
                delta: -1.5
            })
        );
        assert_eq!(
            rotated(ActionKind::Gain, &s, 2, true),
            Some(Command::AdjustGain {
                target: s.channels[0].target,
                delta: 2.0
            })
        );
        assert_eq!(
            tapped(ActionKind::Combo, &s, 99),
            Some(Command::ToggleMute(s.channels[0].target))
        );
        assert_eq!(
            tapped(ActionKind::Combo, &s, 100),
            Some(Command::ToggleMute(s.channels[1].target))
        );
    }
    #[test]
    fn clamps_gain_and_rejects_invalid_values() {
        assert_eq!(adjusted_gain(11.0, 3.0), Some(12.0));
        assert_eq!(adjusted_gain(-59.0, -3.0), Some(-60.0));
        assert_eq!(adjusted_gain(f32::NAN, 3.0), None);
        assert_eq!(adjusted_gain(0.0, f32::INFINITY), None);
    }
    #[test]
    fn edition_bounds_do_not_fallback() {
        for edition in [Edition::Basic, Edition::Banana, Edition::Potato] {
            let (_, strips, _, buses) = edition.counts();
            for (kind, count) in [(TargetKind::Strip, strips), (TargetKind::Bus, buses)] {
                assert!(edition.contains(Target {
                    kind,
                    index: count - 1
                }));
                assert!(!edition.contains(Target { kind, index: count }));
            }
        }
    }
    #[test]
    fn push_releases_original_id_only_after_last_key() {
        let mut p = Presses::default();
        let mut s = Settings::default();
        s.macro_button.logical_id = 4;
        assert_eq!(p.down("a", &s), Some(Command::SetMacro { id: 4, on: true }));
        assert_eq!(p.down("a", &s), None);
        assert_eq!(p.down("b", &s), None);
        s.macro_button.logical_id = 7;
        assert_eq!(p.release("a"), None);
        assert_eq!(p.release("b"), Some(Command::SetMacro { id: 4, on: false }));
        assert_eq!(p.release("b"), None);
    }
    #[test]
    fn toggle_has_no_release_and_disconnect_clears_pushes() {
        let mut p = Presses::default();
        let mut s = Settings::default();
        p.down("a", &s);
        p.clear();
        assert_eq!(p.release("a"), None);
        s.macro_button.mode = ButtonMode::Toggle;
        assert_eq!(p.down("b", &s), Some(Command::ToggleMacro(0)));
        assert_eq!(p.release("b"), None);
    }
}

#[cfg(test)]
mod touch_tests {
    use super::*;
    use crate::settings::TouchOperation;
    #[test]
    fn gestures_have_independent_defaults_and_operations() {
        let mut s = Settings::default();
        let target = s.channels[0].target;
        assert_eq!(
            touched(ActionKind::Gain, &s, 50, false),
            Some(Command::ToggleMute(target))
        );
        assert_eq!(
            touched(ActionKind::Gain, &s, 50, true),
            Some(Command::SetGain { target, gain: 0.0 })
        );
        for hold in [false, true] {
            for op in [
                TouchOperation::None,
                TouchOperation::ToggleMute,
                TouchOperation::ResetGain,
            ] {
                if hold {
                    s.dial.hold_operation = op;
                } else {
                    s.dial.tap_operation = op;
                }
                let expected = match op {
                    TouchOperation::Script => Some(Command::Script(if hold {
                        s.dial.hold_script.clone()
                    } else {
                        s.dial.tap_script.clone()
                    })),
                    TouchOperation::None => None,
                    TouchOperation::ToggleMute => Some(Command::ToggleMute(target)),
                    TouchOperation::ResetGain => Some(Command::SetGain { target, gain: 0.0 }),
                };
                assert_eq!(touched(ActionKind::Gain, &s, 50, hold), expected);
                assert_eq!(
                    touched(ActionKind::Combo, &s, 150, hold),
                    Some(Command::ToggleMute(s.channels[1].target))
                );
            }
        }
        let restored: Settings = serde_json::from_value(serde_json::json!({"dial":{}})).unwrap();
        assert_eq!(restored.dial.tap_operation, TouchOperation::ToggleMute);
        assert_eq!(restored.dial.hold_operation, TouchOperation::ResetGain);
        assert!(
            serde_json::from_value::<Settings>(
                serde_json::json!({"dial":{"tapOperation":"unknown"}})
            )
            .is_err()
        );
    }
}
