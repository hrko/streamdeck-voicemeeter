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
    let dir = "target/transparent-presets";
    std::fs::create_dir_all(dir)?;
    let mut cases = Vec::new();
    for id in 0..4 {
        let mut s = Settings::default();
        let p = &mut s.dial.styles[0].palette;
        p.background_opacity = 0;
        p.muted_background_opacity = 0;
        p.bar_color = "#ffffff".into();
        p.empty_bar_color = "#808080".into();
        p.indicator_background = "#808080".into();
        p.indicator_opacity = 100;
        p.indicator_outline = false;
        p.highlight_positive = false;
        p.meter_track_opacity = 0;
        match id {
            0 => {}
            1 => {
                p.indicator_outline = true;
                p.indicator_background = "#000000".into();
                p.indicator_opacity = 62;
                p.empty_bar_color = "#404040".into();
            }
            2 => {
                p.indicator_background = "#c0c0c0".into();
                p.indicator_opacity = 100;
                p.empty_bar_color = "#404040".into();
                p.meter_track_opacity = 50;
            }
            _ => {
                p.dial_label_color = "#101010".into();
                p.dial_positive_label_color = "#101010".into();
                p.dial_gain_color = "#101010".into();
                p.dial_positive_gain_color = "#101010".into();
                p.indicator_background = "#202020".into();
                p.empty_bar_color = "#b0b0b0".into();
                p.meter_track_opacity = 30;
            }
        }
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
