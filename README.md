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
and initial Rust commit `6d25302` retain their original MIT permissions;
see `licenses/LEGACY-MIT.txt`. The custom terms should receive legal review
before a public release.

## Development status

Install mise 2026.10.3 (the CI-pinned version) and Visual Studio Build Tools (Desktop development
with C++ and a Windows SDK), then run:

```powershell
mise trust
mise run setup
mise run check
mise run test
mise run package
```

`setup` installs only this project's locked Node, pnpm, and Rust tools. Task
auto-installation is disabled so unrelated global mise tools cannot interfere.
`check` includes generated TypeScript contract and locale consistency checks.
Run `mise run types` after changing Rust settings. See
[translation instructions](docs/translating.md) for additional languages.

`mise run package` creates the `.streamDeckPlugin`, SHA-256 checksum, and full
third-party license inventory under `dist/`. It validates the manifest and
layout with Elgato's CLI and compares every archived file with its staged source.
The plugin contains no VoiceMeeter DLL. `mise run ci` runs checks, tests and this
same packaging flow and rejects lockfile changes.

These are development artifacts, not a hardware-validated release. Runtime
integration, visual checks, and hardware acceptance remain in progress;
see [implementation status](docs/implementation-status.md).

Installation, action behavior, development linking and log locations are in
[usage and diagnostics](docs/usage.md).
Dependency update policy and audit commands are in
[dependency updates](docs/dependency-updates.md).
