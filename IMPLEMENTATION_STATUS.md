# Elarydil KVM 1.0 — features and platform support

Elarydil KVM keeps an external monitor's input in step with a USB KVM switch. A small
background agent watches USB devices and switches the monitor over DDC/CI; the
desktop app and the `elarydil-kvm` command line configure it.

## Platforms

| | macOS | Windows | Linux |
|---|---|---|---|
| Supported systems | macOS 13 or later on Apple Silicon | Windows 11 x64 | x86_64 with GTK 3 and WebKitGTK 4.1 |
| Download | DMG | Setup `.exe` (installs WebView2 if needed) | `.deb`, `.rpm` |
| Monitor control (DDC/CI) | IOAVService through the bundled m1ddc helper; displays selected by UUID | Windows physical-monitor DDC API, monitors identified by EDID | `/dev/i2c-*` through DRM connectors, monitors identified by EDID |
| USB monitoring | nusb hot-plug events | nusb hot-plug events | nusb hot-plug events |
| Start at login | LaunchAgent | Startup folder entry | systemd user service |
| Menu bar / tray | Menu bar icon with light and dark icons | Notification area icon | Tray icon through AppIndicator |
| Local IPC | Owner-only Unix socket with a token | Current-user named pipe with a token | Owner-only Unix socket with a token |
| Signing | Ad-hoc signed app and helpers; first launch steps in [MACOS_FIRST_LAUNCH.md](docs/MACOS_FIRST_LAUNCH.md) | Unsigned installer | Unsigned packages |

Intel Macs are not supported. The built-in display of a MacBook is never switched.

## Features

### Setup

- **Setup wizard** in four steps: capture a KVM switch away and back, choose trigger
  devices separately for the disconnect and connect rule, pick the monitor and input
  for each direction, done. It creates one active profile with two live rules.
- Capture shows each device the moment it disconnects or reconnects, with times
  relative to the first device; the first device and everything within 100 ms of it
  is preselected. Columns sort by phase automatically.
- Automatic switching is paused while the wizard is open (a renewable lease that
  expires on its own) and resumes from current device presence without replaying
  the captured changes.
- *Choose devices manually* skips the capture and lists every connected USB device.
- The same wizard runs in a terminal with `elarydil-kvm wizard start`, and scripts
  can drive it step by step with `wizard capture` / `wizard complete`.

### Rules and profiles

- Exactly one active profile; any number of inactive ones, with clone, rename,
  import and export. Duplicate names get a numbered suffix.
- Rules trigger on **disconnect** or **connect** of **all**, **any** or **at least N**
  selected devices. Devices are matched by serial number, VID/PID and USB path, with
  alternatives; ambiguous matches block a rule instead of guessing.
- Timing per rule: debounce, re-arm, cooldown and USB event grouping, plus a global
  minimum interval between writes to one monitor and startup/resume grace periods.
  Every setting shows its valid range and explanation.
- **Test** performs a rule's real switch for 15 seconds and returns automatically.
  **Dry run** evaluates a rule without writing to the monitor.
- Custom shell commands per rule on connect and disconnect, with shell, working
  directory and timeout; they run in the agent even with the app closed. See
  [COMMAND_HOOKS.md](docs/COMMAND_HOOKS.md).

### Monitors and inputs

- Standard inputs (DisplayPort 1/2, HDMI 1/2) are selectable immediately; custom
  values accept decimal or `0x` hex. Inputs can be renamed and shown as
  `HDMI 1 (MacBook)`, hidden, or added.
- The current input is read live; inputs confirmed by a read-back are marked with a
  green dot.
- **Test input** switches for 15 seconds and returns, with *Switch back now*. The
  agent owns the return timer, persists it and recovers it after a restart.
- Monitors are matched exactly by their saved identity; a missing or duplicated
  monitor blocks the action rather than switching the wrong screen.

### Agent and lifecycle

- The agent starts and stops with the app; closing the window keeps the app in the
  menu bar or tray. *Keep agent running when the app quits* makes it independent,
  with its own login start.
- After installation the app waits for macOS to clear the new agent. If the agent
  cannot start, a window explains why, and every failed start is recorded in
  `/tmp/elarydil-kvm-<uid>/agent-start.log` (`%TEMP%\elarydil-kvm\agent-start.log`).
- Sleep and resume are detected from gaps in the watcher; the agent re-reads
  devices and applies the resume grace period instead of acting on stale changes.

### Command line and headless use

- Every app setting and action has a CLI equivalent
  ([GUI_CLI_PARITY.md](docs/GUI_CLI_PARITY.md)); the CLI and agent need no WebView,
  so servers and headless machines can use the CLI-only bundle.
- JSON output with a stable envelope, documented exit codes and NDJSON live streams.
  The [complete reference](docs/CLI_REFERENCE.md) is generated from the CLI's help.

### Safety, privacy and diagnostics

- All processing is local: no account, no network listener, no telemetry, no
  keyboard hooks. IPC is authenticated and limited to the current user.
- Configuration is validated against a schema, written atomically with a backup,
  and locked against concurrent writers ([CONFIG_SCHEMA.md](docs/CONFIG_SCHEMA.md)).
- A per-monitor write limiter is shared by rules and manual switching; a write in
  progress is never duplicated by a second rule.
- Helper processes are bounded by timeouts; timed-out command trees are terminated.
- `doctor`, sanitized diagnostic export (paths, serials and command text redacted),
  an event journal and a live log are built in.

### Interface

- Light and dark appearances that follow the system, offline Geist Mono font, native
  window decorations, help popovers for every setting, tooltips and icons on every
  button, native save dialogs for exports.

## Quality

Every release is built and tested by GitHub Actions:

| Suite | What it covers |
|---|---|
| 52 Rust tests | Rule engine (group masks, KVM cycles, timing), device and monitor matching, write limiter, wizard, setup hold, agent ownership, command hooks, configuration safety |
| 8 CLI tests | Agent start/stop, authenticated IPC, concurrent clients, validation errors, persistence across restarts |
| 16 UI tests | Every page in both appearances and compact windows, wizard paths, rule editor, input tests, settings, driven against a virtual hardware backend |
| String catalog checks | Every user-facing string comes from the English catalog |

Pull requests run the Linux suites; every push to `main` adds the Windows suites, and
a new version builds and publishes the Windows, macOS and Linux installers.

## Known limitations

- **DDC/CI must reach the monitor.** Enable DDC/CI in the monitor's menu. Some docks,
  adapters and KVMs that carry video block DDC; the monitor's input button then
  remains the way back. A monitor reports success when it accepts a command, which
  is not proof that the picture changed.
- **macOS and Windows builds are not notarized or code-signed** with a commercial
  certificate, so the first launch needs the steps in
  [MACOS_FIRST_LAUNCH.md](docs/MACOS_FIRST_LAUNCH.md) or an unknown-publisher
  confirmation on Windows.
- **Linux** needs read/write access to `/dev/i2c-*`; an optional udev rule is in
  `packages/60-elarydil-kvm-i2c.rules`. Start at login requires a systemd user session.
- Configuration written by a newer version cannot be read by an older one; the app
  explains this instead of starting.
- Available from the CLI and configuration files but not yet in the app: advanced
  device selector editing, ordering several monitor actions in one rule, and
  merging configurations between computers.
- Not included: OS notifications, an input submenu in the menu-bar menu, and
  automatic updates.
