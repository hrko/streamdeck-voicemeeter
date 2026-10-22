import { describe, expect, it } from "vitest";
import glyphs from "./generated/glyphs.json";
import { iconCatalogs, searchIcons } from "./icons";

describe("offline icon catalogs", () => {
  it("shows only supported, unique glyphs with audio icons first", () => {
    for (const style of ["outlined", "rounded", "sharp"] as const) {
      const entries = iconCatalogs[style];
      expect(entries.length).toBeGreaterThan(1000);
      expect(entries[0].name).toBe("volume_up");
      expect(new Set(entries.map(entry => entry.codePoint)).size).toBe(entries.length);
      expect(entries.every(entry => glyphs[style].includes(Number.parseInt(entry.codePoint, 16)))).toBe(true);
    }
  });
  it("searches English words, aliases and hexadecimal codes without localized keywords", () => {
    for (const query of ["volume up", "VOLUME_UP", "e050", "U+E050", "0xe050"]) {
      expect(searchIcons("rounded", query).some(entry => entry.name === "volume_up")).toBe(true);
    }
    expect(searchIcons("rounded", "音量")).toEqual([]);
    expect(searchIcons("rounded", "no_such_icon_zzz")).toEqual([]);
    expect(searchIcons("rounded", "  ")).toBe(iconCatalogs.rounded);
  });
});
