//! Generate a browser-viewable sheet from the production Rust renderer.
use streamdeck_voicemeeter::{
    backend::ChannelSnapshot,
    engine::Snapshot,
    gain_button,
    render::IconCache,
    settings::{GainAppearance, Settings},
};

fn main() -> anyhow::Result<()> {
    let mut cache = IconCache::default();
    let mut html = String::from(
        "<!doctype html><meta charset=utf-8><title>Gain key rendering</title><style>body{background:#2d2d2d;color:white;font:14px system-ui} section{display:flex;gap:24px;margin:24px}svg{width:72px;height:72px;border-radius:14px;border:3px solid #484848} .pair{display:flex;gap:14px}.vertical{flex-direction:column}h2{font-size:16px}</style><h1>Gain key rendering</h1>",
    );
    for decorated in [false, true] {
        html.push_str(if decorated {
            "<h2>Optional color, label, dB and meters</h2><section>"
        } else {
            "<h2>Default geometry (−9.6 dB)</h2><section>"
        });
        for appearance in [
            GainAppearance::Horizontal,
            GainAppearance::Vertical,
            GainAppearance::Static,
        ] {
            let mut s = Settings::default();
            s.gain_button.appearance = appearance;
            s.gain_button.highlight_positive = decorated;
            s.gain_button.show_label = decorated;
            s.gain_button.show_db = decorated;
            s.gain_button.show_meter = decorated;
            if decorated {
                s.gain_button.bar_color = "#7bc99e".into();
                s.gain_button.thumb_color = "#7bc99e".into();
            }
            html.push_str(if appearance == GainAppearance::Vertical {
                "<div class='pair vertical'>"
            } else {
                "<div class=pair>"
            });
            for delta in if appearance == GainAppearance::Vertical {
                [3.0, -3.0]
            } else {
                [-3.0, 3.0]
            } {
                s.gain_button.delta = delta;
                let snapshot = Snapshot {
                    channels: vec![(
                        s.channels[0].target,
                        ChannelSnapshot {
                            gain: if decorated { 6.0 } else { -9.6 },
                            label: "Voice 音声".into(),
                            levels: [-12.0, -24.0],
                            ..Default::default()
                        },
                    )],
                    ..Default::default()
                };
                html.push_str(&gain_button::frame(&snapshot, &s, &mut cache, "en").svg);
            }
            html.push_str("</div>");
        }
        html.push_str("</section>");
    }
    let output = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/gain-button-preview.html".into());
    std::fs::write(&output, html)?;
    println!("{output}");
    Ok(())
}
