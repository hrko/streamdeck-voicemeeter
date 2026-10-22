// Development-only entry. Vite's production input is index.html, not this file.
import { createRoot } from "react-dom/client";
import { I18nextProvider } from "react-i18next";
import { App } from "./App";
import { applyDocumentLocale, catalogs, createLocalization } from "./i18n";
import { defaultSettings } from "./generated/contracts";
import type { StreamDeckPiApi } from "@mikanseilaboratory/streamdeck-pi-client";
import "./style.css";

const params = new URLSearchParams(location.search);
const language = params.get("lang") ?? "en";
const source = { ...catalogs };
if (language === "ar") source.ar = { translation: { ...catalogs.en.translation, "section.left": "القناة اليسرى", "section.right": "القناة اليمنى", "field.gainStep": "مقدار تغيير مستوى الصوت لكل خطوة" }, manifest: {} };
if (language === "long") source.qps = { translation: Object.fromEntries(Object.entries(catalogs.en.translation).map(([key, value]) => [key, `${value} — extended translation for layout testing`])), manifest: {} };
const i18n = await createLocalization(language === "long" ? "qps" : language, source);
applyDocumentLocale(i18n);
document.body.style.maxWidth = `${params.get("width") ?? "360"}px`;
const client: StreamDeckPiApi = {
  ready: true, uuid: "preview", actionInfo: { action: `jp.hrko.streamdeck.voicemeeter.rust.${params.get("action") ?? "gain-control-combo"}`, context: "preview", device: "preview", payload: { settings: structuredClone(defaultSettings) } },
  setSettings: () => {}, getSettings: () => {}, setGlobalSettings: () => {}, getGlobalSettings: () => {}, sendToPlugin: () => {}, subscribe: () => () => {},
};
createRoot(document.getElementById("root")!).render(<I18nextProvider i18n={i18n}><App client={client} /></I18nextProvider>);
