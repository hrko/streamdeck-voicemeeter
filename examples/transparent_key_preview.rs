use streamdeck_voicemeeter::{
    backend::ChannelSnapshot,
    dial,
    domain::{ActionKind, Edition},
    engine::Snapshot,
    gain_button,
    render::IconCache,
    settings::{GainAppearance, Settings},
};
fn main() -> anyhow::Result<()> {
    let dir = "target/transparent-key-preview";
    std::fs::create_dir_all(dir)?;
    let mut cases = Vec::new();
    for (id, color) in ["#85adb9", "#a6a6a6"].iter().enumerate() {
        let mut s = Settings::default();
        s.gain_button.meter_low = (*color).into();
        s.gain_button.background_opacity = 0;
        s.gain_button.muted_background_opacity = 0;
        s.gain_button.show_meter = true;
        s.gain_button.show_db = true;
        s.gain_button.fader_label = true;
        s.dial.styles[0].palette = serde_json::from_value(serde_json::to_value(&s.gain_button)?)?;
        s.dial.styles[0].palette.meter_inactive = "#1b1b1b".into();
        let snap = Snapshot {
            edition: Some(Edition::Potato),
            channels: s
                .channels
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    (
                        c.target,
                        ChannelSnapshot {
                            gain: 0.0,
                            levels: if i == 0 {
                                [-120.0, -120.0]
                            } else {
                                [-18.0, -36.0]
                            },
                            label: if i == 0 {
                                "Voice".into()
                            } else {
                                "HDMI".into()
                            },
                            ..Default::default()
                        },
                    )
                })
                .collect(),
            ..Default::default()
        };
        for (name, appearance, delta) in [
            ("upper", GainAppearance::Vertical, 3.0),
            ("lower", GainAppearance::Vertical, -3.0),
            ("left", GainAppearance::Horizontal, -3.0),
            ("right", GainAppearance::Horizontal, 3.0),
        ] {
            s.gain_button.appearance = appearance;
            s.gain_button.delta = delta;
            std::fs::write(
                format!("{dir}/{id}-{name}.svg"),
                gain_button::frame(&snap, &s, &mut IconCache::default(), "en").svg,
            )?;
        }
        let frames = [ActionKind::Gain, ActionKind::Combo]
            .map(|k| dial::feedback(&snap, &s, k, &mut IconCache::default(), "en").0);
        cases.push(serde_json::json!({"frames":frames}));
    }
    std::fs::write(
        format!("{dir}/feedback.json"),
        serde_json::to_vec_pretty(&cases)?,
    )?;
    Ok(())
}
