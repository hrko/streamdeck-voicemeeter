//! Visible key registry; no persistent pairing survives a page/instance lifecycle.
use crate::{
    gain_button::is_fader,
    settings::{GainAppearance, Settings},
};
use std::collections::BTreeMap;
use streamdeck_plugin::{ActionIdentity, CommandSender, Result};

struct Entry {
    identity: ActionIdentity,
    settings: Settings,
    pending: Vec<Settings>,
}
#[derive(Default)]
pub(crate) struct Pairs {
    entries: BTreeMap<String, Entry>,
}
impl Pairs {
    pub fn appear(&mut self, identity: ActionIdentity, settings: Settings) {
        self.entries.insert(
            identity.context.clone(),
            Entry {
                identity,
                settings,
                pending: Vec::new(),
            },
        );
    }
    pub fn remove(&mut self, context: &str) {
        self.entries.remove(context);
    }
    pub fn changed(
        &mut self,
        context: &str,
        settings: Settings,
        sender: &CommandSender,
    ) -> Result<Option<Settings>> {
        if let Some(source) = self.entries.get_mut(context)
            && let Some(index) = source.pending.iter().position(|value| value == &settings)
        {
            source.pending.drain(..=index);
            return Ok((source.settings != settings).then(|| source.settings.clone()));
        }
        let Some(source) = self.entries.get(context) else {
            return Ok(None);
        };
        if source.settings == settings {
            return Ok(None);
        } // host acknowledgement, not another edit
        let mut candidates = self
            .entries
            .iter()
            .filter(|(id, peer)| id.as_str() != context && paired(source, peer))
            .map(|(id, _)| id.clone());
        let peer = candidates.next().filter(|_| candidates.next().is_none());
        let compatible = settings.validate().is_ok()
            && is_fader(&settings)
            && settings.gain_button.sync_pair
            && settings.gain_button.appearance == source.settings.gain_button.appearance
            && settings.gain_button.delta.signum() == source.settings.gain_button.delta.signum();
        self.entries.get_mut(context).unwrap().settings = settings.clone();
        if compatible && let Some(peer_id) = peer {
            let peer = self.entries.get_mut(&peer_id).unwrap();
            let mut next = peer.settings.clone();
            let sign = next.gain_button.delta.signum();
            next.channels[0] = settings.channels[0].clone();
            next.gain_button = settings.gain_button.clone();
            next.gain_button.delta = settings.gain_button.delta.abs() * sign;
            if next != peer.settings {
                sender.set_settings(&peer_id, &serde_json::to_value(&next)?)?;
                // setSettings updates the PI; explicitly request a host event to
                // update the peer's SDK state, worker subscription and rendering.
                sender.get_settings(&peer_id, None)?;
                peer.pending.push(next.clone());
                if peer.pending.len() > 64 {
                    peer.pending.remove(0);
                }
                peer.settings = next;
            }
        }
        Ok(None)
    }
}
fn paired(a: &Entry, b: &Entry) -> bool {
    let (Some(ac), Some(bc)) = (a.identity.coordinates, b.identity.coordinates) else {
        return false;
    };
    let (x, y) = (&a.settings, &b.settings);
    if a.identity.device_id != b.identity.device_id
        || !is_fader(x)
        || !is_fader(y)
        || !x.gain_button.sync_pair
        || !y.gain_button.sync_pair
        || x.validate().is_err()
        || y.validate().is_err()
        || x.channels[0].target != y.channels[0].target
        || x.gain_button.appearance != y.gain_button.appearance
        || x.gain_button.delta.signum() == y.gain_button.delta.signum()
    {
        return false;
    }
    let direction = if x.gain_button.delta < 0.0 { 1 } else { -1 };
    match x.gain_button.appearance {
        GainAppearance::Horizontal => ac.row == bc.row && bc.column - ac.column == direction,
        GainAppearance::Vertical => ac.column == bc.column && bc.row - ac.row == -direction,
        GainAppearance::Static => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use streamdeck_plugin::Coordinates;
    fn entry(id: &str, column: i32, delta: f32) -> Entry {
        let mut settings = Settings::default();
        settings.gain_button.appearance = GainAppearance::Horizontal;
        settings.gain_button.delta = delta;
        Entry {
            identity: ActionIdentity {
                context: id.into(),
                action_id: "gain".into(),
                device_id: "device".into(),
                coordinates: Some(Coordinates { column, row: 0 }),
            },
            settings,
            pending: vec![],
        }
    }
    #[test]
    fn pairing_requires_visible_compatible_unambiguous_neighbors() {
        let a = entry("a", 0, -3.0);
        let mut b = entry("b", 1, 3.0);
        assert!(paired(&a, &b));
        b.identity.device_id = "other".into();
        assert!(!paired(&a, &b));
        b = entry("b", 1, 3.0);
        b.settings.channels[0].target.index = 1;
        assert!(!paired(&a, &b));
        b = entry("b", 1, 3.0);
        b.settings.gain_button.sync_pair = false;
        assert!(!paired(&a, &b));
        b = entry("b", 2, 3.0);
        assert!(!paired(&a, &b));
        b = entry("b", 1, -3.0);
        assert!(!paired(&a, &b));
        b = entry("b", 1, 3.0);
        b.identity.coordinates = None;
        assert!(!paired(&a, &b));
        b = entry("b", 1, 3.0);
        b.settings.gain_button.appearance = GainAppearance::Static;
        assert!(!paired(&a, &b));
        let mut pairs = Pairs::default();
        pairs.entries.insert("a".into(), a);
        pairs.entries.insert("b".into(), entry("b", 1, 3.0));
        pairs
            .entries
            .insert("duplicate".into(), entry("duplicate", 1, 3.0));
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let sender = CommandSender::new(tx, "plugin");
        let mut changed = pairs.entries["a"].settings.clone();
        changed.gain_button.show_db = true;
        pairs.changed("a", changed, &sender).unwrap();
        assert!(rx.try_recv().is_err());
    }
    #[test]
    fn rapid_edits_ignore_older_acknowledgements_and_disabled_peers() {
        let mut pairs = Pairs::default();
        pairs.entries.insert("a".into(), entry("a", 0, -3.0));
        pairs.entries.insert("b".into(), entry("b", 1, 3.0));
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let sender = CommandSender::new(tx, "plugin");
        let mut changed = pairs.entries["a"].settings.clone();
        changed.gain_button.show_db = true;
        pairs.changed("a", changed.clone(), &sender).unwrap();
        let older = pairs.entries["b"].settings.clone();
        changed.gain_button.show_meter = true;
        pairs.changed("a", changed, &sender).unwrap();
        let latest = pairs.entries["b"].settings.clone();
        while rx.try_recv().is_ok() {}
        assert_eq!(
            pairs.changed("b", older, &sender).unwrap(),
            Some(latest.clone())
        );
        assert!(
            pairs
                .changed("b", latest.clone(), &sender)
                .unwrap()
                .is_none()
        );
        assert!(rx.try_recv().is_err());
        let mut disabled = latest;
        disabled.gain_button.sync_pair = false;
        pairs.changed("b", disabled, &sender).unwrap();
        let mut changed = pairs.entries["a"].settings.clone();
        changed.gain_button.show_db = false;
        pairs.changed("a", changed, &sender).unwrap();
        assert!(rx.try_recv().is_err());
    }
}
