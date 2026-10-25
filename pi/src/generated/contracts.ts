// Generated from Rust settings. Do not edit.
export type TargetKind = "strip" | "bus";
export type Target = { kind: TargetKind, index: number, };
export type IconStyle = "outlined" | "rounded" | "sharp";
export type IconSettings = { codePoint: string, style: IconStyle, weight: number, fill: boolean, grade: number, opticalSize: number, };
export type ChannelSettings = { target: Target, gainStep: number, icon: IconSettings, };
export type ButtonMode = "toggle" | "push";
export type MacroSettings = { logicalId: number, mode: ButtonMode, iconOn: IconSettings, iconOff: IconSettings, colorOn: string, colorOff: string, };
export type GainOperation = "adjust" | "set" | "mute";
export type GainAppearance = "static" | "horizontal" | "vertical";
export type GainButtonSettings = { operation: GainOperation, delta: number, gain: number, appearance: GainAppearance, icon: IconSettings, mutedIcon: IconSettings, background: string, mutedBackground: string, barColor: string, thumbColor: string, positiveColor: string, highlightPositive: boolean, showLabel: boolean, showDb: boolean, showMeter: boolean, };
export type Settings = { schemaVersion: number, channels: [ChannelSettings, ChannelSettings], macroButton: MacroSettings, gainButton: GainButtonSettings, };
export type ValidationError = { code: string, field: string, };

export type Edition = "basic" | "banana" | "potato";

export const defaultSettings: Settings = {
  "schemaVersion": 1,
  "channels": [
    {
      "target": {
        "kind": "strip",
        "index": 0
      },
      "gainStep": 3.0,
      "icon": {
        "codePoint": "f71a",
        "style": "rounded",
        "weight": 400,
        "fill": false,
        "grade": 0,
        "opticalSize": 48
      }
    },
    {
      "target": {
        "kind": "bus",
        "index": 0
      },
      "gainStep": 3.0,
      "icon": {
        "codePoint": "f70e",
        "style": "rounded",
        "weight": 400,
        "fill": false,
        "grade": 0,
        "opticalSize": 48
      }
    }
  ],
  "macroButton": {
    "logicalId": 0,
    "mode": "push",
    "iconOn": {
      "codePoint": "e837",
      "style": "outlined",
      "weight": 400,
      "fill": false,
      "grade": 0,
      "opticalSize": 48
    },
    "iconOff": {
      "codePoint": "e836",
      "style": "outlined",
      "weight": 400,
      "fill": false,
      "grade": 0,
      "opticalSize": 48
    },
    "colorOn": "#067ba2",
    "colorOff": "#004162"
  },
  "gainButton": {
    "operation": "adjust",
    "delta": 3.0,
    "gain": 0.0,
    "appearance": "static",
    "icon": {
      "codePoint": "e31d",
      "style": "rounded",
      "weight": 400,
      "fill": false,
      "grade": 0,
      "opticalSize": 48
    },
    "mutedIcon": {
      "codePoint": "e02b",
      "style": "rounded",
      "weight": 400,
      "fill": false,
      "grade": 0,
      "opticalSize": 48
    },
    "background": "#000000",
    "mutedBackground": "#000000",
    "barColor": "#ffffff",
    "thumbColor": "#ffffff",
    "positiveColor": "#ff6347",
    "highlightPositive": false,
    "showLabel": false,
    "showDb": false,
    "showMeter": false
  }
};
