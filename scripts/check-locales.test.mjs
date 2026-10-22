import { test } from 'node:test';
import assert from 'node:assert/strict';
import { checkCatalog } from './check-locales.mjs';

test('rejects missing keys, extra keys, and changed interpolation variables', () => {
  const base = { translation: { title: 'Hello {{name}}', body: 'Body' } };
  assert.equal(checkCatalog(base, { translation: { title: 'Bonjour {{user}}', extra: 'Extra' } }, true).errors.length, 3);
  assert.equal(checkCatalog(base, { translation: { title: 'Bonjour {{name}}' } }, false).errors.length, 0);
});
