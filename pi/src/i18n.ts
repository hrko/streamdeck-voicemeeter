import { createInstance, type Resource } from "i18next";
import { initReactI18next } from "react-i18next";

export interface Catalog {
  translation: Record<string, string>;
  manifest: Record<string, unknown>;
}

const files = import.meta.glob<Catalog>("./locales/*.json", { eager: true, import: "default" });
export const catalogs = Object.fromEntries(Object.entries(files).map(([path, catalog]) => [path.split("/").at(-1)!.replace(/\.json$/, ""), catalog]));

export function normalizeLocale(locale: string): string {
  try { return Intl.getCanonicalLocales(locale.replaceAll("_", "-"))[0] ?? "en"; }
  catch { return "en"; }
}

export async function createLocalization(locale: string, source = catalogs) {
  const instance = createInstance();
  const resources: Resource = Object.fromEntries(Object.entries(source).map(([lang, catalog]) => [lang, { translation: catalog.translation }]));
  await instance.use(initReactI18next).init({
    resources, lng: normalizeLocale(locale), fallbackLng: "en", load: "all",
    keySeparator: false, interpolation: { escapeValue: false }, initAsync: false,
  });
  return instance;
}

export function applyDocumentLocale(instance: Awaited<ReturnType<typeof createLocalization>>) {
  const locale = instance.resolvedLanguage ?? "en";
  document.documentElement.lang = locale;
  document.documentElement.dir = instance.dir(locale);
}
