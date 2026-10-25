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
    let dir = "target/wallpaper-presets";
    std::fs::create_dir_all(dir)?;
    let mut cases = Vec::new();
    for id in 0..4 {
        let mut s = Settings::default();
        let p = &mut s.dial.styles[0].palette;
        p.bar_color = "#ffffff".into();
        p.empty_bar_color = "#808080".into();
        p.indicator_background = "#000000".into();
        p.indicator_opacity = 62;
        p.indicator_outline = true;
        p.meter_mid = "#70c399".into();
        p.dial_positive_gain_color = "#ffb49f".into();
        match id {
            0 => {
                p.background_opacity = 0;
                p.meter_track_opacity = 20;
            }
            1 => {
                p.background_opacity = 40;
                p.meter_track_opacity = 25;
                p.indicator_outline = false;
                p.indicator_background = "#808080".into();
                p.indicator_opacity = 80;
            }
            2 => {
                p.background = "#203447".into();
                p.background_opacity = 60;
                p.bar_color = "#8dd8e8".into();
                p.empty_bar_color = "#536b80".into();
                p.meter_track_opacity = 35;
                p.indicator_outline = false;
            }
            _ => {
                p.background_opacity = 80;
                p.meter_track_opacity = 50;
            }
        }
        p.muted_background = p.background.clone();
        p.muted_background_opacity = p.background_opacity;
        let gain = -6.0;
        let mute = false;
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
