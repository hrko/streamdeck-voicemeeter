use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS,
)]
#[serde(rename_all = "camelCase")]
pub enum TargetKind {
    #[default]
    Strip,
    Bus,
}

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS,
)]
#[serde(default, rename_all = "camelCase")]
pub struct Target {
    pub kind: TargetKind,
    pub index: u8,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum IconStyle {
    Outlined,
    #[default]
    Rounded,
    Sharp,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct IconSettings {
    pub code_point: String,
    pub style: IconStyle,
    pub weight: u16,
    pub fill: bool,
    pub grade: i16,
    pub optical_size: u8,
}

impl Default for IconSettings {
    fn default() -> Self {
        Self {
            code_point: "e050".into(),
            style: IconStyle::Rounded,
            weight: 400,
            fill: false,
            grade: 0,
            optical_size: 48,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct ChannelSettings {
    pub target: Target,
    pub gain_step: f32,
    pub icon: IconSettings,
}

impl Default for ChannelSettings {
    fn default() -> Self {
        Self {
            target: Target::default(),
            gain_step: 3.0,
            icon: IconSettings {
                code_point: "f71a".into(),
                ..Default::default()
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum ButtonMode {
    Toggle,
    #[default]
    Push,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct MacroSettings {
    pub logical_id: u8,
    pub mode: ButtonMode,
    pub icon_on: IconSettings,
    pub icon_off: IconSettings,
    pub color_on: String,
    pub color_off: String,
}

impl Default for MacroSettings {
    fn default() -> Self {
        Self {
            logical_id: 0,
            mode: ButtonMode::Push,
            icon_on: IconSettings {
                code_point: "e837".into(),
                style: IconStyle::Outlined,
                ..Default::default()
            },
            icon_off: IconSettings {
                code_point: "e836".into(),
                style: IconStyle::Outlined,
                ..Default::default()
            },
            color_on: "#067ba2".into(),
            color_off: "#004162".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub dial: DialSettings,
    pub schema_version: u32,
    pub channels: [ChannelSettings; 2],
    pub macro_button: MacroSettings,
    pub gain_button: GainButtonSettings,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum GainOperation {
    #[default]
    Adjust,
    Set,
    Mute,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum GainAppearance {
    #[default]
    Static,
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct GainButtonSettings {
    pub operation: GainOperation,
    pub delta: f32,
    pub gain: f32,
    pub appearance: GainAppearance,
    pub icon: IconSettings,
    pub muted_icon: IconSettings,
    pub background: String,
    pub muted_background: String,
    pub bar_color: String,
    pub thumb_color: String,
    pub positive_color: String,
    pub highlight_positive: bool,
    pub show_label: bool,
    pub fader_label: bool,
    pub sync_pair: bool,
    pub label_color: String,
    pub positive_label_color: String,
    pub show_db: bool,
    pub show_meter: bool,
    pub meter_notch: bool,
    pub swap_meter: bool,
    pub empty_bar_color: String,
    pub background_opacity: u8,
    pub muted_background_opacity: u8,
    pub meter_track_opacity: u8,
    pub dial_label_color: String,
    pub dial_positive_label_color: String,
    pub dial_gain_color: String,
    pub dial_positive_gain_color: String,
    pub indicator_outline_color: String,
    pub indicator_background: String,
    pub indicator_opacity: u8,
    pub indicator_outline: bool,
    pub meter_background: String,
    pub meter_inactive: String,
    pub meter_low: String,
    pub meter_mid: String,
    pub meter_high: String,
    pub meter_muted_low: String,
    pub meter_muted_mid: String,
    pub meter_muted_high: String,
}

impl Default for GainButtonSettings {
    fn default() -> Self {
        Self {
            operation: GainOperation::Adjust,
            delta: 3.0,
            gain: 0.0,
            appearance: GainAppearance::Static,
            icon: IconSettings {
                code_point: "e31d".into(),
                ..Default::default()
            },
            muted_icon: IconSettings {
                code_point: "e02b".into(),
                ..Default::default()
            },
            background: "#000000".into(),
            muted_background: "#000000".into(),
            bar_color: "#ffffff".into(),
            thumb_color: "#ffffff".into(),
            positive_color: "#ff6347".into(),
            highlight_positive: false,
            show_label: false,
            fader_label: false,
            sync_pair: true,
            label_color: "#000000".into(),
            positive_label_color: "#862b20".into(),
            show_db: false,
            show_meter: false,
            meter_notch: false,
            swap_meter: false,
            empty_bar_color: "#808080".into(),
            background_opacity: 100,
            muted_background_opacity: 100,
            meter_track_opacity: 0,
            dial_label_color: "#ffffff".into(),
            dial_positive_label_color: "#ffffff".into(),
            dial_gain_color: "#ffffff".into(),
            dial_positive_gain_color: "#ffffff".into(),
            indicator_outline_color: "#a0a0a0".into(),
            indicator_background: "#808080".into(),
            indicator_opacity: 100,
            indicator_outline: false,
            meter_background: "#000000".into(),
            meter_inactive: "#1b1b1b".into(),
            meter_low: "#85adb9".into(),
            meter_mid: "#1efe5b".into(),
            meter_high: "#fa0000".into(),
            meter_muted_low: "#394547".into(),
            meter_muted_mid: "#384b34".into(),
            meter_muted_high: "#583328".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct DialPalette {
    pub bar_color: String,
    pub label_color: String,
    pub positive_label_color: String,
    pub thumb_color: String,
    pub background: String,
    pub muted_background: String,
    pub positive_color: String,
    pub empty_bar_color: String,
    pub background_opacity: u8,
    pub muted_background_opacity: u8,
    pub meter_track_opacity: u8,
    pub dial_label_color: String,
    pub dial_positive_label_color: String,
    pub dial_gain_color: String,
    pub dial_positive_gain_color: String,
    pub indicator_outline_color: String,
    pub indicator_background: String,
    pub indicator_opacity: u8,
    pub indicator_outline: bool,
    pub meter_background: String,
    pub meter_inactive: String,
    pub meter_low: String,
    pub meter_mid: String,
    pub meter_high: String,
    pub meter_muted_low: String,
    pub meter_muted_mid: String,
    pub meter_muted_high: String,
    pub highlight_positive: bool,
}
impl Default for DialPalette {
    fn default() -> Self {
        let b = GainButtonSettings::default();
        Self {
            bar_color: "#70c399".into(),
            label_color: "#286442".into(),
            positive_label_color: "#862b20".into(),
            thumb_color: "#70c399".into(),
            background: b.background,
            muted_background: b.muted_background,
            positive_color: "#f8634d".into(),
            empty_bar_color: "#2c3d4d".into(),
            background_opacity: 0,
            muted_background_opacity: 0,
            meter_track_opacity: 0,
            dial_label_color: "#ffffff".into(),
            dial_positive_label_color: "#ffffff".into(),
            dial_gain_color: "#ffffff".into(),
            dial_positive_gain_color: "#ffffff".into(),
            indicator_outline_color: "#91a4b5".into(),
            indicator_background: "#2c3d4d".into(),
            indicator_opacity: b.indicator_opacity,
            indicator_outline: b.indicator_outline,
            meter_background: b.meter_background,
            meter_inactive: "#191b1b".into(),
            meter_low: b.meter_low,
            meter_mid: b.meter_mid,
            meter_high: b.meter_high,
            meter_muted_low: b.meter_muted_low,
            meter_muted_mid: b.meter_muted_mid,
            meter_muted_high: b.meter_muted_high,
            highlight_positive: true,
        }
    }
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum DialMeterStyle {
    #[default]
    Cells,
    Continuous,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct DialStyle {
    pub palette: DialPalette,
    pub show_label: bool,
    pub show_gain: bool,
    pub show_icon: bool,
    pub show_meter: bool,
    pub show_bar: bool,
    pub show_mute: bool,
    pub show_solo: bool,
    pub show_routing: bool,
    pub show_mono: bool,
    pub show_eq: bool,
    pub meter_notch: bool,
    pub meter_style: DialMeterStyle,
}
impl Default for DialStyle {
    fn default() -> Self {
        Self {
            palette: DialPalette::default(),
            show_label: true,
            show_gain: true,
            show_icon: true,
            show_meter: true,
            show_bar: true,
            show_mute: true,
            show_solo: true,
            show_routing: true,
            show_mono: true,
            show_eq: true,
            meter_notch: false,
            meter_style: DialMeterStyle::Cells,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct DialSettings {
    pub shared: bool,
    pub styles: [DialStyle; 2],
}
impl Default for DialSettings {
    fn default() -> Self {
        Self {
            shared: true,
            styles: [DialStyle::default(), DialStyle::default()],
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            dial: DialSettings::default(),
            channels: [
                ChannelSettings::default(),
                ChannelSettings {
                    target: Target {
                        kind: TargetKind::Bus,
                        index: 0,
                    },
                    icon: IconSettings {
                        code_point: "f70e".into(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            ],
            macro_button: MacroSettings::default(),
            gain_button: GainButtonSettings::default(),
        }
    }
}

/// Stable wire codes. User-facing explanations belong in translation resources.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ValidationError {
    pub code: String,
    pub field: String,
}

impl ValidationError {
    fn new(code: &str, field: &str) -> Self {
        Self {
            code: code.into(),
            field: field.into(),
        }
    }
}

pub fn valid_color(value: &str) -> bool {
    value.len() == 7
        && value.starts_with('#')
        && value.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
}

impl IconSettings {
    pub fn character(&self) -> Option<char> {
        if self.code_point.is_empty()
            || self.code_point.len() > 6
            || !self.code_point.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return None;
        }
        u32::from_str_radix(&self.code_point, 16)
            .ok()
            .and_then(char::from_u32)
    }

    pub fn validate(&self, field: &str) -> Result<(), ValidationError> {
        if self.character().is_none() {
            return Err(ValidationError::new("invalidCodePoint", field));
        }
        if !(100..=700).contains(&self.weight)
            || !self.weight.is_multiple_of(100)
            || ![-25, 0, 200].contains(&self.grade)
            || ![20, 24, 40, 48].contains(&self.optical_size)
        {
            return Err(ValidationError::new("invalidFontAxes", field));
        }
        Ok(())
    }
}

impl Settings {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(ValidationError::new("unsupportedSchema", "schemaVersion"));
        }
        for (index, channel) in self.channels.iter().enumerate() {
            let field = format!("channels.{index}");
            if !channel.gain_step.is_finite()
                || channel.gain_step <= 0.0
                || channel.gain_step > 72.0
            {
                return Err(ValidationError::new(
                    "invalidGainStep",
                    &format!("{field}.gainStep"),
                ));
            }
            // Edition-specific bounds are checked by the domain before any API call.
            if channel.target.index >= 8 {
                return Err(ValidationError::new(
                    "targetOutOfRange",
                    &format!("{field}.target"),
                ));
            }
            channel.icon.validate(&format!("{field}.icon"))?;
        }
        for style in &self.dial.styles {
            if style.palette.background_opacity > 100
                || style.palette.muted_background_opacity > 100
                || style.palette.meter_track_opacity > 100
                || style.palette.indicator_opacity > 100
            {
                return Err(ValidationError::new("invalidOpacity", "dial"));
            }
            let p = &style.palette;
            for color in [
                &p.indicator_outline_color,
                &p.indicator_background,
                &p.bar_color,
                &p.label_color,
                &p.positive_label_color,
                &p.thumb_color,
                &p.background,
                &p.muted_background,
                &p.positive_color,
                &p.meter_background,
                &p.meter_inactive,
                &p.meter_low,
                &p.meter_mid,
                &p.meter_high,
                &p.meter_muted_low,
                &p.meter_muted_mid,
                &p.meter_muted_high,
                &style.palette.dial_label_color,
                &style.palette.dial_positive_label_color,
                &p.empty_bar_color,
                &style.palette.dial_gain_color,
                &style.palette.dial_positive_gain_color,
            ] {
                if !valid_color(color) {
                    return Err(ValidationError::new("invalidColor", "dial"));
                }
            }
        }
        let m = &self.macro_button;
        let b = &self.gain_button;
        if !b.delta.is_finite() || b.delta == 0.0 || b.delta.abs() > 72.0 {
            return Err(ValidationError::new(
                "invalidButtonDelta",
                "gainButton.delta",
            ));
        }
        if !b.gain.is_finite() || !(-60.0..=12.0).contains(&b.gain) {
            return Err(ValidationError::new("invalidButtonGain", "gainButton.gain"));
        }
        if b.indicator_opacity > 100
            || b.background_opacity > 100
            || b.muted_background_opacity > 100
            || b.meter_track_opacity > 100
        {
            return Err(ValidationError::new(
                "invalidOpacity",
                "gainButton.indicatorOpacity",
            ));
        }
        b.icon.validate("gainButton.icon")?;
        b.muted_icon.validate("gainButton.mutedIcon")?;
        for color in [
            &b.dial_label_color,
            &b.dial_positive_label_color,
            &b.dial_gain_color,
            &b.dial_positive_gain_color,
            &b.empty_bar_color,
            &b.indicator_outline_color,
            &b.indicator_background,
            &b.background,
            &b.muted_background,
            &b.bar_color,
            &b.label_color,
            &b.positive_label_color,
            &b.thumb_color,
            &b.positive_color,
            &b.meter_background,
            &b.meter_inactive,
            &b.meter_low,
            &b.meter_mid,
            &b.meter_high,
            &b.meter_muted_low,
            &b.meter_muted_mid,
            &b.meter_muted_high,
        ] {
            if !valid_color(color) {
                return Err(ValidationError::new("invalidColor", "gainButton"));
            }
        }
        if m.logical_id >= 80 {
            return Err(ValidationError::new(
                "macroOutOfRange",
                "macroButton.logicalId",
            ));
        }
        m.icon_on.validate("macroButton.iconOn")?;
        m.icon_off.validate("macroButton.iconOff")?;
        for (field, color) in [("colorOn", &m.color_on), ("colorOff", &m.color_off)] {
            if !valid_color(color) {
                return Err(ValidationError::new(
                    "invalidColor",
                    &format!("macroButton.{field}"),
                ));
            }
        }
        Ok(())
    }
}

pub fn typescript() -> String {
    let declarations = [
        TargetKind::decl(),
        Target::decl(),
        IconStyle::decl(),
        IconSettings::decl(),
        ChannelSettings::decl(),
        ButtonMode::decl(),
        MacroSettings::decl(),
        GainOperation::decl(),
        GainAppearance::decl(),
        GainButtonSettings::decl(),
        DialPalette::decl(),
        DialMeterStyle::decl(),
        DialStyle::decl(),
        DialSettings::decl(),
        Settings::decl(),
        ValidationError::decl(),
    ];
    format!(
        "// Generated from Rust settings. Do not edit.\n{}\n",
        declarations
            .iter()
            .map(|d| format!("export {d}"))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

// Distinguish a newly placed action ({}) from existing serialized settings.
impl<'de> Deserialize<'de> for Settings {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut value = serde_json::Value::deserialize(deserializer)?;
        if let Some(object) = value.as_object_mut()
            && !object.is_empty()
        {
            let dial = object
                .entry("dial")
                .or_insert_with(|| serde_json::json!({}));
            if let Some(dial) = dial.as_object_mut() {
                let styles = dial
                    .entry("styles")
                    .or_insert_with(|| serde_json::json!([{}, {}]));
                if let Some(styles) = styles.as_array_mut() {
                    for style in styles {
                        if let Some(style) = style.as_object_mut() {
                            for (old, new) in [
                                ("backgroundOpacity", "backgroundOpacity"),
                                ("mutedBackgroundOpacity", "mutedBackgroundOpacity"),
                                ("meterTrackOpacity", "meterTrackOpacity"),
                                ("labelColor", "dialLabelColor"),
                                ("positiveLabelColor", "dialPositiveLabelColor"),
                                ("gainColor", "dialGainColor"),
                                ("positiveGainColor", "dialPositiveGainColor"),
                            ] {
                                if let Some(v) = style.remove(old) {
                                    let palette = style
                                        .entry("palette")
                                        .or_insert_with(|| serde_json::json!({}));
                                    if let Some(palette) = palette.as_object_mut() {
                                        palette.entry(new).or_insert(v);
                                    }
                                }
                            }
                            if let Some(empty) = style.remove("emptyBarColor") {
                                let palette = style
                                    .entry("palette")
                                    .or_insert_with(|| serde_json::json!({}));
                                if let Some(palette) = palette.as_object_mut() {
                                    palette.entry("emptyBarColor").or_insert(empty);
                                }
                            }
                            if let Some(outline) = style.remove("indicatorOutline") {
                                let palette = style
                                    .entry("palette")
                                    .or_insert_with(|| serde_json::json!({}));
                                if let Some(palette) = palette.as_object_mut() {
                                    palette.entry("indicatorOutline").or_insert(outline);
                                }
                            }

                            if let Some(palette) = style
                                .entry("palette")
                                .or_insert_with(|| serde_json::json!({}))
                                .as_object_mut()
                            {
                                palette
                                    .entry("backgroundOpacity")
                                    .or_insert(serde_json::json!(100));
                                palette
                                    .entry("mutedBackgroundOpacity")
                                    .or_insert(serde_json::json!(100));
                            }
                        }
                    }
                }
            }
        }
        #[derive(Deserialize)]
        #[serde(default, rename_all = "camelCase")]
        struct Wire {
            schema_version: u32,
            channels: [ChannelSettings; 2],
            macro_button: MacroSettings,
            gain_button: GainButtonSettings,
            dial: DialSettings,
        }
        impl Default for Wire {
            fn default() -> Self {
                let s = Settings::default();
                Self {
                    schema_version: s.schema_version,
                    channels: s.channels,
                    macro_button: s.macro_button,
                    gain_button: s.gain_button,
                    dial: s.dial,
                }
            }
        }
        let w: Wire = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
        Ok(Self {
            schema_version: w.schema_version,
            channels: w.channels,
            macro_button: w.macro_button,
            gain_button: w.gain_button,
            dial: w.dial,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_settings_receive_valid_defaults() {
        let settings: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(settings, Settings::default());
        assert_eq!(settings.validate(), Ok(()));
        let wire = serde_json::to_value(&settings).unwrap();
        assert_eq!(wire["schemaVersion"], 1);
        assert_eq!(wire["channels"][1]["target"]["kind"], "bus");
    }

    #[test]
    fn gain_key_defaults_preserve_older_settings_and_reject_invalid_steps() {
        let mut old = serde_json::to_value(Settings::default()).unwrap();
        old.as_object_mut().unwrap().remove("gainButton");
        let restored: Settings = serde_json::from_value(old).unwrap();
        assert_eq!(restored.gain_button, GainButtonSettings::default());
        let mut s = restored;
        for delta in [0.0, f32::NAN, f32::INFINITY, -73.0, 73.0] {
            s.gain_button.delta = delta;
            assert_eq!(s.validate().unwrap_err().code, "invalidButtonDelta");
        }
        for delta in [-72.0, -0.5, 0.5, 72.0] {
            s.gain_button.delta = delta;
            assert!(s.validate().is_ok());
        }
        for gain in [-61.0, 13.0, f32::NAN] {
            s.gain_button.gain = gain;
            assert_eq!(s.validate().unwrap_err().code, "invalidButtonGain");
        }
    }

    #[test]
    fn reject_nonfinite_steps_unknown_versions_and_boundary_ids() {
        let mut s = Settings::default();
        for step in [f32::NAN, f32::INFINITY, 0.0, -1.0, 73.0] {
            s.channels[1].gain_step = step;
            assert_eq!(s.validate().unwrap_err().code, "invalidGainStep");
        }
        s = Settings::default();
        s.macro_button.logical_id = 79;
        assert!(s.validate().is_ok());
        s.macro_button.logical_id = 80;
        assert_eq!(s.validate().unwrap_err().code, "macroOutOfRange");
        s.schema_version = 2;
        assert_eq!(s.validate().unwrap_err().code, "unsupportedSchema");
    }

    #[test]
    fn reject_invalid_colors_unicode_and_font_axes() {
        assert!(valid_color("#a0B1c2"));
        for color in ["red", "#123", "#gg0000", "#00000000", "#é1234"] {
            assert!(!valid_color(color));
        }
        for code in ["", "D800", "110000", "<svg>", "0xe050"] {
            let icon = IconSettings {
                code_point: code.into(),
                ..Default::default()
            };
            assert!(icon.validate("icon").is_err());
        }
        let icon = IconSettings {
            weight: 450,
            ..Default::default()
        };
        assert!(icon.validate("icon").is_err());
    }
}
