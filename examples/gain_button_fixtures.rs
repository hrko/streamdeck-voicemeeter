//! Export actual production SVGs for the pixel regression test.
use streamdeck_voicemeeter::{
    backend::ChannelSnapshot,
    engine::Snapshot,
    gain_button,
    render::IconCache,
    settings::{GainAppearance, Settings},
};

fn main() -> anyhow::Result<()> {
    let directory = "target/fader-visual";
    std::fs::create_dir_all(directory)?;
    let mut cache = IconCache::default();
    // Align the thumb to the supplied reference's measured position. The
    // reference does not expose its Windows volume percentage; do not equate
    // that percentage with VoiceMeeter dB or fit position separately per key.
    let gain = 2.394_431_6;
    for (name, appearance, delta) in [
        ("left", GainAppearance::Horizontal, -3.0),
        ("right", GainAppearance::Horizontal, 3.0),
        ("upper", GainAppearance::Vertical, 3.0),
        ("lower", GainAppearance::Vertical, -3.0),
    ] {
        let mut settings = Settings::default();
        settings.gain_button.appearance = appearance;
        settings.gain_button.delta = delta;
        let snapshot = Snapshot {
            channels: vec![(
                settings.channels[0].target,
                ChannelSnapshot {
                    gain,
                    levels: [-12.0, -36.0],
                    label: "Voice / HDMI / a long label".into(),
                    ..Default::default()
                },
            )],
            ..Default::default()
        };
        std::fs::write(
            format!("{directory}/{name}.svg"),
            gain_button::frame(&snapshot, &settings, &mut cache, "en").svg,
        )?;
        settings.gain_button.show_meter = true;
        for swap in [false, true] {
            settings.gain_button.swap_meter = swap;
            std::fs::write(
                format!("{directory}/{name}-meter-{swap}.svg"),
                gain_button::frame(&snapshot, &settings, &mut cache, "en").svg,
            )?;
            settings.gain_button.fader_label = true;
            std::fs::write(
                format!("{directory}/{name}-label-{swap}.svg"),
                gain_button::frame(&snapshot, &settings, &mut cache, "en").svg,
            )?;
            settings.gain_button.fader_label = false;
        }
    }
    Ok(())
}
