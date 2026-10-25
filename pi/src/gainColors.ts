import type { GainButtonSettings } from "./generated/contracts";

type Palette = Pick<GainButtonSettings, "barColor" | "thumbColor" | "background" | "mutedBackground" | "positiveColor" | "highlightPositive">;
// Matches src/dial.rs and the solid colors sampled from VoiceMeeter Potato.
export const gainPalettes = {
  standard: { barColor: "#ffffff", thumbColor: "#ffffff", background: "#000000", mutedBackground: "#000000", positiveColor: "#ff6347", highlightPositive: false },
  voicemeeter: { barColor: "#70c399", thumbColor: "#70c399", background: "#2c3d4d", mutedBackground: "#2c3d4d", positiveColor: "#f8634d", highlightPositive: true },
} satisfies Record<string, Palette>;

export function gainPaletteName(value: GainButtonSettings): keyof typeof gainPalettes | "custom" {
  for (const name of ["standard", "voicemeeter"] as const) {
    const palette = gainPalettes[name];
    if ((Object.keys(palette) as (keyof Palette)[]).every(key =>
      typeof value[key] === "string" ? value[key].toLowerCase() === palette[key] : value[key] === palette[key])) return name;
  }
  return "custom";
}
