# VoiceMeeter for Stream Deck — Rust rewrite

English | [日本語](README.ja.md)

This branch is an independent Rust reimplementation with installable development
packages. Gain Control, Gain Control Combo and Macro have been exercised on
Windows 11 / Stream Deck + / VoiceMeeter Potato. Release acceptance is still in
progress; see the [hardware validation record](docs/hardware-validation.md) for
confirmed behavior and untested environments.

The Rust implementation is on the default branch, `main` (formerly `rust`).
The original Go implementation remains on `legacy-go` and the signed
`legacy-go-before-rust` tag, at commit
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

These are development artifacts. Normal actions, recovery without replaying
offline input, Push release on page/target changes, shared-target synchronization,
meter decay, Japanese labels and font controls have passed the recorded Potato
checks. Subsequent checks confirmed normal-exit Push release, Karaoke indication,
English/Japanese switching and settings retention. Basic/Banana startup, edition
detection and channel choices have been checked; their full feature matrix is
not verified. Windows 10 and the minimum Stream Deck 6.4 version remain untested.
Extended stability observation is deferred to daily use at the owner's request.
See the [delivery audit](docs/delivery-audit.md) for completed scope and the
[implementation status](docs/implementation-status.md) for verification details.

Installation, action behavior, development linking and log locations are in
[usage and diagnostics](docs/usage.md).
Dependency update policy and audit commands are in
[dependency updates](docs/dependency-updates.md).
