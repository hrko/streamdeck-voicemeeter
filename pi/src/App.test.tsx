import { afterEach, beforeAll, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, within, act } from "@testing-library/react";
import { I18nextProvider } from "react-i18next";
import type { StreamDeckPiApi } from "@mikanseilaboratory/streamdeck-pi-client";
import { App } from "./App";
import { createLocalization } from "./i18n";
import { defaultSettings, type Settings } from "./generated/contracts";

afterEach(cleanup);
beforeAll(() => {
  // jsdom has no native dialog implementation; browser behavior is checked in preview.
  HTMLDialogElement.prototype.showModal = function () { this.open = true; };
  HTMLDialogElement.prototype.close = function () { this.open = false; };
});
async function setup(action = "gain-control-combo", settings: unknown = defaultSettings, language = "en") {
  const handlers = new Map<string, (payload: unknown) => void>();
  const client: StreamDeckPiApi = {
    ready: true, uuid: "pi", actionInfo: { action: `jp.hrko.streamdeck.voicemeeter.rust.${action}`, context: "action", device: "device", payload: { settings: settings as Record<string, unknown> } },
    setSettings: vi.fn(), getSettings: vi.fn(), setGlobalSettings: vi.fn(), getGlobalSettings: vi.fn(), sendToPlugin: vi.fn(),
    subscribe: (event, handler) => { handlers.set(event, handler); return () => { handlers.delete(event); }; },
  };
  const i18n = await createLocalization(language);
  render(<I18nextProvider i18n={i18n}><App client={client} /></I18nextProvider>);
  return { client, emit: (event: string, value: unknown) => act(() => { handlers.get(event)?.(value); }) };
}

describe("property inspector forms", () => {
  it("applies and restores fader palettes without changing behavior or detail options", async () => {
    const settings = structuredClone(defaultSettings);
    settings.gainButton.appearance = "vertical";
    settings.gainButton.delta = -2.5;
    settings.gainButton.showDb = true;
    const { client, emit } = await setup("gain-button", settings);
    const palette = screen.getByLabelText("Color preset") as HTMLSelectElement;
    expect(palette.value).toBe("standard");
    fireEvent.change(palette, { target: { value: "voicemeeter" } });
    let saved = vi.mocked(client.setSettings).mock.calls.at(-1)![0] as Settings;
    expect(saved.gainButton).toMatchObject({ barColor: "#70c399", thumbColor: "#70c399", positiveColor: "#f8634d", background: "#2c3d4d", mutedBackground: "#2c3d4d", highlightPositive: true, delta: -2.5, appearance: "vertical", showDb: true });
    expect(saved.channels).toEqual(settings.channels);
    emit("didReceiveSettings", saved);
    expect(palette.value).toBe("voicemeeter");
    fireEvent.change(screen.getByLabelText("Fader color"), { target: { value: "#123456" } });
    expect(palette.value).toBe("custom");
    fireEvent.change(palette, { target: { value: "standard" } });
    saved = vi.mocked(client.setSettings).mock.calls.at(-1)![0] as Settings;
    expect(saved.gainButton).toMatchObject({ barColor: "#ffffff", background: "#000000", highlightPositive: false, delta: -2.5, showDb: true });
  });

  it("configures gain keys with signed steps, optional details and absolute/mute operations", async () => {
    const { client } = await setup("gain-button");
    expect(screen.queryByLabelText("Step (dB)")).toBeNull();
    const delta = screen.getByLabelText("Change (dB)");
    fireEvent.change(delta, { target: { value: "0" } });
    expect(client.setSettings).not.toHaveBeenCalled();
    fireEvent.change(delta, { target: { value: "-2.5" } });
    fireEvent.change(screen.getByLabelText("Display style"), { target: { value: "vertical" } });
    expect(screen.getByText(/Lower half/)).toBeTruthy();
    fireEvent.click(screen.getByLabelText("Highlight above 0 dB"));
    fireEvent.change(screen.getByLabelText("Positive gain color"), { target: { value: "#ff0000" } });
    fireEvent.click(screen.getByLabelText("Show channel label"));
    fireEvent.click(screen.getByLabelText("Show gain (dB)"));
    fireEvent.click(screen.getByLabelText("Show stereo meter"));
    let saved = vi.mocked(client.setSettings).mock.calls.at(-1)![0] as Settings;
    expect(saved.gainButton).toMatchObject({ delta: -2.5, appearance: "vertical", highlightPositive: true, positiveColor: "#ff0000", showLabel: true, showDb: true, showMeter: true });
    fireEvent.change(screen.getByLabelText("Action"), { target: { value: "set" } });
    expect(screen.queryByLabelText("Display style")).toBeNull();
    vi.mocked(client.setSettings).mockClear();
    fireEvent.change(screen.getByLabelText("Target gain (dB)"), { target: { value: "13" } });
    expect(client.setSettings).not.toHaveBeenCalled();
    fireEvent.change(screen.getByLabelText("Target gain (dB)"), { target: { value: "-6" } });
    saved = vi.mocked(client.setSettings).mock.calls.at(-1)![0] as Settings;
    expect(saved.gainButton.gain).toBe(-6);
    fireEvent.change(screen.getByLabelText("Action"), { target: { value: "mute" } });
    expect(screen.queryByLabelText("Target gain (dB)")).toBeNull();
    expect(screen.getByRole("group", { name: "Muted icon" })).toBeTruthy();
  });
  it("synchronizes native color pickers, hex input and previews without saving invalid text", async () => {
    const { client, emit } = await setup("macro");
    const on = screen.getByRole("group", { name: "ON icon" });
    const picker = within(on).getByLabelText("ON background color picker") as HTMLInputElement;
    const code = within(on).getByRole("textbox", { name: "ON background" }) as HTMLInputElement;
    expect(picker.type).toBe("color");
    fireEvent.change(picker, { target: { value: "#123456" } });
    expect(code.value).toBe("#123456");
    expect(within(on).getByRole("img").style.backgroundColor).toBe("rgb(18, 52, 86)");
    let saved = vi.mocked(client.setSettings).mock.calls.at(-1)![0] as Settings;
    expect(saved.macroButton.colorOn).toBe("#123456");
    expect(saved.macroButton.colorOff).toBe(defaultSettings.macroButton.colorOff);
    fireEvent.change(code, { target: { value: "#AABBCC" } });
    expect(picker.value).toBe("#aabbcc");
    expect(within(on).getByRole("img").style.backgroundColor).toBe("rgb(170, 187, 204)");
    vi.mocked(client.setSettings).mockClear();
    for (const value of ["", "#12", "not-a-color"]) {
      fireEvent.change(code, { target: { value } });
      expect(within(on).getByRole("alert").textContent).toContain("#RRGGBB");
      expect(picker.value).toBe("#aabbcc");
      expect(client.setSettings).not.toHaveBeenCalled();
    }
    fireEvent.change(picker, { target: { value: "#010203" } });
    expect(code.value).toBe("#010203");
    expect(within(on).queryByRole("alert")).toBeNull();
    saved = structuredClone(defaultSettings);
    saved.macroButton.colorOn = "#fedcba";
    emit("didReceiveSettings", saved);
    expect(code.value).toBe("#fedcba");
    expect(picker.value).toBe("#fedcba");
  });
  it("opens a searchable picker, cancels without saving and applies only the selected Macro icon", async () => {
    const { client } = await setup("macro");
    const on = screen.getByRole("group", { name: "ON icon" });
    const trigger = within(on).getByRole("button", { name: "Choose icon" });
    trigger.focus();
    fireEvent.click(trigger);
    let picker = screen.getByRole("dialog");
    const search = within(picker).getByRole("searchbox");
    expect(document.activeElement).toBe(search);
    expect(within(picker).getAllByRole("button")[1].getAttribute("aria-label")).toBe("volume_up");
    const firstIcon = within(picker).getByRole("button", { name: "volume_up" });
    firstIcon.focus();
    fireEvent.keyDown(firstIcon, { key: "ArrowRight" });
    expect(document.activeElement).toBe(within(picker).getByRole("button", { name: "volume_down" }));
    const initialCount = within(picker).getAllByRole("button").length;
    fireEvent.click(within(picker).getByRole("button", { name: "Show more" }));
    expect(within(picker).getAllByRole("button").length).toBe(initialCount + 120);
    fireEvent.change(search, { target: { value: "zoom_out" } });
    expect(within(picker).getByRole("button", { name: "zoom_out" })).toBeTruthy();
    fireEvent.change(search, { target: { value: "no_such_icon_zzz" } });
    expect(within(picker).getByText("No matching icons.")).toBeTruthy();
    expect(client.setSettings).not.toHaveBeenCalled();
    fireEvent(picker, new Event("cancel", { cancelable: true }));
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(document.activeElement).toBe(trigger);
    expect(client.setSettings).not.toHaveBeenCalled();
    fireEvent.click(trigger);
    picker = screen.getByRole("dialog");
    fireEvent.change(within(picker).getByRole("searchbox"), { target: { value: "U+E050" } });
    fireEvent.click(within(picker).getByRole("button", { name: "volume_up" }));
    const saved = vi.mocked(client.setSettings).mock.calls.at(-1)![0] as Settings;
    expect(saved.macroButton.iconOn).toEqual({ ...defaultSettings.macroButton.iconOn, codePoint: "e050" });
    expect(saved.macroButton.iconOff).toEqual(defaultSettings.macroButton.iconOff);
    expect(saved.macroButton.colorOn).toBe(defaultSettings.macroButton.colorOn);
    expect(screen.queryByRole("dialog")).toBeNull();
  });
  it("updates channel choices by edition without replacing an unavailable saved target", async () => {
    const settings = structuredClone(defaultSettings);
    settings.channels[0].target.index = 6;
    const { client, emit } = await setup("gain-control-combo", settings);
    const choices = () => screen.getAllByLabelText("Channel") as HTMLSelectElement[];
    const enabled = (select: HTMLSelectElement) => Array.from(select.options).filter(option => !option.disabled);
    for (const [edition, strips, buses] of [["basic", 3, 2], ["banana", 5, 5], ["potato", 8, 8]] as const) {
      emit("sendToPropertyInspector", { event: "status", connected: true, edition });
      expect(enabled(choices()[0])).toHaveLength(strips);
      expect(enabled(choices()[1])).toHaveLength(buses);
      expect(choices()[0].value).toBe("6");
      expect(client.setSettings).not.toHaveBeenCalled();
    }
    emit("sendToPropertyInspector", { event: "status", connected: true, edition: "basic" });
    fireEvent.change(choices()[0], { target: { value: "2" } });
    expect((vi.mocked(client.setSettings).mock.calls.at(-1)![0] as Settings).channels[0].target.index).toBe(2);
    emit("sendToPropertyInspector", { event: "status", connected: false });
    expect(enabled(choices()[0])).toHaveLength(8);
  });
  it("previews independent Macro colors and reports unsupported glyphs", async () => {
    const settings = structuredClone(defaultSettings);
    settings.macroButton.iconOn.codePoint = "10ffff";
    const { client } = await setup("macro", settings);
    const on = screen.getByRole("group", { name: "ON icon" });
    const off = screen.getByRole("group", { name: "OFF icon" });
    expect(within(on).getByRole("img").style.backgroundColor).not.toBe(within(off).getByRole("img").style.backgroundColor);
    expect(screen.queryByLabelText("Icon code point")).toBeNull();
    expect(within(on).getByRole("alert").textContent).toContain("not present");
    expect(within(on).getByRole("img").textContent).toBe("□");
    expect(within(off).queryByRole("alert")).toBeNull();
    expect(client.setSettings).not.toHaveBeenCalled();
    fireEvent.click(within(on).getByRole("button", { name: "Choose icon" }));
    fireEvent.click(within(screen.getByRole("dialog")).getByRole("button", { name: "volume_up" }));
    expect(within(on).queryByRole("alert")).toBeNull();
    expect((vi.mocked(client.setSettings).mock.calls.at(-1)![0] as Settings).macroButton.iconOn.codePoint).toBe("e050");
  });
  it("restores independent Combo values and never saves incomplete or invalid input", async () => {
    const settings = structuredClone(defaultSettings);
    settings.channels[1].gainStep = 0.5;
    const { client } = await setup("gain-control-combo", settings);
    const steps = screen.getAllByLabelText("Step (dB)") as HTMLInputElement[];
    expect(steps.map(input => input.value)).toEqual(["3", "0.5"]);
    fireEvent.change(steps[1], { target: { value: "" } });
    expect(client.setSettings).not.toHaveBeenCalled();
    expect(screen.getByRole("alert").textContent).toContain("positive");
    fireEvent.change(steps[1], { target: { value: "1.5" } });
    const saved = vi.mocked(client.setSettings).mock.calls.at(-1)![0] as Settings;
    expect(saved.channels.map(channel => channel.gainStep)).toEqual([3, 1.5]);
    expect(screen.queryByRole("alert")).toBeNull();
  });
  it("edits Macro ON and OFF independently and restores external settings", async () => {
    const { client, emit } = await setup("macro");
    const on = screen.getByRole("group", { name: "ON icon" });
    fireEvent.click(within(on).getByRole("button", { name: "Choose icon" }));
    fireEvent.change(within(screen.getByRole("dialog")).getByRole("searchbox"), { target: { value: "U+E050" } });
    fireEvent.click(within(screen.getByRole("dialog")).getByRole("button", { name: "volume_up" }));
    let saved = vi.mocked(client.setSettings).mock.calls.at(-1)![0] as Settings;
    expect(saved.macroButton.iconOn.codePoint).toBe("e050");
    expect(saved.macroButton.iconOff.codePoint).toBe(defaultSettings.macroButton.iconOff.codePoint);
    const next = structuredClone(defaultSettings);
    next.macroButton.logicalId = 42;
    emit("didReceiveSettings", next);
    expect((screen.getByLabelText("Button ID") as HTMLInputElement).value).toBe("42");
    fireEvent.change(screen.getByLabelText("Button mode"), { target: { value: "toggle" } });
    saved = vi.mocked(client.setSettings).mock.calls.at(-1)![0] as Settings;
    expect(saved.macroButton.mode).toBe("toggle");
    expect(saved.macroButton.logicalId).toBe(42);
  });
  it("keeps invalid saved settings visible until explicit reset", async () => {
    const { client } = await setup("gain-control", { schemaVersion: 99 });
    expect(screen.getByRole("alert").textContent).toContain("unsupported version");
    expect(client.setSettings).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Reset invalid settings" }));
    expect(client.setSettings).toHaveBeenCalledWith(defaultSettings);
  });
  it("uses Japanese translations for controls and live connection errors", async () => {
    const { emit } = await setup("gain-control", defaultSettings, "ja-JP");
    expect(screen.queryByLabelText("Step (dB)")).toBeNull();
    emit("sendToPropertyInspector", { event: "status", connected: true, edition: "banana" });
    expect(screen.getByRole("status").textContent).toContain("banana");
    emit("sendToPropertyInspector", { event: "status", connected: true, error: { code: "apiError", field: "" } });
    expect(screen.getByRole("alert").textContent).not.toContain("error.apiError");
  });
});
