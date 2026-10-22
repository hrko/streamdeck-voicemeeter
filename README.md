# VoiceMeeter for Stream Deck — Rust rewrite

This branch is an independent Rust reimplementation. It is under development
and is not yet a usable plugin release.

The original Go implementation remains on `main`, at commit
`8ab6ae9522423747bd3be7f9a667c82200d583dd`.

## Target

- Windows x64; VoiceMeeter Basic, Banana, and Potato.
- Gain Control, Gain Control Combo, and Macro actions.
- React property inspectors with Material Symbols customization.
- Japanese and English translations, with resource-based additional locales.
- Local and CI tooling managed by mise; Node dependencies managed by pnpm.
- Seven-day release-age policies and committed tool/package lockfiles.

The new plugin ID is `jp.hrko.streamdeck.voicemeeter.rust`. Existing Go-plugin
settings will not be migrated. The original MIT license is retained.

## Development status

Tooling and implementation are being introduced incrementally. Setup, checks,
packaging, and installation instructions will be added as those tasks become
available. Do not install this working tree as a Stream Deck plugin yet.
