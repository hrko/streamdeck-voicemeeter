import { readdir, readFile, stat, writeFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import path from "node:path";
import { root, destination, id } from "./build-plugin.mjs";

async function walk(directory) {
  const files = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const full = path.join(directory, entry.name);
    if (entry.isSymbolicLink()) throw new Error(`Symlink in distribution: ${full}`);
    if (entry.isDirectory()) files.push(...await walk(full)); else files.push(full);
  }
  return files;
}
const files = await walk(destination);
if (files.some(file => path.basename(file) === "preview.html")) throw new Error("Development preview included in package");
for (const file of files.filter(file => file.endsWith(".js"))) {
  if ((await readFile(file, "utf8")).includes("extended translation for layout testing")) throw new Error("Test locale included in production JavaScript");
}
if (files.some(file => /\.dll$/i.test(file))) throw new Error("DLLs must not be distributed");
for (const required of ["manifest.json", "bin/streamdeck-voicemeeter.exe", "ui/index.html", "layouts/channel.json", "en.json", "ja.json", "LICENSE", "licenses/THIRD-PARTY-NOTICES.md", "licenses/inventory.json"]) {
  if (!(await stat(path.join(destination, required))).size) throw new Error(`Empty package file: ${required}`);
}
const inventory = JSON.parse(await readFile(path.join(destination, "licenses/inventory.json"), "utf8"));
for (const entry of inventory) for (const document of entry.documents) {
  const resolved = path.resolve(destination, "licenses", document);
  const relative = path.relative(path.join(destination, "licenses"), resolved);
  if (relative.startsWith("..") || path.isAbsolute(relative)) throw new Error("License path escapes package");
  if (!(await stat(resolved)).size) throw new Error(`Missing license: ${entry.name}: ${document}`);
}
const source = JSON.parse(await readFile(path.join(root, "assets/fonts/source.json"), "utf8"));
for (const font of source.files) {
  const matching = files.filter(file => path.basename(file).startsWith(font.file.replace(".ttf", "-")) && file.endsWith(".ttf"));
  if (matching.length !== 1) throw new Error(`Expected exactly one bundled ${font.file}`);
  const hash = createHash("sha256").update(await readFile(matching[0])).digest("hex");
  if (hash !== font.sha256) throw new Error(`Font hash differs: ${font.file}`);
}
if (process.argv.includes("--checksum")) {
  const archive = path.join(root, "dist", `${id}.streamDeckPlugin`);
  const digest = createHash("sha256").update(await readFile(archive)).digest("hex");
  await writeFile(`${archive}.sha256`, `${digest}  ${path.basename(archive)}\n`);
}
console.log(`Verified ${files.length} staged files and ${inventory.length} license entries.`);
