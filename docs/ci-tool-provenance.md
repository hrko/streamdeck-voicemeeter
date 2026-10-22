# CI bootstrap provenance

The workflow pins actions by commit. Verified through GitHub's commit API on
2026-10-23 (JST); these commits were older than seven days at selection:

| Action | Version | Commit | Commit date (UTC) |
| --- | --- | --- | --- |
| actions/checkout | v7.0.1 | 3d3c42e5aac5ba805825da76410c181273ba90b1 | 2026-07-17 |
| actions/upload-artifact | v7.0.2 | cf430e030ddbb5b0abf93d22962f4752f3646cd9 | 2026-10-07 |
| jdx/mise-action | v5.1.2 | 08ae2a091c038e1e05ec6f5115dc5c69d5a2a6b8 | 2026-10-12 |
| actions/download-artifact | v5.0.0 | 634f93cb2916e3fdff6788551b99b062d0335ce0 | 2025-08-05 |

mise itself is fixed at 2026.10.3, published 2026-10-05. The action does not
resolve/install tools automatically; setup uses `mise install --locked` for the
three project tools. Experimental features are not enabled. The local
recommended mise version is the same. Updates must review release dates again;
fully pinned versions bypass minimum-release-age resolution filters.

Workflow creation and local `mise run ci` do not establish a successful GitHub
Actions run. Record a remote run separately when this branch is pushed.
