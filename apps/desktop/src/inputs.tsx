import React, { useState, useEffect } from "react";
import {
  Check,
  Play,
  ArrowLeftRight,
  Pencil,
  Trash2,
  Plus,
  Keyboard,
  Mouse,
  HardDrive,
  Radio,
  Usb,
  Camera,
  Speaker,
  Network,
  Printer,
  Bluetooth,
  Save,
  X,
  RotateCcw,
  RefreshCw,
} from "lucide-react";
import { t, request, hex, monitorInputs, inputName, errorMessage } from "./api";
import type { Input, Monitor, MonitorState, Device } from "./api";
import { Field } from "./ui";
export function DeviceIcon({
  device,
  size = 17,
}: {
  device: Pick<Device, "category">;
  size?: number;
}) {
  const icon =
    (
      {
        keyboard: Keyboard,
        mouse: Mouse,
        storage: HardDrive,
        receiver: Radio,
        "input-composite": Keyboard,
        hid: Usb,
        audio: Speaker,
        camera: Camera,
        hub: Network,
        printer: Printer,
        wireless: Bluetooth,
        network: Network,
      } as Record<string, typeof Usb>
    )[device.category] ?? Usb;
  return React.createElement(icon, { size, "aria-hidden": true });
}
export function PortIcon({ value }: { value: number }) {
  return (
    <svg
      width="22"
      height="17"
      viewBox="0 0 24 18"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      aria-label={t([15, 16].includes(value) ? "ui.portDP" : "ui.portHDMI")}
    >
      <path
        d={
          [15, 16].includes(value)
            ? "M3 3h15l3 4v8H3z"
            : "M3 4h18v6l-4 5H7l-4-5z"
        }
      />
      <path d="M7 7h10M8 10h8" />
    </svg>
  );
}
export function useMonitorState(id: string, enabled = true) {
  const [state, setState] = useState<MonitorState | null>(null);
  useEffect(() => {
    setState(null);
    if (!id || !enabled) return;
    let gone = false,
      timer: ReturnType<typeof setTimeout>;
    async function read() {
      try {
        const next = await request<MonitorState>("monitors.state", {
          id,
          watch: true,
        });
        if (!gone) setState(next);
      } catch (e) {
        if (!gone)
          setState((s) => ({
            ...s,
            monitorId: id,
            ddcAvailable: false,
            error: { message: errorMessage(e) },
          }));
      } finally {
        if (!gone) timer = setTimeout(() => void read(), 300);
      }
    }
    void read();
    return () => {
      gone = true;
      clearTimeout(timer);
    };
  }, [id, enabled]);
  return state;
}
export function InputTiles({
  monitor,
  value,
  onChange,
  state,
  onEdit,
  onDelete,
  onTest,
}: {
  monitor?: Monitor;
  value: number;
  onChange: (n: number) => void;
  state?: MonitorState | null;
  onEdit?: (i: Input) => void;
  onDelete?: (i: Input) => void;
  onTest?: (i: Input) => void;
}) {
  return (
    <div
      className="input-tiles"
      role="radiogroup"
      aria-label={t("ui.targetInput")}
    >
      {monitorInputs(monitor, state?.currentInput).map((input) => {
        const current = state?.currentInput === input.value;
        const verified = input.verified || current;
        const base = [15, 16, 17, 18].includes(input.value)
          ? t(`input.${input.value}`)
          : t("ui.inputNumber", { value: input.value });
        const status = t(
          current
            ? "ui.currentInputDot"
            : verified
              ? "ui.verifiedReadback"
              : "ui.tryUnverified",
        );
        return (
          <div
            className={`input-tile ${value === input.value ? "selected" : ""} ${current ? "current" : ""}`}
            key={input.value}
          >
            <button
              type="button"
              role="radio"
              aria-checked={value === input.value}
              aria-label={inputName(input)}
              title={`${inputName(input)} · ${status}`}
              onClick={() => onChange(input.value)}
            >
              <PortIcon value={input.value} />
              <strong className="input-tile-name">{inputName(input)}</strong>
              <span
                className={`verification-dot ${verified ? "verified" : ""} ${current ? "current" : ""}`}
                aria-label={status}
              />
              <code>
                {hex(input.value)} / {input.value}
              </code>
            </button>
            {(onEdit || onDelete || onTest) && (
              <div className="input-tile-actions">
                {onEdit && (
                  <button
                    className="icon-button"
                    aria-label={t("ui.renameInput", { name: base })}
                    title={t("ui.renameInput", { name: base })}
                    onClick={() => onEdit(input)}
                  >
                    <Pencil size={12} />
                  </button>
                )}
                {onDelete && (
                  <button
                    className="icon-button"
                    aria-label={t([15,16,17,18].includes(input.value)&&input.label!==base?"ui.removeInputName":"ui.removeInput", { name: base })}
                    title={t([15,16,17,18].includes(input.value)&&input.label!==base?"ui.removeInputName":"ui.removeInput", { name: base })}
                    onClick={() => onDelete(input)}
                  >
                    <Trash2 size={12} />
                  </button>
                )}
                {onTest && (
                  <button
                    className="icon-button"
                    aria-label={t("ui.testNamedInput", { name: base })}
                    title={t("ui.testNamedInputTip", { name: base })}
                    onClick={() => onTest(input)}
                  >
                    <Play size={12} />
                  </button>
                )}
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}
export function MonitorCard({
  monitor,
  saved,
  act,
  confirm,
  available = true,
}: {
  monitor: Monitor;
  saved?: Monitor;
  available?: boolean;
  act: (c: string, p?: Record<string, unknown>) => Promise<unknown>;
  confirm: (c: { text: string; run: () => Promise<unknown> }) => void;
}) {
  const state = useMonitorState(monitor.id, available);
  const [source, setSource] = useState(18),
    [custom, setCustom] = useState(false),
    [raw, setRaw] = useState(""),
    [label, setLabel] = useState(""),
    [editing, setEditing] = useState(false),
    [error, setError] = useState(""),
    [touched, setTouched] = useState(false),
    [clock, setClock] = useState(Date.now());
  const sourceChosen = React.useRef(false);
  useEffect(() => {
    if (!sourceChosen.current && state?.currentInput)
      setSource(state.currentInput);
  }, [state?.currentInput]);
  const [local, setLocal] = useState<Monitor | undefined>(saved);
  useEffect(() => setLocal(saved), [saved]);
  useEffect(()=>{
    setClock(Date.now());
    if(!state?.test)return;
    if(["active","switching","restoring"].includes(state.test.phase)){const timer=setInterval(()=>setClock(Date.now()),100);return()=>clearInterval(timer);}
    if(state.test.phase==="restored"&&state.test.finishedAtMs){const timer=setTimeout(()=>setClock(Date.now()),Math.max(0,state.test.finishedAtMs+6000-Date.now()));return()=>clearTimeout(timer);}
  },[state?.test?.id,state?.test?.phase]);
  const currentName = state?.currentInput
    ? inputName(
        monitorInputs(local, state?.currentInput).find(
          (i) => i.value === state.currentInput,
        ) ?? {
          value: state.currentInput,
          vcpCode: 96,
          label: "",
          verified: true,
        },
      )
    : t("ui.reading");
  const parsed = /^0x[\da-f]+$/i.test(raw)
    ? parseInt(raw.slice(2), 16)
    : /^\d+$/.test(raw)
      ? Number(raw)
      : NaN;
  const invalid = !Number.isInteger(parsed) || parsed < 1 || parsed > 255;
  async function save() {
    setTouched(true);
    if (custom && invalid) return;
    try {
      setError("");
      const value = custom ? parsed : source;
      const input = (await act("monitors.input.alias.set", {
        id: monitor.id,
        value,
        label: label.trim() || ([15,16,17,18].includes(value)?t(`input.${value}`):t("ui.inputNumber", { value })),
      })) as Input;
      setLocal((m) => ({
        ...monitor,
        ...m,
        inputOptions: [
          ...(m?.inputOptions ?? []).filter((i) => i.value !== value),
          input,
        ],
        hiddenInputs: m?.hiddenInputs?.filter((v) => v !== value),
      }));
      setSource(value);
      setCustom(false);
      setEditing(false);
    } catch (e) {
      setError(errorMessage(e));
    }
  }
  function edit(i: Input) {
    sourceChosen.current = true;
    setSource(i.value);
    setLabel(i.label);
    setEditing(true);
    setCustom(false);
    setError("");
  }
  function test(i: Input) {
    confirm({
      text: t("ui.reversibleTestWarning", {
        target: inputName(i),
        current: currentName,
      }),
      run: async () => {
        try {
          await act("monitors.input.test", {
            id: monitor.id,
            value: i.value,
            confirm: true,
          });
        } catch (e) {
          setError(errorMessage(e));
          throw e;
        }
      },
    });
  }
  const testing =
    state?.test &&
    ["active", "switching", "restoring", "restore-failed"].includes(
      state.test.phase,
    );
  return (
    <section className="panel monitor-card">
      <div className="monitor-overview">
        <div className="display-illustration">
          <svg
            width="44"
            height="36"
            viewBox="0 0 44 36"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
          >
            <rect x="2" y="2" width="40" height="25" rx="3" />
            <path d="M22 27v6m-9 1h18" />
          </svg>
        </div>
        <div>
          <h2>{saved?.alias ?? monitor.name}</h2>
          <div className="inline-actions">
            <span className={`badge ${available ? "success" : "warning"}`}>
              {t(available ? "ui.available" : "ui.notAvailable")}
            </span>
            <span
              className={`badge ${state?.ddcAvailable === true ? "success" : state?.ddcAvailable === false ? "warning" : "neutral"}`}
              title={state?.error?.message}
            >
              {t(
                state?.ddcAvailable === true
                  ? "ui.ddcVerified"
                  : state?.ddcAvailable === false
                    ? "ui.ddcUnavailable"
                    : "ui.checkingDDC",
              )}
            </span>
          </div>
        </div>
        <div className="current-readout">
          <small>{t("ui.currentInput")}</small>
          <strong>{available ? currentName : t("ui.notAvailable")}</strong>
        </div>
      </div>
      <div className="monitor-inputs">
        <InputTiles
          monitor={local}
          state={state}
          value={source}
          onChange={(n) => {
            setSource(n);
            const i = monitorInputs(local, state?.currentInput).find(
              (i) => i.value === n,
            )!;
            edit(i);
          }}
          onEdit={edit}
          onDelete={(i) => {
            void act("monitors.input.alias.remove", {
              id: monitor.id,
              value: i.value,
              reset:[15,16,17,18].includes(i.value)&&i.label!==t(`input.${i.value}`),
            })
              .then((result) => {
                const reset=result as {reset?:boolean;input?:Input};
                setLocal((m) => ({
                  ...monitor,
                  ...m,
                  inputOptions: [...(m?.inputOptions?.filter(v=>v.value!==i.value)??[]),...(reset.input?[reset.input]:[])],
                  hiddenInputs: reset.reset?m?.hiddenInputs?.filter(v=>v!==i.value):[...(m?.hiddenInputs??[]),i.value],
                }));
                setEditing(false);
              })
              .catch((e) => setError(errorMessage(e)));
          }}
          onTest={available ? test : undefined}
        />
        <button
          className="text-button custom-input-button"
          onClick={() => {
            setCustom(true);
            setEditing(true);
            setRaw("");
            setLabel("");
            setTouched(false);
          }}
        >
          <Plus size={14} />
          {t("ui.addCustomInput")}
        </button>
        {editing && (
          <div className="source-form">
            {custom && (
              <Field
                label={t("ui.rawInputExample")}
                help={t("help.rawInput")}
                error={touched && invalid ? t("ui.rawInputRange") : undefined}
              >
                <input
                  value={raw}
                  placeholder={t("ui.rawInputPlaceholder")}
                  onChange={(e) => setRaw(e.target.value)}
                  onBlur={() => setTouched(true)}
                />
              </Field>
            )}
            <Field label={t("ui.inputAlias")} help={t("help.inputAlias")}>
              <input
                value={label}
                onChange={(e) => setLabel(e.target.value)}
                placeholder={t("ui.aliasExample")}
              />
            </Field>
            <div className="source-form-actions">
              <button
                className="button primary"
                disabled={custom && invalid}
                onClick={() => void save()}
              >
                <Save size={14} />
                {t("ui.saveInput")}
              </button>
              <button className="button" onClick={() => setEditing(false)}>
                <X size={14} />
                {t("ui.cancel")}
              </button>
            </div>
          </div>
        )}
        {error && (
          <div className="field-error" role="alert">
            {error}
          </div>
        )}
        {testing && (
          <div
            className={`test-countdown ${state?.test?.phase === "restore-failed" ? "warning" : ""}`}
            role="status"
          >
            <ArrowLeftRight size={17} />
            <div>
              <strong>
                {state?.test?.phase === "restore-failed"
                  ? t("ui.returnFailed")
                  : state?.test?.phase === "restoring"?t("ui.restoringInput"):t("ui.returnIn", {
                      seconds: Math.max(
                        0,
                        Math.ceil((state!.test!.deadlineMs - clock) / 1000),
                      ),
                      input: state!.test!.originalInput,
                    })}
              </strong>
              {state?.test?.error && <small>{state.test.error.message}</small>}
            </div>
            <button
              className="button"
              onClick={() =>
                void act("monitors.input.restore", { id: monitor.id }).catch(
                  (e) => setError(errorMessage(e)),
                )
              }
            >
              <RotateCcw size={13} />
              {t("ui.restoreNow")}
            </button>
          </div>
        )}
        {state?.test?.phase === "restored" && clock<(state.test.finishedAtMs??0)+6000 && (
          <div className="test-restored">
            <Check size={14} />
            {t("ui.inputRestored")}
          </div>
        )}
      </div>
      <div className="panel-foot monitor-actions">
        <button
          onClick={() =>
            void request("monitors.input.get", { id: monitor.id }).catch((e) =>
              setError(errorMessage(e)),
            )
          }
          disabled={!available}
          title={t("ui.refreshSourceTip")}
        >
          <RefreshCw size={13} />
          {t("ui.refreshSource")}
        </button>
        <span>{t("ui.liveReadInterval")}</span>
        <button
          onClick={() =>
            test(
              monitorInputs(local, state?.currentInput).find(
                (i) => i.value === source,
              ) ?? monitorInputs(local)[0],
            )
          }
          disabled={!available || !!testing || !monitorInputs(local).length}
          title={t("ui.testInputTip")}
        >
          <Play size={13} />
          {t("ui.testInput")}
        </button>
      </div>
    </section>
  );
}
