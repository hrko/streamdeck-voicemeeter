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
The new `swapMeter` field defaults to false for existing settings. Meter-enabled
faders use separate tracks: canonical y=48 for the rail and y=98/108 for the
stereo lanes, or y=96 and y=30/40 when swapped. Both lanes span the rounded rail's outer cap extents across
the shared pair (x=20.5..264 horizontally, x=22.5..266 before vertical rotation) and use the same clipping/rotation as the fader. Pixel tests
verify seam continuity and non-overlapping tracks for all four combinations.
The meter includes current-level markers and shared decaying peaks.

Only fader styles render dB and meters; static styles retain but ignore those
preferences. Fader labels are SVG-clipped. Static channel names instead use
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
