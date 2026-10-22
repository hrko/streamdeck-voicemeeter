use crate::settings::{IconSettings, IconStyle};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fmt::Write,
};
use ttf_parser::{Face, OutlineBuilder, Tag};

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
