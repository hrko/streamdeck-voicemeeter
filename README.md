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
settings will not be migrated.

## License

The Rust rewrite is source-available under the license in `LICENSE`.
Commercial use (including work and monetized streaming), modification, and
free redistribution are allowed. Selling the plugin or modified versions,
paywalling access, and including them in paid products require prior written
permission from hrko. This is not an OSI-approved open-source license.

Third-party dependencies retain their own licenses. The original Go version
and previously MIT-licensed versions retain their original MIT permissions;
see `licenses/LEGACY-MIT.txt`. The custom terms should receive legal review
before a public release.

## Development status

Tooling and implementation are being introduced incrementally. Setup, checks,
packaging, and installation instructions will be added as those tasks become
available. Do not install this working tree as a Stream Deck plugin yet.
