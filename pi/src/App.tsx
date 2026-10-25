import { useEffect, useId, useRef, useState, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import type { StreamDeckPiApi } from "@mikanseilaboratory/streamdeck-pi-client";
import { defaultSettings, type Settings, type IconSettings, type ChannelSettings, type Edition, type ValidationError, type GainButtonSettings, type DialStyle } from "./generated/contracts";
import { channelLimits, restoreSettings, validateSettings } from "./settings";
import glyphs from "./generated/glyphs.json";
import { builtinPaletteNames, gainPalettes, gainPaletteName, meterColorKeys, paletteOf, readPalettes, samePalette, type UserPalette } from "./gainColors";
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

function ScriptEditor({label,value,onChange}:{label:string;value:string;onChange:(value:string)=>void}) {
  const {t}=useTranslation();
  const [draft,setDraft]=useState(value);
  useEffect(()=>setDraft(value),[value]);
  const invalid=draft.length>4096 || draft.includes("\0");
  const id=useId();
  return <Row label={label}><textarea className="script-editor" aria-label={label} aria-invalid={invalid} aria-describedby={invalid?id:undefined} dir="ltr" rows={5} spellCheck={false} value={draft} onChange={e=>{
    const next=e.target.value;setDraft(next);
    if(next.length<=4096 && !next.includes("\0")) onChange(next);
  }}/>{invalid && <span id={id} className="error" role="alert">{t("error.invalidScript")}</span>}<p className="help">{t("touch.scriptHelp")}</p></Row>;
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

export function GlobalSettingsEditor({client}:{client:StreamDeckPiApi}) {
  const {t}=useTranslation();
  const [rate,setRate]=useState(15);
  const [ready,setReady]=useState(false);
  const [busy,setBusy]=useState(false);
  const pending=useRef<number|null>(null);
  useEffect(()=>{
    const unsubscribe=client.subscribe("didReceiveGlobalSettings", value=>{
      const global=(value && typeof value === "object" ? value : {}) as Record<string,unknown>;
      const next=pending.current;
      pending.current=null;
      if(next!==null){
        client.setGlobalSettings({...global,refreshRate:next});
        setRate(next);setBusy(false);
      } else setRate([15,20,25,30].includes(global.refreshRate as number)?global.refreshRate as number:15);
      setReady(true);
    });
    client.getGlobalSettings();
    return unsubscribe;
  },[client]);
  return <details><summary>{t("global.title")}</summary>
    <fieldset disabled={!ready || busy}>
      <Select label={t("global.refreshRate")} value={rate} options={[15,20,25,30].map(fps=>[fps,`${fps} fps`])} onChange={fps=>{
        pending.current=fps;setBusy(true);client.getGlobalSettings();
      }}/>
      <p className="help">{t("global.refreshHelp")}</p>
    </fieldset>
  </details>;
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

function PaletteEditor({ client, value, onChange }: { client: StreamDeckPiApi; value: GainButtonSettings; onChange: (value: GainButtonSettings) => void }) {
  const { t } = useTranslation();
  const [items, setItems] = useState<UserPalette[] | null>(null);
  const [error, setError] = useState(false);
  const [busy, setBusy] = useState(false);
  const [name, setName] = useState("");
  const [editing, setEditing] = useState("");
  const [confirmDelete, setConfirmDelete] = useState(false);
  const pending = useRef<((latest: UserPalette[]) => UserPalette[]) | null>(null);
  useEffect(() => {
    const unsubscribe = client.subscribe("didReceiveGlobalSettings", global => {
      const latest = readPalettes(global);
      if (!latest) { pending.current = null; setBusy(false); setError(true); return; }
      setItems(latest); setError(false);
      if (pending.current) {
        const apply = pending.current; pending.current = null;
        try {
          const next = apply(latest);
          client.setGlobalSettings({ ...(global as Record<string, unknown>), gainColorPresets: { version: 1, items: next } });
          setItems(next);
        } catch { setError(true); }
        setBusy(false);
      }
    });
    client.getGlobalSettings();
    return unsubscribe;
  }, [client]);
  const matching = items?.find(item => samePalette(value, item.colors));
  useEffect(() => {
    if (matching && !editing) { setEditing(matching.id); setName(matching.name); }
  }, [matching?.id, editing]);
  const choice = matching?.id ?? gainPaletteName(value);
  const selected = items?.find(item => item.id === editing);
  const validName = !!name.trim() && name.trim().length <= 64;
  const mutate = (operation: (latest: UserPalette[]) => UserPalette[]) => {
    pending.current = operation; setBusy(true); client.getGlobalSettings();
  };
  return <fieldset><legend>{t("button.palette")}</legend>
    <Select label={t("button.palette")} value={choice} options={[
      ...([...builtinPaletteNames, "custom"] as const).map(key => [key, t(`button.palette.${key}`)] as [string,string]),
      ...(items ?? []).map(item => [item.id, item.name] as [string,string])
    ]} onChange={id => {
      setConfirmDelete(false);
      const user = items?.find(item => item.id === id);
      setEditing(user?.id ?? ""); setName(user?.name ?? "");
      if (user) onChange({ ...value, ...user.colors });
      else if (Object.hasOwn(gainPalettes,id)) onChange({ ...value, ...gainPalettes[id as keyof typeof gainPalettes] });
    }} />
    <details><summary>{t("palette.manage")}</summary>
      <p className="help">{t("palette.help")}</p>
      <Row label={t("palette.name")}><input aria-label={t("palette.name")} maxLength={64} value={name} onChange={event => setName(event.target.value)} /></Row>
      {selected && <p>{t("palette.editing", { name: selected.name })}</p>}
      <button disabled={!items || busy || error || !validName || items.length >= 100} onClick={() => {
        const item = { id: `user:${crypto.randomUUID()}`, name: name.trim(), colors: paletteOf(value) };
        mutate(latest => { if (latest.length >= 100) throw new Error("limit"); return [...latest, item]; });
        setEditing(item.id);
      }}>{t("palette.add")}</button>
      <button disabled={!selected || busy || error || !validName} onClick={() => {
        const id = selected!.id, colors = paletteOf(value), nextName = name.trim();
        mutate(latest => { if (!latest.some(item => item.id === id)) throw new Error("removed"); return latest.map(item => item.id === id ? { ...item, name: nextName, colors } : item); });
      }}>{t("palette.update")}</button>
      <button disabled={!selected || busy || error} onClick={() => setConfirmDelete(true)}>{t("palette.delete")}</button>
      {confirmDelete && selected && <div><p>{t("palette.confirm", { name: selected.name })}</p><button onClick={() => {
        const id = selected.id; mutate(latest => latest.filter(item => item.id !== id)); setEditing(""); setConfirmDelete(false);
      }}>{t("palette.confirmDelete")}</button><button onClick={() => setConfirmDelete(false)}>{t("palette.cancel")}</button></div>}
      {!items && !error && <p role="status">{t("palette.loading")}</p>}
      {error && <p role="alert">{t("palette.error")}</p>}
    </details>
  </fieldset>;
}

function GainButtonEditor({ client, value, onChange }: { client: StreamDeckPiApi; value: GainButtonSettings; onChange: (value: GainButtonSettings) => void }) {
  const { t } = useTranslation();
  const patch = (next: Partial<GainButtonSettings>) => onChange({ ...value, ...next });
  const fader = value.operation === "adjust" && value.appearance !== "static";
  return <>
    <Select label={t("button.operation")} value={value.operation} options={(["adjust", "set", "mute"] as const).map(key => [key, t(`button.${key}`)])} onChange={operation => patch({ operation })} />
    {value.operation === "adjust" && <Input label={t("button.delta")} numeric value={value.delta} validate={text => text.trim() && Number.isFinite(Number(text)) && Number(text) !== 0 && Math.abs(Number(text)) <= 72 ? undefined : "invalidButtonDelta"} onCommit={text => patch({ delta: Number(text) })} />}
    {value.operation === "set" && <Input label={t("button.gain")} numeric value={value.gain} validate={text => text.trim() && Number.isFinite(Number(text)) && Number(text) >= -60 && Number(text) <= 12 ? undefined : "invalidButtonGain"} onCommit={text => patch({ gain: Number(text) })} />}
    {value.operation === "adjust" && <Select label={t("button.appearance")} value={value.appearance} options={(["static", "horizontal", "vertical"] as const).map(key => [key, t(`button.${key}`)])} onChange={appearance => patch({ appearance })} />}
    {fader ? <p className="help">{t(value.appearance === "horizontal" ? (value.delta > 0 ? "button.rightHalf" : "button.leftHalf") : (value.delta > 0 ? "button.upperHalf" : "button.lowerHalf"))}</p> : <>
      <fieldset><legend>{t("button.normalIcon")}</legend><IconEditor value={value.icon} background={value.background} onChange={icon => patch({ icon })} /></fieldset>
      <fieldset><legend>{t("button.mutedIcon")}</legend><IconEditor value={value.mutedIcon} background={value.mutedBackground} onChange={mutedIcon => patch({ mutedIcon })} /></fieldset>
    </>}
    {fader && <><Row label={t("button.syncPair")}><input type="checkbox" aria-label={t("button.syncPair")} checked={value.syncPair} onChange={event => patch({ syncPair: event.target.checked })} /></Row><p className="help">{t("button.syncPairHelp")}</p></>}
    <fieldset><legend>{t("button.details")}</legend>
      {((fader ? [...(value.delta < 0 || value.syncPair ? ["faderLabel"] : []), "showDb", "showMeter"] : ["showLabel"]) as ("faderLabel" | "showLabel" | "showDb" | "showMeter")[]).map(key => <Row key={key} label={t(`button.${key}`)}><input type="checkbox" aria-label={t(`button.${key}`)} checked={value[key]} onChange={event => patch({ [key]: event.target.checked })} /></Row>)}
      {fader && value.delta > 0 && !value.syncPair && <p className="help">{t("button.labelNegativeOnly")}</p>}
      {fader && value.showMeter && <Row label={t("button.meterNotch")}><input type="checkbox" aria-label={t("button.meterNotch")} checked={value.meterNotch} onChange={event => patch({ meterNotch: event.target.checked })} /></Row>}
      {fader && value.showMeter && <Row label={t("button.swapMeter")}><input type="checkbox" aria-label={t("button.swapMeter")} checked={value.swapMeter} onChange={event => patch({ swapMeter: event.target.checked })} /></Row>}
    </fieldset>
    {fader ? <>
      <PaletteEditor client={client} value={value} onChange={onChange} />
      <details><summary>{t("palette.adjustColors")}</summary>
        <fieldset><legend>{t("palette.faderColors")}</legend>
          <ColorEditor label={t("button.barColor")} value={value.barColor} onChange={barColor => patch({ barColor })} />
          {(["keyMeterOpacity","keyInactiveOpacity","keyRailOpacity","keyThumbOpacity"] as const).map(key=><Row key={key} label={t(`button.${key}`)}><input type="range" min="0" max="100" aria-label={t(`button.${key}`)} value={value[key]} onChange={e=>patch({[key]:Number(e.target.value)})}/><output>{value[key]}%</output></Row>)}
          <Row label={t("button.keyThumbClearance")}><input type="checkbox" aria-label={t("button.keyThumbClearance")} checked={value.keyThumbClearance} onChange={e=>patch({keyThumbClearance:e.target.checked})}/></Row>
          <ColorEditor label={t("button.thumbColor")} value={value.thumbColor} onChange={thumbColor => patch({ thumbColor })} />
          <ColorEditor label={t("button.labelColor")} value={value.labelColor} onChange={labelColor => patch({ labelColor })} />
          <Row label={t("button.highlightPositive")}><input type="checkbox" aria-label={t("button.highlightPositive")} checked={value.highlightPositive} onChange={event => patch({ highlightPositive: event.target.checked })} /></Row>
          {value.highlightPositive && <ColorEditor label={t("button.positiveLabelColor")} value={value.positiveLabelColor} onChange={positiveLabelColor => patch({ positiveLabelColor })} />}
          {value.highlightPositive && <ColorEditor label={t("button.positiveColor")} value={value.positiveColor} onChange={positiveColor => patch({ positiveColor })} />}
        </fieldset>
        <fieldset><legend>{t("palette.keyColors")}</legend>
          <ColorEditor label={t("button.background")} value={value.background} onChange={background => patch({ background })} />
          <ColorEditor label={t("button.mutedBackground")} value={value.mutedBackground} onChange={mutedBackground => patch({ mutedBackground })} />
        </fieldset>
        <fieldset><legend>{t("meter.colors")}</legend>
          {meterColorKeys.slice(0,2).map(key => <ColorEditor key={key} label={t(`meter.${key}`)} value={value[key]} onChange={color => patch({ [key]: color })} />)}
          <fieldset><legend>{t("meter.normal")}</legend>{meterColorKeys.slice(2,5).map(key => <ColorEditor key={key} label={t(`meter.${key}`)} value={value[key]} onChange={color => patch({ [key]: color })} />)}</fieldset>
          <fieldset><legend>{t("meter.muted")}</legend>{meterColorKeys.slice(5).map(key => <ColorEditor key={key} label={t(`meter.${key}`)} value={value[key]} onChange={color => patch({ [key]: color })} />)}</fieldset>
        </fieldset>
      </details>
    </> : <>
      <ColorEditor label={t("button.background")} value={value.background} onChange={background => patch({ background })} />
      <ColorEditor label={t("button.mutedBackground")} value={value.mutedBackground} onChange={mutedBackground => patch({ mutedBackground })} />
    </>}
    {value.operation === "adjust" && <p className="help">{t("button.help")}</p>}
  </>;
}

function DialEditor({client,value,onChange}: {client:StreamDeckPiApi;value:DialStyle;onChange:(value:DialStyle)=>void}) {
  const {t}=useTranslation();
  const patch=(change:Partial<DialStyle>)=>onChange({...value,...change});
  const color=(key:keyof DialStyle["palette"],v:string|boolean)=>patch({palette:{...value.palette,[key]:v}});
  return <>
    <fieldset><legend>{t("dial.items")}</legend>
      {(["showLabel","showGain","showIcon","showBar","showMeter","showMute","showSolo","showRouting","showMono","showEq"] as const).map(key=><Row key={key} label={t(`dial.${key}`)}><input type="checkbox" aria-label={t(`dial.${key}`)} checked={value[key]} onChange={e=>patch({[key]:e.target.checked})}/></Row>)}
      <Row label={t("dial.indicatorOutline")}><input type="checkbox" aria-label={t("dial.indicatorOutline")} checked={value.palette.indicatorOutline} onChange={e=>color("indicatorOutline",e.target.checked)}/></Row>

    </fieldset>
    {value.showMeter && <fieldset><legend>{t("dial.meterOptions")}</legend>
      <Row label={t("dial.meterStyle")}><select aria-label={t("dial.meterStyle")} value={value.meterStyle} onChange={e=>patch({meterStyle:e.target.value as DialStyle["meterStyle"]})}><option value="cells">{t("dial.meterCells")}</option><option value="continuous">{t("dial.meterContinuous")}</option></select></Row>
      <Row label={t("dial.meterTrackOpacity")}><input type="range" min="0" max="100" step="1" aria-label={t("dial.meterTrackOpacity")} value={value.palette.meterTrackOpacity} onChange={e=>patch({palette:{...value.palette,meterTrackOpacity:Number(e.target.value)}})}/><output>{value.palette.meterTrackOpacity}%</output></Row>
      {value.showMeter && <Row label={t("button.meterNotch")}><input type="checkbox" aria-label={t("button.meterNotch")} checked={value.meterNotch} onChange={e=>patch({meterNotch:e.target.checked})}/></Row>}
    </fieldset>}
    <PaletteEditor client={client} value={{...defaultSettings.gainButton,...value.palette}} onChange={v=>patch({palette:paletteOf(v)})}/>
    <details><summary>{t("palette.adjustColors")}</summary>
      <fieldset><legend>{t("dial.barColors")}</legend>
        {(["barColor","positiveColor"] as const).map(key=><ColorEditor key={key} label={t(`button.${key}`)} value={value.palette[key]} onChange={v=>color(key,v)}/>)}
        <Row label={t("button.highlightPositive")}><input type="checkbox" aria-label={t("button.highlightPositive")} checked={value.palette.highlightPositive} onChange={e=>color("highlightPositive",e.target.checked)}/></Row>
        <ColorEditor label={t("dial.labelColor")} value={value.palette.dialLabelColor} onChange={v=>color("dialLabelColor",v)}/>
        <ColorEditor label={t("dial.positiveLabelColor")} value={value.palette.dialPositiveLabelColor} onChange={v=>color("dialPositiveLabelColor",v)}/>
        <ColorEditor label={t("dial.emptyBarColor")} value={value.palette.emptyBarColor} onChange={v=>color("emptyBarColor",v)}/>
        <ColorEditor label={t("dial.gainColor")} value={value.palette.dialGainColor} onChange={v=>color("dialGainColor",v)}/>
        <ColorEditor label={t("dial.positiveGainColor")} value={value.palette.dialPositiveGainColor} onChange={v=>color("dialPositiveGainColor",v)}/>
      </fieldset>
      <fieldset><legend>{t("dial.indicatorColors")}</legend>
        <ColorEditor label={t("selection.color")} value={value.palette.selectionColor} onChange={v=>color("selectionColor",v)}/>
        <ColorEditor label={t("dial.indicatorOutlineColor")} value={value.palette.indicatorOutlineColor} onChange={v=>color("indicatorOutlineColor",v)}/>
        <ColorEditor label={t("dial.indicatorBackground")} value={value.palette.indicatorBackground} onChange={v=>color("indicatorBackground",v)}/>
        <Row label={t("dial.indicatorOpacity")}><input type="range" min="0" max="100" step="1" aria-label={t("dial.indicatorOpacity")} value={value.palette.indicatorOpacity} onChange={e=>patch({palette:{...value.palette,indicatorOpacity:Number(e.target.value)}})}/><output>{value.palette.indicatorOpacity}%</output></Row>
      </fieldset>
      <fieldset><legend>{t("dial.background")}</legend>
        {(["backgroundOpacity","mutedBackgroundOpacity"] as const).map(key=><Row key={key} label={t(`dial.${key}`)}><input type="range" min="0" max="100" step="10" aria-label={t(`dial.${key}`)} value={value.palette[key]} onChange={e=>patch({palette:{...value.palette,[key]:Number(e.target.value)}})}/><output>{value.palette[key]}%</output></Row>)}
        {(["background","mutedBackground"] as const).map(key=><ColorEditor key={key} label={t(`button.${key}`)} value={value.palette[key]} onChange={v=>color(key,v)}/>)}
      </fieldset>
      <fieldset><legend>{t("meter.colors")}</legend>
        {meterColorKeys.filter(key=>key!=="meterBackground").map(key=><ColorEditor key={key} label={t(`meter.${key}`)} value={value.palette[key]} onChange={v=>color(key,v)}/>)}
      </fieldset>
    </details>
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
      <GainButtonEditor client={client} value={settings.gainButton} onChange={gainButton => save({ ...settings, gainButton })} />
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
      {action === "gain-control" && <fieldset><legend>{t("touch.title")}</legend>
        {(["tapOperation","holdOperation"] as const).map(key => {
          const scriptKey=key==="tapOperation"?"tapScript":"holdScript";
          return <div key={key}><Select label={t(`touch.${key}`)} value={settings.dial[key]} options={[["toggleMute",t("touch.toggleMute")],["resetGain",t("touch.resetGain")],["script",t("touch.script")],["none",t("touch.none")]]} onChange={operation=>save({...settings,dial:{...settings.dial,[key]:operation}})}/>
            {settings.dial[key]==="script" && <ScriptEditor label={t(`touch.${scriptKey}`)} value={settings.dial[scriptKey]} onChange={script=>save({...settings,dial:{...settings.dial,[scriptKey]:script}})}/>}</div>;
        })}
      </fieldset>}
      {action === "gain-control-combo" && <fieldset><legend>{t("selection.mode")}</legend>
        <Select label={t("selection.mode")} value={settings.dial.selectChannel?"select":"hold"} options={[["hold",t("selection.hold")],["select",t("selection.select")]]} onChange={mode=>save({...settings,dial:{...settings.dial,selectChannel:mode==="select"}})}/>
        {settings.dial.selectChannel && <>
          <Select label={t("selection.switchMethod")} value={settings.dial.toggleSelection?"toggle":"position"} options={[["position",t("selection.position")],["toggle",t("selection.toggle")]]} onChange={method=>save({...settings,dial:{...settings.dial,toggleSelection:method==="toggle"}})}/>
          <p className="help">{t("selection.help")}</p>
          <Select label={t("selection.marker")} value={settings.dial.selectionMarker} options={(["A","B","C","D","E","F","G","H"] as const).map(v=>[v,t(`selection.${v}`)])} onChange={selectionMarker=>save({...settings,dial:{...settings.dial,selectionMarker}})}/>
          {([0,1] as const).map(index=><div key={index}>
            <Select label={t(index===0?"selection.leftHold":"selection.rightHold")} value={settings.dial.comboHold[index]} options={[["none",t("touch.none")],["toggleMute",t("touch.toggleMute")],["resetGain",t("touch.resetGain")],["script",t("touch.script")]]} onChange={op=>{const comboHold=[...settings.dial.comboHold] as Settings["dial"]["comboHold"];comboHold[index]=op;save({...settings,dial:{...settings.dial,comboHold}});}}/>
            {settings.dial.comboHold[index]==="script" && <ScriptEditor label={t(index===0?"selection.leftScript":"selection.rightScript")} value={settings.dial.comboScripts[index]} onChange={script=>{const comboScripts=[...settings.dial.comboScripts] as [string,string];comboScripts[index]=script;save({...settings,dial:{...settings.dial,comboScripts}});}}/>}
          </div>)}
        </>}
      </fieldset>}
      {action === "gain-control-combo" && !settings.dial.selectChannel && <p className="help">{t("help.combo")}</p>}
      {action === "gain-control-combo" && <Row label={t("dial.shared")}><input type="checkbox" aria-label={t("dial.shared")} checked={settings.dial.shared} onChange={e=>save({...settings,dial:{...settings.dial,shared:e.target.checked}})}/></Row>}
      {(action === "gain-control-combo" && !settings.dial.shared ? [0,1] as const : [0] as const).map(index=><fieldset key={index}><legend>{t(action !== "gain-control-combo" ? "dial.appearance" : settings.dial.shared ? "dial.sharedAppearance" : index===1 ? "dial.rightAppearance" : "dial.leftAppearance")}</legend><DialEditor client={client} value={settings.dial.styles[index]} onChange={style=>{
        const styles:Settings["dial"]["styles"]=[...settings.dial.styles]; styles[index]=style;
        save({...settings,dial:{...settings.dial,styles}});
      }}/></fieldset>)}
    </>}
    <GlobalSettingsEditor client={client}/>
  </main>;
}
