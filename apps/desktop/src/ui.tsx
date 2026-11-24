import { createPortal } from "react-dom";
import React, { useEffect, useState, useId, useRef } from "react";
import { X, Check, CircleHelp } from "lucide-react";
import { t } from "./api";
export function Modal({
  title,
  close,
  children,
}: {
  title: string;
  close: () => void;
  children: React.ReactNode;
}) {
  const ref = React.useRef<HTMLDivElement>(null);
  useEffect(() => {
    const before = document.activeElement as HTMLElement;
    const node = ref.current;
    node?.querySelector<HTMLElement>("button,input,select")?.focus();
    const handle = (e: KeyboardEvent) => {
      if(Array.from(document.querySelectorAll(".modal")).at(-1)!==node)return;
      if (e.key === "Escape" && !document.querySelector(".fixed-help")) close();
      if (e.key === "Tab" && node) {
        const all = node.querySelectorAll<HTMLElement>(
          "button:not([disabled]),input,select,textarea",
        );
        const first = all[0],
          last = all[all.length - 1];
        if (e.shiftKey && document.activeElement === first) {
          last.focus();
          e.preventDefault();
        } else if (!e.shiftKey && document.activeElement === last) {
          first.focus();
          e.preventDefault();
        }
      }
    };
    document.addEventListener("keydown", handle);
    return () => {
      document.removeEventListener("keydown", handle);
      before?.focus();
    };
  }, []);
  return (
    <div
      className="modal-backdrop"
      onMouseDown={(e) => {
        if (e.target === e.currentTarget) close();
      }}
    >
      <div
        role="dialog"
        aria-modal="true"
        aria-label={title}
        className="modal"
        ref={ref}
      >
        <div className="modal-head">
          <h2>{title}</h2>
          <button
            className="icon-button"
            aria-label={t("ui.close")}
            onClick={close}
          >
            <X size={20} />
          </button>
        </div>
        <div className="modal-body">{children}</div>
      </div>
    </div>
  );
}
export function Switch({
  checked,
  onChange,
  label,
}: {
  checked: boolean;
  onChange: () => void;
  label: string;
}) {
  return (
    <button
      role="switch"
      aria-checked={checked}
      aria-label={label}
      className={`switch ${checked ? "on" : ""}`}
      onClick={onChange}
    >
      <span />
    </button>
  );
}

// Only one help popover may be open at a time; opening another closes it.
let closeOpenHelp: (() => void) | null = null;
export function Help({ label, text }: { label: string; text?: string }) {
  const ref = React.useRef<HTMLButtonElement>(null),
    pop = React.useRef<HTMLDivElement>(null);
  const [open, setOpenState] = useState(false),
    [pinned, setPinned] = useState(false);
  const id = useId();
  const hide = React.useCallback(() => {
    setOpenState(false);
    setPinned(false);
  }, []);
  const show = () => {
    if (closeOpenHelp && closeOpenHelp !== hide) closeOpenHelp();
    closeOpenHelp = hide;
    setOpenState(true);
  };
  useEffect(() => {
    if (!open) {
      if (closeOpenHelp === hide) closeOpenHelp = null;
      return;
    }
    const key = (e: KeyboardEvent) => {
      if (e.key === "Escape") hide();
    };
    const outside = (e: MouseEvent) => {
      const target = e.target as Node;
      if (!ref.current?.contains(target) && !pop.current?.contains(target))
        hide();
    };
    window.addEventListener("keydown", key);
    document.addEventListener("mousedown", outside, true);
    return () => {
      window.removeEventListener("keydown", key);
      document.removeEventListener("mousedown", outside, true);
    };
  }, [open, hide]);
  const rect = ref.current?.getBoundingClientRect();
  const height = Math.min(360, window.innerHeight - 30);
  const top = rect
    ? Math.max(10, Math.min(rect.bottom + 8, window.innerHeight - height - 10))
    : 10;
  const left = rect
    ? Math.max(10, Math.min(rect.left - 80, window.innerWidth - 350))
    : 10;
  return (
    <span
      className="help-anchor"
      onMouseEnter={show}
      onMouseLeave={() => {
        if (!pinned) hide();
      }}
    >
      <button
        ref={ref}
        type="button"
        className="help-button"
        aria-label={t("ui.helpFor", { label })}
        data-no-tooltip
        aria-expanded={open}
        aria-describedby={open ? id : undefined}
        onFocus={show}
        onBlur={() => {
          if (!pinned) hide();
        }}
        onClick={() => {
          if (pinned) hide();
          else {
            show();
            setPinned(true);
          }
        }}
      >
        <CircleHelp size={14} />
      </button>
      {open &&
        createPortal(
          <div
            ref={pop}
            className="help-popover fixed-help"
            id={id}
            role="tooltip"
            style={{ top, left, maxHeight: height }}
            onMouseEnter={show}
            onMouseLeave={() => {
              if (!pinned) hide();
            }}
          >
            <strong>{label}</strong>
            <p>{text ?? t("help.generic")}</p>
          </div>,
          document.body,
        )}
    </span>
  );
}
/** Gives every button a native tooltip: its explicit title, else its
 * accessible label, else its visible text. */
export function useButtonTooltips() {
  useEffect(() => {
    const over = (e: MouseEvent) => {
      const el = (e.target as Element | null)?.closest?.<HTMLElement>(
        "button,[role=switch],[role=radio],summary",
      );
      if (!el || el.hasAttribute("title") || el.hasAttribute("data-no-tooltip"))
        return;
      const label =
        el.getAttribute("aria-label") ?? el.textContent?.trim() ?? "";
      if (label) el.setAttribute("title", label);
    };
    document.addEventListener("mouseover", over, true);
    return () => document.removeEventListener("mouseover", over, true);
  }, []);
}
export function Field({
  label,
  help,
  error,
  children,
}: {
  label: string;
  help?: string;
  error?: string;
  children: React.ReactElement<any>;
}) {
  const id = useId();
  return (
    <div className={`field ${error ? "invalid" : ""}`}>
      <div className="field-title">
        <label htmlFor={id}>{label}</label>
        {help && <Help label={label} text={help} />}
      </div>
      {React.cloneElement(children, {
        id,
        "aria-label": label,
        "aria-invalid": !!error,
        "aria-describedby": error ? id + "-error" : undefined,
      })}
      {error && (
        <span className="field-error" id={id + "-error"} role="alert">
          {error}
        </span>
      )}
    </div>
  );
}
const helpKeys = [
  "theme",
  "uiMode",
  "closeAction",
  "language",
  "agentLogin",
  "guiLogin",
  "agentStandalone",
  "globalCooldown",
  "startupGrace",
  "resumeGrace",
  "dryRun",
  "commandTimeout",
  "commandDirectory",
  "commandsEnabled",
  "commandShell",
  "onConnect",
  "onDisconnect",
  "setting.startupDelayMs",
  "setting.defaultDebounceMs",
  "setting.commandTimeoutMs",
  "diagnosticReport",
  "configuration",
  "daemon",
  "retention",
  "logSize",
];
export function Setting({
  label,
  detail,
  children,
}: {
  label: string;
  detail: string;
  children: React.ReactNode;
}) {
  const key = helpKeys.find((k) => t(`ui.${k}`) === label);
  const text = key ? t(`help.${key}`) : detail;
  return (
    <div className="setting" role="group" aria-label={label}>
      <div>
        <strong>
          {label}
          <Help label={label} text={text === `help.${key}` ? detail : text} />
        </strong>
        <p>{detail}</p>
      </div>
      {children}
    </div>
  );
}
export function NumberSetting({
  value,
  min,
  max,
  save,
}: {
  value: number;
  min: number;
  max: number;
  save: (v: number) => void;
}) {
  const [n, setN] = useState(String(value)),
    [touched, setTouched] = useState(false);
  useEffect(() => setN(String(value)), [value]);
  const invalid =
    n.trim() === "" ||
    !Number.isInteger(Number(n)) ||
    Number(n) < min ||
    Number(n) > max;
  return (
    <div className="number-validation">
      <div className="number-setting">
        <input
          type="number"
          aria-label={t("ui.milliseconds")}
          min={min}
          max={max}
          value={n}
          aria-invalid={touched && invalid}
          onChange={(e) => setN(e.target.value)}
          onBlur={() => setTouched(true)}
        />
        <span>{t("ui.ms")}</span>
        <button
          className="icon-button"
          aria-label={t("ui.save")}
          disabled={invalid}
          onClick={() => save(Number(n))}
        >
          <Check size={17} />
        </button>
      </div>
      {touched && invalid && (
        <span className="field-error" role="alert">
          {t("ui.numberRange", { min, max })}
        </span>
      )}
    </div>
  );
}
