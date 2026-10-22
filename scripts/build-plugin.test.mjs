import test from "node:test";
import assert from "node:assert/strict";
import { localizeManifest, id } from "./build-plugin.mjs";

test("additional partial locales produce SDK UUID-keyed resources without application changes", () => {
  const catalog = { manifest: { Name: "اختبار", Actions: { macro: { Name: "زر", Tooltip: "A long translated description" } } } };
  const localized = localizeManifest(catalog);
  assert.equal(localized.Name, "اختبار");
  assert.deepEqual(localized[`${id}.macro`], catalog.manifest.Actions.macro);
  assert.equal(localized.Actions, undefined);
  assert.equal(localized.Description, undefined); // SDK retains English manifest fallback.
});
