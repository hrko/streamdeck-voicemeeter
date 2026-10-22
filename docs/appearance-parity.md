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
2026-10-23 JST. Owner comparison is pending. Rust checks (28 unit + 4 integration),
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
Stream Deck was restarted. Remaining text appearance confirmation is pending.
