import { readFileSync } from "node:fs";
import { spawnSync } from "node:child_process";

const lock = readFileSync(new URL("../mise.lock", import.meta.url), "utf8");
const version = lock.match(/\[\[tools\.rust\]\]\s+version = "([^"]+)"/)?.[1];
if (!version || !/^\d+\.\d+\.\d+$/.test(version)) throw new Error("Missing exact locked Rust version");
// Existing rustup installations can use a minimal profile, including on CI.
// Install components for the locked toolchain without changing its version.
const result = spawnSync("rustup", ["component", "add", "--toolchain", version, "rustfmt", "clippy"], {
  stdio: "inherit", shell: false, windowsHide: true,
});
if (result.status !== 0) throw new Error(`Rust component installation failed: ${result.error ?? result.status}`);
