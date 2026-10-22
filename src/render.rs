use crate::settings::{IconSettings, IconStyle};
use crate::{domain::ActionKind, engine::Snapshot, settings::Settings};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fmt::Write,
};
use ttf_parser::{Face, OutlineBuilder, Tag};

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub fn data_url(svg: &str) -> String {
    format!("data:image/svg+xml;base64,{}", STANDARD.encode(svg))
}

pub struct Frame {
    pub svg: String,
    pub missing_glyph: bool,
}

fn place_icon(
    cache: &mut IconCache,
    settings: &IconSettings,
    x: f32,
    y: f32,
    size: f32,
    missing: &mut bool,
) -> String {
    match cache.get(settings) {
        Ok(svg) => svg.replacen(
            "viewBox=",
            &format!("x=\"{x}\" y=\"{y}\" width=\"{size}\" height=\"{size}\" viewBox="),
            1,
        ),
        Err(_) => {
            *missing = true;
            format!(
                r##"<rect x="{x}" y="{y}" width="{size}" height="{size}" fill="none" stroke="#ffb1b1" stroke-width="2"/>"##
            )
        }
    }
}

pub fn frame(
    snapshot: &Snapshot,
    settings: &Settings,
    kind: ActionKind,
    cache: &mut IconCache,
) -> Frame {
    let mut missing = false;
    if kind == ActionKind::Macro {
        let state = snapshot.macros.get(&settings.macro_button.logical_id);
        let on = state.copied().unwrap_or(false);
        let icon = if on {
            &settings.macro_button.icon_on
        } else {
            &settings.macro_button.icon_off
        };
        let color = if on {
            &settings.macro_button.color_on
        } else {
            &settings.macro_button.color_off
        };
        let path = place_icon(cache, icon, 12.0, 12.0, 120.0, &mut missing);
        let offline = if state.is_none() {
            r##"<path d="M8 136L136 8" stroke="#ffb1b1" stroke-width="4"/>"##
        } else {
            ""
        };
        return Frame {
            svg: format!(
                r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 144 144"><rect width="144" height="144" fill="{}"/>{path}{offline}</svg>"##,
                escape(color)
            ),
            missing_glyph: missing,
        };
    }
    let count = if kind == ActionKind::Combo { 2 } else { 1 };
    let width = 200.0 / count as f32;
    let mut body = String::new();
    for index in 0..count {
        let settings = &settings.channels[index];
        let channel = snapshot
            .channels
            .iter()
            .find(|(target, _)| *target == settings.target)
            .map(|(_, value)| value);
        let x = index as f32 * width;
        let fallback = format!("{:?} {}", settings.target.kind, settings.target.index);
        let label = channel
            .map(|c| c.label.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or(&fallback);
        // textLength prevents long user labels spilling into the adjacent channel.
        let label_size = (width - 8.0) / label.chars().count().max(1) as f32;
        let font_size = label_size.clamp(7.0, 13.0);
        let _ = write!(
            body,
            r##"<svg x="{x}" width="{width}" height="100" viewBox="0 0 {width} 100"><text x="4" y="15" font-size="{font_size}">{}</text>"##,
            escape(label)
        );
        body.push_str(&place_icon(
            cache,
            &settings.icon,
            4.0,
            21.0,
            28.0,
            &mut missing,
        ));
        let gain = channel
            .map(|c| format!("{:.1}", c.gain))
            .unwrap_or_else(|| "—".into());
        let _ = write!(
            body,
            r##"<text x="36" y="40" font-size="16">{gain}</text>"##
        );
        if let Some(channel) = channel {
            let meter_width = width - 8.0;
            for (side, level) in channel.levels.iter().enumerate() {
                let amount = if level.is_finite() {
                    ((*level + 60.0) / 60.0).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let y = 52 + side * 7;
                let color = if *level > -3.0 {
                    "#ff5959"
                } else if *level > -12.0 {
                    "#ffc44d"
                } else {
                    "#60d394"
                };
                let _ = write!(
                    body,
                    r##"<rect x="4" y="{y}" width="{meter_width}" height="4" fill="#424242"/><rect x="4" y="{y}" width="{}" height="4" fill="{color}"/>"##,
                    amount * meter_width
                );
            }
            let mut flags = vec![("M", channel.mute)];
            flags.extend(
                channel
                    .flags
                    .iter()
                    .map(|(name, enabled)| (name.as_str(), *enabled)),
            );
            let columns = if count == 2 { 5 } else { 8 };
            for (n, (name, enabled)) in flags.into_iter().enumerate() {
                let label = match name {
                    "Solo" => "S",
                    "Mono" => "MO",
                    other => other,
                };
                let color = if enabled { "#ffffff" } else { "#777777" };
                let px = 4.0 + (n % columns) as f32 * (width - 8.0) / columns as f32;
                let py = 77 + (n / columns) * 11;
                let _ = write!(
                    body,
                    r##"<text x="{px}" y="{py}" font-size="9" fill="{color}">{}</text>"##,
                    escape(label)
                );
            }
        } else {
            body.push_str(r##"<path d="M8 60h40" stroke="#ffb1b1" stroke-width="2"/>"##);
        }
        body.push_str("</svg>");
    }
    Frame {
        svg: format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100" fill="#ffffff" font-family="Arial, sans-serif"><rect width="200" height="100" fill="#202020"/>{body}</svg>"##
        ),
        missing_glyph: missing,
    }
}

pub fn font(style: IconStyle) -> &'static [u8] {
    match style {
        IconStyle::Outlined => include_bytes!("../assets/fonts/MaterialSymbolsOutlined.ttf"),
        IconStyle::Rounded => include_bytes!("../assets/fonts/MaterialSymbolsRounded.ttf"),
        IconStyle::Sharp => include_bytes!("../assets/fonts/MaterialSymbolsSharp.ttf"),
    }
}

#[derive(Default)]
struct Path(String);
impl OutlineBuilder for Path {
    fn move_to(&mut self, x: f32, y: f32) {
        let _ = write!(self.0, "M{x} {y}");
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let _ = write!(self.0, "L{x} {y}");
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let _ = write!(self.0, "Q{x1} {y1} {x} {y}");
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let _ = write!(self.0, "C{x1} {y1} {x2} {y2} {x} {y}");
    }
    fn close(&mut self) {
        self.0.push('Z');
    }
}

pub fn icon_svg(settings: &IconSettings) -> anyhow::Result<String> {
    let mut face = Face::parse(font(settings.style), 0)?;
    for (axis, value) in [
        (b"FILL", if settings.fill { 1.0 } else { 0.0 }),
        (b"wght", settings.weight as f32),
        (b"GRAD", settings.grade as f32),
        (b"opsz", settings.optical_size as f32),
    ] {
        anyhow::ensure!(
            face.set_variation(Tag::from_bytes(axis), value).is_some(),
            "invalidFontAxes"
        );
    }
    let character = u32::from_str_radix(&settings.code_point, 16)
        .ok()
        .and_then(char::from_u32)
        .ok_or_else(|| anyhow::anyhow!("invalidCodePoint"))?;
    let glyph = face
        .glyph_index(character)
        .ok_or_else(|| anyhow::anyhow!("missingGlyph"))?;
    let mut path = Path::default();
    let bounds = face
        .outline_glyph(glyph, &mut path)
        .ok_or_else(|| anyhow::anyhow!("missingGlyph"))?;
    let extent = (bounds.width().max(bounds.height())) as f32;
    anyhow::ensure!(extent > 0.0, "missingGlyph");
    let scale = 88.0 / extent;
    let x = 50.0 - (bounds.x_min as f32 + bounds.x_max as f32) * scale / 2.0;
    let y = 50.0 + (bounds.y_min as f32 + bounds.y_max as f32) * scale / 2.0;
    Ok(format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><path fill="#ffffff" transform="translate({x} {y}) scale({scale} -{scale})" d="{}"/></svg>"##,
        path.0
    ))
}

/// Bounded FIFO cache: icon paths depend on all five appearance settings.
#[derive(Default)]
pub struct IconCache {
    values: BTreeMap<String, String>,
    order: VecDeque<String>,
}
impl IconCache {
    pub fn get(&mut self, settings: &IconSettings) -> anyhow::Result<&str> {
        let key = serde_json::to_string(settings)?;
        if !self.values.contains_key(&key) {
            let svg = icon_svg(settings)?;
            if self.order.len() >= 256
                && let Some(oldest) = self.order.pop_front()
            {
                self.values.remove(&oldest);
            }
            self.values.insert(key.clone(), svg);
            self.order.push_back(key.clone());
        }
        Ok(&self.values[&key])
    }
}

pub fn glyph_catalog() -> anyhow::Result<BTreeMap<String, BTreeSet<u32>>> {
    let mut result = BTreeMap::new();
    for (name, style) in [
        ("outlined", IconStyle::Outlined),
        ("rounded", IconStyle::Rounded),
        ("sharp", IconStyle::Sharp),
    ] {
        let face = Face::parse(font(style), 0)?;
        let mut points = BTreeSet::new();
        if let Some(cmap) = face.tables().cmap {
            for subtable in cmap.subtables {
                if subtable.is_unicode() {
                    subtable.codepoints(|point| {
                        if char::from_u32(point)
                            .and_then(|c| face.glyph_index(c))
                            .is_some()
                        {
                            points.insert(point);
                        }
                    });
                }
            }
        }
        result.insert(name.into(), points);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn channel_frames_escape_user_labels_and_keep_stereo_and_combo_values() {
        use crate::backend::ChannelSnapshot;
        let settings = Settings::default();
        let snapshot = Snapshot {
            channels: vec![
                (
                    settings.channels[0].target,
                    ChannelSnapshot {
                        label: "<script>&\"日本語".into(),
                        gain: -12.5,
                        levels: [-20.0, -6.0],
                        mute: true,
                        ..Default::default()
                    },
                ),
                (
                    settings.channels[1].target,
                    ChannelSnapshot {
                        label: "Right".into(),
                        gain: 3.0,
                        levels: [-120.0, 0.0],
                        ..Default::default()
                    },
                ),
            ],
            ..Default::default()
        };
        let output = frame(
            &snapshot,
            &settings,
            ActionKind::Combo,
            &mut IconCache::default(),
        );
        assert!(!output.missing_glyph);
        assert!(!output.svg.contains("<script>"));
        assert!(output.svg.contains("&lt;script&gt;&amp;&quot;日本語"));
        assert!(output.svg.contains(">-12.5</text>"));
        assert!(output.svg.contains(">3.0</text>"));
        assert!(output.svg.contains("#ffc44d"));
        assert!(output.svg.contains("#ff5959"));
        assert_eq!(
            output.svg.matches("height=\"4\" fill=\"#424242\"").count(),
            4
        );
    }
    #[test]
    fn macro_frames_follow_external_state_and_missing_icons_have_fallbacks() {
        let mut settings = Settings::default();
        let mut snapshot = Snapshot::default();
        let mut cache = IconCache::default();
        snapshot.macros.insert(0, true);
        let on = frame(&snapshot, &settings, ActionKind::Macro, &mut cache);
        assert!(on.svg.contains(&settings.macro_button.color_on));
        snapshot.macros.insert(0, false);
        let off = frame(&snapshot, &settings, ActionKind::Macro, &mut cache);
        assert!(off.svg.contains(&settings.macro_button.color_off));
        assert_ne!(on.svg, off.svg);
        settings.macro_button.icon_off.code_point = "10ffff".into();
        assert!(frame(&snapshot, &settings, ActionKind::Macro, &mut cache).missing_glyph);
        snapshot.macros.clear();
        assert!(
            frame(&snapshot, &settings, ActionKind::Macro, &mut cache)
                .svg
                .contains("M8 136L136 8")
        );
    }
    #[test]
    fn all_styles_and_variable_axes_affect_outline() {
        let baseline = IconSettings::default();
        let original = icon_svg(&baseline).unwrap();
        for style in [IconStyle::Outlined, IconStyle::Rounded, IconStyle::Sharp] {
            let settings = IconSettings {
                style,
                ..baseline.clone()
            };
            assert!(icon_svg(&settings).unwrap().contains("<path"));
        }
        for settings in [
            IconSettings {
                fill: true,
                ..baseline.clone()
            },
            IconSettings {
                weight: 700,
                ..baseline.clone()
            },
            IconSettings {
                grade: 200,
                ..baseline.clone()
            },
            IconSettings {
                optical_size: 20,
                ..baseline.clone()
            },
        ] {
            assert_ne!(icon_svg(&settings).unwrap(), original);
        }
    }
    #[test]
    fn missing_glyph_is_explicit_and_cache_separates_settings() {
        let settings = IconSettings::default();
        let mut cache = IconCache::default();
        let first = cache.get(&settings).unwrap().to_owned();
        assert_eq!(cache.get(&settings).unwrap(), first);
        let other = IconSettings {
            fill: true,
            ..settings.clone()
        };
        assert_ne!(cache.get(&other).unwrap(), first);
        assert_eq!(cache.values.len(), 2);
        assert!(
            cache
                .get(&IconSettings {
                    code_point: "10ffff".into(),
                    ..settings
                })
                .unwrap_err()
                .to_string()
                .contains("missingGlyph")
        );
        assert!(glyph_catalog().unwrap()["rounded"].contains(&0xe050));
    }
}
