import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, within, act } from "@testing-library/react";
import { I18nextProvider } from "react-i18next";
import type { StreamDeckPiApi } from "@mikanseilaboratory/streamdeck-pi-client";
import { App } from "./App";
import { createLocalization } from "./i18n";
import { defaultSettings, type Settings } from "./generated/contracts";

afterEach(cleanup);
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
    const { client } = await setup("macro");
    const on = screen.getByRole("group", { name: "ON icon" });
    const off = screen.getByRole("group", { name: "OFF icon" });
    expect(within(on).getByRole("img").style.backgroundColor).not.toBe(within(off).getByRole("img").style.backgroundColor);
    fireEvent.change(within(on).getByLabelText("Icon code point"), { target: { value: "10ffff" } });
    expect(within(on).getByRole("alert").textContent).toContain("not present");
    expect(within(on).getByRole("img").textContent).toBe("□");
    expect(within(off).queryByRole("alert")).toBeNull();
    expect(client.setSettings).toHaveBeenCalled();
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
    fireEvent.change(within(on).getByLabelText("Icon code point"), { target: { value: "e051" } });
    let saved = vi.mocked(client.setSettings).mock.calls.at(-1)![0] as Settings;
    expect(saved.macroButton.iconOn.codePoint).toBe("e051");
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
