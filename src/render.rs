use crate::settings::{IconSettings, IconStyle};
use crate::{engine::Snapshot, settings::Settings};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fmt::Write,
};
use ttf_parser::{Face, OutlineBuilder, Tag};

pub(crate) fn escape(text: &str) -> String {
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

pub(crate) fn place_icon(
    cache: &mut IconCache,
    settings: &IconSettings,
    x: f32,
    y: f32,
    size: f32,
    missing: &mut bool,
) -> String {
    match cache.get(settings) {
        Ok(svg) => {
            // Stream Deck uses a native SVG renderer rather than a browser.
            // Keep one viewport and place cached outlines with group transforms.
            let inner = svg
                .split_once('>')
                .unwrap()
                .1
                .strip_suffix("</svg>")
                .unwrap();
            let scale = size / 100.0;
            format!(r#"<g transform="translate({x} {y}) scale({scale})">{inner}</g>"#)
        }
        Err(_) => {
            *missing = true;
            format!(
                r##"<rect x="{x}" y="{y}" width="{size}" height="{size}" fill="none" stroke="#ffb1b1" stroke-width="2"/>"##
            )
        }
    }
}

pub fn macro_frame(snapshot: &Snapshot, settings: &Settings, cache: &mut IconCache) -> Frame {
    let mut missing = false;
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
    // Go used a 36px symbol centered in a 72px key image.
    let path = place_icon(cache, icon, 36.0, 36.0, 72.0, &mut missing);
    let offline = if state.is_none() {
        r##"<path d="M8 136L136 8" stroke="#ffb1b1" stroke-width="4"/>"##
    } else {
        ""
    };
    Frame {
        svg: format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="144" height="144" viewBox="0 0 144 144"><rect width="144" height="144" fill="{}"/>{path}{offline}</svg>"##,
            escape(color)
        ),
        missing_glyph: missing,
    }
}
pub fn font(style: IconStyle) -> &'static [u8] {
    static OUTLINED: &[u8] = include_bytes!("../assets/fonts/MaterialSymbolsOutlined.ttf");
    static ROUNDED: &[u8] = include_bytes!("../assets/fonts/MaterialSymbolsRounded.ttf");
    static SHARP: &[u8] = include_bytes!("../assets/fonts/MaterialSymbolsSharp.ttf");
    match style {
        IconStyle::Outlined => OUTLINED,
        IconStyle::Rounded => ROUNDED,
        IconStyle::Sharp => SHARP,
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
    // Match the Go font-face size, preserving the glyph's designed bearings
    // instead of independently stretching every symbol to its bounding box.
    let scale = 100.0 / f32::from(face.units_per_em());
    let x = 0.0;
    let y = 100.0;
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
    fn macro_frames_follow_external_state_and_missing_icons_have_fallbacks() {
        let mut settings = Settings::default();
        let mut snapshot = Snapshot::default();
        let mut cache = IconCache::default();
        snapshot.macros.insert(0, true);
        let on = macro_frame(&snapshot, &settings, &mut cache);
        assert!(on.svg.contains(&settings.macro_button.color_on));
        snapshot.macros.insert(0, false);
        let off = macro_frame(&snapshot, &settings, &mut cache);
        assert!(off.svg.contains(&settings.macro_button.color_off));
        assert_ne!(on.svg, off.svg);
        settings.macro_button.icon_off.code_point = "10ffff".into();
        assert!(macro_frame(&snapshot, &settings, &mut cache).missing_glyph);
        snapshot.macros.clear();
        assert!(
            macro_frame(&snapshot, &settings, &mut cache)
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
