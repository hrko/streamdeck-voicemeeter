//! Go-compatible native dial layout and SVG components. Text remains native so
//! Stream Deck supplies exactly the same font, alignment and overflow behavior.
use crate::{
    backend::ChannelSnapshot,
    domain::{ActionKind, Edition},
    engine::Snapshot,
    render::{self, IconCache},
    settings::{Settings, Target, TargetKind},
};
use serde_json::{Map, Value};
use std::fmt::Write;

pub fn layout(kind: ActionKind) -> &'static str {
    if kind == ActionKind::Combo {
        "layouts/combo.json"
    } else {
        "layouts/gain.json"
    }
}

fn svg(width: usize, height: usize, body: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">{body}</svg>"#
    )
}

pub fn feedback(
    snapshot: &Snapshot,
    settings: &Settings,
    kind: ActionKind,
    cache: &mut IconCache,
    language: &str,
) -> (Value, bool) {
    let combo = kind == ActionKind::Combo;
    let mut payload = Map::new();
    let mut missing = false;
    for index in 0..if combo { 2 } else { 1 } {
        let suffix = if index == 0 { "" } else { "1" };
        let config = &settings.channels[index];
        let channel = snapshot
            .channels
            .iter()
            .find(|(target, _)| *target == config.target)
            .map(|(_, c)| c);
        let label = channel
            .map(|c| c.label.clone())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("{:?} {}", config.target.kind, config.target.index));
        payload.insert(format!("title{suffix}"), Value::String(label));
        let gain = channel
            .map(|c| {
                let number = crate::numbers::gain(c.gain, language);
                if combo {
                    number
                } else {
                    format!("{number} dB")
                }
            })
            .unwrap_or_else(|| "—".into());
        payload.insert(format!("gainValue{suffix}"), Value::String(gain));
        let size = if combo { 20 } else { 48 };
        let opacity = if combo { "0.501961" } else { "0.705882" };
        let icon = render::place_icon(cache, &config.icon, 0.0, 0.0, size as f32, &mut missing);
        let stroke = ttf_parser::Face::parse(render::font(config.icon.style), 0)
            .map(|face| 2.0 * f32::from(face.units_per_em()) / size as f32)
            .unwrap_or(40.0);
        let icon = svg(
            size,
            size,
            &format!(
                r#"<g stroke="black" stroke-opacity="{opacity}" stroke-width="{stroke}">{icon}</g>{icon}"#
            ),
        );
        let width = if combo { 84 } else { 108 };
        let levels = channel.map(|c| c.levels).unwrap_or([-120.0; 2]);
        let peaks = snapshot
            .meter_peaks
            .iter()
            .find(|(target, _)| *target == config.target)
            .map(|(_, p)| *p)
            .unwrap_or(levels);
        let meter = meter(width, if combo { 1 } else { 3 }, levels, peaks);
        let fader = fader(width, channel.map(|c| c.gain).unwrap_or(-60.0));
        let status = channel
            .zip(snapshot.edition)
            .map(|(c, edition)| status(edition, config.target, c))
            .unwrap_or_else(|| svg(36, 24, ""));
        for (key, image) in [
            ("icon", icon),
            ("levelMeter", meter),
            ("gainSlider", fader),
            ("status", status),
        ] {
            payload.insert(
                format!("{key}{suffix}"),
                Value::String(render::data_url(&image)),
            );
        }
    }
    (Value::Object(payload), missing)
}

fn meter(width: usize, padding_right: usize, levels: [f32; 2], peaks: [f32; 2]) -> String {
    let cells = (width - 2 - padding_right).div_ceil(2);
    let index = |db: f32| -> usize {
        if db.is_finite() {
            (((db + 60.0) / 72.0).clamp(0.0, 1.0) * cells as f32).round() as usize
        } else {
            0
        }
    };
    let mut body = format!(r#"<rect width="{width}" height="5" fill="black"/>"#);
    for side in 0..2 {
        let current = index(levels[side]);
        let peak = index(peaks[side]);
        for cell in 0..cells {
            let active = cell < peak && !(current != peak && cell == current);
            let color = match (active, cell < index(-24.0), cell < index(0.0)) {
                (true, true, _) => "#85adb9",
                (true, false, true) => "#1efe5b",
                (true, false, false) => "#fa0000",
                (false, _, true) => "#191b1b",
                (false, _, false) => "#1f1715",
            };
            let _ = write!(
                body,
                r#"<rect x="{}" y="{}" width="1" height="1" fill="{color}"/>"#,
                2 + cell * 2,
                1 + side * 2
            );
        }
    }
    svg(width, 5, &body)
}

fn fader(width: usize, gain: f32) -> String {
    let inner = width - 4;
    let amount = if gain.is_finite() {
        ((gain + 60.0) / 72.0).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let filled = inner as f32 * amount;
    let color = if gain > 0.0 { "#f8634d" } else { "#70c399" };
    let mut body = format!(
        r##"<rect width="{width}" height="12" rx="6" fill="black" fill-opacity="0.623529"/><rect x="2" y="2" width="{inner}" height="8" rx="4" fill="#2c3d4d"/>"##
    );
    if filled > 0.0 {
        let _ = write!(
            body,
            r#"<rect x="2" y="2" width="{filled}" height="8" rx="{}" fill="{color}"/>"#,
            filled.min(8.0) / 2.0
        );
    }
    svg(width, 12, &body)
}

fn status(edition: Edition, target: Target, channel: &ChannelSnapshot) -> String {
    let mut body = String::new();
    let flag = |key: &str| channel.flags.get(key).copied().unwrap_or(false);
    let (physical, _, physical_buses, buses) = edition.counts();
    let mut squares = vec![(channel.mute, "#f66051")];
    if target.kind == TargetKind::Strip {
        let a: Vec<_> = (1..=physical_buses)
            .map(|i| flag(&format!("A{i}")))
            .collect();
        let b: Vec<_> = (1..=buses - physical_buses)
            .map(|i| flag(&format!("B{i}")))
            .collect();
        let rows = if edition == Edition::Potato {
            vec![(0, a), (7, b)]
        } else {
            vec![(7, a.into_iter().chain(b).collect())]
        };
        for (y, flags) in rows {
            for (i, enabled) in flags.into_iter().enumerate() {
                let color = if enabled { "#70c399" } else { "#2c3d4d" };
                let _ = write!(
                    body,
                    r#"<circle cx="{}" cy="{}" r="2.5" fill="{color}"/>"#,
                    4.5 + i as f32 * 7.0,
                    y as f32 + 2.5
                );
            }
        }
        squares.push((flag("Solo"), "#e8b15f"));
        if target.index < physical {
            squares.push((flag("Mono"), "#68e6f8"));
            if edition == Edition::Potato {
                squares.push((flag("EQ"), "#296ffd"));
            }
        } else if edition != Edition::Basic {
            squares.push((flag("MC") || flag("K"), "#f66051"));
        }
    } else if edition != Edition::Basic {
        squares.push((flag("EQ"), "#296ffd"));
        squares.push((flag("Mono"), "#68e6f8"));
    }
    for (i, (enabled, color)) in squares.into_iter().enumerate() {
        let color = if enabled { color } else { "#2c3d4d" };
        let _ = write!(
            body,
            r#"<rect x="{}" y="15" width="7" height="7" rx="1.5" fill="{color}"/>"#,
            2 + i * 9
        );
    }
    svg(36, 24, &body)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn karaoke_lights_virtual_strip_center_indicator() {
        let target = Target {
            kind: TargetKind::Strip,
            index: 6,
        };
        let mut channel = ChannelSnapshot::default();
        let off = status(Edition::Potato, target, &channel);
        channel.flags.insert("K".into(), true);
        let on = status(Edition::Potato, target, &channel);
        assert!(!off.contains("#f66051"));
        assert!(on.contains(r##"x="20" y="15" width="7" height="7" rx="1.5" fill="#f66051""##));
        channel.flags.clear();
        channel.flags.insert("MC".into(), true);
        assert_eq!(on, status(Edition::Potato, target, &channel));
    }
    #[test]
    fn go_meter_and_fader_dimensions_and_thresholds() {
        let meter = meter(108, 3, [-60.0, 12.0], [12.0; 2]);
        assert_eq!(meter.matches("height=\"1\"").count(), 104);
        assert!(meter.contains("#fa0000"));
        assert!(meter.contains(r##"x="2" y="1" width="1" height="1" fill="#191b1b""##));
        assert_eq!(meter.matches("<svg").count(), 1);
        assert!(!fader(108, -60.0).contains("#70c399"));
        assert!(fader(108, 12.0).contains(r##"width="104" height="8" rx="4" fill="#f8634d""##));
    }
    #[test]
    fn native_text_and_all_legacy_layout_fields_are_supplied() {
        for kind in [ActionKind::Gain, ActionKind::Combo] {
            let resource = if kind == ActionKind::Gain {
                include_str!("../assets/layouts/gain.json")
            } else {
                include_str!("../assets/layouts/combo.json")
            };
            let layout: Value = serde_json::from_str(resource).unwrap();
            let (payload, missing) = feedback(
                &Snapshot::default(),
                &Settings::default(),
                kind,
                &mut IconCache::default(),
                "ja",
            );
            assert!(!missing);
            for item in layout["items"].as_array().unwrap() {
                assert!(payload.get(item["key"].as_str().unwrap()).is_some());
            }
            assert_eq!(
                payload.as_object().unwrap().len(),
                layout["items"].as_array().unwrap().len()
            );
            assert!(payload["title"].as_str().unwrap().starts_with("Strip"));
        }
    }

    #[test]
    fn labels_are_native_text_and_gain_units_match_each_layout() {
        let settings = Settings::default();
        let snapshot = Snapshot {
            edition: Some(Edition::Potato),
            channels: vec![(
                settings.channels[0].target,
                ChannelSnapshot {
                    label: "<script>&日本語".into(),
                    gain: -12.5,
                    ..Default::default()
                },
            )],
            ..Default::default()
        };
        let (gain, _) = feedback(
            &snapshot,
            &settings,
            ActionKind::Gain,
            &mut IconCache::default(),
            "en",
        );
        assert_eq!(gain["title"], "<script>&日本語");
        assert_eq!(gain["gainValue"], "-12.5 dB");
        let (combo, _) = feedback(
            &snapshot,
            &settings,
            ActionKind::Combo,
            &mut IconCache::default(),
            "en",
        );
        assert_eq!(combo["gainValue"], "-12.5");
        assert_eq!(combo["gainValue1"], "—");
        #[cfg(windows)]
        {
            let (localized, _) = feedback(
                &snapshot,
                &settings,
                ActionKind::Gain,
                &mut IconCache::default(),
                "de-DE",
            );
            assert_eq!(localized["gainValue"], "-12,5 dB");
        }
    }

    #[test]
    fn edition_status_rows_match_go_geometry() {
        let channel = ChannelSnapshot {
            mute: true,
            ..Default::default()
        };
        let physical = Target::default();
        let potato = status(Edition::Potato, physical, &channel);
        assert_eq!(potato.matches("<circle").count(), 8);
        assert_eq!(potato.matches("<rect").count(), 4);
        assert!(potato.contains(r##"x="2" y="15" width="7" height="7" rx="1.5" fill="#f66051""##));
        let banana = status(Edition::Banana, physical, &channel);
        assert_eq!(banana.matches("<circle").count(), 5);
        assert!(!banana.contains("cy=\"2.5\""));
        assert_eq!(banana.matches("<rect").count(), 3);
        let basic = status(
            Edition::Basic,
            Target {
                kind: TargetKind::Bus,
                index: 0,
            },
            &channel,
        );
        assert_eq!(basic.matches("<rect").count(), 1);
        assert!(!basic.contains("<circle"));
    }
}
