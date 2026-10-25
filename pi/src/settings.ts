import { defaultSettings, type Settings, type Edition, type ValidationError } from "./generated/contracts";

export const channelLimits = { basic: { strip: 3, bus: 2 }, banana: { strip: 5, bus: 5 }, potato: { strip: 8, bus: 8 } };

const record = (value: unknown): value is Record<string, unknown> =>
  value !== null && typeof value === "object" && !Array.isArray(value);

// Missing fields receive the same defaults as serde. Invalid present fields are
// preserved for validation rather than silently replaced with another target.
function merge(defaults: unknown, value: unknown): unknown {
  if (value === undefined) return structuredClone(defaults);
  if (Array.isArray(defaults) && Array.isArray(value)) {
    return value.map((item, index) => merge(defaults[index], item));
  }
  if (record(defaults) && record(value)) {
    return Object.fromEntries(Object.keys(defaults).map(key => [key, merge(defaults[key], value[key])]));
  }
  return value;
}

export function restoreSettings(value: unknown): { settings?: Settings; error?: ValidationError } {
  const merged = merge(defaultSettings, value);
  const error = validateSettings(merged);
  return error ? { error } : { settings: merged as Settings };
}

export function validateSettings(value: unknown, edition?: Edition): ValidationError | undefined {
  const error = (code: string, field: string): ValidationError => ({ code, field });
  if (!record(value)) return error("invalidSettings", "");
  if (value.schemaVersion !== 1) return error("unsupportedSchema", "schemaVersion");
  if (!Array.isArray(value.channels) || value.channels.length !== 2) return error("invalidSettings", "channels");
  const icon = (value: unknown, field: string): ValidationError | undefined => {
    if (!record(value)) return error("invalidSettings", field);
    if (typeof value.codePoint !== "string" || !/^[\da-f]{1,6}$/i.test(value.codePoint)) return error("invalidCodePoint", `${field}.codePoint`);
    const cp = Number.parseInt(value.codePoint, 16);
    if (cp > 0x10ffff || (cp >= 0xd800 && cp <= 0xdfff)) return error("invalidCodePoint", `${field}.codePoint`);
    if (!["outlined", "rounded", "sharp"].includes(String(value.style)) ||
        ![100, 200, 300, 400, 500, 600, 700].includes(value.weight as number) ||
        ![-25, 0, 200].includes(value.grade as number) || ![20, 24, 40, 48].includes(value.opticalSize as number) ||
        typeof value.fill !== "boolean") return error("invalidFontAxes", field);
  };
  for (const [index, channel] of value.channels.entries()) {
    const path = `channels.${index}`;
    if (!record(channel) || !record(channel.target)) return error("invalidSettings", path);
    const target = channel.target;
    if (target.kind !== "strip" && target.kind !== "bus") return error("invalidSettings", `${path}.target.kind`);
    const max = edition ? channelLimits[edition][target.kind] : 8;
    if (!Number.isInteger(target.index) || (target.index as number) < 0 || (target.index as number) >= max) return error("targetOutOfRange", `${path}.target.index`);
    if (typeof channel.gainStep !== "number" || !Number.isFinite(channel.gainStep) || channel.gainStep <= 0 || channel.gainStep > 72) return error("invalidGainStep", `${path}.gainStep`);
    const failure = icon(channel.icon, `${path}.icon`);
    if (failure) return failure;
  }
  const macro = value.macroButton;
  const button = value.gainButton;
  if (!record(button)) return error("invalidSettings", "gainButton");
  if (!["adjust", "set", "mute"].includes(String(button.operation)) || !["static", "horizontal", "vertical"].includes(String(button.appearance))) return error("invalidSettings", "gainButton");
  if (typeof button.delta !== "number" || !Number.isFinite(button.delta) || button.delta === 0 || Math.abs(button.delta) > 72) return error("invalidButtonDelta", "gainButton.delta");
  if (typeof button.gain !== "number" || !Number.isFinite(button.gain) || button.gain < -60 || button.gain > 12) return error("invalidButtonGain", "gainButton.gain");
  for (const key of ["highlightPositive", "showLabel", "showDb", "showMeter", "swapMeter"]) {
    if (typeof button[key] !== "boolean") return error("invalidSettings", `gainButton.${key}`);
  }
  for (const key of ["background", "mutedBackground", "barColor", "thumbColor", "positiveColor"]) {
    if (typeof button[key] !== "string" || !/^#[\da-f]{6}$/i.test(button[key])) return error("invalidColor", `gainButton.${key}`);
  }
  const buttonIconError = icon(button.icon, "gainButton.icon") ?? icon(button.mutedIcon, "gainButton.mutedIcon");
  if (buttonIconError) return buttonIconError;
  if (!record(macro)) return error("invalidSettings", "macroButton");
  if (!Number.isInteger(macro.logicalId) || (macro.logicalId as number) < 0 || (macro.logicalId as number) >= 80) return error("macroOutOfRange", "macroButton.logicalId");
  if (macro.mode !== "push" && macro.mode !== "toggle") return error("invalidSettings", "macroButton.mode");
  for (const key of ["colorOn", "colorOff"] as const) {
    if (typeof macro[key] !== "string" || !/^#[\da-f]{6}$/i.test(macro[key])) return error("invalidColor", `macroButton.${key}`);
  }
  return icon(macro.iconOn, "macroButton.iconOn") ?? icon(macro.iconOff, "macroButton.iconOff");
}
