import { test, expect } from "@playwright/test";
import { execFile, spawn } from "node:child_process";
import { promisify } from "node:util";
import { mkdtemp, rm, readFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
const exec = promisify(execFile),
  binary = resolve("../../target/debug/examples/virtual_agent");
let home: string, agent: ReturnType<typeof spawn>;
async function rpc(command: string, payload: object = {}) {
  const { stdout } = await exec(
    binary,
    ["request", command, JSON.stringify(payload)],
    { env: { ...process.env, ELARYDIL_KVM_HOME: home } },
  );
  const result = JSON.parse(stdout);
  if (result.Err) throw result.Err;
  return result.Ok;
}
test.beforeAll(async () => {
  home = await mkdtemp(tmpdir() + "/elarydil-kvm-ui-");
  // Like the desktop app, the test runner owns its agent: the agent reports
  // the default app-bound lifecycle and exits if the runner dies.
  agent = spawn(binary, [], {
    env: {
      ...process.env,
      ELARYDIL_KVM_HOME: home,
      ELARYDIL_KVM_OWNER_PID: String(process.pid),
    },
  });
  for (let i = 0; i < 100; i++) {
    try {
      await rpc("status");
      return;
    } catch {
      await new Promise((r) => setTimeout(r, 50));
    }
  }
  throw Error("Virtual test agent did not start");
});
test.afterAll(async () => {
  try {
    await rpc("agent.stop");
  } catch {}
  agent?.kill();
  await rm(home, { recursive: true, force: true });
});
test.beforeEach(async ({ page }, info) => {
  await rpc("monitors.input.restore", { id: "monitor_desk" }).catch(() => {});
  await rpc("test.input.reset").catch(() => {});
  await rpc("config.reset", { confirm: true });
  await rpc("test.usb", { present: ["usb_0", "usb_1", "usb_2"] });
  if (!info.title.startsWith("manual") && !info.title.startsWith("wizard")) {
    for (const id of ["usb_0", "usb_1"]) await rpc("devices.track", { id });
    await rpc("monitors.remember", { id: "monitor_desk" });
    await rpc("monitors.input.alias.set", {
      id: "monitor_desk",
      value: 18,
      label: "Work MacBook",
    });
    const profile = await rpc("profiles.create", { name: "Desk KVM" });
    await rpc("rules.create", {
      profile: profile.id,
      name: "Switch to MacBook",
      devices: ["usb_0", "usb_1"],
      monitor: "monitor_desk",
      input: 18,
      mode: "all",
      on: "disconnect",
      "dry-run": false,
    });
  }
  await page.route("**/__elarydil-kvm_test", async (route) => {
    const { command, payload } = route.request().postDataJSON();
    try {
      await route.fulfill({ json: { ok: await rpc(command, payload) } });
    } catch (error) {
      await route.fulfill({ json: { error } });
    }
  });
  await page.addInitScript(() => {
    (
      window as unknown as { __TAURI_INTERNALS__: unknown }
    ).__TAURI_INTERNALS__ = {
      invoke: async (_name: string, args: unknown) => {
        if (_name !== "request") return 1;
        const response = await fetch("/__elarydil-kvm_test", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(args),
        });
        const data = await response.json();
        if (data.error) throw data.error;
        return data.ok;
      },
      transformCallback: () => 0,
      unregisterCallback: () => {},
    };
  });
  await page.addInitScript(() => {
    (
      window as unknown as { __TAURI_EVENT_PLUGIN_INTERNALS__: unknown }
    ).__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} };
  });
  await page.goto("/");
  await expect(page.getByText("Agent running")).toBeVisible();
});
async function nav(page: import("@playwright/test").Page, name: string) {
  await page
    .locator(".sidebar")
    .getByRole("button", { name, exact: true })
    .click();
}
async function editRule(page: import("@playwright/test").Page) {
  await nav(page, "Profiles & rules");
  await page.getByRole("button", { name: "Edit", exact: true }).first().click();
  return page.getByRole("dialog").first();
}
test("manual setup selects unverified defaults without saving aliases or testing", async ({
  page,
}) => {
  expect((await rpc("config.show")).app.theme).toBe("system");
  await nav(page, "USB devices");
  await page
    .getByRole("checkbox", { name: "KVM keyboard", exact: true })
    .check();
  await page
    .getByRole("checkbox", { name: "Wireless mouse", exact: true })
    .check();
  await nav(page, "Profiles & rules");
  await expect(
    page.getByRole("button", { name: "Start setup wizard", exact: true }),
  ).toBeVisible();
  await page
    .getByRole("button", { name: "Create a rule manually", exact: true })
    .click();
  const dialog = page.getByRole("dialog");
  await dialog.getByLabel("Rule name", { exact: true }).fill("Switch away");
  await dialog.getByLabel("KVM keyboard", { exact: true }).check();
  await dialog.getByLabel("Wireless mouse", { exact: true }).check();
  await dialog.getByRole("radio", { name: "HDMI 2", exact: true }).click();
  await dialog.getByText("Advanced", { exact: true }).click();
  await expect(
    dialog.getByRole("switch", { name: "Dry run", exact: true }),
  ).toHaveAttribute("aria-checked", "false");
  await dialog.getByRole("button", { name: "Save rule", exact: true }).click();
  await expect(dialog).not.toBeVisible();
  const c = await rpc("config.show");
  expect(c.profiles[0].rules[0].debounceMs).toBe(0);
  expect(c.profiles[0].rules[0].dryRun).toBe(false);
  expect(c.profiles[0].rules[0].actions[0].inputValue).toBe(18);
});
test("monitor tiles read current input and support rename custom input deletion", async ({
  page,
}) => {
  await nav(page, "Monitors");
  await expect(page.getByText("DDC available", { exact: true })).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Monitor selected" }),
  ).toHaveCount(0);
  await expect(page.locator(".input-tile.current strong")).toHaveText("HDMI 1");
  await page
    .getByRole("button", { name: "Rename HDMI 2", exact: true })
    .click();
  await page
    .getByLabel("Friendly input name", { exact: true })
    .fill("Work MacBook");
  await page.screenshot({ path: "test-results/desktop-input-rename.png" });
  await page.getByRole("button", { name: "Save input", exact: true }).click();
  await expect(
    page.getByRole("radio", { name: "HDMI 2 (Work MacBook)", exact: true }),
  ).toBeVisible();
  await page.getByRole("button",{name:"Remove friendly name from HDMI 2",exact:true}).click();
  await expect(page.getByRole("radio",{name:"HDMI 2",exact:true})).toBeVisible();
  expect((await rpc("config.show")).profiles[0].rules[0].actions[0].inputValue).toBe(18);
  await page
    .getByRole("button", { name: "Delete DisplayPort 2", exact: true })
    .click();
  await expect(
    page.getByRole("radio", { name: "DisplayPort 2", exact: true }),
  ).toHaveCount(0);
  await page
    .getByRole("button", { name: "Add custom input", exact: true })
    .click();
  const raw = page.getByLabel("Raw input — decimal (18) or hex (0x12)", {
    exact: true,
  });
  await raw.fill("oops");
  await raw.blur();
  await expect(
    page.getByText(
      "Enter a decimal value from 1 to 255, or hex from 0x01 to 0xFF.",
      { exact: true },
    ),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Save input", exact: true }),
  ).toBeDisabled();
  await raw.fill("0x1F");
  await page.getByLabel("Friendly input name", { exact: true }).fill("Dock");
  await page.getByRole("button", { name: "Save input", exact: true }).click();
  await expect(
    page.getByRole("radio", { name: "Input 31 (Dock)", exact: true }),
  ).toBeVisible();
  await page
    .getByRole("button", { name: "Delete Input 31", exact: true })
    .click();
  await expect(
    page.getByRole("radio", { name: "Input 31 (Dock)", exact: true }),
  ).toHaveCount(0);
});
test("rule validation is inline cooldown 100 is valid and summary uses friendly names", async ({
  page,
}) => {
  await rpc("monitors.input.alias.set", {
    id: "monitor_desk",
    value: 18,
    label: "Work MacBook",
  });
  const d = await editRule(page);
  await expect(
    d.getByRole("radio", { name: "HDMI 2 (Work MacBook)", exact: true }),
  ).toBeVisible();
  const cooldown = d.getByLabel("Cooldown (ms)", { exact: true });
  await cooldown.fill("-1");
  await cooldown.blur();
  await expect(cooldown).toHaveAttribute("aria-invalid", "true");
  await expect(
    d.getByText("Enter a whole number from 0 to 120000.", { exact: true }),
  ).toBeVisible();
  await expect(
    d.getByRole("button", { name: "Save rule", exact: true }),
  ).toBeDisabled();
  await cooldown.fill("100");
  await cooldown.blur();
  await d.getByLabel("Debounce (ms)", { exact: true }).fill("0");
  await expect(d.locator(".preview")).toContainText("HDMI 2 (Work MacBook)");
  await expect(d.locator(".preview em")).not.toHaveCount(0);
  await d.locator(".modal-body").evaluate((body) => { body.scrollTop = body.scrollHeight; });
  await page.screenshot({ path: "test-results/desktop-rule-editor.png" });
  await d.getByRole("button", { name: "Save rule", exact: true }).click();
  await expect(d).not.toBeVisible();
  expect((await rpc("config.show")).profiles[0].rules[0].cooldownMs).toBe(100);
  await expect(page.getByText("Agent running", { exact: true })).toBeVisible();
});
test("wizard live capture shows edge timing then creates two usable rules", async ({
  page,
}) => {
  await rpc("config.set", { id: "safety.startupGraceMs", value: 0 });
  await page
    .getByRole("button", { name: "Setup wizard", exact: true })
    .first()
    .click();
  const d = page.getByRole("dialog");
  await d.getByRole("button", { name: "Start capture", exact: true }).click();
  await expect(d.locator(".capture-table tbody tr")).toHaveCount(0);
  await rpc("test.usb", { present: ["usb_1", "usb_2"] });
  await expect(d.locator(".capture-table tbody tr")).toHaveCount(1);
  await expect(d.getByLabel("Waiting for this event")).toHaveCount(1);
  await rpc("test.usb", { present: ["usb_2"] });
  await expect(d.locator(".capture-table tbody tr")).toHaveCount(2);
  await rpc("test.usb", { present: ["usb_0", "usb_2"] });
  await rpc("test.usb", { present: ["usb_0", "usb_1", "usb_2"] });
  await expect(d.getByLabel("Waiting for this event")).toHaveCount(0);
  const observed = (await rpc("wizard.candidates")).observations;
  expect(observed[0].disconnectedAtMs).toBeLessThanOrEqual(
    observed[1].disconnectedAtMs,
  );
  expect(observed[0].connectedAtMs).toBeGreaterThan(
    observed[0].disconnectedAtMs,
  );
  await page.screenshot({ path: "test-results/desktop-wizard-capture.png" });
  await d.getByRole("button", { name: "Next: select devices", exact: true }).click();
  await d.getByRole("button", { name: "Choose screens", exact: true }).click();
  await d
    .locator(".wizard-target")
    .nth(0)
    .getByRole("radio", { name: "HDMI 1", exact: true })
    .click();
  await d
    .locator(".wizard-target")
    .nth(1)
    .getByRole("radio", { name: "HDMI 2", exact: true })
    .click();
  await page.screenshot({ path: "test-results/desktop-wizard-screens.png" });
  await d
    .getByRole("button", { name: "Create profile & finish", exact: true })
    .click();
  await expect(d.getByText("Your KVM is ready", { exact: true })).toBeVisible();
  const c = await rpc("config.show");
  expect(c.profiles).toHaveLength(1);
  expect(c.profiles[0].enabled).toBe(true);
  expect(c.profiles[0].rules).toHaveLength(2);
  for (const r of c.profiles[0].rules) {
    expect(r.dryRun).toBe(false);
    expect(r.debounceMs).toBe(0);
  }
  await d.getByRole("button", { name: "Done", exact: true }).click();
  await rpc("test.usb", { present: ["usb_2"] });
  await expect
    .poll(
      async () =>
        (await rpc("monitors.input.get", { id: "monitor_desk" })).value,
    )
    .toBe(18);
  await page.waitForTimeout(600);
  await rpc("test.usb", { present: ["usb_0", "usb_1", "usb_2"] });
  await expect
    .poll(
      async () =>
        (await rpc("monitors.input.get", { id: "monitor_desk" })).value,
    )
    .toBe(17);
});
test("wizard starts from scratch on every opening and cancel stops the capture", async ({
  page,
}) => {
  const open = () =>
    page.getByRole("button", { name: "Setup wizard", exact: true }).first().click();
  await open();
  let d = page.getByRole("dialog");
  // Rules must not switch monitors while the user toggles the KVM during setup.
  await expect.poll(async () => (await rpc("status")).setupHold).toBe(true);
  await d.getByRole("button", { name: "Start capture", exact: true }).click();
  await expect(d.getByText("Press the switch on your KVM", { exact: true })).toBeVisible();
  await rpc("test.usb", { present: ["usb_1", "usb_2"] });
  await expect(d.locator(".capture-table tbody tr")).toHaveCount(1);
  await d.getByRole("button", { name: "Cancel", exact: true }).click();
  await expect(d).not.toBeVisible();
  expect((await rpc("wizard.candidates")).session.active).toBe(false);
  await expect.poll(async () => (await rpc("status")).setupHold).toBe(false);
  await rpc("test.usb", { present: ["usb_0", "usb_1", "usb_2"] });
  await open();
  d = page.getByRole("dialog");
  await expect(d.getByRole("button", { name: "Start capture", exact: true })).toBeVisible();
  await expect(d.locator(".capture-table")).toHaveCount(0);
});
test("wizard capture sorts by phase and selects trigger devices per direction", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Setup wizard", exact: true }).first().click();
  const d = page.getByRole("dialog");
  await d.getByRole("button", { name: "Start capture", exact: true }).click();
  await rpc("test.usb", { present: ["usb_1", "usb_2"] });
  await rpc("test.usb", { present: ["usb_2"] });
  await expect(d.locator(".capture-table tbody tr")).toHaveCount(2);
  await expect(
    d.locator("th[aria-sort=ascending]").getByText("Disconnected"),
  ).toBeVisible();
  // The first device of the column is the zero point; its absolute time is shown too.
  await expect(d.locator("tbody tr").first()).toContainText("+0 ms");
  await expect(d.locator("tbody tr").first()).toContainText("at ");
  await rpc("test.usb", { present: ["usb_0", "usb_2"] });
  await expect(
    d.locator("th[aria-sort=ascending]").getByText("Connected"),
  ).toBeVisible();
  await rpc("test.usb", { present: ["usb_0", "usb_1", "usb_2"] });
  await d.getByRole("button", { name: "Next: select devices", exact: true }).click();
  const disconnect = d.getByRole("checkbox", {
    name: "Use Wireless mouse for the disconnect trigger",
    exact: true,
  });
  const connect = d.getByRole("checkbox", {
    name: "Use Wireless mouse for the connect trigger",
    exact: true,
  });
  await disconnect.check();
  await connect.uncheck();
  await d.getByRole("checkbox", { name: "Use KVM keyboard for the connect trigger", exact: true }).check();
  await page.screenshot({ path: "test-results/desktop-wizard-triggers.png" });
  await d.getByRole("button", { name: "Choose screens", exact: true }).click();
  await d.getByRole("button", { name: "Create profile & finish", exact: true }).click();
  await expect(d.getByText("Your KVM is ready", { exact: true })).toBeVisible();
  const rules = (await rpc("config.show")).profiles[0].rules;
  const ids = (trigger: string) =>
    rules.find((r: { trigger: string }) => r.trigger === trigger).deviceIds;
  expect(ids("disconnect")).toContain("usb_1");
  expect(ids("connect")).not.toContain("usb_1");
  expect(ids("connect")).toContain("usb_0");
});
test("wizard manual path lists every connected device and completes both rules", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Setup wizard", exact: true }).first().click();
  const d = page.getByRole("dialog");
  await d.getByRole("button", { name: "Choose devices manually", exact: true }).click();
  await expect(d.locator(".trigger-table tbody tr")).toHaveCount(3);
  await expect(d.getByRole("button", { name: "Choose screens", exact: true })).toBeDisabled();
  await d.getByRole("checkbox", { name: "Use KVM keyboard for the disconnect trigger", exact: true }).check();
  await d.getByRole("checkbox", { name: "Use Wireless mouse for the connect trigger", exact: true }).check();
  await d.getByRole("button", { name: "Choose screens", exact: true }).click();
  await d.getByRole("button", { name: "Create profile & finish", exact: true }).click();
  await expect(d.getByText("Your KVM is ready", { exact: true })).toBeVisible();
  const rules = (await rpc("config.show")).profiles[0].rules;
  expect(rules).toHaveLength(2);
  const ids = (trigger: string) =>
    rules.find((r: { trigger: string }) => r.trigger === trigger).deviceIds;
  expect(ids("disconnect")).toEqual(["usb_0"]);
  expect(ids("connect")).toEqual(["usb_1"]);
});
test("input test confirms countdown and restores immediately on request", async ({
  page,
}) => {
  await nav(page, "Monitors");
  await expect(page.locator(".input-tile.current strong")).toHaveText("HDMI 1");
  await page.getByRole("button", { name: "Test HDMI 2", exact: true }).click();
  await expect(page.getByRole("dialog")).toContainText("15 seconds");
  expect((await rpc("monitors.input.get", { id: "monitor_desk" })).value).toBe(
    17,
  );
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "Confirm", exact: true })
    .click();
  await expect(
    page.getByRole("button", { name: "Switch back now", exact: true }),
  ).toBeVisible();
  await expect(page.locator(".input-tile.current strong")).toHaveText(
    "HDMI 2 (Work MacBook)",
  );
  await page
    .getByRole("button", { name: "Switch back now", exact: true })
    .click();
  await expect(
    page.getByText("Original input restored", { exact: true }),
  ).toBeVisible();
  await expect(page.locator(".input-tile.current strong")).toHaveText("HDMI 1");
});
test("fifteen second rollback survives a closed UI", async ({ page }) => {
  const testRun = await rpc("monitors.input.test", {
    id: "monitor_desk",
    value: 18,
    confirm: true,
  });
  expect(testRun.originalInput).toBe(17);
  expect((await rpc("monitors.input.get", { id: "monitor_desk" })).value).toBe(
    18,
  );
  await page.goto("about:blank");
  await expect
    .poll(
      async () =>
        (await rpc("monitors.state", { id: "monitor_desk" })).test?.phase,
      { timeout: 19000, intervals: [500, 1000] },
    )
    .toBe("restored");
  expect((await rpc("monitors.input.get", { id: "monitor_desk" })).value).toBe(
    17,
  );
  expect((await rpc("status")).guiIndependent).toBe(true);
});
test("commands live in rule advanced and use the independent agent", async ({
  page,
}) => {
  let d = await editRule(page);
  await d.getByText("Advanced", { exact: true }).click();
  await d
    .getByLabel("When KVM connects", { exact: true })
    .fill("printf connected > command-connect.txt");
  await d.getByLabel("Working directory", { exact: true }).fill(home);
  await page.screenshot({ path: "test-results/desktop-rule-advanced.png" });
  await d
    .getByRole("switch", { name: "Run commands automatically", exact: true })
    .click();
  await d.getByRole("button", { name: "Save rule", exact: true }).click();
  await expect(d).not.toBeVisible();
  d = await editRule(page);
  await d.getByText("Advanced", { exact: true }).click();
  await d.getByRole("button", { name: "Test connect", exact: true }).click();
  await page.getByRole("button", { name: "Confirm", exact: true }).click();
  await expect
    .poll(() =>
      readFile(resolve(home, "command-connect.txt"), "utf8").catch(() => ""),
    )
    .toBe("connected");
  await expect(page.getByRole("button", { name: "Confirm", exact: true })).toBeHidden();
  await page.keyboard.press("Escape");
  await expect(d).not.toBeVisible();
  await nav(page, "Settings");
  await expect(page.getByText("Custom commands", { exact: true })).toHaveCount(
    0,
  );
  const report = await rpc("diagnostics.export");
  expect(report.config.profiles[0].rules[0].commands.onConnect).toBe(
    "<redacted>",
  );
});
test("one active profile and each rule has an independent switch", async ({
  page,
}) => {
  const second = await rpc("profiles.create", { name: "Second desk" });
  await nav(page, "Profiles & rules");
  await expect.poll(async () => page.locator(".profile-panel").count()).toBe(2);
  await page
    .locator(".profile-panel")
    .filter({ hasText: "Second desk" })
    .getByRole("button", { name: "Activate profile", exact: true })
    .click();
  await expect
    .poll(async () =>
      (await rpc("config.show")).profiles
        .filter((p: { enabled: boolean }) => p.enabled)
        .map((p: { id: string }) => p.id),
    )
    .toEqual([second.id]);
  await expect(
    page.getByText("Profile inactive", { exact: true }),
  ).toBeVisible();
  await expect(
    page.locator(".profile-panel.is-active").getByText("Active profile", { exact: true }),
  ).toBeVisible();
  await page.screenshot({ path: "test-results/desktop-profiles-active.png" });
});
test("rule test performs the real switch reversibly after confirmation", async ({
  page,
}) => {
  await nav(page, "Profiles & rules");
  await page.getByRole("button", { name: "Test", exact: true }).first().click();
  await expect(page.getByRole("dialog")).toContainText("15 seconds");
  expect((await rpc("monitors.input.get", { id: "monitor_desk" })).value).toBe(17);
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "Confirm", exact: true })
    .click();
  await expect
    .poll(async () => (await rpc("monitors.input.get", { id: "monitor_desk" })).value)
    .toBe(18);
  await rpc("monitors.input.restore", { id: "monitor_desk" });
});
test("standalone agent is an explicit opt-in that reveals agent login", async ({
  page,
}) => {
  await nav(page, "Settings");
  await expect(
    page.getByRole("switch", { name: "Start agent at login", exact: true }),
  ).toHaveCount(0);
  await page
    .getByRole("switch", { name: "Keep agent running when the app quits", exact: true })
    .click();
  await expect
    .poll(async () => (await rpc("config.show")).app.agentStandalone)
    .toBe(true);
  await expect(
    page.getByRole("switch", { name: "Start agent at login", exact: true }),
  ).toBeVisible();
});
test("rule editor fields keep a consistent gap in the dark theme", async ({
  page,
}) => {
  await rpc("ui.theme.set", { id: "dark" });
  await page.reload();
  const d = await editRule(page);
  await d.getByText("Advanced", { exact: true }).click();
  const gaps = await d.evaluate((dialog) => {
    const fields = [...dialog.querySelectorAll<HTMLElement>(".modal-body .field")]
      .map((f) => f.getBoundingClientRect())
      .filter((r) => r.height > 0);
    const out: number[] = [];
    for (let i = 1; i < fields.length; i++)
      if (Math.abs(fields[i].left - fields[i - 1].left) < 2)
        out.push(fields[i].top - fields[i - 1].bottom);
    return out;
  });
  expect(gaps.length).toBeGreaterThan(0);
  for (const gap of gaps) expect(gap).toBeGreaterThanOrEqual(12);
  await d.locator(".modal-body").evaluate((body) => {
    body.scrollTop = body.scrollHeight;
  });
  await page.screenshot({ path: "test-results/desktop-rule-advanced-dark.png" });
  await page.keyboard.press("Escape");
  await nav(page, "Event log");
  await expect(page.locator(".live-badge")).toBeVisible();
  const heights = await page.evaluate(() => ({
    badge: document.querySelector(".live-badge")!.getBoundingClientRect().height,
    button: document.querySelector(".toolbar .button")!.getBoundingClientRect().height,
  }));
  expect(Math.abs(heights.badge - heights.button)).toBeLessThanOrEqual(1);
  await page.screenshot({ path: "test-results/desktop-events-dark-live.png" });
  await nav(page, "Profiles & rules");
  await page.screenshot({ path: "test-results/desktop-profiles-dark.png" });
});
test("settings errors stay visible helpers explain timings controls have matching heights", async ({
  page,
}) => {
  await nav(page, "Settings");
  const number = page.getByRole("spinbutton").first();
  await number.fill("-1");
  await number.blur();
  await expect(
    page.getByText("Enter a whole number from 0 to 120000.", { exact: true }),
  ).toBeVisible();
  await expect(page.getByText("Agent running", { exact: true })).toBeVisible();
  await page
    .getByRole("button", {
      name: "Help: Minimum interval between monitor writes",
      exact: true,
    })
    .click();
  await expect(page.getByRole("tooltip")).toContainText("across all rules");
  // Only one help popover at a time, and clicking elsewhere closes it.
  await page
    .getByRole("button", { name: "Help: Color theme", exact: true })
    .click();
  await expect(page.getByRole("tooltip")).toHaveCount(1);
  await expect(page.getByRole("tooltip")).not.toContainText("across all rules");
  await page.locator(".topbar h1").click();
  await expect(page.getByRole("tooltip")).toHaveCount(0);
  const heights = await page.evaluate(() => ({
    select: document.querySelector("select")!.getBoundingClientRect().height,
    input: document.querySelector("input[type=number]")!.getBoundingClientRect()
      .height,
    selection: getComputedStyle(document.body).userSelect,
  }));
  expect(heights.select).toBe(heights.input);
  expect(heights.selection).toBe("none");
});
test("desktop pages and rule modal fit both appearances and compact windows", async ({
  page,
}) => {
  for (const theme of ["light", "dark"]) {
    await rpc("ui.theme.set", { id: theme });
    await page.reload();
    await expect(
      page.getByText("Agent running", { exact: true }),
    ).toBeVisible();
    await page.evaluate(() => document.fonts.ready);
    for (const [title, name] of [
      ["Overview", "overview"],
      ["USB devices", "devices"],
      ["Monitors", "monitors"],
      ["Profiles & rules", "profiles"],
      ["Event log", "events"],
      ["Settings", "settings"],
    ]) {
      await nav(page, title);
      await page.screenshot({
        path: `test-results/desktop-${name}-${theme}.png`,
      });
      expect(
        await page.evaluate(
          () =>
            document.documentElement.scrollWidth <= innerWidth &&
            document.body.scrollHeight <= innerHeight,
        ),
      ).toBe(true);
    }
    await page.setViewportSize({ width: 760, height: 540 });
    const d = await editRule(page);
    await expect(
      d.getByRole("button", { name: "Save rule", exact: true }),
    ).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(d).not.toBeVisible();
    await page.setViewportSize({ width: 980, height: 700 });
  }
});
// README screenshots: the real interface on explicitly virtual test hardware,
// framed as a macOS window. Regenerate with:
//   ELARYDIL_SCREENSHOTS=1 npx playwright test -g "readme screenshots"
test.describe("readme screenshots", () => {
  test.use({ deviceScaleFactor: 2 });
  test.skip(!process.env.ELARYDIL_SCREENSHOTS, "set ELARYDIL_SCREENSHOTS=1");
  async function framed(
    page: import("@playwright/test").Page,
    name: string,
    dark = false,
  ) {
    await page.evaluate(() => document.fonts.ready);
    await page.waitForTimeout(250);
    const shot = (await page.screenshot()).toString("base64");
    const { width, height } = page.viewportSize()!;
    const frame = await page.context().newPage();
    await frame.setViewportSize({ width: width + 120, height: height + 150 });
    const bar = dark ? "#2f2b33" : "#f3f1f4";
    const title = dark ? "#e8e4ea" : "#2a2530";
    const edge = dark ? "rgba(255,255,255,.14)" : "rgba(0,0,0,.18)";
    await frame.setContent(`<!doctype html><html><body style="margin:0;background:transparent">
      <div id="shot" style="display:inline-block;padding:40px 60px 80px">
        <div style="width:${width}px;border-radius:12px;overflow:hidden;
          box-shadow:0 0 0 .5px ${edge},0 24px 60px rgba(0,0,0,.30),0 6px 18px rgba(0,0,0,.16)">
          <div style="position:relative;height:28px;background:${bar};
            border-bottom:.5px solid ${edge};display:flex;align-items:center;justify-content:center;
            font:600 13px -apple-system,BlinkMacSystemFont,'Helvetica Neue',sans-serif;color:${title}">
            <div style="position:absolute;left:12px;top:8px;display:flex;gap:8px">
              <span style="width:12px;height:12px;border-radius:50%;background:#ff5f57;box-shadow:inset 0 0 0 .5px rgba(0,0,0,.18)"></span>
              <span style="width:12px;height:12px;border-radius:50%;background:#febc2e;box-shadow:inset 0 0 0 .5px rgba(0,0,0,.18)"></span>
              <span style="width:12px;height:12px;border-radius:50%;background:#28c840;box-shadow:inset 0 0 0 .5px rgba(0,0,0,.18)"></span>
            </div>
            Elarydil KVM
          </div>
          <img src="data:image/png;base64,${shot}" width="${width}" height="${height}" style="display:block">
        </div>
      </div></body></html>`);
    await frame.locator("#shot").screenshot({
      path: resolve(`../../docs/screenshots/${name}.png`),
      omitBackground: true,
    });
    await frame.close();
  }
  test("readme screenshots", async ({ page }) => {
    test.setTimeout(120000);
    await rpc("monitors.input.alias.set", {
      id: "monitor_desk",
      value: 17,
      label: "Windows PC",
    });
    for (const theme of ["light", "dark"]) {
      await rpc("ui.theme.set", { id: theme });
      await page.reload();
      await expect(page.getByText("Agent running", { exact: true })).toBeVisible();
      await framed(page, theme === "dark" ? "overview-dark" : "overview", theme === "dark");
    }
    await rpc("ui.theme.set", { id: "light" });
    await page.reload();
    await expect(page.getByText("Agent running", { exact: true })).toBeVisible();
    await nav(page, "Profiles & rules");
    await expect(page.locator(".profile-panel")).toHaveCount(1);
    await framed(page, "profiles");
    await page.getByRole("button", { name: "Edit", exact: true }).first().click();
    await framed(page, "rule-editor");
    await page.keyboard.press("Escape");
    await nav(page, "Monitors");
    await expect(page.getByText("DDC available", { exact: true })).toBeVisible();
    await framed(page, "monitors");
    await nav(page, "USB devices");
    await framed(page, "devices");
    // Setup wizard: live capture, per-direction trigger selection, screens.
    await page.getByRole("button", { name: "Setup wizard", exact: true }).first().click();
    const d = page.getByRole("dialog");
    await d.getByRole("button", { name: "Start capture", exact: true }).click();
    await rpc("test.usb", { present: ["usb_1", "usb_2"] });
    await rpc("test.usb", { present: ["usb_2"] });
    await rpc("test.usb", { present: [] });
    await rpc("test.usb", { present: ["usb_0"] });
    await rpc("test.usb", { present: ["usb_0", "usb_1"] });
    await expect(d.locator(".capture-table tbody tr")).toHaveCount(3);
    await framed(page, "wizard-capture");
    await rpc("test.usb", { present: ["usb_0", "usb_1", "usb_2"] });
    await expect(d.getByLabel("Waiting for this event")).toHaveCount(0);
    await d.getByRole("button", { name: "Next: select devices", exact: true }).click();
    await framed(page, "wizard-devices");
    await d.getByRole("button", { name: "Choose screens", exact: true }).click();
    await framed(page, "wizard-screens");
    await page.keyboard.press("Escape");
    await rpc("ui.theme.set", { id: "dark" });
    await page.reload();
    await expect(page.getByText("Agent running", { exact: true })).toBeVisible();
    await nav(page, "Settings");
    await framed(page, "settings-dark", true);
  });
});
