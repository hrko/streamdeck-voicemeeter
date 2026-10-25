use std::{fmt::Write, sync::OnceLock};
use ttf_parser::{Face, OutlineBuilder};
#[derive(Default)]
struct Outline {
    contours: Vec<Vec<(f32, f32)>>,
}
impl OutlineBuilder for Outline {
    fn move_to(&mut self, x: f32, y: f32) {
        self.contours.push(vec![(x, y)]);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.contours.last_mut().unwrap().push((x, y));
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (a, b) = *self.contours.last().unwrap().last().unwrap();
        for i in 1..=16 {
            let t = i as f32 / 16.0;
            let u = 1.0 - t;
            self.line_to(
                u * u * a + 2.0 * u * t * x1 + t * t * x,
                u * u * b + 2.0 * u * t * y1 + t * t * y,
            );
        }
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let (a, b) = *self.contours.last().unwrap().last().unwrap();
        for i in 1..=16 {
            let t = i as f32 / 16.0;
            let u = 1.0 - t;
            self.line_to(
                u * u * u * a + 3.0 * u * u * t * x1 + 3.0 * u * t * t * x2 + t * t * t * x,
                u * u * u * b + 3.0 * u * u * t * y1 + 3.0 * u * t * t * y2 + t * t * t * y,
            );
        }
    }
    fn close(&mut self) {}
}
/// Clip glyph outlines numerically; no SVG masks or host text clipping needed.
pub fn clipped(text: &str, left: f32, right: f32, color: &str) -> String {
    static FONTS: OnceLock<Vec<Vec<u8>>> = OnceLock::new();
    let fonts = FONTS.get_or_init(|| {
        let root = std::env::var_os("WINDIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| "C:/Windows".into())
            .join("Fonts");
        ["seguisb.ttf", "YuGothM.ttc", "meiryo.ttc", "segoeui.ttf"]
            .iter()
            .filter_map(|name| std::fs::read(root.join(name)).ok())
            .collect()
    });
    let mut cursor = left;
    let mut path = String::new();
    for ch in text.chars() {
        let Some((face, glyph)) = fonts
            .iter()
            .filter_map(|bytes| Face::parse(bytes, 0).ok())
            .find_map(|f| f.glyph_index(ch).map(|g| (f, g)))
        else {
            continue;
        };
        let scale = 17.0 / f32::from(face.units_per_em());
        let mut outline = Outline::default();
        face.outline_glyph(glyph, &mut outline);
        for contour in outline.contours {
            let points: Vec<_> = contour
                .into_iter()
                .map(|(x, y)| (cursor + x * scale, 78.0 - y * scale))
                .collect();
            let mut clipped = Vec::new();
            if let Some(mut prev) = points.last().copied() {
                for next in points {
                    if (prev.0 <= right) != (next.0 <= right) {
                        let t = (right - prev.0) / (next.0 - prev.0);
                        clipped.push((right, prev.1 + t * (next.1 - prev.1)));
                    }
                    if next.0 <= right {
                        clipped.push(next);
                    }
                    prev = next;
                }
            }
            if clipped.len() > 2 {
                for (i, (x, y)) in clipped.into_iter().enumerate() {
                    let _ = write!(path, "{} {x:.3} {y:.3} ", if i == 0 { "M" } else { "L" });
                }
                path.push('Z');
            }
        }
        cursor += f32::from(face.glyph_hor_advance(glyph).unwrap_or(0)) * scale;
        if cursor >= right {
            break;
        }
    }
    format!(r#"<path d="{path}" fill="{color}" fill-rule="evenodd"/>"#)
}
