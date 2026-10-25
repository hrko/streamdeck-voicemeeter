import { useEffect, useId, useState, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import type { StreamDeckPiApi } from "@mikanseilaboratory/streamdeck-pi-client";
import { defaultSettings, type Settings, type IconSettings, type ChannelSettings, type Edition, type ValidationError, type GainButtonSettings } from "./generated/contracts";
import { channelLimits, restoreSettings, validateSettings } from "./settings";
import glyphs from "./generated/glyphs.json";
import { IconPicker } from "./IconPicker";

export interface Status { connected: boolean; edition?: Edition; error?: ValidationError | null }

function Row({ label, children }: { label: string; children: ReactNode }) {
  return <div className="row"><span className="row-label">{label}</span><div className="row-value">{children}</div></div>;
}

function Input({ label, value, onCommit, validate, numeric = false }: {
  label: string; value: string | number; onCommit: (value: string) => void;
  validate: (value: string) => string | undefined; numeric?: boolean;
}) {
  const { t } = useTranslation();
  const id = useId();
  const [draft, setDraft] = useState(String(value));
  const [error, setError] = useState<string>();
  useEffect(() => { setDraft(String(value)); setError(undefined); }, [value]);
  return <Row label={label}><input dir="ltr" aria-label={label} aria-invalid={!!error} aria-describedby={error ? `${id}-error` : undefined}
    inputMode={numeric ? "decimal" : "text"} value={draft} onChange={event => {
      const next = event.target.value;
      setDraft(next);
      const failure = validate(next);
      setError(failure);
      if (!failure) onCommit(next);
    }} />{error && <span id={`${id}-error`} className="error" role="alert">{t(`error.${error}`)}</span>}</Row>;
}

function ColorEditor({ label, value, onChange }: { label: string; value: string; onChange: (value: string) => void }) {
  const { t } = useTranslation();
  const id = useId();
  const [draft, setDraft] = useState(value);
  useEffect(() => setDraft(value), [value]);
  const invalid = !/^#[\da-f]{6}$/i.test(draft);
  return <Row label={label}>
    <div className="color-editor">
      <input type="color" aria-label={t("field.colorPicker", { label })} value={value.toLowerCase()} onChange={event => {
        setDraft(event.target.value);
        onChange(event.target.value);
      }} />
      <input type="text" dir="ltr" aria-label={label} aria-invalid={invalid} aria-describedby={invalid ? `${id}-error` : undefined} value={draft} spellCheck={false} onChange={event => {
        const next = event.target.value;
        setDraft(next);
        if (/^#[\da-f]{6}$/i.test(next)) onChange(next);
      }} />
    </div>
    {invalid && <span id={`${id}-error`} className="error" role="alert">{t("error.invalidColor")}</span>}
  </Row>;
}

function Select<T extends string | number>({ label, value, options, onChange }: {
  label: string; value: T; options: [T, string][]; onChange: (value: T) => void;
}) {
  return <Row label={label}><select aria-label={label} value={value} onChange={event => {
    const option = options.find(([value]) => String(value) === event.target.value);
    if (option) onChange(option[0]);
  }}>{options.map(([value, title]) => <option key={value} value={value}>{title}</option>)}</select></Row>;
}

function IconEditor({ value, onChange, background }: { value: IconSettings; onChange: (value: IconSettings) => void; background?: string }) {
  const { t } = useTranslation();
  const [picking, setPicking] = useState(false);
  const patch = (next: Partial<IconSettings>) => onChange({ ...value, ...next });
  const codePoint = Number.parseInt(value.codePoint, 16);
  const present = glyphs[value.style].includes(codePoint);
  return <>
    <button type="button" className="icon-picker-trigger" aria-label={t("picker.open")} aria-haspopup="dialog" onClick={() => setPicking(true)}>
      <span className="icon-preview" role="img" aria-label={t("preview.icon")} style={{ backgroundColor: background, fontFamily: `Symbols-${value.style}`, fontVariationSettings: `'FILL' ${Number(value.fill)}, 'wght' ${value.weight}, 'GRAD' ${value.grade}, 'opsz' ${value.opticalSize}` }}>{present ? String.fromCodePoint(codePoint) : "□"}</span>
      <span>{t("picker.open")}</span>
    </button>
    {picking && <IconPicker value={value} onClose={() => setPicking(false)} onSelect={codePoint => { patch({ codePoint }); setPicking(false); }} />}
    {!present && <p className="error" role="alert">{t("error.missingGlyph")}</p>}
    <details><summary>{t("section.font")}</summary>
      <Select label={t("field.style")} value={value.style} options={["outlined", "rounded", "sharp"].map(style => [style as IconSettings["style"], t(`style.${style}`)])} onChange={style => patch({ style })} />
      <Select label={t("field.weight")} value={value.weight} options={[100, 200, 300, 400, 500, 600, 700].map(n => [n, String(n)])} onChange={weight => patch({ weight })} />
      <Row label={t("field.fill")}><input aria-label={t("field.fill")} type="checkbox" checked={value.fill} onChange={e => patch({ fill: e.target.checked })} /></Row>
      <Select label={t("field.grade")} value={value.grade} options={[-25, 0, 200].map(n => [n, String(n)])} onChange={grade => patch({ grade })} />
      <Select label={t("field.opticalSize")} value={value.opticalSize} options={[20, 24, 40, 48].map(n => [n, String(n)])} onChange={opticalSize => patch({ opticalSize })} />
    </details>
  </>;
}

function ChannelEditor({ title, value, edition, onChange, targetOnly = false }: { title: string; value: ChannelSettings; edition?: Edition; onChange: (value: ChannelSettings) => void; targetOnly?: boolean }) {
  const { t, i18n } = useTranslation();
  const count = edition ? channelLimits[edition][value.target.kind] : 8;
  const label = (index: number) => t("channel.name", { kind: t(`kind.${value.target.kind}`), index: new Intl.NumberFormat(i18n.language).format(index) });
  return <fieldset><legend>{title}</legend>
    <Select label={t("field.kind")} value={value.target.kind} options={[["strip", t("kind.strip")], ["bus", t("kind.bus")]]} onChange={kind => onChange({ ...value, target: { ...value.target, kind } })} />
    <Row label={t("field.index")}><select aria-label={t("field.index")} value={value.target.index} onChange={event => {
      const index = Number(event.target.value);
      if (Number.isInteger(index) && index >= 0 && index < count) onChange({ ...value, target: { ...value.target, index } });
    }}>
      {value.target.index >= count && <option value={value.target.index} disabled>{label(value.target.index)} — {t("error.targetOutOfRange")}</option>}
      {Array.from({ length: count }, (_, index) => <option key={index} value={index}>{label(index)}</option>)}
    </select></Row>
    {!targetOnly && <><Input label={t("field.gainStep")} numeric value={value.gainStep} validate={text => text.trim() && Number.isFinite(Number(text)) && Number(text) > 0 && Number(text) <= 72 ? undefined : "invalidGainStep"} onCommit={text => onChange({ ...value, gainStep: Number(text) })} />
    <IconEditor value={value.icon} onChange={icon => onChange({ ...value, icon })} /></>}
  </fieldset>;
}

function GainButtonEditor({ value, onChange }: { value: GainButtonSettings; onChange: (value: GainButtonSettings) => void }) {
  const { t } = useTranslation();
  const patch = (next: Partial<GainButtonSettings>) => onChange({ ...value, ...next });
  const fader = value.operation === "adjust" && value.appearance !== "static";
  return <>
    <Select label={t("button.operation")} value={value.operation} options={(["adjust", "set", "mute"] as const).map(key => [key, t(`button.${key}`)])} onChange={operation => patch({ operation })} />
    {value.operation === "adjust" && <Input label={t("button.delta")} numeric value={value.delta} validate={text => text.trim() && Number.isFinite(Number(text)) && Number(text) !== 0 && Math.abs(Number(text)) <= 72 ? undefined : "invalidButtonDelta"} onCommit={text => patch({ delta: Number(text) })} />}
    {value.operation === "set" && <Input label={t("button.gain")} numeric value={value.gain} validate={text => text.trim() && Number.isFinite(Number(text)) && Number(text) >= -60 && Number(text) <= 12 ? undefined : "invalidButtonGain"} onCommit={text => patch({ gain: Number(text) })} />}
    {value.operation === "adjust" && <Select label={t("button.appearance")} value={value.appearance} options={(["static", "horizontal", "vertical"] as const).map(key => [key, t(`button.${key}`)])} onChange={appearance => patch({ appearance })} />}
    {fader ? <>
      <p className="help">{t(value.appearance === "horizontal" ? (value.delta > 0 ? "button.rightHalf" : "button.leftHalf") : (value.delta > 0 ? "button.upperHalf" : "button.lowerHalf"))}</p>
      <ColorEditor label={t("button.barColor")} value={value.barColor} onChange={barColor => patch({ barColor })} />
      <ColorEditor label={t("button.thumbColor")} value={value.thumbColor} onChange={thumbColor => patch({ thumbColor })} />
      <Row label={t("button.highlightPositive")}><input type="checkbox" aria-label={t("button.highlightPositive")} checked={value.highlightPositive} onChange={event => patch({ highlightPositive: event.target.checked })} /></Row>
      {value.highlightPositive && <ColorEditor label={t("button.positiveColor")} value={value.positiveColor} onChange={positiveColor => patch({ positiveColor })} />}
    </> : <>
      <fieldset><legend>{t("button.normalIcon")}</legend><IconEditor value={value.icon} background={value.background} onChange={icon => patch({ icon })} /></fieldset>
      <fieldset><legend>{t("button.mutedIcon")}</legend><IconEditor value={value.mutedIcon} background={value.mutedBackground} onChange={mutedIcon => patch({ mutedIcon })} /></fieldset>
    </>}
    <ColorEditor label={t("button.background")} value={value.background} onChange={background => patch({ background })} />
    <ColorEditor label={t("button.mutedBackground")} value={value.mutedBackground} onChange={mutedBackground => patch({ mutedBackground })} />
    <fieldset><legend>{t("button.details")}</legend>
      {(["showLabel", "showDb", "showMeter"] as const).map(key => <Row key={key} label={t(`button.${key}`)}><input type="checkbox" aria-label={t(`button.${key}`)} checked={value[key]} onChange={event => patch({ [key]: event.target.checked })} /></Row>)}
    </fieldset>
    <p className="help">{t("button.help")}</p>
  </>;
}

export function App({ client }: { client: StreamDeckPiApi }) {
  const { t } = useTranslation();
  const [restored, setRestored] = useState(() => restoreSettings(client.actionInfo?.payload?.settings ?? {}));
  const [status, setStatus] = useState<Status>({ connected: false });
  useEffect(() => client.subscribe("didReceiveSettings", value => setRestored(restoreSettings(value))), [client]);
  useEffect(() => {
    const unsubscribe = client.subscribe("sendToPropertyInspector", payload => {
      if (payload && typeof payload === "object" && "event" in payload && payload.event === "status" && "connected" in payload && typeof payload.connected === "boolean") {
        const message = payload as Status;
        const edition = ["basic", "banana", "potato"].includes(String(message.edition)) ? message.edition : undefined;
        const error = message.error && typeof message.error.code === "string" ? message.error : undefined;
        setStatus({ connected: message.connected, edition, error });
      }
    });
    client.sendToPlugin({ event: "getStatus" });
    return unsubscribe;
  }, [client]);
  const settings = restored.settings;
  const action = client.actionInfo?.action.split(".").at(-1);
  const save = (next: Settings) => {
    if (validateSettings(next)) return;
    setRestored({ settings: next });
    client.setSettings(next);
  };
  const error = restored.error ?? status.error ?? (settings ? validateSettings(settings, status.edition) : undefined);
  return <main>
    <p role="status" className="status">{status.connected ? t("status.connected", { edition: status.edition }) : t("status.disconnected")}</p>
    {error && <p role="alert" className="error">{t(`error.${error.code}`, { defaultValue: t("error.invalidSettings") })}</p>}
    {!settings ? <button onClick={() => save(structuredClone(defaultSettings))}>{t("settings.reset")}</button> : action === "gain-button" ? <>
      <ChannelEditor title={t("section.target")} value={settings.channels[0]} edition={status.connected ? status.edition : undefined} targetOnly onChange={channel => save({ ...settings, channels: [channel, settings.channels[1]] })} />
      <GainButtonEditor value={settings.gainButton} onChange={gainButton => save({ ...settings, gainButton })} />
    </> : action === "macro" ? <>
      <Input label={t("field.logicalId")} numeric value={settings.macroButton.logicalId} validate={text => /^\d+$/.test(text) && Number(text) < 80 ? undefined : "macroOutOfRange"} onCommit={text => save({ ...settings, macroButton: { ...settings.macroButton, logicalId: Number(text) } })} />
      <Select label={t("field.mode")} value={settings.macroButton.mode} options={[["push", t("mode.push")], ["toggle", t("mode.toggle")]]} onChange={mode => save({ ...settings, macroButton: { ...settings.macroButton, mode } })} />
      <p className="help">{t("help.macro")}</p>
      {(["On", "Off"] as const).map(state => <fieldset key={state}><legend>{t(`field.icon${state}`)}</legend>
        <IconEditor value={settings.macroButton[`icon${state}`]} background={settings.macroButton[`color${state}`]} onChange={icon => save({ ...settings, macroButton: { ...settings.macroButton, [`icon${state}`]: icon } })} />
        <ColorEditor label={t(`field.color${state}`)} value={settings.macroButton[`color${state}`]} onChange={color => save({ ...settings, macroButton: { ...settings.macroButton, [`color${state}`]: color } })} />
      </fieldset>)}
    </> : <>
      {(action === "gain-control-combo" ? [0, 1] as const : [0] as const).map(index => <ChannelEditor key={index} edition={status.connected ? status.edition : undefined} title={t(action === "gain-control-combo" ? (index === 0 ? "section.left" : "section.right") : "section.target")} value={settings.channels[index]} onChange={channel => {
        const channels: Settings["channels"] = [...settings.channels];
        channels[index] = channel;
        save({ ...settings, channels });
      }} />)}
      {action === "gain-control-combo" && <p className="help">{t("help.combo")}</p>}
    </>}
    {!status.connected && <p className="help">{t("help.offline")}</p>}
  </main>;
}
