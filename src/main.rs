#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(windows)]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    streamdeck_voicemeeter::logging::initialize()?;
    tracing::info!(
        pid = std::process::id(),
        version = env!("CARGO_PKG_VERSION"),
        "Plugin starting"
    );
    let result = run().await;
    if let Err(error) = &result {
        tracing::error!(%error, "Plugin terminated with an error");
    } else {
        tracing::info!("Plugin stopped");
    }
    result
}

#[cfg(windows)]
async fn run() -> anyhow::Result<()> {
    use streamdeck_voicemeeter::{actions::DeckAction, voicemeeter::Remote, worker::Worker};
    let args: Vec<_> = std::env::args().skip(1).collect();
    // Validate before the SDK builder, which panics on malformed arguments.
    let registration = streamdeck_plugin::RegistrationArguments::parse(&args)?;
    let shutdown = streamdeck_voicemeeter::shutdown::ShutdownEvent::new()?;
    let mut worker = Worker::spawn(Remote::default)?;
    worker.client.language = registration.info.application.language;
    let plugin = streamdeck_plugin::Plugin::builder(&args)
        .state(worker.client.clone())
        .add_action::<DeckAction<0>>()
        .add_action::<DeckAction<1>>()
        .add_action::<DeckAction<2>>()
        .run();
    let result = tokio::select! {
        result = plugin => result,
        () = shutdown.requested() => {
            tracing::info!("Graceful shutdown requested");
            Ok(())
        }
    };
    drop(worker);
    result?;
    Ok(())
}

#[cfg(not(windows))]
fn main() {
    eprintln!("VoiceMeeter requires Windows x64.");
    std::process::exit(1);
}
