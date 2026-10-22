import { createRoot } from "react-dom/client";
import { I18nextProvider } from "react-i18next";
import { StreamDeckPiClient } from "@mikanseilaboratory/streamdeck-pi-client";
import { App } from "./App";
import { createLocalization, applyDocumentLocale } from "./i18n";
import "./style.css";

const root = createRoot(document.getElementById("root")!);
const client = new StreamDeckPiClient();
let localization = await createLocalization("en");
function render() {
  applyDocumentLocale(localization);
  document.title = localization.t("app.name");
  root.render(<I18nextProvider i18n={localization}>{client.ready ? <App client={client} /> : <p role="status">{localization.t("status.connecting")}</p>}</I18nextProvider>);
}
client.subscribe("ready", render);
render();

// The library owns registration and the WebSocket; only host-language handling
// is added here because 0.1.0 ignores the connection's info argument.
(window as Window & { connectElgatoStreamDeckSocket?: typeof client.connect }).connectElgatoStreamDeckSocket = (port, uuid, event, info, actionInfo) => {
  let language = "en";
  try { language = JSON.parse(info).application?.language ?? "en"; } catch { /* English fallback for malformed host data. */ }
  void createLocalization(language).then(instance => { localization = instance; render(); });
  client.connect(port, uuid, event, info, actionInfo);
};
