import { lstat, realpath, rename, mkdir } from "node:fs/promises";
import { spawnSync } from "node:child_process";
import path from "node:path";
process.env.STREAMDECK_DEV = "1";
const { root, id, destination } = await import("./build-plugin.mjs");

if (process.platform !== "win32" || !process.env.APPDATA) {
  throw new Error("Development installation requires Windows and APPDATA");
}
const installed = path.resolve(process.env.APPDATA, "Elgato", "StreamDeck", "Plugins", `${id}.sdPlugin`);
const stat = await lstat(installed).catch(error => {
  if (error.code === "ENOENT") return null;
  throw error;
});
if (stat?.isSymbolicLink() &&
    (await realpath(installed)).toLowerCase() !== destination.toLowerCase()) {
  throw new Error(`Another development checkout is linked at ${installed}; unlink it first.`);
}
function run(command, ...args) {
  const result = spawnSync("mise", ["exec", "node", "pnpm", "rust", "--", command, ...args],
    { cwd: root, stdio: "inherit", shell: false, windowsHide: true });
  if (result.status !== 0) throw new Error(`${command} failed: ${result.error ?? result.status}`);
}
const cli = (...args) => run("pnpm", "exec", "streamdeck", ...args);
let backup;
let hostStopped = false;
try {
  // Stop before replacing the staged EXE, which may be the running linked EXE.
  if (stat) {
    run("pwsh", "-NoProfile", "-File", "scripts/stop-dev-plugin.ps1", "-PluginDirectory", installed);
  }
  // Stop the host only after the native worker has logged out.
  run("pwsh", "-NoProfile", "-File", "scripts/dev-host.ps1", "-Mode", "Stop");
  hostStopped = true;
  run("cargo", "build", "--locked", "--bin", "streamdeck-voicemeeter");
  run("pnpm", "--filter", "@streamdeck-voicemeeter/pi", "exec", "vite", "build");
  run("node", "scripts/build-plugin.mjs", "debug");
  run("node", "scripts/licenses.mjs");
  run("node", "scripts/verify-package.mjs");
  cli("validate", "--no-update-check", destination);
  if (stat && !stat.isSymbolicLink()) {
    // Keep the installed release outside Plugins; never touch ProfilesV2.
    const backups = path.resolve(process.env.APPDATA, "Elgato", "StreamDeck", "PluginBackups");
    await mkdir(backups, { recursive: true });
    backup = path.join(backups, `${id}-${Date.now()}.sdPlugin`);
    await rename(installed, backup);
    console.log(`Previous installation preserved at ${backup}`);
  }
  cli("link", destination);
  run("pwsh", "-NoProfile", "-File", "scripts/dev-host.ps1", "-Mode", "Start");
  hostStopped = false;
  // Starting the host already launches the plugin. A second CLI restart can
  // terminate that new process without giving its worker time to log out.
  run("pwsh", "-NoProfile", "-File", "scripts/wait-dev-plugin.ps1");
} catch (error) {
  // Restore a moved release only if link failed without creating a destination.
  if (backup && !await lstat(installed).catch(e => {
    if (e.code === "ENOENT") return null;
    throw e;
  })) await rename(backup, installed);
  throw error;
} finally {
  if (hostStopped) run("pwsh", "-NoProfile", "-File", "scripts/dev-host.ps1", "-Mode", "Start");
}
