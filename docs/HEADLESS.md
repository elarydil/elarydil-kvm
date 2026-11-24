# Headless operation

Install the CLI bundle containing `elarydil-kvm`, `elarydil-kvm-agent`,
`elarydil-kvm-ddc`, and `elarydil-kvm-command` together. Apple Silicon bundles also
contain `m1ddc`. Keep these executables together; no desktop executable is needed.

```sh
elarydil-kvm agent start
elarydil-kvm status --json
elarydil-kvm wizard start --cycles 2
elarydil-kvm startup enable --component agent
elarydil-kvm startup disable --component gui
```

The agent is a separate per-user process with no window or tray. USB monitoring,
monitor switching, custom shell commands, logs and authenticated local IPC all live
in the agent.

## Lifecycle

- **Started by the desktop app** (the default): the agent stops when the app quits
  or crashes. With the default close action, closing the window only hides the app
  to the menu bar or tray, so switching continues.
- **Started from a terminal or at login** (`agent start`, `startup enable --component
  agent`): the agent is standalone and keeps running until `agent stop`.
- **Desktop users who want an always-on agent** turn on Settings → *Keep agent
  running when the app quits* (`config set app.agentStandalone true`). Only then does
  the app offer *Start agent at login*.

`elarydil-kvm status --json` reports `"standalone": true` when the agent will outlive
the app. To stop a standalone agent and remove its login start:

```sh
elarydil-kvm agent stop
elarydil-kvm startup disable --component agent
```

On macOS the CLI is inside the app: `"/Applications/Elarydil KVM.app/Contents/Resources/bin/elarydil-kvm"`.

## Login startup

Login registration uses Windows Startup, macOS LaunchAgents,
or systemd --user on Linux. Login registration is not a system-wide boot service;
it starts in the signed-in user's session and requires no administrator account.
Do not start a second system service against the same configuration directory.

Interactive `wizard start` guides setup in a terminal. Scripts use `--json` or
`--non-interactive` and the `wizard capture` / `wizard apply` operations, or the
manual setup in [CLI guide](CLI.md). All UI settings can be configured through
CLI commands or validated `config import`.

```sh
elarydil-kvm config export settings.json
elarydil-kvm config import settings.json --dry-run
elarydil-kvm config import settings.json
elarydil-kvm agent pause
elarydil-kvm agent resume
elarydil-kvm logs --follow --duration 60
elarydil-kvm agent stop
```

Configuration operations also work while the agent is stopped. Hardware reads,
manual command execution, and live status require the agent. `agent stop` cancels
active and queued commands. `agent restart` reloads configuration with a fresh USB
baseline. `agent disable` removes login registration; it does not stop a running agent.
Use `gui close` to close the installed desktop interface and tray, and `gui open`
to reopen it. An agent started by the app stops with it unless it is standalone;
the `gui close` result reports this as `agentContinues`.

`--home PATH` or `ELARYDIL_KVM_HOME` selects a separate private configuration root.
Default roots follow platform per-user configuration directories. The agent uses an
owner-only Unix socket or a current-user Windows named pipe and never opens a TCP
listener. Full exports and event logs may contain private command output.

Use `elarydil-kvm --help`, group help such as `elarydil-kvm rules --help`, and each
command's `--help`. [Complete reference](CLI_REFERENCE.md) is generated from those
actual help screens. [Troubleshooting](TROUBLESHOOTING.md) covers USB/DDC limitations.
