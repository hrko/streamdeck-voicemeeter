# VoiceMeeter for Stream Deck

English | [日本語](README.ja.md)

Control VoiceMeeter gain, mute and MacroButtons from your Stream Deck.
Use Stream Deck + dials to adjust channels and view levels on the touch display,
or assign MacroButtons to keys.

## Features

| Action | Controls |
| --- | --- |
| Gain Control | Turn to adjust gain; press or tap to toggle mute. |
| Gain Control Combo | Turn for the left channel; hold and turn for the right. Tap either half to toggle its mute. |
| Macro | Toggle a MacroButton on each press, or keep it ON while holding the key in Push mode. |
| Gain Button (development version) | Adjust or set channel gain, or toggle mute from a key. Static icons or paired horizontal/vertical faders. |

- Display channel labels, gain, stereo levels and channel states.
- Customize Material Symbols icons, including style, weight and fill, and separate Macro ON/OFF icons and background colors.
- Find icons in an offline picker with audio icons first and English-name/code-point search.
- Gain Button supports optional channel labels, dB values, stereo meters and positive-gain colors. This action is included in the 0.1.6 development build, not the published 0.1.5 package.
- Set independent channels and gain steps for Combo.
- Use Japanese or English settings screens; additional languages can be added through translation resources.

## Requirements and availability

- Windows 10 or later, x64, and Stream Deck software 6.4 or later.
- VoiceMeeter Basic, Banana or Potato, installed separately.
- Stream Deck + for dial actions; Stream Deck keys for Macro actions.
- VoiceMeeter MacroButtons configured for the buttons used by Macro actions.

Version 0.1.5 is available as a [public prerelease](https://github.com/hrko/streamdeck-voicemeeter/releases/tag/rust-v0.1.5), not a stable release.
The main hardware testing used Windows 11 / Stream Deck + / Potato.
Basic/Banana startup, edition detection and channel choices were checked, but
their full feature matrix was not. Windows 10 and Stream Deck 6.4 remain untested.
See the [hardware validation record](docs/hardware-validation.md) for details.

## Installation

1. Install VoiceMeeter and Stream Deck software separately. The plugin does not include the VoiceMeeter DLL.
2. Download the `.streamDeckPlugin` file from the [0.1.5 release](https://github.com/hrko/streamdeck-voicemeeter/releases/tag/rust-v0.1.5). Its `.sha256` file is available for checksum verification. To build from source, see the [development guide](docs/development.md).
3. Open the package with Stream Deck and approve installation.
4. Add an action from **VoiceMeeter (Rust)**, its current name in Stream Deck, and select the channel or MacroButton ID in its settings.

Channel indices and MacroButton IDs start at zero. Previous plugin settings are
not migrated; configure the new actions after installation. Both plugins can coexist.

## Usage and troubleshooting

The plugin reconnects when VoiceMeeter becomes available. Disconnected operations
are not replayed. If VoiceMeeter restarts during a held Macro Push, the old release
is not sent into the new session; use a fresh press/release to clear a retained ON state.

For installation details, checksum verification and log collection, see
[usage and diagnostics](docs/usage.md). When reporting an issue, include Windows,
Stream Deck and VoiceMeeter versions, the action and reproduction steps.

## License

The plugin is source-available under [LICENSE](LICENSE). Commercial use,
including work and monetized streaming, modification and free redistribution
are allowed. Selling the plugin or modified versions, paywalling access and
bundling it with paid products require prior written permission from hrko.
This is not an OSI-approved open-source license.

Third-party components retain their own licenses. Earlier MIT grants remain
valid; see [LEGACY-MIT.txt](licenses/LEGACY-MIT.txt).

## Development and translations

See the [development guide](docs/development.md) for build tools, tests and
release preparation, and [translation instructions](docs/translating.md) to add
or improve a language.
