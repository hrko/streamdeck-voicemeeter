import outlined from "../../assets/fonts/MaterialSymbolsOutlined.codepoints?raw";
import rounded from "../../assets/fonts/MaterialSymbolsRounded.codepoints?raw";
import sharp from "../../assets/fonts/MaterialSymbolsSharp.codepoints?raw";
import glyphs from "./generated/glyphs.json";
import type { IconStyle } from "./generated/contracts";

export interface IconEntry { name: string; codePoint: string; aliases: string[] }
const featured = ["volume_up", "volume_down", "volume_off", "volume_mute", "mic", "mic_off", "headphones", "headset_mic", "speaker", "speaker_group", "equalizer", "graphic_eq", "tune", "music_note", "play_arrow", "pause", "stop", "skip_next", "skip_previous", "radio_button_checked", "radio_button_unchecked", "power_settings_new"];
const normalize = (text: string) => text.normalize("NFKC").toLowerCase().replace(/[_-]+/g, " ").trim();

function catalog(text: string, style: IconStyle): IconEntry[] {
  const supported = new Set(glyphs[style]);
  const entries = new Map<string, IconEntry>();
  for (const line of text.trim().split(/\r?\n/)) {
    const [name, codePoint] = line.trim().split(/\s+/);
    if (!supported.has(Number.parseInt(codePoint, 16))) continue;
    const previous = entries.get(codePoint);
    if (previous) previous.aliases.push(name);
    else entries.set(codePoint, { name, codePoint, aliases: [name] });
  }
  const rank = (entry: IconEntry) => {
    const index = featured.findIndex(name => entry.aliases.includes(name));
    if (index >= 0) entry.name = featured[index];
    return index < 0 ? featured.length : index;
  };
  const result = [...entries.values()].map(entry => ({ entry, rank: rank(entry) }));
  return result.sort((a, b) => a.rank - b.rank || a.entry.name.localeCompare(b.entry.name, "en")).map(({ entry }) => entry);
}

export const iconCatalogs = { outlined: catalog(outlined, "outlined"), rounded: catalog(rounded, "rounded"), sharp: catalog(sharp, "sharp") };

export function searchIcons(style: IconStyle, query: string): IconEntry[] {
  const normalized = normalize(query);
  if (!normalized) return iconCatalogs[style];
  const code = normalized.replace(/^(u\+|0x)/, "");
  const words = normalized.split(/\s+/);
  return iconCatalogs[style].filter(entry => entry.codePoint === code || entry.aliases.some(name => words.every(word => normalize(name).includes(word))));
}
