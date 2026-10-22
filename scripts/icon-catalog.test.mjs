import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { createHash } from "node:crypto";

test("icon name catalogs match pinned font provenance", async () => {
  const directory = new URL("../assets/fonts/", import.meta.url);
  const source = JSON.parse(await readFile(new URL("source.json", directory), "utf8"));
  assert.equal(source.catalogs.length, 3);
  for (const catalog of source.catalogs) {
    assert.ok(catalog.url.includes(source.commit));
    const data = await readFile(new URL(catalog.file, directory));
    assert.equal(createHash("sha256").update(data).digest("hex"), catalog.sha256);
    assert.ok(data.toString("utf8").trim().split(/\r?\n/).every(line => /^[a-z0-9_]+ [a-f0-9]+$/.test(line)));
  }
});
