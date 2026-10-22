# Installation and diagnostics

The current artifact is for development validation. Hardware acceptance is not
complete; see implementation-status.md before treating it as a stable release.

## Install and configure

1. Install VoiceMeeter separately from VB-Audio, and Stream Deck 6.4 or later on
   Windows x64. The plugin does not redistribute the VoiceMeeter DLL.
2. Build with `mise run package`, or obtain a reviewed GitHub release artifact.
   Compare `Get-FileHash -Algorithm SHA256 <file>` with its `.sha256` sidecar.
3. Open the `.streamDeckPlugin` file with Stream Deck and accept its installation.
4. Add actions from **VoiceMeeter (Rust)**. Select Strip/Bus and its zero-based
   index, or a zero-based MacroButton ID. Configure MacroButtons in VoiceMeeter.
5. Recreate the desired settings. The Rust plugin has a separate ID and does not
   migrate the old Go plugin's settings. Both versions can coexist.

Gain Control: turn to change gain, press or tap to toggle mute. Combo: turn for
the left channel; hold and turn for the right; tap the corresponding half to
mute. Macro Push holds ON until release; Toggle changes state on each press.
Saved targets are retained while VoiceMeeter is unavailable, but disconnected
operations are not replayed after recovery.

## Local development

`mise run dev` builds debug Rust and PI assets, validates the assembled plugin,
links it into Stream Deck and restarts this plugin ID. Stream Deck must already
be installed and running. This task intentionally changes the local development
installation; it has not yet been validated against the actual host UI.

`mise run test:native` uses a loopback test WebSocket server and launches the
EXE without opening Stream Deck. It validates registration, rejection of malformed
settings and clean shutdown. Test logs are isolated under `target/smoke-runtime`.

## Collect logs

Logs are in `%LOCALAPPDATA%\jp.hrko.streamdeck.voicemeeter.rust\logs`:
`plugin.log`, `plugin.1.log`, and `plugin.2.log`. Each is limited to 1 MiB.
Startup failures and panics are recorded, with timestamps, levels and fields.
The default level is info; `RUST_LOG` can override it for a process launched from
a diagnostic shell. This does not change settings saved in Stream Deck.

When reporting a problem, provide the logs, Windows/Stream Deck/VoiceMeeter
versions and edition, the action type, and reproduction steps. Inspect logs
before sharing them because local paths and context identifiers may be present.

For translations, see [translating.md](translating.md). User-defined labels and
VoiceMeeter names remain unchanged by localization.
