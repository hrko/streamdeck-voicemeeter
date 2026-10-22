import { readFile, writeFile, mkdir, cp, rm, readdir } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { pluginPng } from "./png.mjs";

export const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
export const id = "jp.hrko.streamdeck.voicemeeter.rust";
export const destination = path.join(root, "dist", `${id}.sdPlugin`);
const json = async file => JSON.parse(await readFile(file, "utf8"));
const writeJson = (file, value) => writeFile(file, JSON.stringify(value, null, 2) + "\n");

export function localizeManifest(catalog) {
  const { Actions, ...metadata } = catalog.manifest;
  return { ...metadata, ...Object.fromEntries(Object.entries(Actions ?? {}).map(([action, value]) => [`${id}.${action}`, value])) };
}

export async function buildPlugin(profile = "release") {
  if (!["debug", "release"].includes(profile)) throw new Error("Unsupported build profile");
  const dist = path.join(root, "dist");
  if (path.dirname(destination) !== dist || path.basename(destination) !== `${id}.sdPlugin`) throw new Error("Unsafe output path");
  // Delete only this generated plugin directory, after checking its absolute path.
  await rm(destination, { recursive: true, force: true });
  await mkdir(path.join(destination, "bin"), { recursive: true });
  await mkdir(path.join(destination, "layouts"), { recursive: true });
  await mkdir(path.join(destination, "imgs"), { recursive: true });
  await cp(path.join(root, "target", profile, "streamdeck-voicemeeter.exe"), path.join(destination, "bin", "streamdeck-voicemeeter.exe"));
  await cp(path.join(root, "pi", "dist"), path.join(destination, "ui"), { recursive: true });
  await cp(path.join(root, "LICENSE"), path.join(destination, "LICENSE"));
  const english = await json(path.join(root, "pi/src/locales/en.json"));
  const version = (await readFile(path.join(root, "Cargo.toml"), "utf8")).match(/^version = "([\d.]+)"/m)?.[1];
  if (!version) throw new Error("Missing package version");
  const actions = Object.entries(english.manifest.Actions).map(([suffix, text]) => ({
    ...text, UUID: `${id}.${suffix}`, Icon: "imgs/action", PropertyInspectorPath: "ui/index.html",
    Controllers: [suffix === "macro" ? "Keypad" : "Encoder"],
    DisableAutomaticStates: true,
    States: Array.from({ length: suffix === "macro" ? 2 : 1 }, () => ({ Image: "imgs/key", ShowTitle: false })),
    ...(suffix === "macro" ? {} : { Encoder: { layout: "layouts/channel.json" } }),
  }));
  await writeJson(path.join(destination, "manifest.json"), {
    $schema: "https://schemas.elgato.com/streamdeck/plugins/manifest.json",
    Name: english.manifest.Name, Description: english.manifest.Description, Category: english.manifest.Name,
    Author: "hrko", Version: `${version}.0`, UUID: id, SDKVersion: 2,
    CodePath: "bin/streamdeck-voicemeeter.exe", CodePathWin: "bin/streamdeck-voicemeeter.exe",
    Software: { MinimumVersion: "6.4" }, OS: [{ Platform: "windows", MinimumVersion: "10" }],
    Icon: "imgs/plugin", CategoryIcon: "imgs/action", Actions: actions,
    URL: "https://github.com/hrko/streamdeck-voicemeeter",
  });
  await writeJson(path.join(destination, "layouts/channel.json"), {
    $schema: "https://schemas.elgato.com/streamdeck/plugins/layout.json",
    id: "voicemeeter-channel", controller: "Encoder", items: [{ key: "canvas", type: "pixmap", rect: [0, 0, 200, 100] }],
  });
  for (const file of await readdir(path.join(root, "pi/src/locales"))) {
    if (!file.endsWith(".json")) continue;
    const catalog = await json(path.join(root, "pi/src/locales", file));
    await writeJson(path.join(destination, file.replaceAll("-", "_")), localizeManifest(catalog));
  }
  for (const [name, size] of [["action", 20], ["key", 72], ["plugin", 128]]) {
    for (const scale of [1, 2]) {
      const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${size * scale}" height="${size * scale}" viewBox="0 0 72 72"><rect width="72" height="72" rx="12" fill="#202020"/><path d="M12 36h8l5-19 10 38 10-38 5 19h10" fill="none" stroke="#60d394" stroke-width="4" stroke-linejoin="round"/></svg>`;
      await writeFile(path.join(destination, "imgs", `${name}${scale === 2 ? "@2x" : ""}.svg`), svg);
      if (name === "plugin") await writeFile(path.join(destination, "imgs", `${name}${scale === 2 ? "@2x" : ""}.png`), pluginPng(size * scale));
    }
  }
  console.log(`Built ${destination}`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await buildPlugin(process.argv[2]);
