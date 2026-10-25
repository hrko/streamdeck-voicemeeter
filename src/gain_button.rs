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
    let fader = b.operation == GainOperation::Adjust && b.appearance != GainAppearance::Static;
    let vertical = fader && b.appearance == GainAppearance::Vertical;
    if fader {
        let transform = if vertical {
            if b.delta > 0.0 {
                "translate(0 288) rotate(-90)"
            } else {
                "translate(0 144) rotate(-90)"
            }
        } else if b.delta > 0.0 {
            "translate(-144 0)"
        } else {
            "translate(0 0)"
        };
        let start = position(-60.0, vertical);
        let end = position(12.0, vertical);
        let _ = write!(
            body,
            r#"<g transform="{transform}"><path d="M{start} 72H{end}" fill="none" stroke="{bar}" stroke-width="28" stroke-linecap="round"/><path d="M{start} 72H{end}" fill="none" stroke="{bg}" stroke-width="16" stroke-linecap="round"/>"#
        );
        if let Some(channel) = channel {
            let x = position(channel.gain, vertical);
            let _ = write!(
                body,
                r#"<path d="M{start} 72H{x}" fill="none" stroke="{bar}" stroke-width="28" stroke-linecap="round"/><circle cx="{x}" cy="72" r="26" fill="{bg}"/><circle cx="{x}" cy="72" r="17" fill="{bg}" stroke="{thumb}" stroke-width="6"/>"#
            );
        }
        body.push_str("</g>");
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
    if b.show_label {
        let label = channel
            .map(|c| c.label.as_str())
            .filter(|label| !label.is_empty())
            .map(str::to_owned)
            .unwrap_or_else(|| format!("{:?} {}", target.kind, target.index));
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
    if b.show_db {
        let value = channel
            .map(|c| format!("{} dB", crate::numbers::gain(c.gain, language)))
            .unwrap_or_else(|| "—".into());
        if vertical {
            let _ = write!(
                body,
                r##"<g transform="translate(0 144) rotate(-90)"><text x="72" y="32" text-anchor="middle" font-family="Segoe UI, sans-serif" font-size="17" fill="#ffffff">{}</text></g>"##,
                render::escape(&value)
            );
        } else {
            let _ = write!(
                body,
                r##"<text x="72" y="120" text-anchor="middle" font-family="Segoe UI, sans-serif" font-size="18" fill="#ffffff">{}</text>"##,
                render::escape(&value)
            );
        }
    }
    if b.show_meter {
        let levels = channel.map(|c| c.levels).unwrap_or([-120.0; 2]);
        let peaks = snapshot
            .meter_peaks
            .iter()
            .find(|(key, _)| *key == target)
            .map(|(_, peaks)| *peaks)
            .unwrap_or(levels);
        for (index, peak) in peaks.iter().enumerate() {
            let length = ((peak.clamp(-60.0, 12.0) + 60.0) / 72.0) * 128.0;
            let marker = ((levels[index].clamp(-60.0, 12.0) + 60.0) / 72.0) * 128.0;
            let color = if *peak > 0.0 { "#ff6347" } else { "#7bc99e" };
            let y = 130 + index * 7;
            let transform = if vertical {
                "translate(-128 144) rotate(-90)"
            } else {
                ""
            };
            let _ = write!(
                body,
                r##"<g transform="{transform}"><rect x="8" y="{y}" width="128" height="5" fill="#303030"/><rect x="8" y="{y}" width="{length}" height="5" fill="{color}"/><rect x="{}" y="{y}" width="1" height="5" fill="#ffffff"/></g>"##,
                8.0 + marker
            );
        }
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
