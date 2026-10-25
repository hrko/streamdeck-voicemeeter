# Gain Button implementation and validation

Introduced in development version 0.1.6 as `gain-button`, a Keypad action.
The target uses `channels[0].target`; additional settings are under `gainButton`.
Serde defaults and PI restoration keep existing schema-version-1 settings valid.
The existing worker owns all mixer reads/writes, connection generations, gain
limits, deduplicated snapshots and meter decay. Set-gain requests validate both
the edition's target bounds and the finite −60..+12 dB range before writing.

## Reference geometry

The owner supplied a pixel-size screenshot of the built-in Volume Controller.
`src/gain_button.rs` draws a shared 288x144 rail, then clips/transforms it into
one 144x144 key image. At 72px display size, the rail's outer thickness is 14px,
its outline is 3px, and the thumb uses a round white ring with background-colored
separation. The rail centers span x=34..254 in the combined image. Horizontal
negative/positive keys show the left/right half; vertical positive/negative keys
show the upper/lower half. The magnitude of the step never changes geometry.

The visual shape was compared in a browser against the supplied reference.
Native Stream Deck rendering and the physical key gaps still require owner
confirmation; exact raster equivalence is not claimed from a browser preview.
The gain mapping is linear in dB over −60..+12, independent of Windows volume
percentages. The visual midpoint corresponds to −24 dB.

Color and detail options are independent, with all optional detail fields off by
default. Labels are clipped to the key. Vertical labels and values run alongside
the rail; horizontal/static labels and values use the top/bottom areas. Stereo
meters include the current-level marks and reuse the shared decaying peaks.
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
