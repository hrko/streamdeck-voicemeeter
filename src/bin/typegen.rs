use std::{env, fs, path::Path};
use ts_rs::TS;

fn main() -> anyhow::Result<()> {
    let path = Path::new("pi/src/generated/contracts.ts");
    let body = format!(
        "{}\nexport {}\n\nexport const defaultSettings: Settings = {};\n",
        streamdeck_voicemeeter::settings::typescript(),
        streamdeck_voicemeeter::domain::Edition::decl(),
        serde_json::to_string_pretty(&streamdeck_voicemeeter::settings::Settings::default())?
    );
    if env::args().any(|arg| arg == "--check") {
        anyhow::ensure!(
            fs::read_to_string(path)? == body,
            "Generated contracts differ. Run mise run types."
        );
    } else {
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, body)?;
    }
    let glyphs = serde_json::to_string(&streamdeck_voicemeeter::render::glyph_catalog()?)? + "\n";
    let glyph_path = "pi/src/generated/glyphs.json";
    if env::args().any(|arg| arg == "--check") {
        anyhow::ensure!(
            fs::read_to_string(glyph_path)? == glyphs,
            "Generated glyph catalog differs. Run mise run types."
        );
    } else {
        fs::write(glyph_path, glyphs)?;
    }
    Ok(())
}
