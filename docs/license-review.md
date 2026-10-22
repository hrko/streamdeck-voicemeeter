# Distribution license review

The single GPT-6 Astra medium reviewer examined the installed lockfile versions
read-only. This record is a collection checklist, not legal clearance.

## Upstream documents missing from published crates

Retrieve and verify licenses and any NOTICE at these exact upstream commits:

| Crates | Repository | Commit |
| --- | --- | --- |
| streamdeck-plugin, streamdeck-plugin-protocol, streamdeck-plugin-macros 0.1.0 | MikanseiLaboratory/StreamDeckPluginRust | 098e79bbe9b56ea7d8f9c9c1f6a3c8b20f1e75b5 |
| aliri_braid, aliri_braid_impl 0.4.0 | neoeinstein/aliri_braid | c53cc5d7b5e6830e2b0eafa4459ce71953b2a1d1 |
| ts-rs, ts-rs-macros 10.1.0 | Aleph-Alpha/ts-rs | 0ba56be43af1e438053af2b422a82fcff252f7ef |

Absence of NOTICE in a crate archive does not prove absence upstream.

## Collection requirements

- Preserve individual copyright notices, not only generic MIT/Apache texts.
- Include libloading's ISC license and Rand crates' COPYRIGHT files.
- unicode-ident declares `(MIT OR Apache-2.0) AND Unicode-3.0`; retain the Unicode
  license when its code/data is distributed. Build-time use alone does not prove
  inclusion in the final executable.
- Record the selected alternative for dual/multiple-license dependencies.
- Normalize version_check's legacy `MIT/Apache-2.0` declaration only as an
  explicitly verified exception.
- Collect PI transitive runtime dependencies including scheduler,
  use-sync-external-store, @babel/runtime and html-parse-stringify, in addition to
  React, i18next, react-i18next and the PI client.
- Check generated JS for bundled helper code from build tools.
- Record actual font files, fixed source revision, hashes, license texts and
  notices once Material Symbols assets are selected.
- Copy the inventory, full texts and applicable notices into the final plugin;
  verify package contents rather than treating a lockfile as a license inventory.

The initial Rust commit and older Go code retain their prior MIT grants. The
current source-available license cannot revoke them. Third-party contributions
need appropriate rights before the project owner can separately license them.

## Material Symbols follow-up

The reviewer matched all three local font blobs and LICENSE.txt to upstream
revision `49d4db35df873165d6bd6ba09b063c7dafbac2f4`, and matched source.json's
SHA-256 values to both source fonts and PI build output. The complete root and
variablefont directory trees contain no NOTICE. This statement concerns those
directories, not unrelated parts of the repository.

The standalone Vite output does not copy license documents. The assembled
plugin build now always runs `scripts/licenses.mjs`, including the font license
and dependency notices and texts listed above.

## Packaging review corrections

The reviewer found that a rebuild could remove previously collected documents,
and five inventory paths included an explanatory suffix rather than a real file
name. Both were corrected. Package validation now checks all document paths,
and archive verification compares every staged file by SHA-256.

Upstream overrides are restricted to exact names/versions and source revisions;
StreamDeck and ts-rs license texts also have expected SHA-256 values. Unicode's
additional license is required independently of MIT/Apache files. The aliri
exception records the inspected source revision, the missing upstream text,
the Apache-2.0 selection and the original Cargo declaration with its author.

Future optional/peer dependencies and actual bundled module coverage still
need auditing. The license-expression allowlist is intentionally not described
as a full SPDX parser.
