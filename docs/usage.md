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

## Gain Button (0.1.6 development build)

Add **Gain Button** to a key and select a Strip/Bus channel. Choose **Adjust gain**
with a nonzero signed change (−72..+72 dB), **Set gain** (−60..+12 dB), or
**Toggle mute**. Adjust performs one operation on press, then repeats every 0.25 seconds while
held. Set gain and toggle mute operate only once. Repetition stops on release,
settings changes, page exit or disconnect; reconnect never resumes a held command.
Gain is clamped to −60..+12 dB. Offline input is discarded, and external changes
update the display through the same channel snapshot used by the dial actions.

For relative adjustments, choose a static icon or a horizontal/vertical fader.
Horizontal faders use the left half for negative changes and right half for
positive changes. Vertical faders use the upper half for positive changes and
lower half for negative changes. Place two keys for the same channel next to
each other and match their display options/colors. Step magnitudes can differ.
The pair displays the full −60..+12 dB range.
These are key images, not draggable sliders.

The default fader uses a black background and white rounded rail/ring thumb.
Customize rail, thumb, normal/muted background and positive-gain color separately.
**Highlight above 0 dB** changes the rail and thumb only when the gain is positive.
Current gain and stereo meters are available only for fader styles. Gain is
displayed horizontally inside the thumb (for example −12.5, 0.0 or +6.0),
without the dB suffix. Enabling it enlarges the thumb; the meter clearance
remains 10 px and both axes stay symmetric. A thumb crossing the key boundary
is split between the two keys, including its number. Enabling a
meter shifts the fader to a separate track. Both stereo lanes span the full
key pair with their ends aligned to the fader bar. The default is meter
left / fader right (vertical), or meter above / fader below (horizontal); **Swap fader and meter** exchanges the tracks (above/below for
horizontal, left/right for vertical). Both center axes are equally spaced from
the key center, with 10 px between meter background and thumb ring at 72px key
size. Set the same options on both keys.

The decrease (left/lower) key has a separate **Label inside fader** option. The
text is clipped inside its bar and never continues onto the right/upper key.
Its color is under **Adjust colors → Fader and label** and included in presets.
Static styles ignore saved dB/meter preferences. Their channel label uses Stream
Deck's native title; new keys default to bottom alignment. Existing keys retain
their saved title formatting, which can be changed in the title controls.
Disabling the channel label restores the user-entered title. Meters reuse
the shared decay state and instantaneous-level marks. Select the VoiceMeeter
color preset for the corresponding rail, label, background and meter colors.

Static mode reuses the icon picker/font controls and separate normal/muted icons
and backgrounds. Adjust buttons have a +/− badge; absolute buttons show their
target gain. Set and mute operations always use the static display while retaining
the saved fader preference for switching back to Adjust. Existing actions/settings
remain compatible: missing gain-button settings receive defaults.

### Meter colors and shared presets

The stereo meter uses VoiceMeeter-style cells. Expand **Adjust colors → Meter colors** to edit
its background, unlit cells, low/middle/high level colors and their muted
counterparts. The level/decay continues while muted; only the active-cell colors
change. These colors are independent of the key background.

Choose **Color preset** to apply a built-in or shared user palette. Under
**Manage presets**, enter a name and **Save as new preset** to copy the current
colors. Select a user preset, adjust its colors/name and **Update selected
preset** to edit it, or delete it with confirmation. Built-ins are read-only.
All Gain Button panels share the list. Edits/deletions do not affect keys that
already copied the palette; reapply to update those keys. Presets include meter
colors and positive-gain highlighting, but not target, step, layout or visibility.

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

## Choose a background color

Macro ON/OFF backgrounds each have a color swatch that opens the native color
picker and a text field for `#RRGGBB` input. Both update the same setting and icon
preview. Incomplete or invalid text is not saved; the picker keeps the last valid
color until a valid value is selected or entered.

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
