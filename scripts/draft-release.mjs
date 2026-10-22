import { readFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import path from "node:path";
import { root, id } from "./build-plugin.mjs";

const version = (await readFile(path.join(root, "Cargo.toml"), "utf8")).match(/^version = "([\d.]+)"/m)?.[1];
const tag = process.env.RELEASE_TAG;
if (!version || tag !== `rust-v${version}`) throw new Error("Release tag must match Cargo.toml exactly");
const filenames = [`${id}.streamDeckPlugin`, `${id}.streamDeckPlugin.sha256`];
const files = await Promise.all(filenames.map(async name => ({ name, bytes: await readFile(path.join(root, "dist", name)) })));
const digest = createHash("sha256").update(files[0].bytes).digest("hex");
if (files[1].bytes.toString().trim() !== `${digest}  ${filenames[0]}`) throw new Error("Release checksum does not match package");
const notes = await readFile(path.join(root, "docs", "release-notes", `${version}.md`), "utf8");
const body = notes + "\n\nThe plugin uses a new ID and requires reconfiguration; old settings are not migrated. Windows x64 and VoiceMeeter installed separately are required. Third-party notices and license texts are included in the plugin archive. See docs/usage.md, docs/hardware-validation.md and LICENSE for installation, validation limits and distribution terms.";
if (process.argv.includes("--dry-run")) {
  console.log(JSON.stringify({ tag, draft: true, assets: files.map(file => ({ name: file.name, bytes: file.bytes.length })), sha256: digest }, null, 2));
} else {
  const repository = process.env.GITHUB_REPOSITORY;
  const commit = process.env.GITHUB_SHA;
  if (!/^[\w.-]+\/[\w.-]+$/.test(repository ?? "") || !/^[a-f0-9]{40}$/.test(commit ?? "")) throw new Error("Missing GitHub repository/commit context");
  if (!process.env.GITHUB_TOKEN) throw new Error("Missing GitHub release token");
  async function api(url, method, bytes, contentType = "application/json") {
    const response = await fetch(url, { method, headers: { Authorization: `Bearer ${process.env.GITHUB_TOKEN}`, Accept: "application/vnd.github+json", "X-GitHub-Api-Version": "2022-11-28", "Content-Type": contentType }, body: bytes, signal: AbortSignal.timeout(120000) });
    if (!response.ok) throw new Error(`GitHub release request failed: ${response.status}`);
    return response.json();
  }
  // Always create a draft. Existing tags/releases are not overwritten or deleted.
  const release = await api(`https://api.github.com/repos/${repository}/releases`, "POST", JSON.stringify({ tag_name: tag, target_commitish: commit, name: `VoiceMeeter (Rust) ${version}`, body, draft: true, prerelease: true }));
  for (const file of files) {
    await api(`${release.upload_url.split("{")[0]}?name=${encodeURIComponent(file.name)}`, "POST", file.bytes, "application/octet-stream");
  }
  console.log(`Draft release prepared: ${release.html_url}`);
}
