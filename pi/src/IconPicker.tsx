import { useEffect, useId, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import type { IconSettings } from "./generated/contracts";
import { searchIcons, type IconEntry } from "./icons";

export function IconPicker({ value, onSelect, onClose }: { value: IconSettings; onSelect: (code: string) => void; onClose: () => void }) {
  const { t } = useTranslation();
  const titleId = useId();
  const dialog = useRef<HTMLDialogElement>(null);
  const search = useRef<HTMLInputElement>(null);
  const scroller = useRef<HTMLDivElement>(null);
  const [query, setQuery] = useState("");
  const [limit, setLimit] = useState(120);
  const [hovered, setHovered] = useState<IconEntry>();
  const results = useMemo(() => searchIcons(value.style, query), [value.style, query]);
  useEffect(() => {
    const previous = document.activeElement as HTMLElement | null;
    const element = dialog.current!;
    element.showModal();
    search.current?.focus();
    return () => { element.close(); previous?.focus(); };
  }, []);
  const more = () => setLimit(previous => previous + 120);
  return <dialog ref={dialog} className="icon-picker" aria-labelledby={titleId} onCancel={event => { event.preventDefault(); onClose(); }} onClick={event => { if (event.target === event.currentTarget) onClose(); }}>
    <div className="picker-content">
      <header><strong id={titleId}>{t("picker.title")}</strong><button type="button" aria-label={t("picker.close")} onClick={onClose}>×</button></header>
      <input ref={search} type="search" dir="ltr" aria-label={t("picker.search")} placeholder={t("picker.search")} value={query} onChange={event => {
        setQuery(event.target.value); setLimit(120); setHovered(undefined);
        if (scroller.current) scroller.current.scrollTop = 0;
      }} />
      <p className="picker-count" role="status">{t("picker.count", { count: results.length })}</p>
      <div ref={scroller} className="picker-scroll" onScroll={event => {
        const element = event.currentTarget;
        if (element.scrollTop + element.clientHeight >= element.scrollHeight - 80 && limit < results.length) more();
      }}>
        <div className="picker-grid" onKeyDown={event => {
          if (!["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown", "Home", "End"].includes(event.key)) return;
          const buttons = Array.from(event.currentTarget.querySelectorAll("button"));
          const index = buttons.indexOf(event.target as HTMLButtonElement);
          if (index < 0) return;
          const columns = getComputedStyle(event.currentTarget).gridTemplateColumns.split(" ").length;
          const rtl = getComputedStyle(event.currentTarget).direction === "rtl";
          const horizontal = event.key === "ArrowRight" ? (rtl ? -1 : 1) : (rtl ? 1 : -1);
          const next = event.key === "Home" ? 0 : event.key === "End" ? buttons.length - 1
            : index + (event.key === "ArrowUp" ? -columns : event.key === "ArrowDown" ? columns : horizontal);
          event.preventDefault();
          buttons[Math.max(0, Math.min(buttons.length - 1, next))]?.focus();
        }} style={{ fontFamily: `Symbols-${value.style}`, fontVariationSettings: `'FILL' ${Number(value.fill)}, 'wght' ${value.weight}, 'GRAD' ${value.grade}, 'opsz' ${value.opticalSize}` }}>
          {results.slice(0, limit).map(entry => <button type="button" key={entry.codePoint} title={`${entry.name} · U+${entry.codePoint.toUpperCase()}`} aria-label={entry.name} aria-pressed={Number.parseInt(value.codePoint, 16) === Number.parseInt(entry.codePoint, 16)} onMouseEnter={() => setHovered(entry)} onFocus={() => setHovered(entry)} onClick={() => onSelect(entry.codePoint)}>
            <span aria-hidden="true">{String.fromCodePoint(Number.parseInt(entry.codePoint, 16))}</span>
          </button>)}
        </div>
        {results.length === 0 && <p className="help">{t("picker.empty")}</p>}
        {limit < results.length && <button type="button" className="picker-more" onClick={more}>{t("picker.more")}</button>}
      </div>
      <footer dir="ltr">{hovered ? `${hovered.name} · U+${hovered.codePoint.toUpperCase()}` : t("picker.hint")}</footer>
    </div>
  </dialog>;
}
