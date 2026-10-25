import { expect, it } from "vitest";
import { gainPalettes, readPalettes } from "./gainColors";
it("loads older shared palettes without losing meter colors when label color is absent", () => {
  const { labelColor: _label, ...older } = gainPalettes.voicemeeter;
  const items = readPalettes({ gainColorPresets: { version: 1, items: [{ id: "user:old", name: "Old VM", colors: older }] } });
  expect(items?.[0].colors).toMatchObject({ labelColor: "#000000", meterLow: "#85adb9", meterMutedHigh: "#583328" });
  expect(readPalettes({ gainColorPresets: { version: 1, items: [{ id: "user:old", name: "Bad", colors: { ...older, labelColor: "not a color" } }] } })).toBeUndefined();
});
