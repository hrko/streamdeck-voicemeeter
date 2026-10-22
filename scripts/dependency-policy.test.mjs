import test from "node:test";
import assert from "node:assert/strict";
import { releaseAge, selectVersion } from "./dependency-policy.mjs";

test("updates exclude recent, unknown-date, prerelease and different-major versions", () => {
  const now = Date.parse("2026-10-23T00:00:00Z");
  const versions = Object.fromEntries(["1.0.0", "1.1.0", "1.2.0", "1.3.0", "1.4.0-beta.1", "2.0.0"].map(v => [v, {}]));
  const time = { "1.0.0": "2026-01-01", "1.1.0": "2026-10-16T00:00:00Z", "1.2.0": "2026-10-16T00:00:01Z", "1.4.0-beta.1": "2026-01-01", "2.0.0": "2026-01-01" };
  assert.equal(selectVersion({ versions, time }, "1.0.0", now), "1.1.0");
  assert.equal(releaseAge(undefined, now), "unknown");
  assert.equal(releaseAge("2027-01-01", now), "too-new");
  assert.equal(selectVersion({ versions: { "1.0.0": {} }, time: {} }, "1.0.0", now), undefined);
});
