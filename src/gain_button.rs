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
    let background_opacity = f32::from(if muted {
        b.muted_background_opacity
    } else {
        b.background_opacity
    }) / 100.0;
    let mut body = format!(
        r#"<rect width="144" height="144" fill="{bg}" fill-opacity="{background_opacity}"/>"#
    );
    let mut missing = false;
    let fader = is_fader(settings);
    let vertical = fader && b.appearance == GainAppearance::Vertical;
    if fader {
        let transform = pair_transform(vertical, b.delta > 0.0);
        // H layout: 12px meter, 12px bar gap, 17px outer margins.
        // Reflect both tracks when swapped; number visibility never moves axes.
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
                r#"<path d="M{start} 72H{x}" fill="none" stroke="{bar}" stroke-width="28" stroke-linecap="round"/>"#
            );
        }
        // Only the negative half owns the label. Clip within that key and rail;
        // and filled segment; never draw into the empty rail or positive half.
        // The thumb overlays the remaining text.
        if b.fader_label && b.delta < 0.0 {
            let left = start - 6.0;
            let filled_end = channel
                .map(|value| position(value.gain, vertical))
                .unwrap_or(left);
            let width = (filled_end.min(144.0) - left).max(0.0);
            let _ = write!(
                body,
                r#"<defs><clipPath id="fader-label"><rect x="{left}" y="60" width="{width}" height="24"/></clipPath></defs><text x="{}" y="78" text-anchor="start" font-family="Segoe UI, sans-serif" font-size="17" font-weight="600" fill="{}" clip-path="url(#fader-label)">{}</text>"#,
                left,
                render::escape(if positive {
                    &b.positive_label_color
                } else {
                    &b.label_color
                }),
                render::escape(&label(snapshot, settings))
            );
            // Stream Deck's SVG renderer may ignore text clip paths. Paint the
            // empty rail over the label as well, using only basic SVG primitives.
            if filled_end < end {
                let hidden_width = end + 14.0 - filled_end;
                let _ = write!(
                    body,
                    r#"<rect x="{filled_end}" y="58" width="{hidden_width}" height="28" fill="{bg}"/><path d="M{filled_end} 72H{end}" fill="none" stroke="{bar}" stroke-width="28" stroke-linecap="round"/><path d="M{filled_end} 72H{end}" fill="none" stroke="{bg}" stroke-width="16" stroke-linecap="round"/>"#
                );
            }
        }
        if let Some(channel) = channel {
            let x = position(channel.gain, vertical);
            let (radius, clearance) = if b.show_db { (25, 34) } else { (17, 26) };
            let _ = write!(
                body,
                r#"<circle cx="{x}" cy="72" r="{clearance}" fill="{bg}"/><circle cx="{x}" cy="72" r="{radius}" fill="{bg}" stroke="{thumb}" stroke-width="6"/>"#
            );
            if b.show_db {
                let gain = if channel.gain.abs() < 0.05 {
                    0.0
                } else {
                    channel.gain
                };
                let number = crate::numbers::gain(gain, language);
                let number = if gain > 0.0 {
                    format!("+{number}")
                } else {
                    number
                };
                let rotation = if vertical { 90 } else { 0 };
                // Keep text horizontal after the vertical pair's -90 degree turn.
                let rgb = u32::from_str_radix(bg.trim_start_matches('#'), 16).unwrap_or(0);
                let brightness =
                    ((rgb >> 16) & 255) * 299 + ((rgb >> 8) & 255) * 587 + (rgb & 255) * 114;
                let ink = if brightness > 128000 {
                    "#000000"
                } else {
                    "#ffffff"
                };
                let _ = write!(
                    body,
                    r#"<defs><clipPath id="gain-number"><circle cx="{x}" cy="72" r="22"/></clipPath></defs><text x="{x}" y="76.5" transform="rotate({rotation} {x} 72)" text-anchor="middle" font-family="Consolas, monospace" font-size="13" font-weight="bold" fill="{ink}" clip-path="url(#gain-number)">{}</text>"#,
                    render::escape(&number)
                );
            }
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
        let top = if b.swap_meter { 88 } else { 36 };
        let _ = write!(
            body,
            r#"<g transform="{transform}"><rect x="{meter_start}" y="{}" width="{meter_width}" height="24" fill="{}"/>"#,
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
                // Optional notch marks the instantaneous signal below the decaying peak.
                let active = cell < peak && !(b.meter_notch && current != peak && cell == current);
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
                    r#"<rect x="{}" y="{}" width="{}" height="8" fill="{}"/>"#,
                    meter_start + cell as f32 * pitch + 1.0,
                    top + side * 12,
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
    fn key_background_respects_normal_and_muted_palette_opacity() {
        let mut settings = Settings::default();
        settings.gain_button.background_opacity = 0;
        settings.gain_button.muted_background_opacity = 35;
        let target = settings.channels[0].target;
        let mut snapshot = Snapshot {
            channels: vec![(target, ChannelSnapshot::default())],
            ..Default::default()
        };
        for appearance in [
            GainAppearance::Static,
            GainAppearance::Horizontal,
            GainAppearance::Vertical,
        ] {
            settings.gain_button.appearance = appearance;
            snapshot.channels[0].1.mute = false;
            let normal = frame(&snapshot, &settings, &mut IconCache::default(), "en").svg;
            assert!(
                normal.contains(r##"width="144" height="144" fill="#000000" fill-opacity="0""##)
            );
            snapshot.channels[0].1.mute = true;
            let muted = frame(&snapshot, &settings, &mut IconCache::default(), "en").svg;
            assert!(muted.contains(r#"fill-opacity="0.35""#));
        }
    }

    #[test]
    fn current_level_notch_is_opt_in_and_does_not_change_peak_extent() {
        let mut settings = Settings::default();
        settings.gain_button.appearance = GainAppearance::Horizontal;
        settings.gain_button.show_meter = true;
        assert!(!settings.gain_button.meter_notch);
        let target = settings.channels[0].target;
        let snapshot = Snapshot {
            channels: vec![(
                target,
                ChannelSnapshot {
                    levels: [-36.0, -18.0],
                    ..Default::default()
                },
            )],
            meter_peaks: vec![(target, [0.0, 6.0])],
            ..Default::default()
        };
        let mut cache = IconCache::default();
        let plain = frame(&snapshot, &settings, &mut cache, "en").svg;
        settings.gain_button.meter_notch = true;
        let marked = frame(&snapshot, &settings, &mut cache, "en").svg;
        assert_eq!(
            marked.matches("#1b1b1b").count(),
            plain.matches("#1b1b1b").count() + 2
        );
        assert_eq!(
            marked.matches("<rect ").count(),
            plain.matches("<rect ").count()
        );
        let mut equal = snapshot;
        equal.meter_peaks[0].1 = equal.channels[0].1.levels;
        let marked = frame(&equal, &settings, &mut cache, "en").svg;
        settings.gain_button.meter_notch = false;
        assert_eq!(marked, frame(&equal, &settings, &mut cache, "en").svg);
    }

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
    fn fader_label_only_renders_inside_negative_half_and_uses_custom_color() {
        let mut settings = Settings::default();
        settings.gain_button.fader_label = true;
        settings.gain_button.label_color = "#123456".into();
        settings.gain_button.show_meter = true;
        let snapshot = Snapshot {
            channels: vec![(
                settings.channels[0].target,
                ChannelSnapshot {
                    label: "Very long 日本語 Voice <&> channel label".into(),
                    ..Default::default()
                },
            )],
            ..Default::default()
        };
        let mut cache = IconCache::default();
        for appearance in [GainAppearance::Horizontal, GainAppearance::Vertical] {
            settings.gain_button.appearance = appearance;
            settings.gain_button.delta = -3.0;
            let negative = frame(&snapshot, &settings, &mut cache, "en").svg;
            assert!(negative.contains("clip-path=\"url(#fader-label)\""));
            assert!(negative.contains("fill=\"#123456\""));
            assert!(negative.contains("日本語 Voice &lt;&amp;&gt;"));
            settings.gain_button.delta = 3.0;
            let positive = frame(&snapshot, &settings, &mut cache, "en").svg;
            assert!(!positive.contains("fader-label"));
            assert!(!positive.contains("Very long"));
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
        assert_eq!(normal.matches("height=\"8\"").count(), 80);
        assert!(normal.contains("#123456"));
        for color in ["#a6a6a6", "#1efe5b", "#fa0000"] {
            assert!(normal.contains(color));
        }
        snapshot.channels[0].1.mute = true;
        let muted = frame(&snapshot, &settings, &mut cache, "en").svg;
        for color in ["#424242", "#384b34", "#583328"] {
            assert!(muted.contains(color));
        }
        for color in ["#a6a6a6", "#1efe5b", "#fa0000"] {
            assert!(!muted.contains(color));
        }
        assert!(muted.contains("#123456"));
        snapshot.channels[0].1.levels = [-120.0; 2];
        let silence = frame(&snapshot, &settings, &mut cache, "en").svg;
        assert_eq!(silence.matches("fill=\"#1b1b1b\"").count(), 80);
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
        s.gain_button.fader_label = true;
        s.gain_button.delta = -3.0;
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
        assert!(svg.contains(">+6.0</text>"));
        assert!(!svg.contains(" dB"));
        assert!(svg.contains("rotate(90"));
        assert!(svg.contains("#ff6347"));
        assert!(svg.contains("fill=\"#862b20\""));
        assert!(svg.contains("text-anchor=\"start\""));
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
