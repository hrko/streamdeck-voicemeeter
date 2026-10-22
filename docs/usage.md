# Installation and diagnostics

The current artifact is for development validation. Hardware acceptance is not
complete; see implementation-status.md before treating it as a stable release.
Use [hardware-validation.md](hardware-validation.md) to record actual device
results and untested environments.

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
and links it into Stream Deck. It first requests normal plugin shutdown, allowing
held buttons to be released and the Remote API session to log out. It refuses
to forcibly terminate an unresponsive plugin. It then temporarily closes and restarts Stream Deck
to release file locks, so all Stream Deck actions are briefly unavailable.
The standard Windows installation under Program Files is required.
Development output lives in `target/dev-plugin`, separate from release `dist`.
An existing packaged installation is preserved under
`%APPDATA%\Elgato\StreamDeck\PluginBackups\<plugin-id>-<timestamp>.sdPlugin`;
profile settings are not modified. A link to another checkout is rejected.
Repeated runs reuse this checkout's link. Avoid running development and package
tasks concurrently because both build the PI assets.

On Windows 11 / Stream Deck 7.0.3, conversion from a packaged installation,
a second development build, and subsequent release packaging with the debug
plugin running were verified on 2026-10-23. The CLI's stop response alone did
not release all native process/directory locks; the host restart is required
by this observed environment.
The initial development helper forcibly stopped residual native processes; the
owner subsequently observed exhausted VoiceMeeter connection slots. That path
has been removed. Older binaries without the shutdown event require a one-time
manual upgrade while VoiceMeeter is stopped; do not repeatedly force-stop them.

To return to a packaged release, close Stream Deck, remove only the development
junction at `Plugins\jp.hrko.streamdeck.voicemeeter.rust.sdPlugin` (not its target),
move the saved directory back to that location, and restart Stream Deck.
Alternatively unlink the development installation and install a new package.

`mise run test:native` uses a loopback test WebSocket server and launches the
EXE without opening Stream Deck. It validates registration, rejection of malformed
settings and clean shutdown. Each run gets a new directory under
`target/smoke-runtime`, so old log records cannot satisfy a later check.
Smoke processes set `STREAMDECK_SMOKE_TEST=1` to disable real VoiceMeeter login.
The task also checks the per-process shutdown event while its test WebSocket
remains open. Never set this variable for the installed plugin.

## Prepare a GitHub release

The `Prepare draft release` workflow runs when a `rust-v*` tag is pushed. The
tag must match the Cargo package version, such as `rust-v0.1.0`. This supports
the independent `rust` branch without changing the repository's default branch.
Create a signed release tag only when the desired commit is ready for review,
then push that specific tag. The tag workflow first runs the reusable
Windows build workflow, downloads that run's artifacts, verifies the package
checksum, and creates a draft prerelease with the package and checksum attached.
It does not publish the release. Existing releases are not overwritten; if an
upload fails, inspect the draft before retrying.

Manual dispatch is also supported once GitHub recognizes the workflow. Its UI
requires the workflow on the default branch; see
[GitHub's manual-trigger documentation](https://docs.github.com/en/actions/how-tos/manage-workflows/manually-running-a-workflow).
Pass the same version-matching tag when using manual dispatch. Ordinary branch
pushes run CI only and do not create releases.

Before publishing, review the hardware acceptance record and included license
texts. The current development candidate has not completed hardware acceptance.
For a local check without creating a release, set `RELEASE_TAG` and run
`mise exec node -- node scripts/draft-release.mjs --dry-run` after packaging.

## Collect logs

Logs are in `%LOCALAPPDATA%\jp.hrko.streamdeck.voicemeeter.rust\logs`:
`plugin.log`, `plugin.1.log`, and `plugin.2.log`. Each is limited to 1 MiB.
Startup failures and panics are recorded, with timestamps, levels and fields.
The default level is info; `RUST_LOG` can override it for a process launched from
a diagnostic shell. This does not change settings saved in Stream Deck.
Stream Deck also passes its inherited environment to plugins: for example,
`RUST_LOG=warn` suppresses startup and normal-shutdown INFO records. For a full
diagnostic session, close Stream Deck and launch it from a shell with
`$env:RUST_LOG = 'info'`, then reproduce the issue. Changing the variable after
Stream Deck has started does not change its existing child processes.

When reporting a problem, provide the logs, Windows/Stream Deck/VoiceMeeter
versions and edition, the action type, and reproduction steps. Inspect logs
before sharing them because local paths and context identifiers may be present.

For translations, see [translating.md](translating.md). User-defined labels and
VoiceMeeter names remain unchanged by localization.
