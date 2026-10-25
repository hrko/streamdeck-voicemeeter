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
  const incoming=structuredClone(value);
  if(record(incoming) && Object.keys(incoming).length) {
    if(incoming.dial===undefined) incoming.dial={};
    if(record(incoming.dial)) {
      if(incoming.dial.styles===undefined) incoming.dial.styles=[{},{}];
      if(Array.isArray(incoming.dial.styles)) for(const style of incoming.dial.styles) if(record(style)) {
        if(style.emptyBarColor!==undefined) {
          if(style.palette===undefined) style.palette={};
          if(record(style.palette) && style.palette.emptyBarColor===undefined) style.palette.emptyBarColor=style.emptyBarColor;
        }
        if(style.indicatorOutline!==undefined) {
          if(style.palette===undefined) style.palette={};
          if(record(style.palette) && style.palette.indicatorOutline===undefined) style.palette.indicatorOutline=style.indicatorOutline;
        }
        if(style.palette===undefined) style.palette={};
        if(record(style.palette)) {
          if(style.backgroundOpacity!==undefined && style.palette.backgroundOpacity===undefined) style.palette.backgroundOpacity=style.backgroundOpacity;
          if(style.mutedBackgroundOpacity!==undefined && style.palette.mutedBackgroundOpacity===undefined) style.palette.mutedBackgroundOpacity=style.mutedBackgroundOpacity;
          if(style.meterTrackOpacity!==undefined && style.palette.meterTrackOpacity===undefined) style.palette.meterTrackOpacity=style.meterTrackOpacity;
          if(style.labelColor!==undefined && style.palette.dialLabelColor===undefined) style.palette.dialLabelColor=style.labelColor;
          if(style.positiveLabelColor!==undefined && style.palette.dialPositiveLabelColor===undefined) style.palette.dialPositiveLabelColor=style.positiveLabelColor;
          if(style.gainColor!==undefined && style.palette.dialGainColor===undefined) style.palette.dialGainColor=style.gainColor;
          if(style.positiveGainColor!==undefined && style.palette.dialPositiveGainColor===undefined) style.palette.dialPositiveGainColor=style.positiveGainColor;
          if(style.palette.backgroundOpacity===undefined) style.palette.backgroundOpacity=100;
          if(style.palette.mutedBackgroundOpacity===undefined) style.palette.mutedBackgroundOpacity=100;
        }
      }
    }
  }
  const merged = merge(defaultSettings, incoming);
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
  const dial=value.dial;
  if (!record(dial) || typeof dial.shared!=="boolean" || !Array.isArray(dial.styles) || dial.styles.length!==2) return error("invalidSettings","dial");
  for(const style of dial.styles) {
    if(!record(style) || !record(style.palette)) return error("invalidSettings","dial");
    for(const key of ["showLabel","showGain","showIcon","showBar","showMeter","showMute","showSolo","showRouting","showMono","showEq","meterNotch"]) {
      if(typeof style[key]!=="boolean") return error("invalidSettings",`dial.${key}`);
    }
    if(style.meterStyle!=="cells" && style.meterStyle!=="continuous") return error("invalidSettings","dial.meterStyle");
    for(const key of Object.keys(defaultSettings.dial.styles[0].palette)) {
      const v=style.palette[key];
      if(["indicatorOpacity","backgroundOpacity","mutedBackgroundOpacity","meterTrackOpacity"].includes(key) ? !Number.isInteger(v) || (v as number)<0 || (v as number)>100 : key==="highlightPositive" || key==="indicatorOutline" ? typeof v!=="boolean" : typeof v!=="string" || !/^#[\da-f]{6}$/i.test(v)) return error(["indicatorOpacity","backgroundOpacity","mutedBackgroundOpacity","meterTrackOpacity"].includes(key)?"invalidOpacity":"invalidColor",`dial.palette.${key}`);
    }
  }
  const macro = value.macroButton;
  const button = value.gainButton;
  if (!record(button)) return error("invalidSettings", "gainButton");
  if(["indicatorOpacity","backgroundOpacity","mutedBackgroundOpacity","meterTrackOpacity"].some(key=>!Number.isInteger(button[key]) || (button[key] as number)<0 || (button[key] as number)>100)) return error("invalidOpacity","gainButton.indicatorOpacity");
  if (!["adjust", "set", "mute"].includes(String(button.operation)) || !["static", "horizontal", "vertical"].includes(String(button.appearance))) return error("invalidSettings", "gainButton");
  if (typeof button.delta !== "number" || !Number.isFinite(button.delta) || button.delta === 0 || Math.abs(button.delta) > 72) return error("invalidButtonDelta", "gainButton.delta");
  if (typeof button.gain !== "number" || !Number.isFinite(button.gain) || button.gain < -60 || button.gain > 12) return error("invalidButtonGain", "gainButton.gain");
  for (const key of ["indicatorOutline", "syncPair", "meterNotch", "highlightPositive", "showLabel", "faderLabel", "showDb", "showMeter", "swapMeter"]) {
    if (typeof button[key] !== "boolean") return error("invalidSettings", `gainButton.${key}`);
  }
  for (const key of ["indicatorOutlineColor", "dialLabelColor", "dialPositiveLabelColor", "dialGainColor", "dialPositiveGainColor", "emptyBarColor", "indicatorBackground", "background", "mutedBackground", "barColor", "labelColor", "positiveLabelColor", "thumbColor", "positiveColor", "meterBackground", "meterInactive", "meterLow", "meterMid", "meterHigh", "meterMutedLow", "meterMutedMid", "meterMutedHigh"]) {
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
