#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(windows)]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use streamdeck_voicemeeter::{actions::DeckAction, voicemeeter::Remote, worker::Worker};
    let args: Vec<_> = std::env::args().collect();
    // Validate before the SDK builder, which panics on malformed arguments.
    streamdeck_plugin::RegistrationArguments::parse(&args)?;
    let worker = Worker::spawn(Remote::default)?;
    let result = streamdeck_plugin::Plugin::builder(&args)
        .state(worker.client.clone())
        .add_action::<DeckAction<0>>()
        .add_action::<DeckAction<1>>()
        .add_action::<DeckAction<2>>()
        .run()
        .await;
    drop(worker);
    result?;
    Ok(())
}

#[cfg(not(windows))]
fn main() {
    eprintln!("VoiceMeeter requires Windows x64.");
    std::process::exit(1);
}
