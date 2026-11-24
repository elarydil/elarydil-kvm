import React, { useState, useEffect, useCallback } from "react";
import { createRoot } from "react-dom/client";
import {
  LayoutDashboard,
  Usb,
  Monitor as MonitorIcon,
  Workflow,
  Activity,
  Settings,
  ArrowRight,
  Plus,
  RefreshCw,
  Search,
  Check,
  ChevronRight,
  Play,
  Pause,
  ShieldCheck,
  Radio,
  AlertCircle,
  Trash2,
  Copy,
  Sun,
  Moon,
  Terminal,
  Download,
  X,
  Cable,
  CheckCircle2,
  Info,
  WandSparkles,
  Power,
  Pencil,
  FolderOpen,
  Square,
  Save,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { request, t, hex, monitorInputs, inputName } from "./api";
import type {
  Config,
  Device,
  Monitor,
  Profile,
  Rule,
  Input,
  CommandHooks,
} from "./api";
import "@fontsource-variable/geist-mono";
import "./style.css";
import appIcon from "../src-tauri/icons/32x32.png";
import {
  Modal,
  Switch,
  Setting,
  NumberSetting,
  Help,
  useButtonTooltips,
} from "./ui";
import { MonitorCard, DeviceIcon } from "./inputs";
import { RuleEditor, Wizard, RuleSummary } from "./workflow-ui";
import { listen } from "@tauri-apps/api/event";
const nav = [
  ["dashboard", LayoutDashboard],
  ["devices", Usb],
  ["monitors", MonitorIcon],
  ["profiles", Workflow],
  ["events", Activity],
  ["settings", Settings],
] as const;
type Data = {
  config: Config | null;
  devices: Device[];
  monitors: Monitor[];
  status: {
    agent: string;
    usbAvailable: boolean;
    uptimeMs: number;
    standalone?: boolean;
  } | null;
  events: Event[];
};
type Event = {
  timestamp: string;
  eventType: string;
  monotonicSequence: number;
  severity: string;
  result: unknown;
};
function deviceDescription(device: Device) {
  const model = [device.manufacturer, device.model].filter(Boolean).join(" ");
  return [model, t(`usb.type.${device.category}`)].filter(Boolean).join(" · ");
}
function App() {
  const [page, setPage] = useState("dashboard"),
    [data, setData] = useState<Data>({
      config: null,
      devices: [],
      monitors: [],
      status: null,
      events: [],
    }),
    [error, setError] = useState(""),
    [actionError, setActionError] = useState(""),
    [pageRestored, setPageRestored] = useState(false),
    [notice, setNotice] = useState(""),
    [noticePath, setNoticePath] = useState(""),
    [busy, setBusy] = useState(false),
    [search, setSearch] = useState(""),
    [selected, setSelected] = useState<string[]>([]),
    [wizard, setWizard] = useState(false),
    [editing, setEditing] = useState<Rule | null | undefined>(undefined),
    [profile, setProfile] = useState(""),
    [details, setDetails] = useState<Device | null>(null),
    [confirm, setConfirm] = useState<{
      text: string;
      run: () => Promise<unknown>;
    } | null>(null);
  const message = (e: unknown) =>
    typeof e === "object" && e && "message" in e
      ? String(e.message) +
        ("detail" in e && e.detail ? ` (${String(e.detail).slice(0, 500)})` : "")
      : String(e);
  const load = useCallback(async () => {
    try {
      const [config, status, usb, monitors, events] = await Promise.all([
        request<Config>("config.show"),
        request<Data["status"]>("status"),
        request<{ live: Device[] }>("devices.list"),
        request<{ live: Monitor[] }>("monitors.list"),
        request<Event[]>("events.list", { limit: 100 }),
      ]);
      setData({
        config,
        status,
        devices: usb.live,
        monitors: monitors.live,
        events,
      });
      setError("");
    } catch (e) {
      setError(message(e));
    }
  }, []);
  useEffect(() => {
    void load();
    const timer = setInterval(() => void load(), 2500);
    return () => clearInterval(timer);
  }, [load]);
  useEffect(() => {
    const theme = data.config?.app.theme ?? "system";
    document.documentElement.dataset.theme = theme;
  }, [data.config?.app.theme]);
  useButtonTooltips();
  useEffect(() => {
    if (!notice) return;
    const timer = setTimeout(
      () => {
        setNotice("");
        setNoticePath("");
      },
      noticePath ? 9000 : 3200,
    );
    return () => clearTimeout(timer);
  }, [notice, noticePath]);
  useEffect(() => {
    if (!actionError) return;
    const timer = setTimeout(() => setActionError(""), 8000);
    return () => clearTimeout(timer);
  }, [actionError]);
  useEffect(() => {
    const unlisten = listen<string>("navigate", (event) => {
      if (nav.some(([id]) => id === event.payload)) {
        setPageRestored(true);
        setPage(event.payload);
        setSearch("");
      }
    }).catch(() => () => {});
    const errors = listen<string>("native-menu-error", (event) =>
      setActionError(event.payload),
    ).catch(() => () => {});
    return () => {
      void unlisten.then((f) => f());
      void errors.then((f) => f());
    };
  }, []);
  useEffect(() => {
    const off = listen("agent-status-changed", () => void load()).catch(
      () => () => {},
    );
    return () => {
      void off.then((f) => f());
    };
  }, [load]);
  useEffect(() => {
    if (data.config && !pageRestored) {
      setPageRestored(true);
      if (nav.some(([id]) => id === data.config!.app.lastPage))
        setPage(data.config.app.lastPage);
    }
  }, [data.config, pageRestored]);
  useEffect(() => {
    if (pageRestored && data.config?.app.lastPage !== page)
      void request("config.set", { id: "app.lastPage", value: page }).catch(
        () => {},
      );
  }, [page, pageRestored]);
  async function act(command: string, payload: Record<string, unknown> = {}) {
    setBusy(true);
    try {
      const result = await request(command, payload);
      if(typeof result==="object"&&result&&"message"in result)setNotice(String(result.message));
      else if(!["rules.simulate","rules.commands.run"].includes(command)&&["config.","ui.","profiles.","rules.","devices.","monitors.input.alias."].some(prefix=>command.startsWith(prefix)))setNotice(t("ui.saved"));
      await load();
      return result;
    } catch (e) {
      setActionError(message(e));
      throw e;
    } finally {
      setBusy(false);
    }
  }
  const run = (command: string, payload: Record<string, unknown> = {}) => {
    void act(command, payload).catch(() => {});
  };
  const navigate = (p: string) => {
    setPageRestored(true);
    setPage(p);
    setSearch("");
  };
  const config = data.config,
    profiles = config?.profiles ?? [],
    tracked = config?.usbDevices ?? [],
    saved = config?.monitors ?? [],
    running = data.status?.agent === "running";
  const toggle = (id: string) =>
    setSelected((ids) =>
      ids.includes(id) ? ids.filter((i) => i !== id) : [...ids, id],
    );
  const startRule = (p: string) => {
    setProfile(p);
    setEditing(null);
  };
  /** Exports JSON through the native save panel and confirms where it went. */
  async function download(name: string, value: unknown) {
    try {
      const path = await invoke<string | null>("save_file", {
        name,
        contents: JSON.stringify(value, null, 2),
      });
      if (typeof path !== "string") return;
      setNoticePath(path);
      setNotice(t("ui.exportSaved", { path }));
    } catch (e) {
      setActionError(t("ui.exportFailed", { error: message(e) }));
    }
  }
  /** Performs the rule's monitor switch for real, as the reversible 15-second
   * input test: the monitor returns to its current input automatically. */
  function testSwitch(rule: Rule) {
    const a = rule.actions[0];
    if (!a) return;
    const monitor =
      saved.find((m) => m.id === a.monitorId) ??
      data.monitors.find((m) => m.id === a.monitorId);
    const input = monitorInputs(monitor).find((i) => i.value === a.inputValue);
    const vars = {
      rule: rule.name,
      monitor: monitor?.alias ?? monitor?.name ?? a.monitorId,
      input: input
        ? inputName(input)
        : t("ui.inputNumber", { value: a.inputValue }),
    };
    setConfirm({
      text: t("ui.testSwitchConfirm", vars),
      run: () =>
        act("monitors.input.test", {
          id: a.monitorId,
          value: a.inputValue,
          confirm: true,
        }).then(() => setNotice(t("ui.testSwitchStarted", vars))),
    });
  }
  return (
    <div className="shell">
      <aside className="sidebar">
        <div className="brand">
          <span className="brandmark">
            <img draggable={false} src={appIcon} width={24} height={24} alt="" />
          </span>
          <strong>{t("ui.brand")}</strong>
        </div>
        <div className="sidebar-section">{t("ui.controls")}</div>
        <nav aria-label={t("ui.navigation")}>
          {nav.map(([id, Icon]) => (
            <button
              key={id}
              aria-label={t(`ui.nav.${id}`)}
              aria-current={page === id ? "page" : undefined}
              onClick={() => navigate(id)}
              className={page === id ? "nav active" : "nav"}
            >
              <Icon size={16} />
              <span>{t(`ui.nav.${id}`)}</span>
              {id === "devices" && <b>{data.devices.length}</b>}
            </button>
          ))}
        </nav>
        <div className="sidebar-bottom">
          <button
            className="sidebar-action"
            title={t("ui.setupOpenTip")}
            onClick={() => setWizard(true)}
          >
            <WandSparkles size={15} />
            {t("ui.setupOpen")}
          </button>
          <div className="agent-indicator">
            <span
              className={`dot ${running ? "green" : data.status?.agent === "paused" ? "amber" : ""}`}
            />
            <span>
              {t(
                running
                  ? "ui.agentRunning"
                  : data.status?.agent === "paused"
                    ? "ui.agentPaused"
                    : "ui.agentOffline",
              )}
            </span>
          </div>
          <div className="sidebar-version">{t("ui.version")}</div>
        </div>
      </aside>
      <main>
        <header className="topbar">
          <h1>{t(`ui.title.${page}`)}</h1>
          <div className="heading-actions">
            <button
              className="icon-button"
              aria-label={t("ui.refresh")}
              title={t("ui.refresh")}
              onClick={() => void load()}
            >
              <RefreshCw size={15} className={busy ? "spin" : ""} />
            </button>
            {page === "dashboard" ? (
              <button
                className="button"
                title={t("ui.setupOpenTip")}
                onClick={() => setWizard(true)}
              >
                <WandSparkles size={14} />
                {t("ui.setupOpen")}
              </button>
            ) : page === "profiles" ? (
              <button
                className="button primary"
                onClick={() => {
                  setProfile("");
                  setEditing(null);
                }}
              >
                <Plus size={14} />
                {t("ui.newRule")}
              </button>
            ) : page === "events" ? (
              <button
                className="button"
                title={t("ui.exportEventsTip")}
                onClick={() =>
                  void download("elarydil-kvm-events.json", data.events)
                }
              >
                <Download size={14} />
                {t("ui.export")}
              </button>
            ) : null}
          </div>
        </header>
        <div className="content" data-page={page}>
          {page !== "dashboard" && (
            <p className="page-description">{t(`ui.subtitle.${page}`)}</p>
          )}
          {error && (
            <div role="alert" className="banner error">
              <AlertCircle size={18} />
              <div>
                <strong>{t("ui.connectionIssue")}</strong>
                <span>{error}</span>
              </div>
              <button onClick={() => run("agent.start")}>
                <Play size={13} />
                {t("ui.startAgent")}
              </button>
            </div>
          )}
          {notice && (
            <div role="status" className="toast">
              <CheckCircle2 size={16} />
              <span className="toast-text">{notice}</span>
              {noticePath && (
                <button
                  className="toast-action"
                  onClick={() =>
                    void invoke("reveal_file", { path: noticePath }).catch(
                      (e) => setActionError(message(e)),
                    )
                  }
                >
                  <FolderOpen size={13} />
                  {t("ui.showInFolder")}
                </button>
              )}
              <button
                aria-label={t("ui.dismiss")}
                onClick={() => {
                  setNotice("");
                  setNoticePath("");
                }}
              >
                <X size={14} />
              </button>
            </div>
          )}
          {page === "dashboard" && (
            <>
              <section className="automation-status">
                <div
                  className={`automation-symbol ${running ? "is-running" : ""}`}
                >
                  <Cable size={24} />
                </div>
                <div className="automation-copy">
                  <h2>{t("ui.automation")}</h2>
                  <p>
                    <span
                      className={`dot ${running ? "green" : data.status?.agent === "paused" ? "amber" : ""}`}
                    />
                    {t(
                      running
                        ? "ui.running"
                        : data.status?.agent === "paused"
                          ? "ui.paused"
                          : "ui.unavailable",
                    )}
                    <span className="status-divider" />
                    {t(
                      data.status?.standalone
                        ? "ui.backgroundIndependent"
                        : "ui.runsWithApp",
                    )}
                  </p>
                </div>
                <button
                  className="button"
                  disabled={!data.status}
                  onClick={() =>
                    run(
                      running
                        ? "agent.pause"
                        : data.status?.agent === "paused"
                          ? "agent.resume"
                          : "agent.start",
                    )
                  }
                >
                  {running ? <Pause size={14} /> : <Play size={14} />}
                  {t(
                    running
                      ? "ui.pauseAutomation"
                      : data.status?.agent === "paused"
                        ? "ui.resumeAutomation"
                        : "ui.startAgent",
                  )}
                </button>
              </section>
              <div className="section-heading">
                <h2>{t("ui.connectedHardware")}</h2>
                <span>{t("ui.thisComputer")}</span>
              </div>
              <section className="panel resource-list">
                <button
                  className="resource-row"
                  onClick={() => navigate("devices")}
                >
                  <span className="resource-icon">
                    <Usb size={21} />
                  </span>
                  <span className="resource-copy">
                    <strong>{t("ui.nav.devices")}</strong>
                    <small>
                      {t("ui.hardwareDevices", {
                        connected: data.devices.length,
                        tracked: tracked.length,
                      })}
                    </small>
                  </span>
                  <span className="resource-count">{data.devices.length}</span>
                  <ChevronRight size={15} />
                </button>
                <button
                  className="resource-row"
                  onClick={() => navigate("monitors")}
                >
                  <span className="resource-icon">
                    <MonitorIcon size={21} />
                  </span>
                  <span className="resource-copy">
                    <strong>{t("ui.nav.monitors")}</strong>
                    <small>
                      {data.monitors
                        .map(
                          (m) =>
                            saved.find((x) => x.id === m.id)?.alias ?? m.name,
                        )
                        .join(", ") || t("ui.noMonitorsShort")}
                    </small>
                  </span>
                  <span className="resource-count">{data.monitors.length}</span>
                  <ChevronRight size={15} />
                </button>
              </section>
              <div className="two-columns">
                <section className="panel overview-profiles">
                  <PanelHead
                    title={t("ui.activeProfiles")}
                    count={profiles.length}
                    action={() => navigate("profiles")}
                    label={t("ui.manage")}
                  />
                  {profiles.length ? (
                    profiles.slice(0, 4).map((p) => (
                      <div className="profile-row" key={p.id}>
                        <div className="small-icon">
                          <Workflow size={17} />
                        </div>
                        <div>
                          <strong>{p.name}</strong>
                          <span>
                            {t("ui.rulesCount", { count: p.rules.length })}
                          </span>
                        </div>
                        <Switch
                          label={t("ui.enableProfile")}
                          checked={p.enabled}
                          onChange={() =>
                            run(
                              `profiles.${p.enabled ? "disable" : "enable"}`,
                              { id: p.id },
                            )
                          }
                        />
                      </div>
                    ))
                  ) : (
                    <Empty
                      icon={<Workflow size={24} />}
                      title={t("ui.noProfiles")}
                      detail={t("ui.noProfilesDetail")}
                      action={() => setWizard(true)}
                      label={t("ui.setupOpen")}
                    />
                  )}
                </section>
                <section className="panel overview-events">
                  <PanelHead
                    title={t("ui.recentActivity")}
                    action={() => navigate("events")}
                    label={t("ui.viewAll")}
                  />
                  {data.events.length ? (
                    data.events
                      .slice(0, 4)
                      .map((e) => (
                        <EventRow key={e.monotonicSequence} event={e} />
                      ))
                  ) : (
                    <Empty
                      icon={<Activity size={24} />}
                      title={t("ui.noEvents")}
                      detail={t("ui.noEventsDetail")}
                    />
                  )}
                </section>
              </div>
              <div className="overview-note">
                <Info size={14} />
                <span>{t("ui.backgroundNote")}</span>
              </div>
            </>
          )}
          {page === "devices" && (
            <section className="panel">
              <div className="toolbar">
                <div className="search">
                  <Search size={17} />
                  <input
                    aria-label={t("ui.searchDevices")}
                    placeholder={t("ui.searchDevices")}
                    value={search}
                    onChange={(e) => setSearch(e.target.value)}
                  />
                </div>
                <span className="muted">
                  {t("ui.selectedCount", { count: selected.length })}
                </span>
                <button
                  className="button"
                  disabled={!selected.length}
                  onClick={() => {
                    void Promise.all(
                      selected.map((id) => act("devices.track", { id })),
                    ).catch(() => {});
                  }}
                >
                  <Plus size={15} />
                  {t("ui.trackSelected")}
                </button>
              </div>
              <div className="table-scroll">
                <table>
                  <thead>
                    <tr>
                      <th>
                        <input
                          type="checkbox"
                          aria-label={t("ui.selectAll")}
                          checked={
                            data.devices.length > 0 &&
                            data.devices.every((d) => selected.includes(d.id))
                          }
                          onChange={(e) =>
                            setSelected(
                              e.target.checked
                                ? data.devices.map((d) => d.id)
                                : [],
                            )
                          }
                        />
                      </th>
                      <th>{t("ui.device")}</th>
                      <th>{t("ui.usbId")}</th>
                      <th>{t("ui.status")}</th>
                      <th>{t("ui.tracking")}</th>
                      <th />
                    </tr>
                  </thead>
                  <tbody>
                    {data.devices
                      .filter((d) =>
                        `${d.name} ${hex(d.vid)} ${hex(d.pid)}`
                          .toLowerCase()
                          .includes(search.toLowerCase()),
                      )
                      .map((d) => (
                        <tr
                          key={d.id}
                          className={
                            selected.includes(d.id) ? "selected-row" : undefined
                          }
                        >
                          <td>
                            <input
                              type="checkbox"
                              aria-label={d.name}
                              checked={selected.includes(d.id)}
                              onChange={() => toggle(d.id)}
                            />
                          </td>
                          <td>
                            <div className="table-device">
                              <div className="small-icon">
                                <DeviceIcon device={d} size={18} />
                              </div>
                              <div>
                                <strong>
                                  {tracked.find((x) => x.id === d.id)?.alias ??
                                    d.name}
                                </strong>
                                <span>{deviceDescription(d)}</span>
                              </div>
                            </div>
                          </td>
                          <td>
                            <code>
                              {hex(d.vid)}:{hex(d.pid)}
                            </code>
                          </td>
                          <td>
                            <span className="badge success">
                              <span className="dot green" />
                              {t("ui.connected")}
                            </span>
                          </td>
                          <td>
                            {t(
                              tracked.some((x) => x.id === d.id)
                                ? "ui.tracked"
                                : "ui.notTracked",
                            )}
                          </td>
                          <td>
                            <button
                              className="icon-button"
                              aria-label={t("ui.details")}
                              onClick={() => setDetails(d)}
                            >
                              <ChevronRight size={17} />
                            </button>
                          </td>
                        </tr>
                      ))}
                  </tbody>
                </table>
              </div>
              {!data.devices.length && (
                <Empty
                  icon={<Usb size={32} />}
                  title={t("ui.noDevices")}
                  detail={t("ui.noDevicesDetail")}
                  action={() => run("devices.rescan")}
                  label={t("ui.rescan")}
                />
              )}
              <div className="panel-foot">
                <ShieldCheck size={14} />
                {t("ui.presenceOnly")}
              </div>
            </section>
          )}
          {page === "monitors" && (
            <>
              <div className="banner">
                <Info size={18} />
                <span>{t("ui.inputWarning")}</span>
              </div>
              {data.monitors.length || saved.length ? (
                [
                  ...data.monitors,
                  ...saved.filter(
                    (m) => !data.monitors.some((l) => l.id === m.id),
                  ),
                ].map((m) => (
                  <MonitorCard
                    key={m.id}
                    available={data.monitors.some((l) => l.id === m.id)}
                    monitor={m}
                    saved={saved.find((s) => s.id === m.id)}
                    act={act}
                    confirm={setConfirm}
                  />
                ))
              ) : (
                <section className="panel">
                  <Empty
                    icon={<MonitorIcon size={35} />}
                    title={t("ui.noMonitors")}
                    detail={t("ui.noMonitorsDetail")}
                    action={() => void load()}
                    label={t("ui.rescan")}
                  />
                </section>
              )}
            </>
          )}
          {page === "profiles" && (
            <>
              {profiles.length ? (
                profiles.map((p) => (
                  <section
                    className={`panel profile-panel ${p.enabled ? "is-active" : ""}`}
                    key={p.id}
                  >
                    <div className="panel-head">
                      <div>
                        <h2>{p.name}</h2>
                        <span className="muted">
                          {t("ui.rulesCount", { count: p.rules.length })}
                        </span>
                      </div>
                      <div className="inline-actions">
                        {p.enabled ? (
                          <span
                            className="profile-status"
                            role="status"
                            title={t("ui.activeProfileTip")}
                          >
                            <span className="dot green" />
                            {t("ui.activeProfile")}
                          </span>
                        ) : (
                          <button
                            className="button"
                            title={t("ui.activateProfileTip")}
                            onClick={() =>
                              run("profiles.activate", { id: p.id })
                            }
                          >
                            <Power size={13} />
                            {t("ui.activateProfile")}
                          </button>
                        )}
                        <Help
                          label={t("ui.activeProfile")}
                          text={t("help.activeProfile")}
                        />
                        <button
                          className="icon-button"
                          aria-label={t("ui.clone")}
                          title={t("ui.cloneTip")}
                          onClick={() =>
                            run("profiles.clone", { id: p.id })
                          }
                        >
                          <Copy size={16} />
                        </button>
                        <button
                          className="icon-button"
                          aria-label={t("ui.exportProfile")}
                          title={t("ui.exportProfileTip")}
                          onClick={() => void download(`${p.name}.json`, p)}
                        >
                          <Download size={16} />
                        </button>
                        <button
                          className="icon-button danger"
                          aria-label={t("ui.deleteProfile")}
                          title={t("ui.deleteProfile")}
                          onClick={() =>
                            setConfirm({
                              text: t("ui.deleteConfirm", { name: p.name }),
                              run: () =>
                                act("profiles.delete", {
                                  id: p.id,
                                  confirm: true,
                                }),
                            })
                          }
                        >
                          <Trash2 size={16} />
                        </button>
                      </div>
                    </div>
                    {p.rules.map((r) => (
                      <div className="rule-row" key={r.id}>
                        <div className="small-icon">
                          <Workflow size={19} />
                        </div>
                        <div>
                          <strong>
                            {r.name}
                            {!p.enabled && (
                              <small className="inactive-profile-label">
                                {t("ui.profileInactive")}
                              </small>
                            )}
                          </strong>
                          <span>
                            <RuleSummary
                              rule={r}
                              monitor={saved.find(
                                (m) => m.id === r.actions[0]?.monitorId,
                              )}
                            />
                          </span>
                          <span className="rule-badges">
                            {r.dryRun && (
                              <b
                                className="badge warning"
                                title={t("ui.dryRunDetail")}
                              >
                                {t("ui.dryRun")}
                              </b>
                            )}
                            <b className="badge neutral">
                              {t("ui.shortCooldown", { value: r.cooldownMs })}
                            </b>
                          </span>
                        </div>
                        <button
                          className="button"
                          title={t("ui.editRuleTip")}
                          onClick={() => {
                            setProfile(p.id);
                            setEditing(r);
                          }}
                        >
                          <Pencil size={13} />
                          {t("ui.edit")}
                        </button>
                        <button
                          className="button"
                          title={t("ui.testSwitchTip")}
                          disabled={
                            !r.actions[0] ||
                            !data.monitors.some(
                              (m) => m.id === r.actions[0].monitorId,
                            )
                          }
                          onClick={() => testSwitch(r)}
                        >
                          <Play size={13} />
                          {t("ui.testSwitch")}
                        </button>
                        <Switch
                          label={t("ui.enableRule")}
                          checked={r.enabled}
                          onChange={() =>
                            run(`rules.${r.enabled ? "disable" : "enable"}`, {
                              id: r.id,
                            })
                          }
                        />
                      </div>
                    ))}
                    <div className="panel-foot">
                      <button
                        title={t("ui.addRuleTip")}
                        onClick={() => startRule(p.id)}
                      >
                        <Plus size={15} />
                        {t("ui.addRule")}
                      </button>
                    </div>
                  </section>
                ))
              ) : (
                <section className="panel">
                  <div className="empty empty-cta">
                    <div className="empty-icon">
                      <Workflow size={35} />
                    </div>
                    <strong>{t("ui.noProfiles")}</strong>
                    <p>{t("ui.noProfilesWizard")}</p>
                    <button
                      className="button primary cta"
                      title={t("ui.setupOpenTip")}
                      onClick={() => setWizard(true)}
                    >
                      <WandSparkles size={16} />
                      {t("ui.startSetupWizard")}
                    </button>
                    <span className="or-divider">{t("ui.or")}</span>
                    <button
                      className="text-button"
                      title={t("ui.createRuleManuallyTip")}
                      onClick={() => {
                        setProfile("");
                        setEditing(null);
                      }}
                    >
                      <Plus size={13} />
                      {t("ui.createRuleManually")}
                    </button>
                  </div>
                </section>
              )}
            </>
          )}
          {page === "events" && (
            <section className="panel">
              <div className="toolbar">
                <div className="search">
                  <Search size={17} />
                  <input
                    value={search}
                    onChange={(e) => setSearch(e.target.value)}
                    placeholder={t("ui.searchEvents")}
                    aria-label={t("ui.searchEvents")}
                  />
                </div>
                <span className="badge success live-badge" role="status">
                  <span className="dot green" />
                  {t("ui.liveStream")}
                </span>
                <button
                  className="button"
                  onClick={() => {
                    void act("diagnostics.export")
                      .then((v) =>
                        download("elarydil-kvm-diagnostics.json", v),
                      )
                      .catch(() => {});
                  }}
                >
                  <Download size={15} />
                  {t("ui.diagnostics")}
                </button>
              </div>
              {data.events
                .filter((e) =>
                  JSON.stringify(e)
                    .toLowerCase()
                    .includes(search.toLowerCase()),
                )
                .map((e) => (
                  <EventRow key={e.monotonicSequence} event={e} expanded />
                ))}
              {!data.events.length && (
                <Empty
                  icon={<Activity size={30} />}
                  title={t("ui.noEvents")}
                  detail={t("ui.noEventsDetail")}
                />
              )}
            </section>
          )}
          {page === "settings" && config && (
            <>
              <section className="panel settings-panel">
                <PanelHead title={t("ui.appearance")} />
                <Setting label={t("ui.theme")} detail={t("ui.themeDetail")}>
                  <select
                    value={config.app.theme}
                    onChange={(e) =>
                      run("ui.theme.set", { id: e.target.value })
                    }
                  >
                    {["system", "light", "dark"].map((k) => (
                      <option key={k} value={k}>
                        {t(`ui.${k}`)}
                      </option>
                    ))}
                  </select>
                </Setting>
                <Setting label={t("ui.uiMode")} detail={t("ui.uiModeDetail")}>
                  <select
                    value={config.app.uiMode}
                    onChange={(e) => run("ui.mode.set", { id: e.target.value })}
                  >
                    {[
                      "window-only",
                      "tray-only",
                      "both",
                      "headless",
                      "menu-bar-only",
                    ].map((k) => (
                      <option value={k} key={k}>
                        {t(`ui.mode.${k}`)}
                      </option>
                    ))}
                  </select>
                </Setting>
                <Setting
                  label={t("ui.closeAction")}
                  detail={t(
                    config.app.closeWindowAction === "hide"
                      ? "ui.backgroundNote"
                      : config.app.agentStandalone
                        ? "ui.closeQuitStandaloneNote"
                        : "ui.closeQuitNote",
                  )}
                >
                  <select
                    value={config.app.closeWindowAction}
                    onChange={(e) =>
                      run("config.set", {
                        id: "app.closeWindowAction",
                        value: e.target.value,
                      })
                    }
                  >
                    <option value="hide">{t("ui.hide")}</option>
                    <option value="quit-gui">{t("ui.quitGui")}</option>
                  </select>
                </Setting>
                <Setting
                  label={t("ui.language")}
                  detail={t("ui.localizationReady")}
                >
                  <span className="badge neutral">{t("ui.english")}</span>
                </Setting>
              </section>
              <section className="panel settings-panel">
                <PanelHead title={t("ui.background")} />
                <Setting
                  label={t("ui.guiLogin")}
                  detail={t("ui.guiLoginDetail")}
                >
                  <Switch
                    label={t("ui.guiLogin")}
                    checked={config.app.launchGuiAtLogin}
                    onChange={() =>
                      run(
                        `startup.${config.app.launchGuiAtLogin ? "disable" : "enable"}`,
                        { component: "gui" },
                      )
                    }
                  />
                </Setting>
                <Setting
                  label={t("ui.agentStandalone")}
                  detail={t("ui.agentStandaloneDetail")}
                >
                  <Switch
                    label={t("ui.agentStandalone")}
                    checked={config.app.agentStandalone}
                    onChange={() => {
                      const next = !config.app.agentStandalone;
                      void act("config.set", {
                        id: "app.agentStandalone",
                        value: next,
                      })
                        .then(() =>
                          !next && config.app.launchAgentAtLogin
                            ? act("startup.disable", { component: "agent" })
                            : undefined,
                        )
                        .catch(() => {});
                    }}
                  />
                </Setting>
                {(config.app.agentStandalone ||
                  config.app.launchAgentAtLogin) && (
                  <Setting
                    label={t("ui.agentLogin")}
                    detail={t("ui.agentLoginDetail")}
                  >
                    <Switch
                      label={t("ui.agentLogin")}
                      checked={config.app.launchAgentAtLogin}
                      onChange={() =>
                        run(
                          `startup.${config.app.launchAgentAtLogin ? "disable" : "enable"}`,
                          { component: "agent" },
                        )
                      }
                    />
                  </Setting>
                )}
                <Setting
                  label={t("ui.globalCooldown")}
                  detail={t("ui.globalCooldownDetail")}
                >
                  <NumberSetting
                    value={config.safety.globalMonitorCooldownMs}
                    min={0}
                    max={120000}
                    save={(v) =>
                      run("config.set", {
                        id: "safety.globalMonitorCooldownMs",
                        value: v,
                      })
                    }
                  />
                </Setting>
                <Setting
                  label={t("ui.startupGrace")}
                  detail={t("ui.startupGraceDetail")}
                >
                  <NumberSetting
                    value={config.safety.startupGraceMs}
                    min={0}
                    max={120000}
                    save={(v) =>
                      run("config.set", {
                        id: "safety.startupGraceMs",
                        value: v,
                      })
                    }
                  />
                </Setting>
                <Setting
                  label={t("ui.resumeGrace")}
                  detail={t("ui.resumeGraceDetail")}
                >
                  <NumberSetting
                    value={config.safety.resumeGraceMs}
                    min={0}
                    max={120000}
                    save={(v) =>
                      run("config.set", {
                        id: "safety.resumeGraceMs",
                        value: v,
                      })
                    }
                  />
                </Setting>
                {(["startupDelayMs"] as const).map((k) => (
                  <Setting
                    key={k}
                    label={t(`ui.setting.${k}`)}
                    detail={t("ui.milliseconds")}
                  >
                    <NumberSetting
                      value={config.app[k]}
                      min={0}
                      max={120000}
                      save={(v) =>
                        run("config.set", { id: `app.${k}`, value: v })
                      }
                    />
                  </Setting>
                ))}
                {(["defaultDebounceMs", "commandTimeoutMs"] as const).map(
                  (k) => (
                    <Setting
                      key={k}
                      label={t(`ui.setting.${k}`)}
                      detail={t("ui.milliseconds")}
                    >
                      <NumberSetting
                        value={config.safety[k]}
                        min={k === "commandTimeoutMs" ? 100 : 0}
                        max={k === "commandTimeoutMs" ? 30000 : 60000}
                        save={(v) =>
                          run("config.set", { id: `safety.${k}`, value: v })
                        }
                      />
                    </Setting>
                  ),
                )}
              </section>
              <section className="panel settings-panel">
                <PanelHead title={t("ui.diagnostics")} />
                <Setting
                  label={t("ui.diagnosticReport")}
                  detail={t("ui.diagnosticReportDetail")}
                >
                  <button
                    className="button"
                    onClick={() => {
                      void act("diagnostics.export")
                        .then((report) =>
                          download("elarydil-kvm-diagnostics.json", report),
                        )
                        .catch(() => {});
                    }}
                  >
                    <Download size={15} />
                    {t("ui.exportReport")}
                  </button>
                </Setting>
                <Setting
                  label={t("ui.configuration")}
                  detail={t("ui.configDetail")}
                >
                  <button
                    className="button"
                    onClick={() =>
                      void download("elarydil-kvm-config.json", config)
                    }
                  >
                    <Download size={15} />
                    {t("ui.export")}
                  </button>
                </Setting>
                <Setting
                  label={t("ui.daemon")}
                  detail={t("ui.backgroundIndependent")}
                >
                  <div className="inline-actions">
                    <button
                      className="button"
                      onClick={() => run("agent.restart")}
                    >
                      <RefreshCw size={15} />
                      {t("ui.restart")}
                    </button>
                    <button
                      className="button danger"
                      onClick={() =>
                        setConfirm({
                          text: t("ui.stopConfirm"),
                          run: () => act("agent.stop"),
                        })
                      }
                    >
                      <Square size={13} />
                      {t("ui.stopAgent")}
                    </button>
                  </div>
                </Setting>
              </section>
            </>
          )}
        </div>
        <footer className="statusbar">
          <span>
            <ShieldCheck size={12} />
            {t("ui.localOnly")}
          </span>
          <span>{t("ui.brand")}</span>
        </footer>
      </main>
      {actionError && (
        <div className="action-error-toast" role="alert">
          <AlertCircle size={16} />
          {actionError}
          <button
            className="icon-button"
            aria-label={t("ui.close")}
            onClick={() => setActionError("")}
          >
            <X size={14} />
          </button>
        </div>
      )}
      {details && (
        <Modal title={t("ui.deviceDetails")} close={() => setDetails(null)}>
          <div className="detail-icon">
            <DeviceIcon device={details} size={30} />
          </div>
          <h3>{details.name}</h3>
          <dl>
            <dt>{t("ui.deviceModel")}</dt>
            <dd>{details.model || t("ui.notReported")}</dd>
            <dt>{t("ui.deviceType")}</dt>
            <dd>{t(`usb.type.${details.category}`)}</dd>
            <dt>{t("ui.manufacturer")}</dt>
            <dd>{details.manufacturer || t("ui.notReported")}</dd>
            <dt>{t("ui.usbId")}</dt>
            <dd>
              <code>
                {hex(details.vid)}:{hex(details.pid)}
              </code>
            </dd>
            <dt>{t("ui.path")}</dt>
            <dd>
              <code>{details.path}</code>
            </dd>
            <dt>{t("ui.serial")}</dt>
            <dd>{t(details.serial ? "ui.available" : "ui.unavailable")}</dd>
          </dl>
          <label className="field">
            {t("ui.alias")}
            <input
              id="alias"
              defaultValue={
                tracked.find((d) => d.id === details.id)?.alias ?? details.name
              }
            />
          </label>
          <button
            className="button primary full"
            onClick={() => {
              void act("devices.track", {
                id: details.id,
                name: (document.getElementById("alias") as HTMLInputElement)
                  .value,
              })
                .then(() => setDetails(null))
                .catch(() => {});
            }}
          >
            <Save size={14} />
            {t("ui.saveTracked")}
          </button>
        </Modal>
      )}
      {editing !== undefined && (
        <RuleEditor
          rule={editing}
          profileId={profile}
          config={config}
          devices={data.devices}
          monitors={data.monitors}
          selected={selected}
          act={act}
          confirm={setConfirm}
          close={() => setEditing(undefined)}
        />
      )}
      {wizard && (
        <Wizard
          data={data}
          act={act}
          close={() => setWizard(false)}
        />
      )}
      {confirm && (
        <Modal title={t("ui.confirmAction")} close={() => setConfirm(null)}>
          <div className="banner warning">
            <AlertCircle size={20} />
            <span>{confirm.text}</span>
          </div>
          <div className="modal-actions">
            <button className="button" onClick={() => setConfirm(null)}>
              <X size={14} />
              {t("ui.cancel")}
            </button>
            <button
              className="button primary"
              onClick={() => {
                void confirm
                  .run()
                  .then(() => setConfirm(null))
                  .catch(() => {});
              }}
            >
              <Check size={14} />
              {t("ui.confirm")}
            </button>
          </div>
        </Modal>
      )}
    </div>
  );
}
function PanelHead({
  title,
  count,
  action,
  label,
}: {
  title: string;
  count?: number;
  action?: () => void;
  label?: string;
}) {
  return (
    <div className="panel-head">
      <h2>
        {title}
        {count !== undefined && <b className="count">{count}</b>}
      </h2>
      {action && (
        <button className="text-button" onClick={action}>
          {label}
          <ArrowRight size={14} />
        </button>
      )}
    </div>
  );
}
function Empty({
  icon,
  title,
  detail,
  action,
  label,
}: {
  icon: React.ReactNode;
  title: string;
  detail: string;
  action?: () => void;
  label?: string;
}) {
  return (
    <div className="empty">
      <div className="empty-icon">{icon}</div>
      <strong>{title}</strong>
      <p>{detail}</p>
      {action && (
        <button className="button" onClick={action}>
          {label}
          <ArrowRight size={14} />
        </button>
      )}
    </div>
  );
}
function EventRow({
  event,
  expanded = false,
}: {
  event: Event;
  expanded?: boolean;
}) {
  return (
    <div className={`event-row ${expanded ? "expanded" : ""}`}>
      <div
        className={`event-icon ${event.severity === "error" ? "error" : ""}`}
      >
        {event.severity === "error" ? (
          <AlertCircle size={15} />
        ) : (
          <Activity size={15} />
        )}
      </div>
      <div>
        <strong>{t(`event.${event.eventType}`)}</strong>
        {expanded && <code>{JSON.stringify(event.result)}</code>}
      </div>
      <time>
        {new Date(event.timestamp).toLocaleTimeString("en-US", {
          hour: "2-digit",
          minute: "2-digit",
          second: "2-digit",
          hour12: false,
        })}
      </time>
    </div>
  );
}
createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
