import type { GainButtonSettings } from "./generated/contracts";
import { defaultSettings } from "./generated/contracts";

export const meterColorKeys = ["meterBackground", "meterInactive", "meterLow", "meterMid", "meterHigh", "meterMutedLow", "meterMutedMid", "meterMutedHigh"] as const;
export const paletteKeys = ["barColor", "labelColor", "positiveLabelColor", "thumbColor", "background", "mutedBackground", "positiveColor", "highlightPositive", "emptyBarColor", "indicatorBackground", "indicatorOpacity", "indicatorOutline", "backgroundOpacity", "mutedBackgroundOpacity", "meterTrackOpacity", "dialLabelColor", "dialPositiveLabelColor", "dialGainColor", "dialPositiveGainColor", ...meterColorKeys] as const;
export type Palette = Pick<GainButtonSettings, typeof paletteKeys[number]>;
export interface UserPalette { id: string; name: string; colors: Palette }
export function paletteOf(value: GainButtonSettings): Palette {
  return Object.fromEntries(paletteKeys.map(key => [key, value[key]])) as Palette;
}
const base = paletteOf(defaultSettings.gainButton);
const voicemeeter: Palette = { ...base, emptyBarColor:"#2c3d4d", indicatorBackground:"#000000", indicatorOpacity:62, indicatorOutline:false, barColor: "#70c399", labelColor: "#286442", positiveLabelColor: "#862b20", thumbColor: "#70c399", background: "#2c3d4d", mutedBackground: "#2c3d4d", positiveColor: "#f8634d", highlightPositive: true };
const transparent = (palette:Palette, indicatorOutline:boolean):Palette => ({
  ...palette, backgroundOpacity:0, mutedBackgroundOpacity:0,
  indicatorBackground:"#000000", indicatorOpacity:62, indicatorOutline,
});
export const gainPalettes = {
  standard: { ...base },
  voicemeeter,
  standardTransparent: transparent(base,false),
  standardTransparentOutlined: transparent(base,true),
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
    const colors: Record<string, unknown> = { backgroundOpacity:base.backgroundOpacity, mutedBackgroundOpacity:base.mutedBackgroundOpacity, meterTrackOpacity:base.meterTrackOpacity, dialLabelColor:base.dialLabelColor, dialPositiveLabelColor:base.dialPositiveLabelColor, dialGainColor:base.dialGainColor, dialPositiveGainColor:base.dialPositiveGainColor, emptyBarColor:base.emptyBarColor, indicatorBackground:base.indicatorBackground, indicatorOpacity:base.indicatorOpacity, indicatorOutline:base.indicatorOutline, labelColor: base.labelColor, positiveLabelColor: base.positiveLabelColor, ...p.colors };
    if (!paletteKeys.every(key => ["indicatorOpacity","backgroundOpacity","mutedBackgroundOpacity","meterTrackOpacity"].includes(key) ? Number.isInteger(colors[key]) && (colors[key] as number)>=0 && (colors[key] as number)<=100 : key === "highlightPositive" || key === "indicatorOutline" ? typeof colors[key] === "boolean" : typeof colors[key] === "string" && /^#[\da-f]{6}$/i.test(colors[key]))) return undefined;
    ids.add(p.id);
  }
  return store.items.map(p => ({ ...p, colors: { backgroundOpacity:base.backgroundOpacity, mutedBackgroundOpacity:base.mutedBackgroundOpacity, meterTrackOpacity:base.meterTrackOpacity, dialLabelColor:base.dialLabelColor, dialPositiveLabelColor:base.dialPositiveLabelColor, dialGainColor:base.dialGainColor, dialPositiveGainColor:base.dialPositiveGainColor, emptyBarColor:base.emptyBarColor, indicatorBackground:base.indicatorBackground, indicatorOpacity:base.indicatorOpacity, indicatorOutline:base.indicatorOutline, labelColor: base.labelColor, positiveLabelColor: base.positiveLabelColor, ...p.colors } })) as UserPalette[];
}
