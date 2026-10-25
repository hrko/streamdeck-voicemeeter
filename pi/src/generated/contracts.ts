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
export type GainButtonSettings = { operation: GainOperation, delta: number, gain: number, appearance: GainAppearance, icon: IconSettings, mutedIcon: IconSettings, background: string, mutedBackground: string, barColor: string, thumbColor: string, positiveColor: string, highlightPositive: boolean, showLabel: boolean, faderLabel: boolean, syncPair: boolean, labelColor: string, positiveLabelColor: string, showDb: boolean, showMeter: boolean, meterNotch: boolean, swapMeter: boolean, emptyBarColor: string, backgroundOpacity: number, mutedBackgroundOpacity: number, meterTrackOpacity: number, dialLabelColor: string, dialPositiveLabelColor: string, dialGainColor: string, dialPositiveGainColor: string, indicatorOutlineColor: string, indicatorBackground: string, indicatorOpacity: number, indicatorOutline: boolean, meterBackground: string, meterInactive: string, meterLow: string, meterMid: string, meterHigh: string, meterMutedLow: string, meterMutedMid: string, meterMutedHigh: string, };
export type DialPalette = { barColor: string, labelColor: string, positiveLabelColor: string, thumbColor: string, background: string, mutedBackground: string, positiveColor: string, emptyBarColor: string, backgroundOpacity: number, mutedBackgroundOpacity: number, meterTrackOpacity: number, dialLabelColor: string, dialPositiveLabelColor: string, dialGainColor: string, dialPositiveGainColor: string, indicatorOutlineColor: string, indicatorBackground: string, indicatorOpacity: number, indicatorOutline: boolean, meterBackground: string, meterInactive: string, meterLow: string, meterMid: string, meterHigh: string, meterMutedLow: string, meterMutedMid: string, meterMutedHigh: string, highlightPositive: boolean, };
export type DialMeterStyle = "cells" | "continuous";
export type DialStyle = { palette: DialPalette, showLabel: boolean, showGain: boolean, showIcon: boolean, showMeter: boolean, showBar: boolean, showMute: boolean, showSolo: boolean, showRouting: boolean, showMono: boolean, showEq: boolean, meterNotch: boolean, meterStyle: DialMeterStyle, };
export type TouchOperation = "none" | "toggleMute" | "resetGain";
export type DialSettings = { tapOperation: TouchOperation, holdOperation: TouchOperation, shared: boolean, styles: [DialStyle, DialStyle], };
export type Settings = { dial: DialSettings, schemaVersion: number, channels: [ChannelSettings, ChannelSettings], macroButton: MacroSettings, gainButton: GainButtonSettings, };
export type ValidationError = { code: string, field: string, };

export type Edition = "basic" | "banana" | "potato";

export const defaultSettings: Settings = {
  "dial": {
    "tapOperation": "toggleMute",
    "holdOperation": "resetGain",
    "shared": true,
    "styles": [
      {
        "palette": {
          "barColor": "#70c399",
          "labelColor": "#286442",
          "positiveLabelColor": "#862b20",
          "thumbColor": "#70c399",
          "background": "#000000",
          "mutedBackground": "#000000",
          "positiveColor": "#f8634d",
          "emptyBarColor": "#2c3d4d",
          "backgroundOpacity": 0,
          "mutedBackgroundOpacity": 0,
          "meterTrackOpacity": 0,
          "dialLabelColor": "#ffffff",
          "dialPositiveLabelColor": "#ffffff",
          "dialGainColor": "#ffffff",
          "dialPositiveGainColor": "#ffffff",
          "indicatorOutlineColor": "#91a4b5",
          "indicatorBackground": "#2c3d4d",
          "indicatorOpacity": 100,
          "indicatorOutline": false,
          "meterBackground": "#000000",
          "meterInactive": "#191b1b",
          "meterLow": "#85adb9",
          "meterMid": "#1efe5b",
          "meterHigh": "#fa0000",
          "meterMutedLow": "#394547",
          "meterMutedMid": "#384b34",
          "meterMutedHigh": "#583328",
          "highlightPositive": true
        },
        "showLabel": true,
        "showGain": true,
        "showIcon": true,
        "showMeter": true,
        "showBar": true,
        "showMute": true,
        "showSolo": true,
        "showRouting": true,
        "showMono": true,
        "showEq": true,
        "meterNotch": false,
        "meterStyle": "cells"
      },
      {
        "palette": {
          "barColor": "#70c399",
          "labelColor": "#286442",
          "positiveLabelColor": "#862b20",
          "thumbColor": "#70c399",
          "background": "#000000",
          "mutedBackground": "#000000",
          "positiveColor": "#f8634d",
          "emptyBarColor": "#2c3d4d",
          "backgroundOpacity": 0,
          "mutedBackgroundOpacity": 0,
          "meterTrackOpacity": 0,
          "dialLabelColor": "#ffffff",
          "dialPositiveLabelColor": "#ffffff",
          "dialGainColor": "#ffffff",
          "dialPositiveGainColor": "#ffffff",
          "indicatorOutlineColor": "#91a4b5",
          "indicatorBackground": "#2c3d4d",
          "indicatorOpacity": 100,
          "indicatorOutline": false,
          "meterBackground": "#000000",
          "meterInactive": "#191b1b",
          "meterLow": "#85adb9",
          "meterMid": "#1efe5b",
          "meterHigh": "#fa0000",
          "meterMutedLow": "#394547",
          "meterMutedMid": "#384b34",
          "meterMutedHigh": "#583328",
          "highlightPositive": true
        },
        "showLabel": true,
        "showGain": true,
        "showIcon": true,
        "showMeter": true,
        "showBar": true,
        "showMute": true,
        "showSolo": true,
        "showRouting": true,
        "showMono": true,
        "showEq": true,
        "meterNotch": false,
        "meterStyle": "cells"
      }
    ]
  },
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
    "faderLabel": false,
    "syncPair": true,
    "labelColor": "#000000",
    "positiveLabelColor": "#862b20",
    "showDb": false,
    "showMeter": false,
    "meterNotch": false,
    "swapMeter": false,
    "emptyBarColor": "#808080",
    "backgroundOpacity": 100,
    "mutedBackgroundOpacity": 100,
    "meterTrackOpacity": 0,
    "dialLabelColor": "#ffffff",
    "dialPositiveLabelColor": "#ffffff",
    "dialGainColor": "#ffffff",
    "dialPositiveGainColor": "#ffffff",
    "indicatorOutlineColor": "#a0a0a0",
    "indicatorBackground": "#808080",
    "indicatorOpacity": 100,
    "indicatorOutline": false,
    "meterBackground": "#000000",
    "meterInactive": "#1b1b1b",
    "meterLow": "#a6a6a6",
    "meterMid": "#1efe5b",
    "meterHigh": "#fa0000",
    "meterMutedLow": "#424242",
    "meterMutedMid": "#384b34",
    "meterMutedHigh": "#583328"
  }
};
