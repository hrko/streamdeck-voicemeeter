import { defaultSettings } from "./generated/contracts";
import { expect, it } from "vitest";
import { gainPalettes, readPalettes, gainPaletteName, builtinPaletteNames } from "./gainColors";
it("loads older shared palettes without losing meter colors when label color is absent", () => {
  const { labelColor: _label, positiveLabelColor: _positiveLabel, ...older } = gainPalettes.voicemeeter;
  const items = readPalettes({ gainColorPresets: { version: 1, items: [{ id: "user:old", name: "Old VM", colors: older }] } });
  expect(items?.[0].colors).toMatchObject({ labelColor: "#000000", positiveLabelColor: "#862b20", meterLow: "#85adb9", meterMutedHigh: "#583328" });
  expect(readPalettes({ gainColorPresets: { version: 1, items: [{ id: "user:old", name: "Bad", colors: { ...older, labelColor: "not a color" } }] } })).toBeUndefined();
});

it("round-trips indicator palettes and upgrades older palettes", () => {
 const colors={...gainPalettes.standard,indicatorBackground:"#123456",indicatorOpacity:37,indicatorOutline:true,emptyBarColor:"#654321"};
 const store={gainColorPresets:{version:1,items:[{id:"user:test",name:"Test",colors}]}};
 expect(readPalettes(store)?.[0].colors).toEqual(colors);
 for(const bad of [-1,101,0.5,"37"]) {
  expect(readPalettes({gainColorPresets:{...store.gainColorPresets,items:[{...store.gainColorPresets.items[0],colors:{...colors,indicatorOpacity:bad}}]}})).toBeUndefined();
 }
 const old={...colors} as Record<string,unknown>;
 for(const key of ["indicatorBackground","indicatorOpacity","indicatorOutline","emptyBarColor"]) delete old[key];
 expect(readPalettes({gainColorPresets:{version:1,items:[{id:"user:old",name:"Old",colors:old}]}})?.[0].colors).toMatchObject({indicatorBackground:"#808080",indicatorOpacity:100,indicatorOutline:false,emptyBarColor:"#808080"});
});

it("preserves opacity and independent dial text colors in shared presets", () => {
 const colors={...gainPalettes.standard,backgroundOpacity:30,mutedBackgroundOpacity:70,meterTrackOpacity:45,dialLabelColor:"#123456",dialPositiveLabelColor:"#234567",dialGainColor:"#345678",dialPositiveGainColor:"#456789"};
 const wrap=(c:unknown)=>({gainColorPresets:{version:1,items:[{id:"user:colors",name:"Colors",colors:c}]}});
 expect(readPalettes(wrap(colors))?.[0].colors).toEqual(colors);
 expect(colors.labelColor).toBe("#000000");
 const old={...colors} as Record<string,unknown>;
 for(const key of ["backgroundOpacity","mutedBackgroundOpacity","meterTrackOpacity","dialLabelColor","dialPositiveLabelColor","dialGainColor","dialPositiveGainColor"]) delete old[key];
 expect(readPalettes(wrap(old))?.[0].colors).toMatchObject({backgroundOpacity:100,mutedBackgroundOpacity:100,meterTrackOpacity:0,dialLabelColor:"#ffffff",dialGainColor:"#ffffff"});
 for(const key of ["backgroundOpacity","mutedBackgroundOpacity","meterTrackOpacity"]) expect(readPalettes(wrap({...colors,[key]:101}))).toBeUndefined();
});

it("recognizes all four transparent variants without changing their base colors", () => {
 for(const name of ["standardTransparent","standardTransparentOutlined","voicemeeterTransparent","voicemeeterTransparentOutlined"] as const){
  const value=gainPalettes[name];
  const base=name.startsWith("standard")?gainPalettes.standard:gainPalettes.voicemeeter;
  expect(value).toEqual({...base,backgroundOpacity:0,mutedBackgroundOpacity:0,indicatorBackground:"#000000",indicatorOpacity:62,indicatorOutline:name.endsWith("Outlined")});
  expect(gainPaletteName({...defaultSettings.gainButton,...value})).toBe(name);
 }
});

it("places the default first and groups each opaque palette with its transparent variants", () => {
 expect(builtinPaletteNames).toEqual(["default","standard","standardTransparent","standardTransparentOutlined","voicemeeter","voicemeeterTransparent","voicemeeterTransparentOutlined"]);
 expect(gainPaletteName({...defaultSettings.gainButton,...defaultSettings.dial.styles[0].palette})).toBe("default");
 expect(gainPalettes.default).toMatchObject({barColor:"#70c399",positiveColor:"#f8634d",emptyBarColor:"#2c3d4d",indicatorBackground:"#2c3d4d",indicatorOpacity:100,indicatorOutline:false,backgroundOpacity:0,mutedBackgroundOpacity:0});
});

it("keeps the default button thumb and label colors distinct from dial text", () => {
 expect(gainPalettes.default).toMatchObject({barColor:"#70c399",thumbColor:"#70c399",labelColor:"#286442",positiveLabelColor:"#862b20",dialLabelColor:"#ffffff",dialPositiveLabelColor:"#ffffff"});
});

it("uses neutral inactive cells and outlines only in monochrome presets", () => {
 for(const key of ["standard","standardTransparent","standardTransparentOutlined"] as const)
  expect(gainPalettes[key]).toMatchObject({meterInactive:"#1b1b1b",indicatorOutlineColor:"#a0a0a0"});
 for(const key of ["default","voicemeeter","voicemeeterTransparent","voicemeeterTransparentOutlined"] as const)
  expect(gainPalettes[key]).toMatchObject({meterInactive:"#191b1b",indicatorOutlineColor:"#91a4b5"});
 const old={...gainPalettes.standard} as Record<string,unknown>;
 delete old.indicatorOutlineColor;
 expect(readPalettes({gainColorPresets:{version:1,items:[{id:"user:old",name:"Old",colors:old}]}})?.[0].colors.indicatorOutlineColor).toBe("#91a4b5");
});
