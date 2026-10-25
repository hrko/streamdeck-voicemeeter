//! A shared 288x144 rail cropped to one 144px key. Geometry is based on the
//! owner's 72px reference: 14px rail, 3px outline, round caps and a ring thumb.
use crate::{
    engine::Snapshot,
    render::{self, Frame, IconCache},
    settings::{GainAppearance, GainOperation, Settings},
};
use std::fmt::Write;

pub fn position(gain: f32, vertical: bool) -> f32 {
    // The whole pair covers the same -60..+12 dB range as the existing dial.
    let start = if vertical { 36.5 } else { 34.5 };
    start + 215.5 * ((gain.clamp(-60.0, 12.0) + 60.0) / 72.0)
}

pub fn is_fader(settings: &Settings) -> bool {
    settings.gain_button.operation == GainOperation::Adjust
        && settings.gain_button.appearance != GainAppearance::Static
}

fn label(snapshot: &Snapshot, settings: &Settings) -> String {
    let target = settings.channels[0].target;
    snapshot
        .channels
        .iter()
        .find(|(key, _)| *key == target)
        .map(|(_, channel)| channel.label.as_str())
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| format!("{:?} {}", target.kind, target.index))
}

/// None restores the host's user-entered title when automatic labels are off.
pub fn title(snapshot: &Snapshot, settings: &Settings) -> Option<String> {
    (!is_fader(settings) && settings.gain_button.show_label).then(|| label(snapshot, settings))
}

fn pair_transform(vertical: bool, positive: bool) -> &'static str {
    match (vertical, positive) {
        (true, true) => "translate(0 288) rotate(-90)",
        (true, false) => "translate(0 144) rotate(-90)",
        (false, true) => "translate(-144 0)",
        (false, false) => "translate(0 0)",
    }
}

pub fn frame(
    snapshot: &Snapshot,
    settings: &Settings,
    cache: &mut IconCache,
    language: &str,
) -> Frame {
    let b = &settings.gain_button;
    let target = settings.channels[0].target;
    let channel = snapshot
        .channels
        .iter()
        .find(|(key, _)| *key == target)
        .map(|(_, value)| value);
    let muted = channel.is_some_and(|value| value.mute);
    let bg = render::escape(if muted {
        &b.muted_background
    } else {
        &b.background
    });
    let positive = b.highlight_positive && channel.is_some_and(|value| value.gain > 0.0);
    let bar = render::escape(if positive {
        &b.positive_color
    } else {
        &b.bar_color
    });
    let thumb = render::escape(if positive {
        &b.positive_color
    } else {
        &b.thumb_color
    });
    let mut body = format!(r#"<rect width="144" height="144" fill="{bg}"/>"#);
    let mut missing = false;
    let fader = is_fader(settings);
    let vertical = fader && b.appearance == GainAppearance::Vertical;
    if fader {
        let transform = pair_transform(vertical, b.delta > 0.0);
        let offset = if b.show_meter {
            if b.swap_meter { -24 } else { 24 }
        } else {
            0
        };
        let start = position(-60.0, vertical);
        let end = position(12.0, vertical);
        let _ = write!(
            body,
            r#"<g transform="{transform}"><g transform="translate(0 {offset})"><path d="M{start} 72H{end}" fill="none" stroke="{bar}" stroke-width="28" stroke-linecap="round"/><path d="M{start} 72H{end}" fill="none" stroke="{bg}" stroke-width="16" stroke-linecap="round"/>"#
        );
        if let Some(channel) = channel {
            let x = position(channel.gain, vertical);
            let _ = write!(
                body,
                r#"<path d="M{start} 72H{x}" fill="none" stroke="{bar}" stroke-width="28" stroke-linecap="round"/><circle cx="{x}" cy="72" r="26" fill="{bg}"/><circle cx="{x}" cy="72" r="17" fill="{bg}" stroke="{thumb}" stroke-width="6"/>"#
            );
        }
        body.push_str("</g></g>");
    } else {
        let icon = if muted { &b.muted_icon } else { &b.icon };
        body.push_str(&render::place_icon(
            cache,
            icon,
            36.0,
            36.0,
            72.0,
            &mut missing,
        ));
        let badge = match b.operation {
            GainOperation::Adjust => {
                if b.delta > 0.0 {
                    "+".into()
                } else {
                    "−".into()
                }
            }
            GainOperation::Set => format!("→ {}", crate::numbers::gain(b.gain, language)),
            GainOperation::Mute => String::new(),
        };
        let (x, y, size, anchor) = if b.operation == GainOperation::Set {
            (72, 108, 17, "middle")
        } else {
            (120, 84, 32, "middle")
        };
        let _ = write!(
            body,
            r##"<text x="{x}" y="{y}" text-anchor="{anchor}" font-family="Arial" font-size="{size}" font-weight="bold" fill="#ffffff">{}</text>"##,
            render::escape(&badge)
        );
    }
    body.push_str(r#"<defs><clipPath id="label"><rect x="6" y="0" width="132" height="24"/></clipPath></defs>"#);
    if fader && b.show_label {
        let label = label(snapshot, settings);
        let text = format!(
            r##"<text x="72" y="19" text-anchor="middle" font-family="Segoe UI, sans-serif" font-size="17" fill="#ffffff" clip-path="url(#label)">{}</text>"##,
            render::escape(&label)
        );
        if vertical {
            let _ = write!(
                body,
                r#"<g transform="translate(110 144) rotate(-90)">{text}</g>"#
            );
        } else {
            body.push_str(&text);
        }
    }
    if fader && b.show_db {
        let value = channel
            .map(|c| format!("{} dB", crate::numbers::gain(c.gain, language)))
            .unwrap_or_else(|| "—".into());
        if vertical {
            let _ = write!(
                body,
                r##"<g transform="translate(0 144) rotate(-90)"><text x="72" y="16" text-anchor="middle" font-family="Segoe UI, sans-serif" font-size="17" fill="#ffffff">{}</text></g>"##,
                render::escape(&value)
            );
        } else {
            let _ = write!(
                body,
                r##"<text x="72" y="132" text-anchor="middle" font-family="Segoe UI, sans-serif" font-size="18" fill="#ffffff">{}</text>"##,
                render::escape(&value)
            );
        }
    }
    if fader && b.show_meter {
        // Match the rail's outer rounded-cap extents, not just its centerline.
        let meter_start = position(-60.0, vertical) - 14.0;
        let meter_width = position(12.0, vertical) + 14.0 - meter_start;
        let levels = channel.map(|c| c.levels).unwrap_or([-120.0; 2]);
        let peaks = snapshot
            .meter_peaks
            .iter()
            .find(|(key, _)| *key == target)
            .map(|(_, peaks)| *peaks)
            .unwrap_or(levels);
        let transform = pair_transform(vertical, b.delta > 0.0);
        let top = if b.swap_meter { 98 } else { 30 };
        let _ = write!(
            body,
            r#"<g transform="{transform}"><rect x="{meter_start}" y="{}" width="{meter_width}" height="20" fill="{}"/>"#,
            top - 2,
            render::escape(&b.meter_background)
        );
        const CELLS: usize = 40;
        let pitch = meter_width / CELLS as f32;
        let index = |db: f32| -> usize {
            if db.is_finite() {
                (((db + 60.0) / 72.0).clamp(0.0, 1.0) * CELLS as f32).round() as usize
            } else {
                0
            }
        };
        for side in 0..2 {
            let current = index(levels[side]);
            let peak = index(peaks[side]);
            for cell in 0..CELLS {
                // The one-cell notch preserves the existing current/decay marker.
                let active = cell < peak && !(current != peak && cell == current);
                let color = if !active {
                    &b.meter_inactive
                } else if cell < index(-24.0) {
                    if muted {
                        &b.meter_muted_low
                    } else {
                        &b.meter_low
                    }
                } else if cell < index(0.0) {
                    if muted {
                        &b.meter_muted_mid
                    } else {
                        &b.meter_mid
                    }
                } else if muted {
                    &b.meter_muted_high
                } else {
                    &b.meter_high
                };
                let _ = write!(
                    body,
                    r#"<rect x="{}" y="{}" width="{}" height="6" fill="{}"/>"#,
                    meter_start + cell as f32 * pitch + 1.0,
                    top + side * 10,
                    pitch - 2.0,
                    render::escape(color)
                );
            }
        }
        body.push_str("</g>");
    }

    if channel.is_none() {
        body.push_str(r##"<path d="M8 136L136 8" stroke="#ffb1b1" stroke-width="4"/>"##);
    }
    Frame {
        svg: format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="144" height="144" viewBox="0 0 144 144">{body}</svg>"#
        ),
        missing_glyph: missing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::ChannelSnapshot;

    #[test]
    fn complementary_halves_share_position_and_ignore_step_magnitude() {
        assert_eq!(position(-60.0, false), 34.5);
        assert_eq!(position(12.0, false), 250.0);
        assert_eq!(position(-60.0, true), 36.5);
        assert_eq!(position(12.0, true), 252.0);
        let mut s = Settings::default();
        s.gain_button.appearance = GainAppearance::Horizontal;
        let snapshot = Snapshot {
            channels: vec![(
                s.channels[0].target,
                ChannelSnapshot {
                    gain: -24.0,
                    ..Default::default()
                },
            )],
            ..Default::default()
        };
        let mut cache = IconCache::default();
        for (appearance, delta, transform) in [
            (GainAppearance::Horizontal, -1.0, "translate(0 0)"),
            (GainAppearance::Horizontal, 3.0, "translate(-144 0)"),
            (
                GainAppearance::Vertical,
                -0.5,
                "translate(0 144) rotate(-90)",
            ),
            (
                GainAppearance::Vertical,
                6.0,
                "translate(0 288) rotate(-90)",
            ),
        ] {
            s.gain_button.appearance = appearance;
            s.gain_button.delta = delta;
            let svg = frame(&snapshot, &s, &mut cache, "en").svg;
            assert!(svg.contains(transform));
            assert!(svg.contains(r#"r="26""#));
            assert!(svg.contains(r#"r="17""#));
            assert!(svg.contains(r#"stroke-width="28""#));
        }
    }

    #[test]
    fn cell_meter_uses_independent_background_and_muted_zone_colors() {
        let mut settings = Settings::default();
        settings.gain_button.appearance = GainAppearance::Horizontal;
        settings.gain_button.show_meter = true;
        let mut snapshot = Snapshot {
            channels: vec![(
                settings.channels[0].target,
                ChannelSnapshot {
                    gain: 0.0,
                    levels: [12.0, 12.0],
                    ..Default::default()
                },
            )],
            ..Default::default()
        };
        let mut cache = IconCache::default();
        settings.gain_button.meter_background = "#123456".into();
        let normal = frame(&snapshot, &settings, &mut cache, "en").svg;
        assert_eq!(normal.matches("height=\"6\"").count(), 80);
        assert!(normal.contains("#123456"));
        for color in ["#85adb9", "#1efe5b", "#fa0000"] {
            assert!(normal.contains(color));
        }
        snapshot.channels[0].1.mute = true;
        let muted = frame(&snapshot, &settings, &mut cache, "en").svg;
        for color in ["#394547", "#384b34", "#583328"] {
            assert!(muted.contains(color));
        }
        for color in ["#85adb9", "#1efe5b", "#fa0000"] {
            assert!(!muted.contains(color));
        }
        assert!(muted.contains("#123456"));
        snapshot.channels[0].1.levels = [-120.0; 2];
        let silence = frame(&snapshot, &settings, &mut cache, "en").svg;
        assert_eq!(silence.matches("fill=\"#191b1b\"").count(), 80);
        settings.gain_button.meter_background = "invalid".into();
        assert!(settings.validate().is_err());
    }

    #[test]
    fn static_styles_use_native_titles_and_ignore_saved_fader_details() {
        let mut settings = Settings::default();
        settings.gain_button.show_label = true;
        let snapshot = Snapshot {
            channels: vec![(
                settings.channels[0].target,
                ChannelSnapshot {
                    label: "Voice <&>".into(),
                    ..Default::default()
                },
            )],
            ..Default::default()
        };
        let mut cache = IconCache::default();
        for operation in [
            GainOperation::Adjust,
            GainOperation::Set,
            GainOperation::Mute,
        ] {
            settings.gain_button.operation = operation;
            for appearance in [
                GainAppearance::Static,
                GainAppearance::Horizontal,
                GainAppearance::Vertical,
            ] {
                settings.gain_button.appearance = appearance;
                if is_fader(&settings) {
                    continue;
                }
                settings.gain_button.show_meter = false;
                settings.gain_button.show_db = false;
                let plain = frame(&snapshot, &settings, &mut cache, "en").svg;
                settings.gain_button.show_meter = true;
                settings.gain_button.show_db = true;
                assert_eq!(plain, frame(&snapshot, &settings, &mut cache, "en").svg);
                assert!(!plain.contains("Voice"));
                assert_eq!(title(&snapshot, &settings).as_deref(), Some("Voice <&>"));
            }
        }
        settings.gain_button.show_label = false;
        assert_eq!(title(&snapshot, &settings), None);
        settings.gain_button.show_label = true;
        settings.gain_button.operation = GainOperation::Adjust;
        settings.gain_button.appearance = GainAppearance::Horizontal;
        assert_eq!(title(&snapshot, &settings), None);
    }

    #[test]
    fn optional_details_escape_labels_and_highlight_only_positive_gain() {
        let mut s = Settings::default();
        s.gain_button.appearance = GainAppearance::Vertical;
        s.gain_button.highlight_positive = true;
        s.gain_button.show_label = true;
        s.gain_button.show_db = true;
        s.gain_button.show_meter = true;
        let mut snapshot = Snapshot {
            channels: vec![(
                s.channels[0].target,
                ChannelSnapshot {
                    gain: 6.0,
                    label: "Voice <&>".into(),
                    levels: [-6.0, -18.0],
                    ..Default::default()
                },
            )],
            ..Default::default()
        };
        let mut cache = IconCache::default();
        let svg = frame(&snapshot, &s, &mut cache, "en").svg;
        assert!(svg.contains("Voice &lt;&amp;&gt;"));
        assert!(svg.contains("6.0 dB"));
        assert!(svg.contains("#ff6347"));
        assert_eq!(svg.matches("<svg ").count(), 1);
        snapshot.channels[0].1.gain = 0.0;
        assert!(
            !frame(&snapshot, &s, &mut cache, "en")
                .svg
                .contains("#ff6347")
        );
        snapshot.channels.clear();
        assert!(
            frame(&snapshot, &s, &mut cache, "en")
                .svg
                .contains("M8 136L136 8")
        );
    }
}
