# Go appearance reference

Scope: touch display and Macro keys. The owner explicitly excluded the React
property inspector from this change.

The reference is the Go tree at
`8ab6ae9522423747bd3be7f9a667c82200d583dd`. The two JSON files in
`assets/layouts/` are the original `layouts/gain_controll.json` and
`layouts/gain_controll_combo.json`, renamed only at the filesystem level.
They are copied into the package directly; Draw.io and Go are not build inputs.

| Component | Reference behavior restored |
| --- | --- |
| Gain layout | Icon [16,40,48,48], title [16,10,136,24], gain [112,40,72,24], status [76,40,36,24], meter [76,66,108,5], slider [76,74,108,12] |
| Combo layout | Original left-hand rectangles and a 100px translation for the right side; 20px icons, 84px meters/sliders |
| Text | Native Stream Deck text, size 16, weight 600, original alignments and Combo title fade; Gain includes `dB`, Combo omits it |
| Status | Original edition/physical/virtual strip and bus rows; colored route circles and rounded Mute/Solo/Mono/MC/EQ squares; original active/inactive colors |
| Meter | 1px stereo cells and gaps, original padding, -60..+12 dB range, -24/0 dB color boundaries, 12 dB/s falling peak and current-level marker |
| Gain slider | Original 2px translucent-black border, rounded blue-gray background, green fill up to 0 dB and red above 0 dB |
| Icons | Font em sizing and bearings, original dial sizes and black outlines (Gain alpha 180/255, Combo 128/255); configured font axes retained |
| Macro | Symbol occupies half the key, centered on its configured background; standard ON/OFF state titles restored |

The original screenshot `screenshots/actions.png` corroborates the placement.
The old full-canvas Rust dial renderer has been removed. Dial text is no longer
approximated with SVG/Arial; only icon, meter, slider and status assets use SVG,
each with a single root viewport. Missing-glyph/offline diagnostics remain.

Automated checks cover field/layout agreement, meter/fader dimensions and color
boundaries, edition-specific status geometry, native label strings, locale-aware
gain formatting, and EXE registration/layout/feedback messages. The package is
validated by the official CLI. Actual Stream Deck visual comparison remains
necessary: the pinned bundled fonts and native SVG rasterizer can differ from
the old dynamically downloaded fonts and Go rasterizer in edge antialiasing.
Do not describe pixel-for-pixel parity as verified without that comparison.

Installed candidate EXE SHA-256:
`312f7b9e535e6122f11448882d2ce0ccc9770d04cd457e81ca936a803cb601e7`.
The EXE, both layouts and manifest were replaced and the plugin restarted on
2026-10-23 JST. Owner comparison is recorded below. Rust checks (28 unit + 4 integration),
13 PI tests, 3 script tests and native smoke checks passed; the official CLI
package contains 201 verified files and 102 dependency/asset license entries.

The owner's first comparison confirmed icons, status indicators, gain sliders
and Macro icon size. Gain/left-Combo labels and Macro state titles were absent.
Read-only profile inspection found the early Rust manifest's `ShowTitle: false`
persisted in placed actions. The reserved dial key `title` follows those host
preferences; `title1` does not. New manifest defaults now explicitly show titles
and restore the Go dial title size/alignment. Packaging rejects hidden defaults.
On 2026-10-23, the owner's nine placed Rust states were repaired with byte-for-byte
profile backups, preserving other plugin/profile data and custom dial titles.
Stream Deck was restarted. The owner then confirmed that Gain, left-Combo and
Macro titles appeared and that their size and position looked correct.

## Official built-in layout comparison (2026-10-23)

Compared the shipped JSON with Elgato's published
[built-in layouts](https://docs.elgato.com/streamdeck/sdk/guides/dials/).
The installed Stream Deck 7.0.3 brightness plugin also selects `$B1` in its
manifest. Rectangles below are `[x, y, width, height]`, in pixels.

| Item | Current Go-compatible layout | Official reference | Finding |
| --- | --- | --- | --- |
| Gain title | [16,10,136,24], declared size 16 / weight 600 / left | `$B1`: same | Geometry and declarations match |
| Gain icon | [16,40,48,48] | `$B1`: same | Matches |
| Gain value | [112,40,72,24], size 16 / weight 600 / right | `$B1`: [76,40,108,32], size 24 / weight 600 / right | Same top and right edge; Go reserves 36px for status and uses smaller text |
| Gain slider | [76,74,108,12] | `$B1`: same outer rectangle | Go adds its own border/colors |
| Combo | Two horizontal halves, separate titles and size-16 values | `$C1`: one title and two vertically stacked indicators | Go-specific composition |

The meter and status symbols are additional Go features. Combo titles retain
the standard title row's y=10, height=24 and declared size=16, but use narrower
rectangles alongside their icons. These differences are retained deliberately
to reproduce Go; this comparison does not change the layout.

The [text reference](https://docs.elgato.com/streamdeck/sdk/references/layouts/)
specifies that the reserved `title` item's font and alignment use the user's
title preferences instead of the JSON declarations. Actual title weight is
therefore not guaranteed by `weight: 600`; Combo's non-reserved `title1` uses
its declared style. Changing native title preferences can make the two differ.
Macro ON/OFF uses native key state titles rather than custom SVG text, so its
appearance likewise follows the host's title settings. The dial layout
reference does not prescribe a matching Macro key text rectangle.

This is a definition-level comparison with published layouts, corroborated by
the installed brightness manifest, not a pixel comparison of every stock
plugin. The owner's hardware confirmation covers the restored Go appearance.
Windows CI for `79bed2c` passed (run `39819336073`).

## Aligned gain and status rows (2026-10-25)

The approved readability update retains the fader at y=74 and all label/icon
positions. With meters visible, gain text starts at y=43 and status at y=40.
With meters hidden, gain text starts at y=48 and status at y=45. The status
shapes and gain glyphs have aligned visual bottoms in the preview; host font
rasterization still requires hardware confirmation. Separate native slots are
toggled through feedback because item rectangles cannot change at runtime.
Combo channels choose their slots independently unless their styles are shared.
