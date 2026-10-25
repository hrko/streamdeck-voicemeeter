# Gain Button implementation and validation

Introduced in development version 0.1.6 as `gain-button`, a Keypad action.
The target uses `channels[0].target`; additional settings are under `gainButton`.
Serde defaults and PI restoration keep existing schema-version-1 settings valid.
The existing worker owns all mixer reads/writes, connection generations, gain
limits, deduplicated snapshots and meter decay. Set-gain requests validate both
the edition's target bounds and the finite −60..+12 dB range before writing.

## Reference geometry

The owner supplied the built-in Volume Controller screenshot used in
`tests/fixtures/fader`. The production renderer draws a shared 288x144 rail
and clips/transforms it into one 144x144 key image. At 72px display size:

| Measurement | Horizontal | Vertical (before rotation) |
| --- | --- | --- |
| Rail endpoint centers | 17.25..125 px | 18.25..126 px |
| Outer rail width / outline | 14 / 3 px | 14 / 3 px |
| Thumb ring outer / inner diameter | 20 / 14 px | 20 / 14 px |
| Background clearance outside ring | 3 px | 3 px |

Negative/positive horizontal keys show the left/right half; positive/negative
vertical keys show the upper/lower half. Step magnitude does not change geometry.
Gain maps linearly over −60..+12 dB, independently of Windows volume percentages.

Run `mise run test:visual` for the pixel regression check (also included in CI).
A Rust example exports the **production renderer's** four SVGs, and locked sharp
0.35.5 (published 2026-09-27; resolved under the 7-day policy on 2026-10-25)
rasterizes them directly at 72px. Comparison excludes only rounded host
key-frame corners. Each key must have grayscale RMSE <= 2/255 and no more than
4 differing silhouette pixels (threshold 128). This allows minor rasterizer
antialiasing differences while catching 1px shifts and changed clearances.
Reference thumb position is aligned with one shared test gain because the
screenshot does not expose its Windows volume percentage. This tests geometry,
not equivalence between Windows percentage and VoiceMeeter gain.

On 2026-10-25, the four keys had RMSE 0.98–1.27/255 and 1–2 silhouette pixels
different out of 4,400 compared pixels each. Reinstating the old geometry at
the same thumb positions produces 4/118/96/18 silhouette differences for
left/right/upper/lower and fails the thresholds. Output PNGs, red/green difference
images and numerical results are in `target/fader-visual`. This is a raster
comparison with the supplied screenshot; updated hardware rendering still
requires confirmation. No claim of byte-identical antialiasing is made.

Color and detail options are independent, with all optional detail fields off by
default. The PI offers White on black and VoiceMeeter color presets. VoiceMeeter
uses #70c399 for the rail/thumb, #f8634d for positive gain (highlight enabled),
and #2c3d4d for both backgrounds, matching `src/dial.rs` and the owner's Potato
screenshot. Applying a palette changes only colors and positive highlighting;
channel, operation, step, orientation and label/dB/meter options are retained.
Presets are inferred from saved color values; individual edits display Custom.
The `swapMeter` flag defaults to false: the meter is above a horizontal fader
and left of a vertical fader. Swapping reverses these positions. The rail is
at canonical y=96 and stereo lanes at y=36/48 (swapped: y=48 and y=88/100).
The 24px meter background starts at 34 (86 swapped). At 72px key size,
outer margins measured from the meter background and 14px bar are both 17px;
the gap between them is 12px. These axes stay fixed with or without gain text.
Both lanes span the rounded rail's outer cap extents across the shared pair
(x=20.5..264 horizontally, x=22.5..266 before vertical rotation).
The meter has 40 cells per channel, with a shared grid across both keys.
Pixel tests compare two rendered keys against one uninterrupted production
canvas, checking both orientations and placements, alignment and separation.

Cells below −24 dB are cyan, −24..0 dB green, and above 0 dB red. Their normal
and muted colors are independently configurable. Muting changes colors, not
level readings. Meter background and unlit-cell color are separate from the
key background and remain unchanged by mute. The current-level notch and
shared peak decay are retained. The default muted colors were sampled from
the owner's 2026-10-25 screenshot; normal colors match the existing dial.

User palettes are stored in Stream Deck global settings under `gainColorPresets`
(version 1) and shared across Gain Button inspectors. Each has a UUID, name and
only palette fields. Applying copies values into action settings. Updating,
renaming or deleting the shared entry never rewrites existing action settings.
Built-ins cannot be mutated; saving a copy creates a user entry. Before writes
the inspector requests the latest global settings, preserves unrelated fields,
and refuses unknown or malformed preset stores. Up to 100 names of 64 characters
are supported. Stored palette IDs cannot address built-ins. No external storage
or network is used.

Only fader styles render dB and meters; static styles retain but ignore those
preferences. `faderLabel` is independent of the static `showLabel`. It renders
inside the negative (left/lower) rail only, clipped before the split and
overlaid by the thumb. Labels start 4px from the outer rounded bar end, rather than being centered.
`positiveLabelColor` selects the text color when positive highlighting is active
(default and VoiceMeeter preset: dark red #862b20). Older settings and user
palettes receive this default. `labelColor` is a palette field; older user palettes
without it receive the default black. Raster tests verify label pixels stay
inside the rail and the positive half remains completely unchanged. Static channel names instead use
`setTitle`, including external name updates; disabling labels or returning to a
fader sends a null title to restore the host's title. New Gain Button instances
have bottom title alignment. Existing host title-format preferences are retained.
SDK integration tests cover label updates and resetting automatic titles.
Static icons use the existing Material Symbols renderer, normal/muted icons and
backgrounds; direction or absolute-target badges identify the operation.

Run `mise exec rust -- cargo run --locked --example gain_button_preview` to
generate `target/gain-button-preview.html` from the production renderer.

## Validation on 2026-10-23

- Formatting, Clippy, generated types, TypeScript and translation checks passed.
- 33 Rust unit tests, 9 SDK/worker integration tests, 19 PI tests and 4 script
  tests passed. New coverage includes signed steps, absolute gain validation,
  limits, mute, externally changed state, offline input, disappearance, old
  settings defaults, complementary geometry and optional display controls.
- Debug and release native WebSocket smoke tests passed without mixer login.
- Official CLI packaging passed; all 204 archived files match staging and the
  102 dependency/asset license entries were verified.
- Browser previews checked default horizontal/vertical geometry, static badges,
  optional display placement and Japanese PI controls.

The owner has not yet confirmed this new action on hardware. The existing 0.1.5
release remains public; 0.1.6 has not been published.

## Held gain adjustment

Adjust keys apply immediately and repeat every 250 ms. Missed ticks are skipped,
not replayed in a burst. The captured connection generation is shared by the
initial command and repeats. Release, settings replacement, disappearance and
Drop abort the task; connection loss or generation change ends it. Offline
presses never start a repeat task. Set and mute operations do not repeat.
SDK/worker tests cover first-tick delay, repeated changes, release, settings
changes, disappearance and reconnect while held.

## Gain inside the thumb

With `showDb`, the thumb outer diameter is 28px at 72px key size (otherwise
20px). Monospaced signed numbers use no unit and remain horizontal in either
orientation. A light/dark foreground is selected from the background color.
The thumb covers the channel label. Meter and fader centers remain at 46/96
(canonical 144px coordinates), or 98/48 when swapped, with or without gain text. Numbers are clipped to
the inner circle; pixel tests verify that removing that clip reveals no missing
glyphs for −60, −24, −12.5, 0, +6 and +12. Key-boundary splitting remains visible.

## Adjacent setting synchronization

The worker client's shared visible-key registry is populated by `willAppear`
and removed by `willDisappear` and Drop. It matches device, coordinates,
orientation, opposite signs and the pre-edit target. There is no persisted page
identity or background editing of hidden profiles. Duplicate matching contexts
are treated as ambiguous and skipped. Multi-actions are never registered.

Settings edits copy the first channel and gain-button settings to the peer,
preserving its delta sign. SDK state is refreshed with `getSettings` after
`setSettings`; host replies update the worker and rendering through the normal
settings handler. Pending snapshots suppress acknowledgement loops and restore
the latest settings if an older reply arrives during rapid edits. The registry
is scoped to a worker/client rather than process-global static state.

`meterNotch` optionally marks the instantaneous level with an unlit cell below
the decaying peak on gain-button meters. It defaults to false for both new and
existing settings and follows pair synchronization. Dial meters are unchanged.

Fader labels are clipped to the filled segment: lowering the gain hides text
above the thumb (vertical) or to its right (horizontal). Text stays anchored
at the bar end instead of moving or shrinking.

## Dial color interoperability

`dial.styles[2]` stores independent `DialStyle` values; `dial.shared` selects the
left style for both halves without copying over the saved right style. The
`DialPalette` keys match the common gain-button palette so saved user presets
work in both inspectors. Dial-only gain-number and empty-bar colors are kept
outside the shared palette. Boolean visibility controls and channel/icon settings
are never overwritten by palette selection.

Native feedback text uses `{value, enabled, color}` and pixmaps use
`{value, enabled}`. Per-half background pixmaps have zOrder 0; existing layout
items remain at their original rectangles with zOrder 1. Status visibility skips
the original slot rather than compacting the other indicators. The native
loopback test checks these structured feedback messages. Renderer tests cover
muted colors, shared/independent Combo styles and retained status positions.

Dial backgrounds use numeric SVG dimensions (200x100 for single, 100x100 for
Combo halves), avoiding percentage interpretation differences in the host SVG
renderer. Opacity is applied to the background pixmap only, and 0 disables it.
`channelLabel`/`channelLabel1` replace the host-special `title` key so both sides
respect explicit PI label colors. Shared palettes retain button label fields,
but dial text uses independent `DialStyle.labelColor`/`positiveLabelColor`.
Rust deserialization and PI restore both distinguish empty new settings from
nonempty legacy settings to preserve the existing opaque background.

### Dial meter and indicator readability

Dial meters default to meterStyle "cells"; "continuous" draws two continuous
stereo lanes. Both use the same level scale and symmetric 2 px inner margin as
the gain bar. Cells use whole-pixel 1 px lights and 1 px gaps within each 2 px interval,
so native-size rasterization retains distinct cells. Switching styles does not
change the level interval boundaries. Meter decay, mute colors and the optional
current-level notch are unchanged.

The outer meter background matches the gain bar rim (black, opacity 160/255).
meterTrackOpacity controls the inner inactive track from 0 to 100% (default 0),
using emptyBarColor. At zero opacity, cells retain dim inactive cells; continuous
meters show only the active portions. The legacy palette meterBackground field
is retained for shared preset compatibility but is not used by dial meters.

palette.indicatorOutline defaults to false and controls outlines on inactive indicators.
Routing indicators have a 5 px footprint and 2 px outer corner radius. Other
status indicators retain their 7 px footprint. Inset 1 px strokes align straight
edges to pixels while curves remain antialiased. These settings follow the existing
shared/independent Combo appearance selection. Old settings acquire these defaults
without changing targets, steps, palettes or per-channel settings.

Inactive indicator background color, opacity (0–100%) and outline are included
in shared color presets. The standard preset uses #808080 at 100% with no outline;
VoiceMeeter uses black at 62% with no outline. The dial's empty bar color is also
part of the palette: #808080 for standard, #2c3d4d for VoiceMeeter. Previously
saved top-level dial outline and empty bar values migrate into the palette.
Changing inactive opacity does not affect the active colors or outline opacity.

Background opacity (normal and muted), inactive meter-track opacity, and the
four dial text colors are now included in shared palettes. Dial label/gain
colors have separate dial-prefixed keys from the button's in-fader label colors.
Existing top-level dial settings migrate without overriding already-saved palette
values. Older user presets receive opaque normal/muted backgrounds, no meter
track, and white dial text. Built-in presets use those same opacity/text defaults;
new actions remain transparent until a preset is applied. Meter style, visibility,
notch and shared/independent appearance are not changed by palette selection.

Four built-in transparent presets are available: standard and VoiceMeeter,
each with and without inactive indicator outlines. They retain their base
palette colors, set both background opacities to zero, and use black inactive
indicators at 62% opacity. Like the other built-ins, they are read-only and can
be copied into a named user preset.

The first built-in palette, Default, is the initial appearance for new dials:
transparent backgrounds, green/red gain bar, #2c3d4d empty rail and inactive
indicators (100%, no outline), and white dial text. Other built-ins are grouped
as white/black then VoiceMeeter, each followed by transparent no-outline and
outlined variants. Saved action palettes are not replaced by the new default.

Monochrome palettes (including transparent variants) use neutral #1b1b1b inactive
meter cells and #a0a0a0 inactive indicator outlines. Default and VoiceMeeter retain
their existing colors. Indicator outline color is editable and saved in shared
palettes; older presets retain the former #91a4b5 outline. Active status and meter
colors remain semantic colors.

Monochrome palettes also use #a6a6a6 for active low-level meter cells. Mid/high,
and Default/VoiceMeeter low-level colors remain unchanged. Muted low-level cells
in monochrome palettes use neutral #424242; muted mid/high colors are retained.

### Single-channel dial touch commands

Tap and hold independently select mute, reset-to-zero, disabled, or a Remote
API script. The script fields are separate from appearance presets and retain
contents when switching operations. Scripts use `VBVMR_SetParametersW` on the
existing worker/connection, support multiple semicolon-separated commands, and
are limited to 4096 UTF-16 code units with no embedded NUL. Empty drafts can be
saved but fail on execution. Editing never executes a script. API errors remain
visible in the PI until a successful command or settings update; return codes
are logged without logging script contents. Commands rejected while disconnected
or belonging to an old connection generation are never replayed. MacroButtons-
specific scripting commands are not implicitly supported; edition-specific
parameters are interpreted by VoiceMeeter. A failed multi-command script may
have applied earlier commands; no rollback or automatic retry is performed.

### Combo channel selection

Combo defaults to touch-selection mode; hold-and-turn remains selectable. In touch-selection mode, tap
selects a half without muting; rotation (including pressed rotation) adjusts the
selected channel, and dial press toggles its mute. Selection is persisted per
action. Holding selects the touched side and runs its independently configured
hold action (default mute toggle for both sides). Arbitrary scripts retain their literal targets.
All eight selection markers A–H are available; G is the short bottom line and
is the default. Markers remain visible when labels/meters are hidden. The
selected channel's palette supplies selectionColor (white in all built-ins).
User palettes save this color; older palettes receive white. Marker shape and
operation mode are not palette fields.

The tap switching method defaults to selecting the touched side. An optional
anywhere-toggle method alternates the selection on each tap, regardless of its
position. Long press always selects the touched side and executes that side's
hold action; it does not toggle first. Saved explicit mode/hold values remain
unchanged when defaults change.

Gain-button backgrounds also honor backgroundOpacity and mutedBackgroundOpacity
from the shared palette, for static and both fader styles. Transparent presets
therefore produce transparent key backgrounds; foreground elements retain their
own colors and opacity.
