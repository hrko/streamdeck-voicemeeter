import type { GainButtonSettings } from "./generated/contracts";
import { defaultSettings } from "./generated/contracts";

export const meterColorKeys = ["meterBackground", "meterInactive", "meterLow", "meterMid", "meterHigh", "meterMutedLow", "meterMutedMid", "meterMutedHigh"] as const;
export const paletteKeys = ["keyMeterOpacity","keyInactiveOpacity","keyRailOpacity","keyThumbOpacity","keyThumbClearance","selectionColor", "barColor", "labelColor", "positiveLabelColor", "thumbColor", "background", "mutedBackground", "positiveColor", "highlightPositive", "emptyBarColor", "indicatorOutlineColor", "indicatorBackground", "indicatorOpacity", "indicatorOutline", "backgroundOpacity", "mutedBackgroundOpacity", "meterTrackOpacity", "dialLabelColor", "dialPositiveLabelColor", "dialGainColor", "dialPositiveGainColor", ...meterColorKeys] as const;
export type Palette = Pick<GainButtonSettings, typeof paletteKeys[number]>;
export interface UserPalette { id: string; name: string; colors: Palette }
export function paletteOf(value: GainButtonSettings): Palette {
  return Object.fromEntries(paletteKeys.map(key => [key, value[key]])) as Palette;
}
const base = paletteOf(defaultSettings.gainButton);
const monochrome: Palette={...base,meterMutedLow:"#424242",meterLow:"#a6a6a6",meterInactive:"#1b1b1b",indicatorOutlineColor:"#a0a0a0"};
const voicemeeter: Palette = { ...base, meterMutedLow:"#394547", meterLow:"#85adb9", meterInactive:"#191b1b", indicatorOutlineColor:"#91a4b5", emptyBarColor:"#2c3d4d", indicatorBackground:"#000000", indicatorOpacity:62, indicatorOutline:false, barColor: "#70c399", labelColor: "#286442", positiveLabelColor: "#862b20", thumbColor: "#70c399", background: "#2c3d4d", mutedBackground: "#2c3d4d", positiveColor: "#f8634d", highlightPositive: true };
const transparent = (palette:Palette, indicatorOutline:boolean):Palette => ({
  ...palette, backgroundOpacity:0, mutedBackgroundOpacity:0,
  keyMeterOpacity:40,keyInactiveOpacity:40,keyRailOpacity:40,keyThumbOpacity:40,keyThumbClearance:false,
  indicatorBackground:"#000000", indicatorOpacity:62, indicatorOutline,
});
export const gainPalettes = {
  default: paletteOf({...defaultSettings.gainButton,...defaultSettings.dial.styles[0].palette}),
  standard: { ...monochrome },
  standardTransparent: transparent(monochrome,false),
  standardTransparentOutlined: transparent(monochrome,true),
  voicemeeter,
  voicemeeterTransparent: transparent(voicemeeter,false),
  voicemeeterTransparentOutlined: transparent(voicemeeter,true),
} satisfies Record<string, Palette>;
export const builtinPaletteNames=Object.keys(gainPalettes) as (keyof typeof gainPalettes)[];
export function samePalette(a: Palette, b: Palette): boolean {
  return paletteKeys.every(key => typeof a[key] === "string" ? a[key].toLowerCase() === b[key] : a[key] === b[key]);
}
export function gainPaletteName(value: GainButtonSettings): keyof typeof gainPalettes | "custom" {
  return builtinPaletteNames.find(name => samePalette(value, gainPalettes[name])) ?? "custom";
}
const record = (v: unknown): v is Record<string, unknown> => !!v && typeof v === "object" && !Array.isArray(v);
// Refuse malformed/unknown-version stores rather than overwrite user presets.
export function readPalettes(global: unknown): UserPalette[] | undefined {
  if (!record(global)) return undefined;
  const store = global.gainColorPresets;
  if (store === undefined) return [];
  if (!record(store) || store.version !== 1 || !Array.isArray(store.items) || store.items.length > 100) return undefined;
  const ids = new Set<string>();
  for (const p of store.items) {
    if (!record(p) || typeof p.id !== "string" || !p.id.startsWith("user:") || ids.has(p.id) || typeof p.name !== "string" || !p.name.trim() || p.name.length > 64 || !record(p.colors)) return undefined;
    const colors: Record<string, unknown> = { keyMeterOpacity:100,keyInactiveOpacity:100,keyRailOpacity:100,keyThumbOpacity:100,keyThumbClearance:true, selectionColor:"#ffffff", indicatorOutlineColor:"#91a4b5", backgroundOpacity:base.backgroundOpacity, mutedBackgroundOpacity:base.mutedBackgroundOpacity, meterTrackOpacity:base.meterTrackOpacity, dialLabelColor:base.dialLabelColor, dialPositiveLabelColor:base.dialPositiveLabelColor, dialGainColor:base.dialGainColor, dialPositiveGainColor:base.dialPositiveGainColor, emptyBarColor:base.emptyBarColor, indicatorBackground:base.indicatorBackground, indicatorOpacity:base.indicatorOpacity, indicatorOutline:base.indicatorOutline, labelColor: base.labelColor, positiveLabelColor: base.positiveLabelColor, ...p.colors };
    if (!paletteKeys.every(key => ["keyMeterOpacity","keyInactiveOpacity","keyRailOpacity","keyThumbOpacity","indicatorOpacity","backgroundOpacity","mutedBackgroundOpacity","meterTrackOpacity"].includes(key) ? Number.isInteger(colors[key]) && (colors[key] as number)>=0 && (colors[key] as number)<=100 : key === "keyThumbClearance" || key === "highlightPositive" || key === "indicatorOutline" ? typeof colors[key] === "boolean" : typeof colors[key] === "string" && /^#[\da-f]{6}$/i.test(colors[key]))) return undefined;
    ids.add(p.id);
  }
  return store.items.map(p => ({ ...p, colors: { keyMeterOpacity:100,keyInactiveOpacity:100,keyRailOpacity:100,keyThumbOpacity:100,keyThumbClearance:true, selectionColor:"#ffffff", indicatorOutlineColor:"#91a4b5", backgroundOpacity:base.backgroundOpacity, mutedBackgroundOpacity:base.mutedBackgroundOpacity, meterTrackOpacity:base.meterTrackOpacity, dialLabelColor:base.dialLabelColor, dialPositiveLabelColor:base.dialPositiveLabelColor, dialGainColor:base.dialGainColor, dialPositiveGainColor:base.dialPositiveGainColor, emptyBarColor:base.emptyBarColor, indicatorBackground:base.indicatorBackground, indicatorOpacity:base.indicatorOpacity, indicatorOutline:base.indicatorOutline, labelColor: base.labelColor, positiveLabelColor: base.positiveLabelColor, ...p.colors } })) as UserPalette[];
}
