# Installation and diagnostics

Version 0.1.5 is a [public prerelease](https://github.com/hrko/streamdeck-voicemeeter/releases/tag/rust-v0.1.5). Hardware acceptance is not
complete; see implementation-status.md before treating it as a stable release.
Use [hardware-validation.md](hardware-validation.md) to record actual device
results and untested environments.

## Install and configure

1. Install VoiceMeeter separately from VB-Audio, and Stream Deck 6.4 or later on
   Windows x64. The plugin does not redistribute the VoiceMeeter DLL.
2. Obtain a reviewed package for this version, or follow the development guide to build it.
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

For building, development installation and release preparation, see the
[development guide](development.md).

## Choose an icon

Click the icon preview or **Choose icon** to open the picker. Frequently useful
audio icons appear first, followed by the remaining icons in name order, without
categories. Search by English name (for example `volume up` or `mic`) or code
point (`e050`, `U+E050`). No localized search keywords are required.

Click an icon to apply it and close the picker. Escape, the close button or the
backdrop dismisses it without changing settings. Tab and arrow keys navigate
the choices; Enter selects. Scrolling loads more icons; **Show more** is also
available for keyboard users. Font style/axes and Macro colors are preserved.
Code points can be searched in the picker; there is no separate input field.
All catalog data and fonts are bundled,
so searching does not require network access.

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
