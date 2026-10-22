import { describe, expect, it } from "vitest";
import { applyDocumentLocale, catalogs, createLocalization, normalizeLocale } from "./i18n";

describe("resource-driven localization", () => {
  it("resolves regional locales and falls back to English for unknown locales", async () => {
    expect((await createLocalization("ja_JP")).t("field.weight")).toBe("太さ");
    expect((await createLocalization("de-DE")).t("field.weight")).toBe("Weight");
    expect(normalizeLocale("not a locale")).toBe("en");
  });
  it("adds a partial locale without changing application logic", async () => {
    const instance = await createLocalization("fr-CA", { ...catalogs, fr: { translation: { "field.weight": "Graisse" }, manifest: {} } });
    expect(instance.t("field.weight")).toBe("Graisse");
    expect(instance.t("field.grade")).toBe("Grade");
    expect(instance.t("status.connected", { edition: "Banana" })).toBe("Connected: Banana");
  });
  it("sets document direction for an additional RTL resource", async () => {
    const instance = await createLocalization("ar", { ...catalogs, ar: { translation: { "field.weight": "الوزن" }, manifest: {} } });
    applyDocumentLocale(instance);
    expect(document.documentElement.dir).toBe("rtl");
    expect(document.documentElement.lang).toBe("ar");
  });
  it("preserves long translations and user-provided names", async () => {
    const long = "A deliberately extended translation ".repeat(10);
    const instance = await createLocalization("fr", { ...catalogs, fr: { translation: { "field.weight": long }, manifest: {} } });
    expect(instance.t("field.weight")).toBe(long);
    expect(instance.t("channel.name", { kind: "My 日本語 microphone", index: 2 })).toBe("My 日本語 microphone 2");
  });
});
