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
            icon: IconSettings::default(),
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
                ..Default::default()
            },
            icon_off: IconSettings {
                code_point: "e836".into(),
                ..Default::default()
            },
            color_on: "#067ba2".into(),
            color_off: "#004162".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub schema_version: u32,
    pub channels: [ChannelSettings; 2],
    pub macro_button: MacroSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            channels: [
                ChannelSettings::default(),
                ChannelSettings {
                    target: Target {
                        kind: TargetKind::Bus,
                        index: 0,
                    },
                    ..Default::default()
                },
            ],
            macro_button: MacroSettings::default(),
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
        let m = &self.macro_button;
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
