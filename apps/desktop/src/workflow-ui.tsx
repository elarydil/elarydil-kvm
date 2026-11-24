import React, { useState, useEffect } from "react";
import {
  Check,
  Workflow,
  ArrowRight,
  ArrowLeft,
  ArrowLeftRight,
  ArrowUpDown,
  ArrowUp,
  ArrowDown,
  Radio,
  LoaderCircle,
  Plug,
  Unplug,
  Terminal,
  Play,
  X,
  RotateCcw,
  ListChecks,
  Info,
  Pause,
} from "lucide-react";
import { request, t, monitorInputs, inputName, errorMessage } from "./api";
import type {
  Config,
  Device,
  Monitor,
  Profile,
  Rule,
  CommandHooks,
} from "./api";
import { Modal, Field, Setting, Switch, Help } from "./ui";
import { InputTiles, DeviceIcon, useMonitorState } from "./inputs";
const defaultCommands = (): CommandHooks => ({
  enabled: false,
  onConnect: "",
  onDisconnect: "",
  shell: "system",
  workingDirectory: "",
  timeoutMs: 10000,
});
const range = (v: number, min: number, max: number) =>
  !Number.isInteger(v) || v < min || v > max
    ? t("ui.numberRange", { min, max })
    : "";
export function RuleSummary({
  rule,
  monitor,
}: {
  rule: Pick<
    Rule,
    "mode" | "minimumCount" | "deviceIds" | "trigger" | "debounceMs" | "actions"
  >;
  monitor?: Monitor;
}) {
  const value = rule.actions[0]?.inputValue ?? 18;
  const input = monitorInputs(monitor).find((i) => i.value === value);
  return (
    <span className="rule-narrative">
      {t("ui.previewWhen")}{" "}
      {t(`ui.modeSummary.${rule.mode}`, { count: rule.minimumCount })}{" "}
      <em>{t("ui.selectedCount", { count: rule.deviceIds.length })}</em>{" "}
      {t("ui.deviceNoun", { count: rule.deviceIds.length })}{" "}
      {t(`ui.eventVerb.${rule.trigger}`)},{" "}
      {t("ui.previewWait")}{" "}
      <em>
        {rule.debounceMs} {t("ui.ms")}
      </em>
      , {t("ui.previewThen")}{" "}
      {t("ui.requestInput")}{" "}
      <em>
        {input ? inputName(input) : t("ui.inputNumber", { value })}
      </em>
      .
    </span>
  );
}
export function RuleEditor({
  rule,
  profileId,
  config,
  devices,
  monitors,
  selected,
  act,
  close,
  confirm,
}: {
  rule: Rule | null;
  profileId: string;
  config: Config | null;
  devices: Device[];
  monitors: Monitor[];
  selected: string[];
  act: (c: string, p?: Record<string, unknown>) => Promise<unknown>;
  close: () => void;
  confirm?: (c: { text: string; run: () => Promise<unknown> }) => void;
}) {
  const [name, setName] = useState(
      rule?.name ?? t(`wizard.rule.${rule?.trigger ?? "disconnect"}`),
    ),
    [nameEdited, setNameEdited] = useState(!!rule),
    [profileName, setProfileName] = useState(""),
    [ids, setIds] = useState(rule?.deviceIds ?? selected),
    [mode, setMode] = useState(rule?.mode ?? "all"),
    [event, setEvent] = useState(rule?.trigger ?? "disconnect"),
    [minimum, setMinimum] = useState(rule?.minimumCount ?? 1),
    [monitor, setMonitor] = useState(
      rule?.actions[0]?.monitorId ??
        monitors[0]?.id ??
        config?.monitors[0]?.id ??
        "",
    ),
    [input, setInput] = useState(rule?.actions[0]?.inputValue ?? 18),
    [debounce, setDebounce] = useState(rule?.debounceMs ?? 0),
    [rearm, setRearm] = useState(rule?.rearmMs ?? 500),
    [cooldown, setCooldown] = useState(rule?.cooldownMs ?? 500),
    [window, setWindow] = useState(rule?.coincidenceWindowMs ?? 0),
    [dry, setDry] = useState(rule?.dryRun ?? false),
    [commands, setCommands] = useState<CommandHooks>(
      rule?.commands ?? defaultCommands(),
    ),
    [touched, setTouched] = useState<Record<string, boolean>>({}),
    [error, setError] = useState(""),
    [server, setServer] = useState<Record<string, string>>({}),
    [saving, setSaving] = useState(false);
  const available = [
    ...monitors,
    ...(config?.monitors ?? []).filter(
      (m) => !monitors.some((l) => l.id === m.id),
    ),
  ];
  const chosen =
    config?.monitors.find((m) => m.id === monitor) ??
    available.find((m) => m.id === monitor);
  const state = useMonitorState(
    monitor,
    monitors.some((m) => m.id === monitor),
  );
  const allDevices = [
    ...(config?.usbDevices ?? []).map((d) => ({
      ...devices.find((x) => x.id === d.id),
      id: d.id,
      name: d.alias,
      category: devices.find((x) => x.id === d.id)?.category ?? "other",
    })),
    ...devices.filter((d) => !config?.usbDevices.some((s) => s.id === d.id)),
  ];
  const errors: Record<string, string> = {
    name: name.trim() ? "" : t("ui.required"),
    ids: ids.length ? "" : t("ui.selectOneDevice"),
    monitor: monitor ? "" : t("ui.chooseMonitor"),
    input: monitorInputs(chosen, state?.currentInput).some(
      (i) => i.value === input,
    )
      ? ""
      : t("ui.chooseInput"),
    debounce: range(debounce, 0, 60000),
    rearm: range(rearm, 0, 60000),
    cooldown: range(cooldown, 0, 120000),
    window: range(window, 0, 120000),
    minimum:
      mode === "at-least" ? range(minimum, 1, Math.max(ids.length, 1)) : "",
    timeout: range(commands.timeoutMs, 100, 300000),
    onConnect:
      new TextEncoder().encode(commands.onConnect).length > 16384 ||
      commands.onConnect.includes("\0")
        ? t("ui.commandTextLimit")
        : "",
    onDisconnect:
      new TextEncoder().encode(commands.onDisconnect).length > 16384 ||
      commands.onDisconnect.includes("\0")
        ? t("ui.commandTextLimit")
        : "",
    directory:
      new TextEncoder().encode(commands.workingDirectory).length > 4096 ||
      commands.workingDirectory.includes("\0")
        ? t("ui.commandDirectoryLimit")
        : "",
    commands:
      commands.enabled &&
      !commands.onConnect.trim() &&
      !commands.onDisconnect.trim()
        ? t("ui.commandRequired")
        : "",
  };
  const valid = Object.values(errors).every((e) => !e);
  const touch = (key: string) => setTouched((v) => ({ ...v, [key]: true }));
  const err = (key: string) =>
    server[key] || (touched[key] ? errors[key] : undefined);
  async function save() {
    setTouched(Object.fromEntries(Object.keys(errors).map((k) => [k, true])));
    if (!valid) return;
    setSaving(true);
    setError("");
    setServer({});
    try {
      if (!config?.monitors.some((m) => m.id === monitor))
        await act("monitors.remember", { id: monitor });
      for (const id of ids)
        if (!config?.usbDevices.some((d) => d.id === id))
          await act("devices.track", { id });
      let p = profileId;
      if (!p)
        p = (
          (await act("profiles.create", {
            name: profileName.trim() || t("wizard.default_profile"),
          })) as Profile
        ).id;
      await act(rule ? "rules.set" : "rules.create", {
        id: rule?.id,
        profile: p,
        name: name.trim(),
        on: event,
        devices: ids,
        mode,
        "min-count": minimum,
        monitor,
        input,
        "debounce-ms": debounce,
        "rearm-ms": rearm,
        "cooldown-ms": cooldown,
        "coincidence-window-ms": window,
        "dry-run": dry,
        commands,
      });
      close();
    } catch (e) {
      const key =
        typeof e === "object" && e && "field" in e
          ? String(e.field).split(".").pop()
          : undefined;
      const map: Record<string, string> = {
        cooldownMs: "cooldown",
        debounceMs: "debounce",
        rearmMs: "rearm",
        coincidenceWindowMs: "window",
        timeoutMs: "timeout",
      };
      if (key && map[key]) setServer({ [map[key]]: errorMessage(e) });
      setError(errorMessage(e));
    } finally {
      setSaving(false);
    }
  }
  const numeric = (
    key: string,
    label: string,
    value: number,
    set: (v: number) => void,
    min: number,
    max: number,
    help: string,
  ) => (
    <Field key={key} label={label} help={t(help)} error={err(key)}>
      <input
        type="number"
        min={min}
        max={max}
        value={value}
        onChange={(e) => set(Number(e.target.value))}
        onBlur={() => touch(key)}
      />
    </Field>
  );
  return (
    <Modal title={t(rule ? "ui.editRule" : "ui.newRule")} close={close}>
      <div className="form-grid">
        <Field
          label={t("ui.ruleName")}
          help={t("help.ruleName")}
          error={err("name")}
        >
          <input
            value={name}
            onChange={(e) => {
              setName(e.target.value);
              setNameEdited(true);
            }}
            onBlur={() => touch("name")}
          />
        </Field>
        {!profileId && (
          <Field label={t("ui.profileName")} help={t("help.profileName")}>
            <input
              value={profileName}
              placeholder={t("wizard.default_profile")}
              onChange={(e) => setProfileName(e.target.value)}
            />
          </Field>
        )}
        <Field label={t("ui.trigger")} help={t("help.trigger")}>
          <select
            value={event}
            onChange={(e) => {
              setEvent(e.target.value);
              if (!nameEdited || !name.trim())
                setName(t(`wizard.rule.${e.target.value}`));
            }}
          >
            <option value="connect">{t("ui.onConnect")}</option>
            <option value="disconnect">{t("ui.onDisconnect")}</option>
          </select>
        </Field>
        <Field label={t("ui.combination")} help={t("help.combination")}>
          <select value={mode} onChange={(e) => setMode(e.target.value)}>
            {["all", "any", "at-least"].map((k) => (
              <option key={k} value={k}>
                {t(`ui.combination.${k}`)}
              </option>
            ))}
          </select>
        </Field>
        {mode === "at-least" &&
          numeric(
            "minimum",
            t("ui.minimum"),
            minimum,
            setMinimum,
            1,
            ids.length,
            "help.minimum",
          )}
      </div>
      <fieldset className={err("ids") ? "invalid" : ""}>
        <legend>
          {t("ui.selectDevices")}{" "}
          <Help label={t("ui.selectDevices")} text={t("help.devices")} />
        </legend>
        <div className="device-checklist">
          {allDevices.map((d) => (
            <label key={d.id}>
              <input
                type="checkbox"
                checked={ids.includes(d.id)}
                onChange={() => {
                  setIds((old) =>
                    old.includes(d.id)
                      ? old.filter((x) => x !== d.id)
                      : [...old, d.id],
                  );
                  touch("ids");
                }}
              />
              <DeviceIcon device={d} />
              {d.name}
            </label>
          ))}
        </div>
        {err("ids") && <span className="field-error">{err("ids")}</span>}
      </fieldset>
      <Field
        label={t("ui.targetMonitor")}
        help={t("help.targetMonitor")}
        error={err("monitor")}
      >
        <select
          value={monitor}
          onChange={(e) => {
            setMonitor(e.target.value);
            touch("monitor");
          }}
        >
          <option value="">{t("ui.chooseMonitor")}</option>
          {available.map((m) => (
            <option key={m.id} value={m.id}>
              {m.alias ?? m.name}
            </option>
          ))}
        </select>
      </Field>
      <div className="field">
        <div className="field-title">
          <strong>{t("ui.targetInput")}</strong>
          <Help label={t("ui.targetInput")} text={t("help.targetInput")} />
        </div>
        <InputTiles
          monitor={chosen}
          state={state}
          value={input}
          onChange={setInput}
        />
        {err("input") && <span className="field-error">{err("input")}</span>}
      </div>
      <div className="form-grid">
        {numeric(
          "debounce",
          t("ui.debounce"),
          debounce,
          setDebounce,
          0,
          60000,
          "help.debounce",
        )}
        {numeric(
          "cooldown",
          t("ui.cooldown"),
          cooldown,
          setCooldown,
          0,
          120000,
          "help.cooldown",
        )}
      </div>
      <div className="preview">
        <Workflow size={18} />
        <RuleSummary
          rule={{
            mode,
            minimumCount: minimum,
            deviceIds: ids,
            trigger: event,
            debounceMs: debounce,
            actions: [
              {
                id: "preview",
                monitorId: monitor,
                inputValue: input,
                vcpCode: 96,
              },
            ],
          }}
          monitor={chosen}
        />
      </div>
      <details className="advanced-settings" open={dry || undefined}>
        <summary>{t("ui.advanced")}</summary>
        <Setting label={t("ui.dryRun")} detail={t("ui.dryRunDetail")}>
          <Switch
            label={t("ui.dryRun")}
            checked={dry}
            onChange={() => setDry(!dry)}
          />
        </Setting>
        <div className="form-grid">
          {numeric(
            "rearm",
            t("ui.rearm"),
            rearm,
            setRearm,
            0,
            60000,
            "help.rearm",
          )}
          {numeric(
            "window",
            t("ui.coincidence"),
            window,
            setWindow,
            0,
            120000,
            "help.coincidence",
          )}
        </div>
        <h3>
          <Terminal size={15} />
          {t("ui.customCommands")}
        </h3>
        <Setting
          label={t("ui.commandsEnabled")}
          detail={t("ui.commandEnableDetail")}
        >
          <Switch
            label={t("ui.commandsEnabled")}
            checked={commands.enabled}
            onChange={() => {
              setCommands((c) => ({ ...c, enabled: !c.enabled }));
              touch("commands");
            }}
          />
        </Setting>
        {err("commands") && (
          <div className="field-error">{err("commands")}</div>
        )}
        {(["onConnect", "onDisconnect"] as const).map((key) => (
          <Field
            key={key}
            label={t(`ui.${key}`)}
            help={t("help.command")}
            error={err(key)}
          >
            <textarea
              rows={3}
              spellCheck={false}
              value={commands[key]}
              maxLength={16384}
              onBlur={() => touch(key)}
              onChange={(e) =>
                setCommands((c) => ({ ...c, [key]: e.target.value }))
              }
            />
          </Field>
        ))}
        <div className="form-grid">
          <Field label={t("ui.commandShell")} help={t("help.commandShell")}>
            <select
              value={commands.shell}
              onChange={(e) =>
                setCommands((c) => ({ ...c, shell: e.target.value }))
              }
            >
              {["system", "bash", "powershell", "cmd"].map((k) => (
                <option key={k} value={k}>
                  {t(`ui.shell.${k}`)}
                </option>
              ))}
            </select>
          </Field>
          {numeric(
            "timeout",
            t("ui.commandTimeout"),
            commands.timeoutMs,
            (n) => setCommands((c) => ({ ...c, timeoutMs: n })),
            100,
            300000,
            "help.commandTimeout",
          )}
        </div>
        <Field
          label={t("ui.commandDirectory")}
          help={t("help.commandDirectory")}
          error={err("directory")}
        >
          <input
            value={commands.workingDirectory}
            onBlur={() => touch("directory")}
            placeholder={t("ui.commandHome")}
            onChange={(e) =>
              setCommands((c) => ({ ...c, workingDirectory: e.target.value }))
            }
          />
        </Field>
        {rule && confirm && (
          <div className="command-tests">
            {(["connect", "disconnect"] as const).map((event) => (
              <button
                key={event}
                className="button"
                title={t("ui.commandTestTip")}
                disabled={
                  JSON.stringify(commands) !==
                    JSON.stringify(rule.commands ?? defaultCommands()) ||
                  !(
                    event === "connect"
                      ? commands.onConnect
                      : commands.onDisconnect
                  ).trim()
                }
                onClick={() =>
                  confirm({
                    text: t("ui.commandConfirm", { event }),
                    run: () =>
                      act("rules.commands.run", {
                        id: rule.id,
                        event,
                        confirm: true,
                      }),
                  })
                }
              >
                <Play size={13} />
                {t(`ui.test.${event}`)}
              </button>
            ))}
          </div>
        )}
      </details>
      <div className="modal-actions">
        <div className="form-error" role={error ? "alert" : undefined}>
          {error}
        </div>
        <button className="button" onClick={close}>
          <X size={14} />
          {t("ui.cancel")}
        </button>
        <button
          className="button primary"
          disabled={saving || !valid}
          onClick={() => void save()}
        >
          <Check size={15} />
          {t("ui.saveRule")}
        </button>
      </div>
    </Modal>
  );
}

type Observation = {
  device: Device;
  disconnectedAtMs: number | null;
  connectedAtMs: number | null;
  lastEvent: string;
};
type Capture = {
  session: { id: string; active: boolean };
  observations: Observation[];
};
type Direction = "disconnect" | "connect";
/** Devices that change within this window of the first one are selected by
 * default; slower ones usually re-enumerate late and only delay switching. */
export const TRIGGER_WINDOW_MS = 100;
const at = (o: Observation, d: Direction) =>
  d === "disconnect" ? o.disconnectedAtMs : o.connectedAtMs;
function firstEvent(list: Observation[], d: Direction) {
  let first: Observation | undefined;
  for (const o of list) {
    const v = at(o, d);
    if (v !== null && (first === undefined || v < at(first, d)!)) first = o;
  }
  return first;
}
export function defaultTriggerIds(list: Observation[], d: Direction) {
  const first = firstEvent(list, d);
  if (!first) return [];
  const base = at(first, d)!;
  return list
    .filter((o) => {
      const v = at(o, d);
      return v !== null && v - base <= TRIGGER_WINDOW_MS;
    })
    .map((o) => o.device.id);
}
function sortObservations(
  list: Observation[],
  key: Direction,
  descending: boolean,
) {
  return [...list].sort((a, b) => {
    const x = at(a, key),
      y = at(b, key);
    if (x === null && y === null)
      return a.device.name.localeCompare(b.device.name);
    if (x === null) return 1;
    if (y === null) return -1;
    return descending ? y - x : x - y;
  });
}
function SortHeader({
  direction,
  sort,
  descending,
  onSort,
}: {
  direction: Direction;
  sort: Direction;
  descending: boolean;
  onSort: (d: Direction) => void;
}) {
  const active = sort === direction;
  const Arrow = !active ? ArrowUpDown : descending ? ArrowUp : ArrowDown;
  return (
    <th aria-sort={active ? (descending ? "descending" : "ascending") : "none"}>
      <button
        type="button"
        className={`sort-header ${active ? "active" : ""}`}
        title={t("ui.sortBy", { column: t(`ui.${direction}ed`) })}
        onClick={() => onSort(direction)}
      >
        {direction === "disconnect" ? <Unplug size={14} /> : <Plug size={14} />}
        {t(`ui.${direction}ed`)}
        <Arrow size={12} className="sort-arrow" />
      </button>
    </th>
  );
}
function EventTime({
  observation,
  direction,
  list,
}: {
  observation: Observation;
  direction: Direction;
  list: Observation[];
}) {
  const ms = at(observation, direction);
  if (ms === null) return null;
  const first = firstEvent(list, direction)!;
  const offset = ms - at(first, direction)!;
  const tip = t(`ui.eventTimeTip.${direction}`, {
    absolute: ms,
    offset,
    first: first.device.name,
  });
  return (
    <span className="event-time" title={tip}>
      <strong className="green">{t("ui.offsetMs", { value: offset })}</strong>
      {offset === 0 && <small>{t("ui.atMs", { value: ms })}</small>}
    </span>
  );
}
function useSort(initial: Direction) {
  const [sort, setSort] = useState<Direction>(initial),
    [descending, setDescending] = useState(false);
  const onSort = (d: Direction) => {
    if (d === sort) setDescending((v) => !v);
    else {
      setSort(d);
      setDescending(false);
    }
  };
  const force = (d: Direction) => {
    setSort(d);
    setDescending(false);
  };
  return { sort, descending, onSort, force };
}
export function Wizard({
  data,
  act,
  close,
}: {
  data: { devices: Device[]; monitors: Monitor[]; config: Config | null };
  act: (c: string, p?: Record<string, unknown>) => Promise<unknown>;
  close: () => void;
}) {
  const [step, setStep] = useState(0),
    [manual, setManual] = useState(false),
    [capture, setCapture] = useState<Capture | null>(null),
    [triggers, setTriggers] = useState<Record<Direction, string[]>>({
      disconnect: [],
      connect: [],
    }),
    [error, setError] = useState(""),
    [saving, setSaving] = useState(false),
    [connectMonitor, setConnectMonitor] = useState(data.monitors[0]?.id ?? ""),
    [disconnectMonitor, setDisconnectMonitor] = useState(
      data.monitors[0]?.id ?? "",
    ),
    [connectInput, setConnectInput] = useState(17),
    [disconnectInput, setDisconnectInput] = useState(18),
    [chosen, setChosen] = useState(false),
    [profile, setProfile] = useState<Profile | null>(null);
  const liveSort = useSort("disconnect"),
    reviewSort = useSort("disconnect");
  const saved = (id: string) =>
    data.config?.monitors.find((m) => m.id === id) ??
    data.monitors.find((m) => m.id === id);
  const cs = useMonitorState(connectMonitor, step === 3),
    ds = useMonitorState(
      disconnectMonitor,
      step === 3 && connectMonitor !== disconnectMonitor,
    );
  useEffect(() => {
    if (
      !chosen &&
      cs?.currentInput &&
      monitorInputs(saved(connectMonitor)).some(
        (i) => i.value === cs.currentInput,
      )
    ) {
      setConnectInput(cs.currentInput);
      setDisconnectInput(cs.currentInput === 18 ? 17 : 18);
    }
  }, [cs?.currentInput, chosen]);
  // Every opening starts from scratch: a capture left running by an earlier
  // window is cancelled instead of resumed, and closing mid-capture stops it.
  // While the wizard is open the agent must not switch monitors: the user is
  // pressing the KVM switch on purpose. The hold is a short lease renewed here,
  // so it lapses on its own if this window disappears.
  useEffect(() => {
    let alive = true;
    const hold = () => request("wizard.hold", { ms: 15000 }).catch(() => {});
    void request("wizard.cancel")
      .catch(() => {})
      .then(() => {
        if (alive) void hold();
      });
    const timer = setInterval(() => void hold(), 5000);
    return () => {
      alive = false;
      clearInterval(timer);
      void request("wizard.cancel")
        .catch(() => {})
        .then(() => request("wizard.hold", { release: true }))
        .catch(() => {});
    };
  }, []);
  useEffect(() => {
    if (step !== 1) return;
    let stopped = false,
      timer: ReturnType<typeof setTimeout>;
    async function update() {
      try {
        const c = await request<Capture>("wizard.candidates");
        if (!stopped) setCapture(c);
      } catch (e) {
        if (!stopped) setError(errorMessage(e));
      } finally {
        if (!stopped) timer = setTimeout(() => void update(), 50);
      }
    }
    void update();
    return () => {
      stopped = true;
      clearTimeout(timer);
    };
  }, [step]);
  const observations = capture?.observations ?? [];
  const phase: Direction | "waiting" = observations.some(
    (o) => o.connectedAtMs !== null,
  )
    ? "connect"
    : observations.length
      ? "disconnect"
      : "waiting";
  useEffect(() => {
    if (phase !== "waiting") liveSort.force(phase);
  }, [phase]);
  async function start() {
    try {
      setError("");
      await act("wizard.start", { cycles: 1 });
      setCapture(null);
      setManual(false);
      liveSort.force("disconnect");
      setStep(1);
    } catch (e) {
      setError(errorMessage(e));
    }
  }
  function chooseManually() {
    void request("wizard.cancel").catch(() => {});
    setManual(true);
    setCapture(null);
    setTriggers({ disconnect: [], connect: [] });
    setError("");
    setStep(2);
  }
  async function cancelCapture() {
    await request("wizard.cancel").catch(() => {});
    close();
  }
  async function finish() {
    try {
      const c = (await act("wizard.capture.finish")) as Capture;
      setCapture(c);
      setTriggers({
        disconnect: defaultTriggerIds(c.observations, "disconnect"),
        connect: defaultTriggerIds(c.observations, "connect"),
      });
      reviewSort.force("disconnect");
      setStep(2);
    } catch (e) {
      setError(errorMessage(e));
    }
  }
  async function complete() {
    setSaving(true);
    try {
      setError("");
      const p = (await act("wizard.complete", {
        ...(manual ? {} : { session: capture?.session.id }),
        "disconnect-devices": triggers.disconnect,
        "connect-devices": triggers.connect,
        "connect-monitor": connectMonitor,
        "connect-input": connectInput,
        "disconnect-monitor": disconnectMonitor,
        "disconnect-input": disconnectInput,
      })) as Profile;
      setProfile(p);
      setStep(4);
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setSaving(false);
    }
  }
  const toggle = (d: Direction, id: string) =>
    setTriggers((v) => ({
      ...v,
      [d]: v[d].includes(id) ? v[d].filter((x) => x !== id) : [...v[d], id],
    }));
  const reviewList: Observation[] = manual
    ? data.devices.map((device) => ({
        device,
        disconnectedAtMs: null,
        connectedAtMs: null,
        lastEvent: "",
      }))
    : observations;
  const allSelected = new Set([...triggers.disconnect, ...triggers.connect]);
  const lateCount = (d: Direction) =>
    observations.filter((o) => at(o, d) !== null).length -
    defaultTriggerIds(observations, d).length;
  return (
    <Modal title={t("ui.setupTitle")} close={step === 1 ? () => void cancelCapture() : close}>
      <div className="wizard-steps">
        {["capture", "devices", "screens", "done"].map((s, i) => (
          <div
            key={s}
            className={
              step === i + 1
                ? "current"
                : step > i + 1 && !(manual && i === 0)
                  ? "complete"
                  : ""
            }
          >
            <span>
              {step > i + 1 && !(manual && i === 0) ? <Check size={12} /> : i + 1}
            </span>
            {t(`ui.wizardStep.${s}`)}
          </div>
        ))}
      </div>
      {step < 4 && (
        <div className="wizard-hold-note" role="status">
          <Pause size={13} />
          {t("ui.setupHoldNote")}
        </div>
      )}
      {step === 0 && (
        <>
          <div className="wizard-hero">
            <Radio size={34} />
            <h3>{t("ui.captureReady")}</h3>
            <p>{t("ui.captureIntro")}</p>
          </div>
          <div className="modal-actions">
            <button
              className="button"
              title={t("ui.manualSetupTip")}
              onClick={chooseManually}
            >
              <ListChecks size={14} />
              {t("ui.manualSetup")}
            </button>
            <button className="button primary" onClick={() => void start()}>
              <Radio size={14} />
              {t("ui.wizard.startCapture")}
              <ArrowRight size={14} />
            </button>
          </div>
        </>
      )}
      {step === 1 && (
        <>
          <div className={`capture-prompt phase-${phase}`}>
            <div className="kvm-press" aria-hidden="true">
              <ArrowLeftRight size={22} />
            </div>
            <div>
              <strong>{t(`ui.capturePrompt.${phase}.title`)}</strong>
              <p>{t(`ui.capturePrompt.${phase}.detail`)}</p>
            </div>
          </div>
          {observations.length > 0 && (
            <table className="capture-table">
              <thead>
                <tr>
                  <th>{t("ui.usbDevice")}</th>
                  <SortHeader direction="disconnect" {...liveSort} />
                  <SortHeader direction="connect" {...liveSort} />
                </tr>
              </thead>
              <tbody>
                {sortObservations(
                  observations,
                  liveSort.sort,
                  liveSort.descending,
                ).map((o) => (
                  <tr key={o.device.id}>
                    <td>
                      <DeviceIcon device={o.device} />
                      <strong>{o.device.name}</strong>
                    </td>
                    {(["disconnect", "connect"] as const).map((d) => (
                      <td key={d}>
                        {at(o, d) === null ? (
                          <LoaderCircle
                            size={16}
                            className={`spin ${phase === d || (phase === "disconnect" && d === "connect") ? "" : "muted"}`}
                            aria-label={t("ui.waitingForEvent")}
                          />
                        ) : (
                          <EventTime observation={o} direction={d} list={observations} />
                        )}
                      </td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          )}
          {!observations.length && (
            <div className="empty-capture">
              <LoaderCircle className="spin" size={20} />
              {t("ui.captureEmpty")}
            </div>
          )}
          <div className="modal-actions">
            <button className="button" onClick={() => void cancelCapture()}>
              <X size={14} />
              {t("ui.cancel")}
            </button>
            <button
              className="button primary"
              disabled={!observations.length}
              onClick={() => void finish()}
            >
              {t("ui.wizard.finish")}
              <ArrowRight size={14} />
            </button>
          </div>
        </>
      )}
      {step === 2 && (
        <>
          <h3>{t(manual ? "ui.triggerTitleManual" : "ui.triggerTitle")}</h3>
          <p className="muted">
            {t(manual ? "ui.triggerHelpManual" : "ui.triggerHelp")}
          </p>
          <div className="trigger-table-scroll">
            <table className="capture-table trigger-table">
              <thead>
                <tr>
                  <th>{t("ui.usbDevice")}</th>
                  {manual ? (
                    <>
                      <th>
                        <Unplug size={14} />
                        {t("ui.disconnectTrigger")}
                      </th>
                      <th>
                        <Plug size={14} />
                        {t("ui.connectTrigger")}
                      </th>
                    </>
                  ) : (
                    <>
                      <SortHeader direction="disconnect" {...reviewSort} />
                      <SortHeader direction="connect" {...reviewSort} />
                    </>
                  )}
                </tr>
              </thead>
              <tbody>
                {(manual
                  ? reviewList
                  : sortObservations(
                      reviewList,
                      reviewSort.sort,
                      reviewSort.descending,
                    )
                ).map((o) => (
                  <tr
                    key={o.device.id}
                    className={allSelected.has(o.device.id) ? "selected-row" : ""}
                  >
                    <td>
                      <DeviceIcon device={o.device} />
                      <strong>{o.device.name}</strong>
                    </td>
                    {(["disconnect", "connect"] as const).map((d) => {
                      const seen = manual || at(o, d) !== null;
                      return (
                        <td key={d}>
                          <label
                            className={`trigger-cell ${seen ? "" : "unseen"}`}
                            title={
                              seen
                                ? t(`ui.useForTrigger.${d}`, { name: o.device.name })
                                : t(`ui.notObserved.${d}`)
                            }
                          >
                            <input
                              type="checkbox"
                              aria-label={t(`ui.useForTrigger.${d}`, {
                                name: o.device.name,
                              })}
                              checked={triggers[d].includes(o.device.id)}
                              onChange={() => toggle(d, o.device.id)}
                            />
                            {!manual && at(o, d) !== null && (
                              <EventTime observation={o} direction={d} list={reviewList} />
                            )}
                            {!seen && <small>{t("ui.notObservedShort")}</small>}
                          </label>
                        </td>
                      );
                    })}
                  </tr>
                ))}
              </tbody>
              <tfoot>
                <tr>
                  <td>{t("ui.selectedTotal")}</td>
                  <td>{t("ui.selectedCount", { count: triggers.disconnect.length })}</td>
                  <td>{t("ui.selectedCount", { count: triggers.connect.length })}</td>
                </tr>
              </tfoot>
            </table>
          </div>
          {!manual && (
            <div className="trigger-hint">
              <Info size={14} />
              <span>
                {t("ui.triggerWindowHint", { window: TRIGGER_WINDOW_MS })}
                {(lateCount("disconnect") > 0 || lateCount("connect") > 0) &&
                  " " +
                    t("ui.triggerWindowLate", {
                      disconnect: lateCount("disconnect"),
                      connect: lateCount("connect"),
                    })}
              </span>
            </div>
          )}
          {(!triggers.disconnect.length || !triggers.connect.length) && (
            <div className="field-error">
              {t("ui.triggerBothRequired")}
            </div>
          )}
          <div className="modal-actions">
            {manual ? (
              <button className="button" onClick={() => setStep(0)}>
                <ArrowLeft size={14} />
                {t("ui.back")}
              </button>
            ) : (
              <button className="button" onClick={() => void start()}>
                <RotateCcw size={14} />
                {t("ui.captureAgain")}
              </button>
            )}
            <button
              className="button primary"
              disabled={!triggers.disconnect.length || !triggers.connect.length}
              onClick={() => setStep(3)}
            >
              {t("ui.chooseScreens")}
              <ArrowRight size={14} />
            </button>
          </div>
        </>
      )}
      {step === 3 && (
        <>
          <p className="muted">{t("ui.wizardReadyHelp")}</p>
          {(["connect", "disconnect"] as const).map((event) => {
            const id = event === "connect" ? connectMonitor : disconnectMonitor;
            const val = event === "connect" ? connectInput : disconnectInput;
            return (
              <section className="wizard-target" key={event}>
                <h3>
                  {event === "connect" ? (
                    <Plug size={16} />
                  ) : (
                    <Unplug size={16} />
                  )}{" "}
                  {t(
                    `ui.${event === "connect" ? "onConnect" : "onDisconnect"}`,
                  )}
                </h3>
                <Field
                  label={t(
                    event === "connect"
                      ? "ui.connectMonitor"
                      : "ui.disconnectMonitor",
                  )}
                  help={t("help.targetMonitor")}
                >
                  <select
                    value={id}
                    onChange={(e) =>
                      (event === "connect"
                        ? setConnectMonitor
                        : setDisconnectMonitor)(e.target.value)
                    }
                  >
                    <option value="">{t("ui.chooseMonitor")}</option>
                    {data.monitors.map((m) => (
                      <option key={m.id} value={m.id}>
                        {m.alias ?? m.name}
                      </option>
                    ))}
                  </select>
                </Field>
                <InputTiles
                  monitor={saved(id)}
                  value={val}
                  state={
                    event === "connect" || connectMonitor === disconnectMonitor
                      ? cs
                      : ds
                  }
                  onChange={(n) => {
                    setChosen(true);
                    (event === "connect"
                      ? setConnectInput
                      : setDisconnectInput)(n);
                  }}
                />
              </section>
            );
          })}
          <div className="modal-actions">
            <button className="button" onClick={() => setStep(2)}>
              <ArrowLeft size={14} />
              {t("ui.back")}
            </button>
            <button
              className="button primary"
              disabled={saving || !connectMonitor || !disconnectMonitor}
              onClick={() => void complete()}
            >
              {t("ui.finishSetup")}
              <Check size={15} />
            </button>
          </div>
        </>
      )}
      {step === 4 && (
        <div className="wizard-complete">
          <Check size={36} />
          <h3>{t("ui.setupComplete")}</h3>
          <p>
            {t("ui.setupCompleteDetail", {
              profile: profile?.name ?? "",
              count: allSelected.size,
            })}
          </p>
          <ul>
            <li>{t("wizard.rule.connect")}</li>
            <li>{t("wizard.rule.disconnect")}</li>
          </ul>
          <button className="button primary" onClick={close}>
            <Check size={14} />
            {t("ui.done")}
          </button>
        </div>
      )}
      {error && (
        <div className="form-error persistent-error" role="alert">
          {error}
        </div>
      )}
    </Modal>
  );
}
