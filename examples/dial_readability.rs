//! Export production dial feedback for readability comparison sheets.
use serde_json::json;
use streamdeck_voicemeeter::{
    backend::ChannelSnapshot,
    dial,
    domain::{ActionKind, Edition},
    engine::Snapshot,
    render::IconCache,
    settings::Settings,
};
fn main() -> anyhow::Result<()> {
    let dir = "target/dial-readability";
    std::fs::create_dir_all(dir)?;
    let mut cases = Vec::new();
    for id in 0..17 {
        let mut s = Settings::default();
        let d = &mut s.dial.styles[0];
        d.palette.background_opacity = 70;
        d.palette.muted_background_opacity = 70;
        let mut gain = -6.0;
        let mut mute = false;
        match id {
            1 => {
                d.palette.bar_color = "#ffffff".into();
                d.palette.highlight_positive = false;
            }
            2 => {
                d.palette.background = "#2c3d4d".into();
                d.palette.muted_background = "#2c3d4d".into();
                d.palette.background_opacity = 100;
                d.palette.muted_background_opacity = 100;
            }
            3 => {
                gain = 6.0;
                d.palette.dial_positive_gain_color = "#ffab91".into();
            }
            4 => {
                mute = true;
            }
            5 => {
                d.show_routing = false;
                d.show_solo = false;
                d.show_mono = false;
                d.show_eq = false;
            }
            6 => {
                d.show_icon = false;
            }
            7 => {
                d.show_meter = false;
                d.show_routing = false;
                d.show_solo = false;
                d.show_mono = false;
                d.show_eq = false;
            }
            8 => {
                s.dial.shared = false;
                s.dial.styles[1].palette.background_opacity = 70;
                s.dial.styles[1].palette.bar_color = "#8db7ff".into();
            }
            9 | 10 => {
                d.palette = serde_json::from_value(serde_json::to_value(
                    streamdeck_voicemeeter::settings::GainButtonSettings::default(),
                )?)?;
                d.palette.background_opacity = 0;
                d.palette.muted_background_opacity = 0;
                d.palette.indicator_background = "#000000".into();
                d.palette.indicator_opacity = 62;
                d.palette.indicator_outline = id == 10;
            }
            11..=13 => {
                d.palette.background_opacity = 0;
                d.palette.muted_background_opacity = 0;
                d.palette.indicator_background = "#2c3d4d".into();
                d.palette.indicator_opacity = 100;
                d.palette.indicator_outline = false;
                d.palette.empty_bar_color = "#2c3d4d".into();
                gain = if id == 12 { 6.0 } else { -6.0 };
                mute = id == 13;
            }
            14..=16 => {
                d.palette = serde_json::from_value(serde_json::to_value(
                    streamdeck_voicemeeter::settings::GainButtonSettings::default(),
                )?)?;
                if id != 15 {
                    d.palette.bar_color = "#70c399".into();
                    d.palette.positive_color = "#f8634d".into();
                    d.palette.highlight_positive = true;
                    d.palette.empty_bar_color = "#2c3d4d".into();
                    d.palette.background = "#2c3d4d".into();
                    d.palette.muted_background = "#2c3d4d".into();
                    d.palette.meter_low = "#85adb9".into();
                    d.palette.meter_muted_low = "#394547".into();
                    d.palette.meter_inactive = "#191b1b".into();
                }
                d.palette.background_opacity = if id == 14 { 100 } else { 0 };
                d.palette.muted_background_opacity = d.palette.background_opacity;
                d.palette.indicator_background = "#000000".into();
                d.palette.indicator_opacity = 62;
                d.palette.indicator_outline = false;
            }
            _ => {}
        }
        let snapshot = Snapshot {
            edition: Some(Edition::Potato),
            channels: s
                .channels
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    (
                        c.target,
                        ChannelSnapshot {
                            gain: if i == 0 { gain } else { 0.0 },
                            mute: mute && i == 0,
                            label: if id == 8 {
                                if i == 0 {
                                    "マイク入力・配信音声"
                                } else {
                                    "ヘッドホン出力・モニター"
                                }
                            } else if i == 0 {
                                "Voice"
                            } else {
                                "HDMI"
                            }
                            .into(),
                            levels: if i == 0 {
                                [-12.0, -18.0]
                            } else {
                                [-24.0, -30.0]
                            },
                            flags: [
                                ("A1".into(), true),
                                ("B1".into(), true),
                                ("Solo".into(), true),
                                ("EQ".into(), true),
                            ]
                            .into(),
                        },
                    )
                })
                .collect(),
            ..Default::default()
        };
        let mut frames = Vec::new();
        for kind in [ActionKind::Gain, ActionKind::Combo] {
            frames.push(dial::feedback(&snapshot, &s, kind, &mut IconCache::default(), "ja").0);
        }
        cases.push(json!({"id":id,"frames":frames,"settings":s}));
    }
    std::fs::write(
        format!("{dir}/feedback.json"),
        serde_json::to_vec_pretty(&cases)?,
    )?;
    Ok(())
}
