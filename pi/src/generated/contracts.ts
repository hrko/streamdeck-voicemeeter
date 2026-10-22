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
