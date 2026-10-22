import { readFile, writeFile } from "node:fs/promises";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { root } from "./build-plugin.mjs";
import { releaseAge, selectVersion } from "./dependency-policy.mjs";

const audit = process.argv.includes("--audit");
const report = { checkedAt: new Date().toISOString(), minimumReleaseAge: "7d", mode: audit ? "audit" : "update", records: [], cargoLibraryPolicy: "Cargo library resolution is not governed by mise minimum_release_age. Cargo updates require separate review.", result: "in-progress" };
function run(command, args) {
  const result = spawnSync(command, args, { cwd: root, stdio: "inherit", shell: false, windowsHide: true });
  if (result.status !== 0) throw new Error(`${command} failed: ${result.error ?? result.status}`);
}
const tool = (command, args) => run("mise", ["exec", "node", "pnpm", "rust", "--", command, ...args]);
async function request(url, format = "json") {
  const response = await fetch(url, { headers: { "User-Agent": "streamdeck-voicemeeter-dependency-review" }, signal: AbortSignal.timeout(30000) });
  if (!response.ok) throw new Error(`${response.status}: ${url}`);
  return format === "json" ? response.json() : response.text();
}
function record(name, version, publishedAt, source) {
  const status = releaseAge(publishedAt);
  report.records.push({ name, version, publishedAt: publishedAt ?? null, status, source });
  if (status !== "eligible") throw new Error(`${name}@${version}: release age ${status}; review required`);
}
try {
  if (!audit) run("mise", ["lock", "--bump", "--platform", "windows-x64", "--minimum-release-age", "7d", "node", "pnpm", "rust"]);
  const lock = await readFile(path.join(root, "mise.lock"), "utf8");
  const version = name => {
    const found = lock.match(new RegExp(`\\[\\[tools\\.${name}\\]\\]\\s+version = "([^"]+)"`));
    if (!found) throw new Error(`Missing locked tool: ${name}`);
    return found[1];
  };
  const nodeSource = "https://nodejs.org/dist/index.json";
  const node = (await request(nodeSource)).find(item => item.version === `v${version("node")}`);
  record("node", version("node"), node?.date, nodeSource);
  const pnpmSource = "https://registry.npmjs.org/pnpm";
  record("pnpm", version("pnpm"), (await request(pnpmSource)).time[version("pnpm")], pnpmSource);
  const rustSource = `https://static.rust-lang.org/dist/channel-rust-${version("rust")}.toml`;
  record("rust", version("rust"), (await request(rustSource, "text")).match(/^date = "([^"]+)"/m)?.[1], rustSource);
  const bootstrapVersion = "2026.10.3";
  const miseSource = `https://api.github.com/repos/jdx/mise/releases/tags/v${bootstrapVersion}`;
  record("mise-bootstrap", bootstrapVersion, (await request(miseSource)).published_at, miseSource);
  const manifests = ["package.json", "pi/package.json"];
  for (const filename of manifests) {
    const file = path.join(root, filename);
    const manifest = JSON.parse(await readFile(file, "utf8"));
    if (filename === "package.json") {
      if (audit && manifest.packageManager !== `pnpm@${version("pnpm")}`) throw new Error("packageManager differs from mise.lock");
      manifest.packageManager = `pnpm@${version("pnpm")}`;
    }
    for (const section of ["dependencies", "devDependencies"]) {
      for (const [name, current] of Object.entries(manifest[section] ?? {})) {
        if (!/^\d+\.\d+\.\d+$/.test(current)) throw new Error(`Dependency is not exact: ${name}: ${current}`);
        const source = `https://registry.npmjs.org/${encodeURIComponent(name)}`;
        const metadata = await request(source);
        // PI-client's reviewed API version remains an explicit adoption decision.
        const next = audit || name === "@mikanseilaboratory/streamdeck-pi-client" ? current : selectVersion(metadata, current);
        if (!next) { report.records.push({ name, status: "unknown", source }); throw new Error(`No age-eligible version: ${name}`); }
        record(name, next, metadata.time[next], source);
        manifest[section][name] = next;
      }
    }
    if (!audit) await writeFile(file, JSON.stringify(manifest, null, 2) + "\n");
  }
  if (!audit) {
    run("mise", ["install", "--locked", "node", "pnpm", "rust"]);
    tool("pnpm", ["install"]); // Workspace minimumReleaseAge also constrains transitive resolution.
    tool("cargo", ["update"]); // Intentionally separate from tool and npm release-age policies.
    tool("cargo", ["run", "--locked", "--bin", "typegen"]);
  }
  report.result = "passed";
} catch (error) {
  report.result = "failed";
  report.error = String(error);
  process.exitCode = 1;
  console.error(error.message);
} finally {
  await writeFile(path.join(root, "docs/dependency-age-report.json"), JSON.stringify(report, null, 2) + "\n");
}
