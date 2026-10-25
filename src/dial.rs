//! Go-compatible native dial layout and SVG components. Text remains native so
//! Stream Deck supplies exactly the same font, alignment and overflow behavior.
use crate::{
    backend::ChannelSnapshot,
    domain::{ActionKind, Edition},
    engine::Snapshot,
    render::{self, IconCache},
    settings::{DialMeterStyle, DialPalette, DialStyle, Settings, Target, TargetKind},
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
        let style = &settings.dial.styles[if settings.dial.shared { 0 } else { index }];
        let colors = &style.palette;
        let channel = snapshot
            .channels
            .iter()
            .find(|(target, _)| *target == config.target)
            .map(|(_, c)| c);
        let label = channel
            .map(|c| c.label.clone())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("{:?} {}", config.target.kind, config.target.index));
        let muted = channel.is_some_and(|c| c.mute);
        let positive = colors.highlight_positive && channel.is_some_and(|c| c.gain > 0.0);
        payload.insert(format!("channelLabel{suffix}"), serde_json::json!({"value":label,"enabled":style.show_label,"color": if positive {&style.palette.dial_positive_label_color} else {&style.palette.dial_label_color}}));
        let bg = if muted {
            &colors.muted_background
        } else {
            &colors.background
        };
        let background_width = if combo { 100 } else { 200 };
        let opacity = if muted {
            style.palette.muted_background_opacity
        } else {
            style.palette.background_opacity
        };
        payload.insert(format!("background{suffix}"), serde_json::json!({
            "value":render::data_url(&svg(background_width,100,&format!(r#"<rect width="{background_width}" height="100" fill="{bg}"/>"#))),
            "enabled":opacity != 0,
            "opacity":f32::from(opacity)/100.0,
        }));
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
        payload.insert(format!("gainValue{suffix}"), serde_json::json!({"value":gain,"enabled":style.show_gain,"color":if positive {&style.palette.dial_positive_gain_color} else {&style.palette.dial_gain_color}}));
        let size = if combo { 20 } else { 48 };
        let opacity = if combo { "0.501961" } else { "0.705882" };
        let icon = if style.show_icon {
            render::place_icon(cache, &config.icon, 0.0, 0.0, size as f32, &mut missing)
        } else {
            String::new()
        };
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
        let meter = meter_styled(
            width,
            if combo { 1 } else { 3 },
            levels,
            peaks,
            colors,
            muted,
            style.meter_notch,
            style,
        );
        let fader = fader_styled(
            width,
            channel.map(|c| c.gain).unwrap_or(-60.0),
            colors,
            &style.palette.empty_bar_color,
        );
        let status = channel
            .zip(snapshot.edition)
            .map(|(c, edition)| status_styled(edition, config.target, c, style))
            .unwrap_or_else(|| svg(36, 24, ""));
        for (key, image) in [
            ("icon", icon),
            ("levelMeter", meter),
            ("gainSlider", fader),
            ("status", status),
        ] {
            payload.insert(
                format!("{key}{suffix}"),
                serde_json::json!({"value":render::data_url(&image),"enabled":match key {"icon"=>style.show_icon,"levelMeter"=>style.show_meter,"gainSlider"=>style.show_bar,_=>true}}),
            );
        }
    }
    (Value::Object(payload), missing)
}

#[allow(clippy::too_many_arguments)]
fn meter_styled(
    width: usize,
    _padding_right: usize,
    levels: [f32; 2],
    peaks: [f32; 2],
    colors: &DialPalette,
    muted: bool,
    notch: bool,
    style: &DialStyle,
) -> String {
    let inner = (width - 4) as f32;
    let cells = (width - 4) / 2;
    let index = |db: f32| -> usize {
        if db.is_finite() {
            (((db + 60.0) / 72.0).clamp(0.0, 1.0) * cells as f32).round() as usize
        } else {
            0
        }
    };
    // Match the fader's outer rim, including alpha. The optional inner track is separate.
    let mut body = format!(
        r#"<rect width="{width}" height="5" rx="2.5" fill="black" fill-opacity="0.623529"/><rect x="2" y="1" width="{inner}" height="3" rx="1.5" fill="{}" fill-opacity="{}"/>"#,
        style.palette.empty_bar_color,
        f32::from(style.palette.meter_track_opacity) / 100.0
    );
    for side in 0..2 {
        let current = index(levels[side]);
        let peak = index(peaks[side]);
        for cell in 0..cells {
            let active = cell < peak && !(notch && current != peak && cell == current);
            let color = if !active {
                &colors.meter_inactive
            } else if cell < index(-24.0) {
                if muted {
                    &colors.meter_muted_low
                } else {
                    &colors.meter_low
                }
            } else if cell < index(0.0) {
                if muted {
                    &colors.meter_muted_mid
                } else {
                    &colors.meter_mid
                }
            } else if muted {
                &colors.meter_muted_high
            } else {
                &colors.meter_high
            };
            let continuous = style.meter_style == DialMeterStyle::Continuous;
            if continuous && !active {
                continue;
            }
            // Keep each cell on a whole pixel. Subpixel gaps blend into the lit pixel
            // and look like a darker continuous line at native dial resolution.
            let gap = if continuous { 0.0 } else { 1.0 };
            let x = 2.0 + cell as f32 * 2.0;
            let y = 1.0 + side as f32 * 2.0 - if continuous { 0.2 } else { 0.0 };
            let height = if continuous { 1.4 } else { 1.0 };
            let opacity = if active { 1.0 } else { 0.28 };
            let _ = write!(
                body,
                r#"<rect x="{x}" y="{y}" width="{}" height="{height}" fill="{color}" fill-opacity="{opacity}"/>"#,
                2.0 - gap
            );
        }
    }
    svg(width, 5, &body)
}

fn fader_styled(width: usize, gain: f32, colors: &DialPalette, empty: &str) -> String {
    let inner = width - 4;
    let amount = if gain.is_finite() {
        ((gain + 60.0) / 72.0).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let filled = inner as f32 * amount;
    let color = if colors.highlight_positive && gain > 0.0 {
        &colors.positive_color
    } else {
        &colors.bar_color
    };
    let mut body = format!(
        r##"<rect width="{width}" height="12" rx="6" fill="black" fill-opacity="0.623529"/><rect x="2" y="2" width="{inner}" height="8" rx="4" fill="{}"/>"##,
        empty
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

fn status_styled(
    edition: Edition,
    target: Target,
    channel: &ChannelSnapshot,
    style: &DialStyle,
) -> String {
    let mut body = String::new();
    let outline_color = &style.palette.indicator_outline_color;
    let off_color = &style.palette.indicator_background;
    let off_opacity = f32::from(style.palette.indicator_opacity) / 100.0;
    let flag = |key: &str| channel.flags.get(key).copied().unwrap_or(false);
    let (physical, _, physical_buses, buses) = edition.counts();
    let mut squares = vec![(channel.mute, "#f66051", style.show_mute)];
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
            if !style.show_routing {
                continue;
            }
            for (i, enabled) in flags.into_iter().enumerate() {
                let x = 2.0 + i as f32 * 7.0;
                if !enabled && style.palette.indicator_outline {
                    let _ = write!(
                        body,
                        r##"<rect x="{}" y="{}" width="4" height="4" rx="1.5" fill="{off_color}" fill-opacity="{off_opacity}" stroke="{outline_color}" stroke-width="1"/>"##,
                        x + 0.5,
                        y as f32 + 0.5
                    );
                } else {
                    let color = if enabled { "#8bdfb1" } else { off_color };
                    let _ = write!(
                        body,
                        r#"<rect x="{x}" y="{y}" width="5" height="5" rx="2" fill="{color}" fill-opacity="{}"/>"#,
                        if enabled { 1.0 } else { off_opacity }
                    );
                }
            }
        }
        squares.push((flag("Solo"), "#e8b15f", style.show_solo));
        if target.index < physical {
            squares.push((flag("Mono"), "#68e6f8", style.show_mono));
            if edition == Edition::Potato {
                squares.push((flag("EQ"), "#296ffd", style.show_eq));
            }
        } else if edition != Edition::Basic {
            squares.push((
                flag("MC") || flag("K"),
                if flag("K") { "#68e6f8" } else { "#f66051" },
                style.show_mono,
            ));
        }
    } else if edition != Edition::Basic {
        squares.push((flag("EQ"), "#296ffd", style.show_eq));
        squares.push((flag("Mono"), "#68e6f8", style.show_mono));
    }
    for (i, (enabled, color, visible)) in squares.into_iter().enumerate() {
        if !visible {
            continue;
        }
        if !enabled && style.palette.indicator_outline {
            let _ = write!(
                body,
                r##"<rect x="{}" y="15.5" width="6" height="6" rx="1" fill="{off_color}" fill-opacity="{off_opacity}" stroke="{outline_color}" stroke-width="1"/>"##,
                2.5 + i as f32 * 9.0
            );
        } else {
            let color = if enabled {
                if color == "#296ffd" { "#72a9ff" } else { color }
            } else {
                off_color
            };
            let _ = write!(
                body,
                r#"<rect x="{}" y="15" width="7" height="7" rx="1.5" fill="{color}" fill-opacity="{}"/>"#,
                2 + i * 9,
                if enabled { 1.0 } else { off_opacity }
            );
        }
    }
    svg(36, 24, &body)
}

#[cfg(test)]
fn status(edition: Edition, target: Target, channel: &ChannelSnapshot) -> String {
    status_styled(edition, target, channel, &DialStyle::default())
}
#[cfg(test)]
fn fader(width: usize, gain: f32) -> String {
    fader_styled(width, gain, &DialPalette::default(), "#2c3d4d")
}
#[cfg(test)]
fn meter(width: usize, padding: usize, levels: [f32; 2], peaks: [f32; 2]) -> String {
    meter_styled(
        width,
        padding,
        levels,
        peaks,
        &DialPalette::default(),
        false,
        true,
        &DialStyle::default(),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn meter_modes_and_track_preserve_level_bounds_and_mute_colors() {
        for width in [84, 108] {
            let mut style = DialStyle::default();
            let a = meter_styled(
                width,
                0,
                [-12.0, -24.0],
                [-6.0, -18.0],
                &style.palette,
                false,
                false,
                &style,
            );
            style.palette.meter_track_opacity = 100;
            let track = meter_styled(
                width,
                0,
                [-12.0, -24.0],
                [-6.0, -18.0],
                &style.palette,
                false,
                false,
                &style,
            );
            // Changing track opacity affects only the track, not any meter cells.
            assert_eq!(
                a.replacen("fill-opacity=\"0\"", "fill-opacity=\"1\"", 1),
                track
            );
            style.meter_style = DialMeterStyle::Continuous;
            let b = meter_styled(
                width,
                0,
                [-12.0, -24.0],
                [-6.0, -18.0],
                &style.palette,
                false,
                false,
                &style,
            );
            assert!(!b.contains(&style.palette.meter_inactive));
            assert!(b.contains("x=\"2\" y=\"0.8\""));
            let muted = meter_styled(
                width,
                0,
                [12.0; 2],
                [12.0; 2],
                &style.palette,
                true,
                false,
                &style,
            );
            assert!(muted.contains(&style.palette.meter_muted_high));
            assert!(!muted.contains(&format!("fill=\"{}\"", style.palette.meter_high)));
            assert!(muted.contains(&format!("x=\"{}\" y=\"0.8\" width=\"2\"", width - 4)));
        }
    }

    #[test]
    fn indicator_fill_opacity_does_not_dim_active_flags_and_migrates_old_settings() {
        let settings: Settings=serde_json::from_value(serde_json::json!({"dial":{"styles":[{"indicatorOutline":true,"emptyBarColor":"#123456"},{}]}})).unwrap();
        assert!(settings.dial.styles[0].palette.indicator_outline);
        assert_eq!(settings.dial.styles[0].palette.empty_bar_color, "#123456");
        let mut style = DialStyle::default();
        style.palette.indicator_background = "#123456".into();
        style.palette.indicator_opacity = 37;
        let channel = ChannelSnapshot {
            mute: true,
            ..Default::default()
        };
        let image = status_styled(Edition::Potato, Target::default(), &channel, &style);
        assert!(image.contains("fill=\"#123456\" fill-opacity=\"0.37\""));
        assert!(image.contains("fill=\"#f66051\" fill-opacity=\"1\""));
        style.palette.indicator_outline = true;
        style.palette.indicator_opacity = 0;
        let image = status_styled(Edition::Potato, Target::default(), &channel, &style);
        assert!(image.contains("fill-opacity=\"0\" stroke=\"#91a4b5\""));
    }

    #[test]
    fn outline_toggle_retains_routing_and_status_geometry() {
        let mut style = DialStyle::default();
        let channel = ChannelSnapshot::default();
        style.palette.indicator_outline = true;
        let outlined = status_styled(Edition::Potato, Target::default(), &channel, &style);
        style.palette.indicator_outline = false;
        let plain = status_styled(Edition::Potato, Target::default(), &channel, &style);
        assert!(outlined.contains("width=\"4\" height=\"4\" rx=\"1.5\""));
        assert!(outlined.contains("width=\"6\" height=\"6\" rx=\"1\""));
        assert!(!plain.contains("stroke="));
        assert_eq!(
            plain.matches("width=\"5\" height=\"5\" rx=\"2\"").count(),
            8
        );
        assert_eq!(
            outlined.matches("<rect").count(),
            plain.matches("<rect").count()
        );
    }

    #[test]
    fn opacity_migration_background_extent_and_label_colors() {
        use base64::Engine;
        let fresh: Settings = serde_json::from_value(serde_json::json!({})).unwrap();
        assert_eq!(fresh.dial.styles[0].palette.background_opacity, 0);
        let old: Settings = serde_json::from_value(serde_json::json!({"schemaVersion":1})).unwrap();
        assert_eq!(old.dial.styles[0].palette.background_opacity, 100);
        let mut settings = old;
        settings.dial.styles[0].palette.background_opacity = 40;
        settings.dial.styles[0].palette.label_color = "#000000".into();
        settings.dial.styles[0].palette.dial_label_color = "#abcdef".into();
        for kind in [ActionKind::Gain, ActionKind::Combo] {
            let (p, _) = feedback(
                &Snapshot::default(),
                &settings,
                kind,
                &mut IconCache::default(),
                "en",
            );
            assert!((p["background"]["opacity"].as_f64().unwrap() - 0.4).abs() < 0.0001);
            let img = p["background"]["value"].as_str().unwrap();
            let svg = String::from_utf8(
                base64::engine::general_purpose::STANDARD
                    .decode(img.split_once(',').unwrap().1)
                    .unwrap(),
            )
            .unwrap();
            assert!(svg.contains(if kind == ActionKind::Gain {
                "<rect width=\"200\" height=\"100\""
            } else {
                "<rect width=\"100\" height=\"100\""
            }));
            assert!(!svg.contains('%'));
            assert_eq!(p["channelLabel"]["color"], "#abcdef");
            if kind == ActionKind::Combo {
                assert_eq!(p["channelLabel1"]["color"], "#abcdef");
            }
        }
    }

    #[test]
    fn appearance_toggles_colors_and_combo_sharing_are_independent_of_channels() {
        use base64::Engine;
        let image = |p: &Value, key: &str| -> String {
            let value = p[key]["value"].as_str().unwrap();
            String::from_utf8(
                base64::engine::general_purpose::STANDARD
                    .decode(value.split_once(',').unwrap().1)
                    .unwrap(),
            )
            .unwrap()
        };
        let mut settings = Settings::default();
        let snapshot = Snapshot {
            edition: Some(Edition::Potato),
            channels: settings
                .channels
                .iter()
                .map(|c| {
                    (
                        c.target,
                        ChannelSnapshot {
                            gain: 6.0,
                            mute: true,
                            levels: [12.0; 2],
                            ..Default::default()
                        },
                    )
                })
                .collect(),
            ..Default::default()
        };
        let style = &mut settings.dial.styles[0];
        style.show_label = false;
        style.show_icon = false;
        style.show_meter = false;
        style.show_mute = false;
        style.show_routing = false;
        style.show_solo = false;
        style.show_mono = false;
        style.show_eq = false;
        style.palette.meter_muted_high = "#123456".into();
        style.palette.dial_positive_gain_color = "#abcdef".into();
        let (shared, _) = feedback(
            &snapshot,
            &settings,
            ActionKind::Combo,
            &mut IconCache::default(),
            "en",
        );
        for suffix in ["", "1"] {
            assert_eq!(shared[format!("channelLabel{suffix}")]["enabled"], false);
            assert_eq!(shared[format!("icon{suffix}")]["enabled"], false);
            assert_eq!(shared[format!("levelMeter{suffix}")]["enabled"], false);
            assert_eq!(shared[format!("gainValue{suffix}")]["color"], "#abcdef");
            assert!(image(&shared, &format!("levelMeter{suffix}")).contains("#123456"));
            assert!(!image(&shared, &format!("status{suffix}")).contains("<rect"));
        }
        settings.dial.shared = false;
        let (split, _) = feedback(
            &snapshot,
            &settings,
            ActionKind::Combo,
            &mut IconCache::default(),
            "en",
        );
        assert_eq!(split["channelLabel"]["enabled"], false);
        assert_eq!(split["channelLabel1"]["enabled"], true);
        assert_eq!(split["gainValue1"]["color"], "#ffffff");
        assert_eq!(split["channelLabel1"]["value"], "Bus 0");
    }

    #[test]
    fn hidden_status_flags_do_not_shift_remaining_slots() {
        let style = DialStyle {
            show_mute: false,
            show_solo: false,
            palette: DialPalette {
                indicator_outline: true,
                ..Default::default()
            },
            ..Default::default()
        };
        let channel = ChannelSnapshot::default();
        let svg = status_styled(Edition::Potato, Target::default(), &channel, &style);
        assert!(!svg.contains("x=\"2\" y=\"15\""));
        assert!(svg.contains("x=\"20.5\" y=\"15.5\""));
        assert!(svg.contains("x=\"29.5\" y=\"15.5\""));
    }

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
        assert!(on.contains(r##"x="20" y="15" width="7" height="7" rx="1.5" fill="#68e6f8""##));
        channel.flags.clear();
        channel.flags.insert("MC".into(), true);
        assert!(
            status(Edition::Potato, target, &channel)
                .contains(r##"x="20" y="15" width="7" height="7" rx="1.5" fill="#f66051""##)
        );
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
            assert!(
                payload["channelLabel"]["value"]
                    .as_str()
                    .unwrap()
                    .starts_with("Strip")
            );
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
        assert_eq!(gain["channelLabel"]["value"], "<script>&日本語");
        assert_eq!(gain["gainValue"]["value"], "-12.5 dB");
        let (combo, _) = feedback(
            &snapshot,
            &settings,
            ActionKind::Combo,
            &mut IconCache::default(),
            "en",
        );
        assert_eq!(combo["gainValue"]["value"], "-12.5");
        assert_eq!(combo["gainValue1"]["value"], "—");
        #[cfg(windows)]
        {
            let (localized, _) = feedback(
                &snapshot,
                &settings,
                ActionKind::Gain,
                &mut IconCache::default(),
                "de-DE",
            );
            assert_eq!(localized["gainValue"]["value"], "-12,5 dB");
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
        assert_eq!(potato.matches("width=\"5\" height=\"5\"").count(), 8);
        assert_eq!(potato.matches("<rect").count(), 12);
        assert!(potato.contains(r##"x="2" y="15" width="7" height="7" rx="1.5" fill="#f66051""##));
        let banana = status(Edition::Banana, physical, &channel);
        assert_eq!(banana.matches("width=\"5\" height=\"5\"").count(), 5);
        assert!(!banana.contains("y=\"0.5\""));
        assert_eq!(banana.matches("<rect").count(), 8);
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
