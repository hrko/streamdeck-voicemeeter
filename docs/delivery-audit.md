# Rust rewrite delivery audit

Recorded on 2026-10-23. Implementation and preparation of the GitHub distribution
candidate are complete. Version 0.1.5 was subsequently published as a prerelease
at the owner's request on 2026-10-23. Marketplace submission has not been
performed. This record does not certify every supported environment.

| Planned area | Evidence |
| --- | --- |
| Independent Rust branch and legacy preservation | `main` branch (originally `rust`); original Go commit `8ab6ae9522423747bd3be7f9a667c82200d583dd` retained on `legacy-go` and signed tag `legacy-go-before-rust` |
| Locked local/CI tools and seven-day update policies | `mise.toml`, `mise.lock`, `pnpm-workspace.yaml`, dependency policy tests; isolated clean-install and update results in `implementation-status.md` |
| Three actions, worker lifecycle, settings and generated types | Rust unit and SDK/worker tests; owner hardware results in `hardware-validation.md` |
| React PI, customization and additional languages | PI and localization tests; `translating.md`; owner-confirmed Japanese/English, keyboard, validation and font controls |
| Go display appearance and meters | `appearance-parity.md`; owner-confirmed layouts, titles, meter decay and shared targets |
| Licensing and bundled assets | `license-review.md`; packaged 102-entry inventory and license texts; no VoiceMeeter DLL |
| Distribution and diagnostics | `usage.md`; signed `rust-v0.1.5`; successful Windows release workflow `39843231127`; downloaded package independently verified against sidecar and GitHub digest |

The 0.1.5 package SHA-256 is
`2ef6d3197937b5288aa12d891882bba8c2282c43fc44dbcb0f49305173a7a34a`.
Its GitHub release is now a public prerelease. The existing installed development build
contains the accepted fixes; the release version bump did not require another
hardware restart.

## Verification limits

The main hardware coverage is Windows 11, Stream Deck + (firmware 2.0.3.2),
and VoiceMeeter Potato. The owner confirmed the final normal-exit Push release,
Mono-cyan Karaoke indicator, and corrected Basic/Banana channel choices.
Basic/Banana startup and edition detection were also checked, but their full
feature matrix was not. Windows 10 and Stream Deck 6.4 remain untested.
Invalid-setting rejection has automated coverage; the entire hardware
invalid-target/status-flag matrix has not been exercised.

Extended stability observation was explicitly deferred to daily use by the
owner and is not claimed as passed. These limits remain visible in the hardware
record and release notes, as required by the plan. They limit compatibility
claims rather than blocking preparation of this development distribution.

When VoiceMeeter is restarted during a held Push, an old-session release is
not replayed into the new session. A retained ON state therefore requires a
fresh press/release. Normal Stream Deck exit while VoiceMeeter is available
releases the held target before logout; this was confirmed by the owner.

The later owner-requested source-available license supersedes the plan's
initial MIT choice for new work. Earlier MIT grants and third-party licenses
remain intact. The license review is not a legal opinion.
