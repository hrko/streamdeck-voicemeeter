# User guide

English | [日本語](usage.ja.md) | [README](../README.md)

This guide describes the **0.1.6 development build**. Features and settings differ from the published [0.1.5 prerelease](https://github.com/hrko/streamdeck-voicemeeter/releases/tag/rust-v0.1.5). To use the development build, follow the [development guide](development.md).

## Installation and channel selection

1. Install VoiceMeeter and Stream Deck software 6.4 or later on Windows x64. The plugin does not include the VoiceMeeter DLL.
2. Download the `.streamDeckPlugin` package from its release page. Compare PowerShell's `Get-FileHash -Algorithm SHA256 <path>` output with the attached `.sha256` file.
3. Open the package and approve installation in Stream Deck.
4. Add an action from **VoiceMeeter (Rust)** and select its target.

Choose **Strip (input)** or **Bus (output)**. Indices start at zero. Named channels appear as `Strip 6 — Voice`, and name changes refresh about once a second. Targets are saved by index, so renaming a channel does not change which channel is controlled.

Available channels follow the connected Basic/Banana/Potato edition. If a saved target is unavailable in that edition, select a valid channel. The old plugin can coexist, but its settings are not migrated automatically.

## Dial controls

### Gain Control

Turn to adjust gain by the configured step; press the knob to toggle mute. Gain is limited to −60..+12 dB. By default, tapping toggles mute and long-pressing resets gain to 0 dB. Change or disable either gesture under **Touch controls**.

### Gain Control Combo

The default mode is **Select channel by touch**. Tap a side to select it; rotation and knob presses control that channel. Choose whether taps select the touched side or switch to the other channel regardless of tap position.

Long press selects the touched side and performs that side's assigned action, defaulting to mute toggle. Eight selection markers are available; the default is G, a short bottom line.

In **Hold and turn for right channel** mode, turn normally for the left channel and press and turn for the right. Tap either half to toggle its mute. Channels and gain steps are independent.

### VoiceMeeter commands

Assign **Run VoiceMeeter command** to single-channel tap/long press or either side's long press in channel-selection mode. Enter Remote API commands separated by semicolons. Editing does not execute commands.

Commands are limited to 4096 UTF-16 code units and cannot contain NUL characters. Execution errors appear in the inspector; details are recorded in the log.

## Key controls

### Gain Button

Choose **Adjust gain**, **Set gain**, or **Toggle mute**. Relative change accepts −72..+72 dB except zero; absolute gain accepts −60..+12 dB. Actual gain is clamped to −60..+12 dB.

Adjust runs once on press, then every 0.25 seconds while held. Set and mute run once per press. Adjust supports static icons or horizontal/vertical faders; Set and Mute use static icons. Faders are key images, not draggable sliders.

| Orientation | Increase key | Decrease key |
| --- | --- | --- |
| Horizontal | Right | Left |
| Vertical | Upper | Lower |

Place complementary keys for the same channel next to each other to form a fader. **Sync adjacent fader** defaults to on. When enabled on both halves, subsequent edits synchronize channel, step magnitude and appearance. Each key retains its increase/decrease sign.

Pairs must be visible on the same device, target the same channel and use matching orientations in the arrangement above. Opening a page does not overwrite existing settings. Changing orientation, operation or step sign breaks the pair. Disable synchronization to use different step magnitudes.

Gain values and stereo meters are fader-only options. Values appear inside the thumb, and the meter sits beside the fader. **Swap fader and meter** exchanges their positions.

Fader labels appear only on the left/lower half and are clipped to the filled part as gain falls. With synchronization enabled, either half can edit the label. Static icons use Stream Deck's native title for the channel name, with bottom alignment by default. Disabling the channel label restores the user-entered title.

### Macro

Enter the ID (0–79) of a button configured in VoiceMeeter MacroButtons. **Toggle** changes ON/OFF on each press; **Push (hold)** turns OFF on release. If multiple Push keys hold the same target, it stays ON until all are released.

## Appearance and colors

### Icons and colors

Open **Choose icon** to search by English name or code point (`e050`, `U+E050`). Audio icons appear first. Click to apply; Escape or Close cancels. Search works offline.

Change font style, weight, fill, grade and optical size. Colors support both a native picker and `#RRGGBB` input. Incomplete or invalid input is not saved.

### Color presets

Choose Default, White on black, VoiceMeeter, or transparent white/black and VoiceMeeter variants with or without outlines. In transparent preset names, “outline” refers to inactive dial indicators.

Presets are shared by Gain Button and dial actions. They include colors, background and component opacities, positive-gain highlighting, indicator outlines and selection-marker color. They exclude channels, steps, icons, visible items, control mode and refresh rate.

Under **Manage presets**, save the current palette with a name, or update/delete a user preset. Built-ins cannot be edited. Applying a preset copies its settings; updating or deleting it leaves previously applied appearances unchanged. Reapply to use updated values.

Transparent presets make the outer background fully transparent. On Gain Button, meter backing, unlit cells, empty rail and thumb fill use 40% opacity, with no clearance outside the thumb ring. Change individual values under **Adjust colors**.

### Dial display

Independently show or hide labels, gain values, icons, gain bars, meters and status indicators. Hiding an item does not disable its operation. Hiding meters compacts the text/status spacing while leaving the gain bar in place.

Combo enables **Use the same appearance on both sides** by default. Disable it for separate appearances. Channels, steps and icons remain independent.

Meters support cells or continuous bars. Levels continue updating while muted, using muted active colors. **Mark current level with an unlit cell** marks the instantaneous signal below the falling peak; it does not mark the gain setting and defaults to off.

## Refresh rate

At the bottom of the inspector, **Plugin-wide settings → Refresh rate** offers 15 (default), 20, 25 and 30 fps. Changes apply immediately to all keys and dials and persist after restart. Lower the rate if the display stutters.

## Connection and troubleshooting

The plugin reconnects when VoiceMeeter becomes available. Disconnected operations and held repeats are not replayed after recovery.

Leaving a page or changing the target during Macro Push attempts to release the original target, as does normal Stream Deck shutdown. Releases are not carried across a VoiceMeeter connection loss. If a restarted session retains ON, press and release the key again.

Logs are in `%LOCALAPPDATA%\jp.hrko.streamdeck.voicemeeter.rust\logs`: `plugin.log`, `plugin.1.log` and `plugin.2.log`, each limited to 1 MiB. Include logs, Windows/Stream Deck/VoiceMeeter versions, action type and reproduction steps in issue reports.

If normal logs are missing, close Stream Deck, set `$env:RUST_LOG = 'info'` in PowerShell, and launch Stream Deck from that shell. Changing the variable after launch does not affect the running plugin. Review local paths in logs before sharing them.

See [hardware validation](hardware-validation.md) for tested environments, the [development guide](development.md) for builds, and [translation instructions](translating.md) to add a language.
