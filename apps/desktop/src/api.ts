import { invoke } from "@tauri-apps/api/core";
import catalog from "../../../locales/en-US.json";
export const t = (key: string, vars: Record<string, string | number> = {}) =>
  Object.entries(vars).reduce(
    (s, [k, v]) => s.replaceAll(`{${k}}`, String(v)),
    (catalog as Record<string, string>)[
      vars.count === 1 && `${key}.one` in catalog ? `${key}.one` : key
    ] ?? key,
  );
export type Device = {
  id: string;
  name: string;
  model?: string | null;
  manufacturer?: string;
  alias?: string;
  vid: number;
  pid: number;
  serial?: string;
  path: string;
  category: string;
  connected: boolean;
};
export type Input = {
  value: number;
  vcpCode: number;
  label: string;
  verified: boolean;
};
export type Monitor = {
  id: string;
  name: string;
  alias?: string;
  connector: string;
  backend: string;
  permissions: string;
  fingerprint: string;
  inputOptions?: Input[];
  hiddenInputs?: number[];
};
export type CommandHooks = {
  enabled: boolean;
  onConnect: string;
  onDisconnect: string;
  shell: string;
  workingDirectory: string;
  timeoutMs: number;
};
export type Rule = {
  commands?: CommandHooks;
  id: string;
  name: string;
  trigger: string;
  deviceIds: string[];
  mode: string;
  minimumCount: number;
  debounceMs: number;
  rearmMs: number;
  cooldownMs: number;
  coincidenceWindowMs: number;
  enabled: boolean;
  dryRun: boolean;
  actions: {
    id: string;
    monitorId: string;
    inputValue: number;
    vcpCode: number;
  }[];
};
export type Profile = {
  id: string;
  name: string;
  enabled: boolean;
  rules: Rule[];
};
export type Config = {
  schemaVersion: number;
  app: {
    theme: string;
    uiMode: string;
    locale: string;
    launchAgentAtLogin: boolean;
    launchGuiAtLogin: boolean;
    agentStandalone: boolean;
    closeWindowAction: string;
    notifications: string;
    lastPage: string;
    logRetentionDays: number;
    logMaxBytes: number;
    startupDelayMs: number;
  };
  safety: {
    globalMonitorCooldownMs: number;
    startupGraceMs: number;
    resumeGraceMs: number;
    defaultDebounceMs: number;
    commandTimeoutMs: number;
  };
  usbDevices: {
    id: string;
    alias: string;
    enabled: boolean;
    selectors: unknown[];
  }[];
  monitors: Monitor[];
  profiles: Profile[];
};
export async function request<T = unknown>(
  command: string,
  payload: Record<string, unknown> = {},
): Promise<T> {
  return invoke<T>("request", { command, payload });
}
export const hex = (n: number) =>
  `0x${n.toString(16).toUpperCase().padStart(2, "0")}`;

export type MonitorState = {
  monitorId: string;
  currentInput?: number | null;
  ddcAvailable?: boolean | null;
  lastReadMs?: number;
  error?: { message: string; detail?: string };
  test?: {
    id: string;
    originalInput: number;
    targetInput: number;
    deadlineMs: number;
    finishedAtMs?:number;
    phase: string;
    error?: { message: string };
  };
};
export const defaultInputs = (): Input[] =>
  [15, 16, 17, 18].map((value) => ({
    value,
    vcpCode: 96,
    label: t(`input.${value}`),
    verified: false,
  }));
export function monitorInputs(m?: Monitor, current?: number | null): Input[] {
  const merged = defaultInputs().filter(
    (i) => !m?.hiddenInputs?.includes(i.value),
  );
  for (const i of m?.inputOptions ?? []) {
    const at = merged.findIndex(
      (x) => x.value === i.value && x.vcpCode === i.vcpCode,
    );
    if (at < 0) merged.push(i);
    else merged[at] = i;
  }
  if (
    current &&
    current > 0 &&
    current <= 255 &&
    !m?.hiddenInputs?.includes(current) &&
    !merged.some((i) => i.value === current)
  )
    merged.push({
      value: current,
      vcpCode: 96,
      label: t("ui.inputNumber", { value: current }),
      verified: true,
    });
  return merged;
}
export function inputName(input: Input): string {
  const base = [15, 16, 17, 18].includes(input.value)
    ? t(`input.${input.value}`)
    : t("ui.inputNumber", { value: input.value });
  return input.label && input.label !== base
    ? `${base} (${input.label})`
    : base;
}
export const errorMessage = (e: unknown) =>
  typeof e === "object" && e && "message" in e
    ? String(e.message) + ("detail" in e && e.detail ? ` · ${e.detail}` : "")
    : String(e);
