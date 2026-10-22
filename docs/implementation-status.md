# Implementation status

This is a progress record, not a release acceptance certificate.

## Verified

- Independent root commit on `rust`; old Go files removed and retained on `main`
  at `8ab6ae9522423747bd3be7f9a667c82200d583dd`.
- Repository-local commit signing disabled at the owner's request.
- mise 2026.10.3, Windows x64 tool lock: Node 24.22.0, pnpm 11.28.6,
  Rust 1.99.0. Seven-day resolution delay; Rust 1.99.0 published 2026-10-01.
- Cargo and pnpm lockfiles generated from upstream registries. Frozen pnpm
  install passes its supply-chain policy checks.
- `mise run setup`: locked tools, dependencies, MSVC and Windows SDK detected.
- `mise run check`: Rust formatting, Clippy (warnings denied), generated TS
  contract consistency, TS type checking, en/ja locale consistency.
- `mise run test`: 15 Rust tests; subsequent PI run: 12 PI/localization tests
  and 1 locale-validator test.
- React PI production bundle builds. Shared channel/icon forms, independent
  Combo settings, Macro controls, explicit invalid-setting reset, translated
  status/errors and local standard-style CSS are implemented. Form interaction
  tests cover invalid-input suppression, independent values and settings refresh.
- Rust-generated defaults and PI validation preserve offline targets, reject
  malformed stored values, and restore missing nested fields without guessing.
- Subsequent `mise run test:rust` and `check:rust`: 15 Rust tests and static
  checks pass with the real API adapter and SDK action implementations.
- Raw API adapter retains login across outages, reads UTF-16 names, maps meter
  offsets by edition, and bridges asynchronous writes with expiring pending values.
- Gain, Combo and Macro SDK callbacks dispatch through the shared worker;
  settings updates and disappearance release held MacroButtons.
- Backend boundary and session engine with deduplicated target snapshots,
  generation checks, lifecycle macro release and fake-backend tests.
- Dedicated worker with 10 ms polling, 15 fps snapshots, bounded command queue
  and 1–10 second reconnect backoff compiles and passes Clippy.
- Additional locale resources require no application code changes; regional
  fallback, partial translation fallback, long strings and RTL direction tested.
- One GPT-6 Astra medium subagent reviewed the source-available license.
  Clarified same-license redistribution and the initial Rust MIT grant.

## Required before release

- Real VoiceMeeter adapter and worker integration tests. Review asynchronous
  gain writes against the SDK cache before claiming rapid-input correctness on
  hardware. Verify per-target API errors, reconnect recovery and queue saturation.
- SDK action registration, lifecycle cleanup, settings and PI messages, event
  integration tests, real WebSocket registration.
- Bundled fixed Material Symbols fonts with source/version/hash/licenses;
  variable-axis SVG rendering, icon cache, labels/meters/status indicators.
- React forms using streamdeck-pi-client, saved settings, validation, previews,
  connection state, standard-PI styling and visual/keyboard checks.
- Build-time manifest localization, package manifest/layouts/icons, direct EXE
  startup, bounded structured logs.
- Remaining mise tasks: dev, build, licenses, package, ci, deps:update.
- GitHub Actions with SHA-pinned actions, fixed mise bootstrap, locked installs,
  complete checks, release artifacts/checksums.
- Exact runtime/transitive dependency and asset license inventory; required
  full texts, notices, modification notices and distribution verification.
- Clean-install verification and release-age policy verification for updates.
- Real Stream Deck PI and hardware, VoiceMeeter Basic/Banana/Potato behavior,
  restarts, offline behavior, overlapping actions, long labels, meter performance.
- Record tested Windows/Stream Deck versions; do not claim untested support.

## Implementation notes

The adapter uses voicemeeter 0.4.0's existing raw bindings because its high-level
handle logs out when dropped and cannot log in again in that process. Login is
retained across disconnected states; logout occurs only at process shutdown.
The wide-string API preserves non-ASCII names. SDK source is in the Cargo registry.

The PI client's provider installs the Stream Deck connect hook and owns one
WebSocket. Its `connect` currently ignores host `info`, so capture locale from
that argument in an adapter while delegating the original connection function.
Its preconnection placeholder is hardcoded English; provide a localized loading
gate without creating another socket. Its settings hook shallow-merges settings;
normalize nested defaults and validate before saving.

The `streamdeck` CLI does not accept `--version`; use its package manifest for
version inspection. A failed version flag is not a packaging failure.

Custom license review is not a legal opinion. Final distribution must retain
third-party rights and earlier MIT grants. Third-party contributions may require
an explicit rights agreement before hrko can separately license those parts.
