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
at canonical y=97 and stereo lanes at y=39/49 (swapped: y=47 and y=89/99).
The meter center and rail center lie at 47/97 around the key center 72; the
10px display-size clearance is between the meter background and thumb ring.
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
overlaid by the thumb. `labelColor` is a palette field; older user palettes
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
The thumb covers the channel label. With a meter, track centers move to 43/101
(canonical 144px coordinates), preserving symmetric axes and 10px clearance.
Without gain text the original geometry is unchanged. Numbers are clipped to
the inner circle; pixel tests verify that removing that clip reveals no missing
glyphs for −60, −24, −12.5, 0, +6 and +12. Key-boundary splitting remains visible.
