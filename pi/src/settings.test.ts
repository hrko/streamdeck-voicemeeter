import { describe, expect, it } from "vitest";
import { defaultSettings } from "./generated/contracts";
import { restoreSettings, validateSettings } from "./settings";

describe("settings contract", () => {
  it("migrates dial display defaults and rejects invalid new values", () => {
    const restored=restoreSettings({dial:{styles:[{},{}]}});
    expect(restored.settings?.dial.styles[0]).toMatchObject({meterStyle:"cells",meterTrackOpacity:0,palette:{indicatorOutline:false,indicatorBackground:"#808080",indicatorOpacity:100}});
    expect(restoreSettings({dial:{styles:[{indicatorOutline:true,emptyBarColor:"#123456"},{}]}}).settings?.dial.styles[0].palette).toMatchObject({indicatorOutline:true,emptyBarColor:"#123456"});
    const settings=structuredClone(defaultSettings);
    settings.dial.styles[0].meterTrackOpacity=101;
    expect(validateSettings(settings)?.code).toBe("invalidOpacity");
    settings.dial.styles[0].meterTrackOpacity=50;
    settings.dial.styles[0].meterStyle="continuous";
    expect(validateSettings(settings)).toBeUndefined();
    const malformed=JSON.parse(JSON.stringify(settings));
    malformed.dial.styles[0].meterStyle="unknown";
    expect(validateSettings(malformed)?.code).toBe("invalidSettings");
  });
  it("restores nested defaults without replacing saved channels", () => {
    const restored = restoreSettings({ channels: [{ target: { index: 2 } }, {}] });
    expect(restored.error).toBeUndefined();
    expect(restored.settings?.channels[0].target).toEqual({ kind: "strip", index: 2 });
    expect(restored.settings?.channels[1]).toEqual(defaultSettings.channels[1]);
  });
  it("rejects malformed saved settings rather than resetting a target", () => {
    for (const value of [null, { schemaVersion: 2 }, { channels: [] }, { channels: [{ target: { index: -1 } }, {}] }]) {
      expect(restoreSettings(value).settings).toBeUndefined();
    }
  });
  it("validates edition bounds without modifying saved offline targets", () => {
    const settings = structuredClone(defaultSettings);
    settings.channels[0].target.index = 7;
    expect(validateSettings(settings)).toBeUndefined();
    expect(validateSettings(settings, "basic")?.code).toBe("targetOutOfRange");
    expect(settings.channels[0].target.index).toBe(7);
  });
  it("rejects invalid input and accepts arbitrary Unicode and independent steps", () => {
    const settings = structuredClone(defaultSettings);
    settings.channels[0].icon.codePoint = "10ffff";
    settings.channels[1].gainStep = 0.5;
    expect(validateSettings(settings)).toBeUndefined();
    settings.channels[0].icon.codePoint = "d800";
    expect(validateSettings(settings)?.code).toBe("invalidCodePoint");
    settings.channels[0].icon.codePoint = "e050";
    settings.channels[1].gainStep = Number.NaN;
    expect(validateSettings(settings)?.field).toBe("channels.1.gainStep");
  });
});
