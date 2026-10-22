# PI visual checks

Checked on 2026-10-23 (JST) in the in-app Chromium browser using the Vite
development-only `preview.html` entry. This is not a Stream Deck embedded-browser
or hardware acceptance result.

- Japanese Combo at 360 px content width: separate channel forms, actual bundled
  glyphs, wrapped help text, and expanded variable-font controls render correctly.
- Setting the gain step to zero shows a Japanese validation error next to the
  field. Expanding font details and pressing Tab focuses the style selector.
- Macro at 300 px with a synthetic Arabic locale: rows and disclosure controls
  follow RTL direction; ON/OFF icon colors remain independent. Numeric, hex and
  color input values explicitly use LTR direction.
- Synthetic long translations at 300 px: labels, legends, help and disclosure
  captions wrap without overlapping controls. Native select labels clip within
  the closed control; their full options remain available in the dropdown.
- Japanese Macro at 360 px: both states display the bundled symbols and separate
  configured backgrounds.

Run a Vite dev server and open `/preview.html?lang=ja&action=macro&width=360`.
Other test choices: `lang=ar`, `lang=long`, and `action=gain-control-combo`.
Synthetic resources exist only in the preview entry. Package validation rejects
the preview HTML and its long-translation marker in production JavaScript.

The owner subsequently confirmed touch-strip rendering, Japanese labels and
font-detail controls in Stream Deck 7.0.3 on Windows 11, plus corrected falling
peak meters on Gain/Combo. See hardware-validation.md for the exact tested build.
Validation on the minimum supported Stream Deck/CEF version remains pending.
The browser-only long-text/RTL
checks above do not establish embedded-browser results on other versions.

The owner subsequently confirmed embedded PI Tab navigation/editing, restoration
of the last valid step after leaving and returning from invalid input, and the
missing-codepoint warning/fallback. The built-in inspector comparison was broadly
acceptable except for the text font: Japanese kana appeared wider. The PI root
font was changed from Arial/Segoe UI to `system-ui, sans-serif` in response.
The owner confirmed that the corrected font appearance is acceptable.
