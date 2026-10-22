# Development guide

The implementation uses Rust for the plugin and React for the property inspector.
Development takes place on `main` (formerly `rust`). The original Go implementation
is preserved on `legacy-go` and the signed `legacy-go-before-rust` tag at commit
`8ab6ae9522423747bd3be7f9a667c82200d583dd`.

## Build and check

Install mise 2026.10.3 (the CI-pinned version) and Visual Studio Build Tools with
Desktop development with C++ and a Windows SDK, then run:

```powershell
mise trust
mise run setup
mise run check
mise run test
mise run package
```

mise manages local and CI tools; pnpm manages Node packages. Seven-day release-age
policies and committed tool/package lockfiles govern dependency resolution.
See [dependency updates](dependency-updates.md) for policy details and audit commands.

`setup` installs this project's locked Node, pnpm and Rust tools. Task tool
auto-installation is disabled to avoid unrelated global tools. `check` includes
formatting, Clippy, TypeScript, generated contracts and translation consistency.
Run `mise run types` after changing Rust settings types.

`mise run package` creates the `.streamDeckPlugin`, SHA-256 sidecar and complete
third-party license inventory/texts under `dist/`. It validates the manifest and
layout with Elgato's CLI and compares every archived file with staging.
`mise run ci` runs checks, tests and packaging and rejects lockfile changes.

See [implementation status](implementation-status.md), [delivery audit](delivery-audit.md),
[hardware validation](hardware-validation.md), [appearance parity](appearance-parity.md)
and [translation instructions](translating.md) for supporting records.
The custom license should receive legal review before public release; see
[license review](license-review.md). Earlier MIT grants remain documented in
[LEGACY-MIT.txt](../licenses/LEGACY-MIT.txt).

## Local development

The icon picker imports the three upstream `.codepoints` files alongside the
fonts from the same pinned revision. Provenance and SHA-256 hashes are in
`assets/fonts/source.json`; `scripts/icon-catalog.test.mjs` verifies those bytes.
The existing Material Symbols license covers the bundled upstream catalog.
`pi/src/icons.ts` filters entries through generated glyph coverage, merges
same-code-point aliases and orders common audio icons before the remaining names.
Search uses official English names and code points in every locale. Only picker
controls need translation; there is no per-language icon keyword inventory.


`mise run dev` builds debug Rust and PI assets, validates the assembled plugin,
and links it into Stream Deck. It first requests normal plugin shutdown, allowing
held buttons to be released and the Remote API session to log out. It refuses
to forcibly terminate an unresponsive plugin. It then temporarily closes and restarts Stream Deck
to release file locks, so all Stream Deck actions are briefly unavailable.
The standard Windows installation under Program Files is required.
Development output lives in `target/dev-plugin`, separate from release `dist`.
An existing packaged installation is preserved under
`%APPDATA%\Elgato\StreamDeck\PluginBackups\<plugin-id>-<timestamp>.sdPlugin`;
profile settings are not modified. A link to another checkout is rejected.
Host startup launches the plugin once; the task verifies that process rather
than issuing another CLI restart that could bypass normal logout.
Repeated runs reuse this checkout's link. Avoid running development and package
tasks concurrently because both build the PI assets.

On Windows 11 / Stream Deck 7.0.3, conversion from a packaged installation,
a second development build, and subsequent release packaging with the debug
plugin running were verified on 2026-10-23. The CLI's stop response alone did
not release all native process/directory locks; the host restart is required
by this observed environment.
The initial development helper forcibly stopped residual native processes; the
owner subsequently observed exhausted VoiceMeeter connection slots. That path
has been removed. Older binaries without the shutdown event require a one-time
manual upgrade while VoiceMeeter is stopped; do not repeatedly force-stop them.

To return to a packaged release, close Stream Deck, remove only the development
junction at `Plugins\jp.hrko.streamdeck.voicemeeter.rust.sdPlugin` (not its target),
move the saved directory back to that location, and restart Stream Deck.
Alternatively unlink the development installation and install a new package.

`mise run test:native` uses a loopback test WebSocket server and launches the
EXE without opening Stream Deck. It validates registration, rejection of malformed
settings and clean shutdown. Each run gets a new directory under
`target/smoke-runtime`, so old log records cannot satisfy a later check.
Smoke processes set `STREAMDECK_SMOKE_TEST=1` to disable real VoiceMeeter login.
The task also checks the per-process shutdown event while its test WebSocket
remains open. Never set this variable for the installed plugin.

## Prepare a GitHub release

The `Prepare draft release` workflow runs when a `rust-v*` tag is pushed. The
tag must match the Cargo package version, such as `rust-v0.1.0`. The Rust rewrite
now uses the default `main` branch; the `rust-v*` release tag convention remains.
Create a signed release tag only when the desired commit is ready for review,
then push that specific tag. The tag workflow first runs the reusable
Windows build workflow, downloads that run's artifacts, verifies the package
checksum, and creates a draft prerelease with the package and checksum attached.
It does not publish the release. Existing releases are not overwritten; if an
upload fails, inspect the draft before retrying.

Manual dispatch is also supported once GitHub recognizes the workflow. Its UI
requires the workflow on the default branch; see
[GitHub's manual-trigger documentation](https://docs.github.com/en/actions/how-tos/manage-workflows/manually-running-a-workflow).
Pass the same version-matching tag when using manual dispatch. Ordinary branch
pushes run CI only and do not create releases.

Before publishing, review the hardware acceptance record and included license
texts. The current development candidate has not completed hardware acceptance.
For a local check without creating a release, set `RELEASE_TAG` and run
`mise exec node -- node scripts/draft-release.mjs --dry-run` after packaging.
