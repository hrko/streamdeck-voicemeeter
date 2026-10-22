# Adding a language

Add a UTF-8 JSON file named after a BCP 47 language tag in `pi/src/locales/`,
for example `fr.json` or `pt-BR.json`. Copy the structure of `en.json`.
Vite discovers these files automatically; no component or action changes are
needed. Use `translation` for PI strings and `manifest` for Stream Deck's plugin
and action descriptions. Preserve keys and interpolation variables such as
`{{edition}}`. Translate whole messages rather than joining fragments.

English and Japanese must contain every key. Other languages can be partial:
missing messages fall back to English and the validation task reports them.
Extra keys and mismatched interpolation variables fail validation in every
language. Regional locales resolve to their base language before English.
Never translate saved setting keys, code points, identifiers, or user labels.

Run `mise run check:locales` and `mise run test:pi`. Exercise long translations
in the actual PI, including input labels, errors, and collapsed details. RTL
languages use the document direction; styles must use logical properties.
System fonts provide UI text. If a language needs additional bundled fonts,
record their origin, exact version, copyright, and redistribution license.

The additional French/Arabic test catalogs exist only in tests and are not
shipped or advertised as supported translations.
