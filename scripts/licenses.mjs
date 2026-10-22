import { readFile, writeFile, mkdir, readdir, cp, rm, realpath } from "node:fs/promises";
import { createRequire } from "node:module";
import { execFileSync } from "node:child_process";
import path from "node:path";
import { createHash } from "node:crypto";
import { root, destination } from "./build-plugin.mjs";

const output = path.join(root, "dist", "licenses");
if (path.dirname(output) !== path.join(root, "dist")) throw new Error("Unsafe license output");
await rm(output, { recursive: true, force: true });
await mkdir(output, { recursive: true });
const inventory = [];
const allowed = new Set(["MIT", "Apache-2.0", "ISC", "BSD-2-Clause", "BSD-3-Clause", "Unicode-3.0", "Zlib", "Unlicense", "CC0-1.0", "0BSD"]);
function checkLicense(expression, name) {
  const normalized = expression === "MIT/Apache-2.0" && name === "version_check" ? "MIT OR Apache-2.0" : expression;
  if (!normalized || normalized.split(/[\s()]+/).filter(token => token && !["AND", "OR"].includes(token)).some(token => !allowed.has(token))) throw new Error(`Unreviewed license: ${name}: ${expression}`);
  return normalized;
}
async function documents(directory, prefix = "") {
  const found = [];
  for (const item of await readdir(directory, { withFileTypes: true })) {
    if (["node_modules", ".git", "target"].includes(item.name)) continue;
    const relative = path.join(prefix, item.name);
    if (item.isFile() && /^(license|licence|copying|copyright|notice)([._-]|$)/i.test(item.name)) found.push(relative);
    else if (item.isDirectory()) found.push(...await documents(path.join(directory, item.name), relative));
  }
  return found;
}
async function collect({ name, version, license, directory, ecosystem, source }) {
  const expression = checkLicense(license, name);
  const key = `${ecosystem}-${name.replaceAll("/", "-").replaceAll("@", "")}-${version}`;
  const folder = path.join(output, key);
  await mkdir(folder, { recursive: true });
  const files = await documents(directory);
  if (expression.includes("Unicode-3.0") && !files.some(file => /LICENSE-UNICODE$/i.test(file))) throw new Error(`Unicode license missing: ${name}`);
  for (const file of files) {
    await mkdir(path.dirname(path.join(folder, file)), { recursive: true });
    await cp(path.join(directory, file), path.join(folder, file));
  }
  let selectedLicense = expression;
  if (!files.some(file => /^(license|licence|copying)/i.test(path.basename(file)))) {
    const upstream = ["streamdeck-plugin", "streamdeck-plugin-protocol", "streamdeck-plugin-macros"].includes(name) && version === "0.1.0" ? "streamdeck-plugin-LICENSE" : ["ts-rs", "ts-rs-macros"].includes(name) && version === "10.1.0" ? "ts-rs-LICENSE" : undefined;
    if (upstream) {
      const expected = upstream.startsWith("streamdeck") ? "c71d239df91726fc519c6eb72d318ec65820627232b2f796219e87dcf35d0ab4" : "db7f7f8e7236a2d0b41b855b2501b7b913caebbcda6ce498637a304ad1706ce1";
      const document = await readFile(path.join(root, "licenses/upstream", upstream));
      if (createHash("sha256").update(document).digest("hex") !== expected) throw new Error(`Upstream license hash differs: ${name}`);
      const revision = upstream.startsWith("streamdeck") ? "098e79bbe9b56ea7d8f9c9c1f6a3c8b20f1e75b5" : "0ba56be43af1e438053af2b422a82fcff252f7ef";
      const vcs = JSON.parse(await readFile(path.join(directory, ".cargo_vcs_info.json"), "utf8"));
      if (vcs.git.sha1 !== revision) throw new Error(`Unreviewed source revision: ${name}`);
      source = { registry: source, upstreamRevision: revision, licenseSha256: expected, note: "License text supplemented from pinned upstream source." };
      await writeFile(path.join(folder, "LICENSE"), document);
      files.push("LICENSE");
    } else if (["aliri_braid", "aliri_braid_impl"].includes(name) && version === "0.4.0") {
      // Both the crate and pinned upstream omit license texts. Their Cargo
      // declaration lists Apache-2.0 as an alternative; rely on that option and retain the declaration.
      selectedLicense = "Apache-2.0";
      const vcs = JSON.parse(await readFile(path.join(directory, ".cargo_vcs_info.json"), "utf8"));
      if (vcs.git.sha1 !== "c53cc5d7b5e6830e2b0eafa4459ce71953b2a1d1") throw new Error(`Unreviewed source revision: ${name}`);
      source = { registry: source, upstreamRevision: vcs.git.sha1, note: "Complete upstream recursive tree (48 entries) contains no license/notice texts. Apache-2.0 standard text supplemented; original license declaration and author retained." };
      await cp(path.join(root, "assets/fonts/LICENSE.txt"), path.join(folder, "LICENSE-APACHE-2.0"));
      await cp(path.join(directory, "Cargo.toml.orig"), path.join(folder, "UPSTREAM-Cargo.toml"));
      files.push("LICENSE-APACHE-2.0", "UPSTREAM-Cargo.toml");
    } else throw new Error(`License text missing: ${name}@${version}`);
  }
  inventory.push({ ecosystem, name, version, declaredLicense: license, selectedLicense, source, documents: files.map(file => `${key}/${file}`) });
}

const metadata = JSON.parse(execFileSync("cargo", ["metadata", "--locked", "--offline", "--format-version", "1", "--filter-platform", "x86_64-pc-windows-msvc"], { cwd: root, encoding: "utf8", maxBuffer: 16*1024*1024 }));
for (const pkg of metadata.packages.filter(pkg => pkg.source)) {
  await collect({ name: pkg.name, version: pkg.version, license: pkg.license, directory: path.dirname(pkg.manifest_path), ecosystem: "cargo", source: pkg.source });
}
const seen = new Set();
async function resolvePackage(require, name) {
  for (const directory of require.resolve.paths(name) ?? []) {
    try {
      const candidate = await realpath(path.join(directory, name, "package.json"));
      if (JSON.parse(await readFile(candidate, "utf8")).name === name) return candidate;
    } catch {}
  }
  throw new Error(`Cannot resolve runtime license: ${name}`);
}
async function npmPackage(manifestPath) {
  const manifest = JSON.parse(await readFile(manifestPath, "utf8"));
  const key = `${manifest.name}@${manifest.version}`;
  if (seen.has(key)) return;
  seen.add(key);
  await collect({ name: manifest.name, version: manifest.version, license: manifest.license, directory: path.dirname(manifestPath), ecosystem: "npm", source: manifest.repository ?? "npm registry (pnpm-lock.yaml)" });
  const require = createRequire(manifestPath);
  for (const dependency of Object.keys(manifest.dependencies ?? {})) {
    // Resolve package.json by walking from the package entry when exports hide it.
    let next;
    try { next = require.resolve(`${dependency}/package.json`); }
    catch {
      let dir = path.dirname(require.resolve(dependency));
      while (dir !== path.dirname(dir)) {
        try { const candidate = path.join(dir, "package.json"); if (JSON.parse(await readFile(candidate, "utf8")).name === dependency) { next = candidate; break; } } catch {}
        dir = path.dirname(dir);
      }
    }
    if (!next) throw new Error(`Cannot resolve runtime license: ${dependency}`);
    await npmPackage(next);
  }
}
const piManifestPath = path.join(root, "pi/package.json");
const pi = JSON.parse(await readFile(piManifestPath, "utf8"));
const require = createRequire(piManifestPath);
for (const name of Object.keys(pi.dependencies)) await npmPackage(await resolvePackage(require, name));
// Conservatively retain Vite's license for generated runtime helpers too.
const viteManifest = require.resolve("vite/package.json");
const vite = JSON.parse(await readFile(viteManifest, "utf8"));
await collect({ name: vite.name, version: vite.version, license: vite.license, directory: path.dirname(viteManifest), ecosystem: "build-helper", source: vite.repository });
await cp(path.join(root, "assets/fonts"), path.join(output, "material-symbols"), { recursive: true, filter: source => !source.endsWith(".ttf") });
inventory.push({ ecosystem: "font", name: "Material Symbols", version: "49d4db35df873165d6bd6ba09b063c7dafbac2f4", selectedLicense: "Apache-2.0", documents: ["material-symbols/LICENSE.txt", "material-symbols/source.json"] });
await cp(path.join(root, "licenses/LEGACY-MIT.txt"), path.join(output, "LEGACY-MIT.txt"));
inventory.sort((a,b) => `${a.ecosystem}/${a.name}`.localeCompare(`${b.ecosystem}/${b.name}`, "en"));
await writeFile(path.join(output, "inventory.json"), JSON.stringify(inventory, null, 2) + "\n");
const notice = "# Third-party notices\n\nThis inventory conservatively includes Cargo build-time dependencies, PI runtime dependencies, Vite helper notices, and bundled fonts. Inclusion here does not imply every build dependency is linked into the executable. Full original documents are in the accompanying directories. Third-party rights are governed by their respective licenses.\n\n" + inventory.map(p => `- ${p.name} ${p.version}: ${p.selectedLicense}`).join("\n") + "\n";
await writeFile(path.join(output, "THIRD-PARTY-NOTICES.md"), notice);
await cp(output, path.join(destination, "licenses"), { recursive: true });
console.log(`Collected documents for ${inventory.length} dependencies/assets.`);
