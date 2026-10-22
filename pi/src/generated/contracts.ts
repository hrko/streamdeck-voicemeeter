// Generated from Rust settings. Do not edit.
export type TargetKind = "strip" | "bus";
export type Target = { kind: TargetKind, index: number, };
export type IconStyle = "outlined" | "rounded" | "sharp";
export type IconSettings = { codePoint: string, style: IconStyle, weight: number, fill: boolean, grade: number, opticalSize: number, };
export type ChannelSettings = { target: Target, gainStep: number, icon: IconSettings, };
export type ButtonMode = "toggle" | "push";
export type MacroSettings = { logicalId: number, mode: ButtonMode, iconOn: IconSettings, iconOff: IconSettings, colorOn: string, colorOff: string, };
export type Settings = { schemaVersion: number, channels: [ChannelSettings, ChannelSettings], macroButton: MacroSettings, };
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
        "codePoint": "e050",
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
        "codePoint": "e050",
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
      "style": "rounded",
      "weight": 400,
      "fill": false,
      "grade": 0,
      "opticalSize": 48
    },
    "iconOff": {
      "codePoint": "e836",
      "style": "rounded",
      "weight": 400,
      "fill": false,
      "grade": 0,
      "opticalSize": 48
    },
    "colorOn": "#067ba2",
    "colorOff": "#004162"
  }
};
