# Elarydil KVM — complete CLI reference

Generated from the compiled CLI by `python packages/generate-cli-docs.py`.
See [setup guide](CLI.md), [headless agent](HEADLESS.md), and [custom commands](COMMAND_HOOKS.md).

## elarydil-kvm

```text
Elarydil KVM — USB-driven monitor switching, with a separate headless agent.

Usage: elarydil-kvm [OPTIONS] <COMMAND>

Commands:
  agent        Agent. See docs/CLI.md.
  commands     Manage commands. Choose a subcommand for its options and examples.
  completion   Print shell completion for bash, zsh, fish, powershell, or elvish.
  config       Config. See docs/CLI.md.
  devices      Devices. See docs/CLI.md.
  diagnostics  Export local sanitized diagnostics.
  doctor       Inspect hardware backends and permissions; optionally save a sanitized diagnostic report.
  events       Events. See docs/CLI.md.
  gui          Gui. See docs/CLI.md.
  logs         Read the private event journal or stream versioned JSON events with --follow.
  monitors     Monitors. See docs/CLI.md.
  profiles     Profiles. See docs/CLI.md.
  rules        Rules. See docs/CLI.md.
  startup      Startup. See docs/CLI.md.
  status       Show agent health, USB availability, uptime, and headless operation status.
  ui           Ui. See docs/CLI.md.
  wizard       Wizard. See docs/CLI.md.
  help         Print this message or the help of the given subcommand(s)

Options:
      --json         Print a versioned JSON response
      --home <home>  Override the private per-user configuration directory
  -h, --help         Print help
  -V, --version      Print version
```

## status

```text
Show agent health, USB availability, uptime, and headless operation status.

Usage: elarydil-kvm status [OPTIONS]

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm status

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## doctor

```text
Inspect hardware backends and permissions; optionally save a sanitized diagnostic report.

Usage: elarydil-kvm doctor [OPTIONS]

Options:
      --export <export>
          Export

      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm doctor

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## diagnostics export

```text
Export diagnostics with hardware identities, shell commands, and command output redacted.

Usage: elarydil-kvm diagnostics export [OPTIONS]

Options:
      --export <export>
          Export

      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm diagnostics export

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## logs

```text
Read the private event journal or stream versioned JSON events with --follow.

Usage: elarydil-kvm logs [OPTIONS]

Options:
      --json
          Print a versioned JSON response

      --profile <profile>
          Profile

      --home <home>
          Override the private per-user configuration directory

      --limit <limit>
          Limit

      --since <since>
          Since

      --level <level>
          Level

      --duration <duration>
          Duration

      --follow
          Follow

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm logs

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## events list

```text
List recent agent, USB, rule, monitor, and command events, newest first.

Usage: elarydil-kvm events list [OPTIONS]

Options:
      --json
          Print a versioned JSON response

      --profile <profile>
          Profile

      --home <home>
          Override the private per-user configuration directory

      --limit <limit>
          Limit

      --since <since>
          Since

      --level <level>
          Level

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm events list

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## events clear

```text
Clear the local event journal after explicit confirmation.

Usage: elarydil-kvm events clear [OPTIONS]

Options:
      --confirm
          Confirm

      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm events clear --confirm

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## agent start

```text
Start the independent background agent. A running older agent is restarted when its API lacks required features.

Usage: elarydil-kvm agent start [OPTIONS]

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm agent start

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## agent stop

```text
Stop the agent and attempt pending input returns; --wait waits for the instance lock to be released.

Usage: elarydil-kvm agent stop [OPTIONS]

Options:
      --json
          Print a versioned JSON response

      --wait
          Wait for completion and return the command exit code.

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm agent stop

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## agent restart

```text
Restart the agent and reload saved configuration.

Usage: elarydil-kvm agent restart [OPTIONS]

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm agent restart

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## agent status

```text
Show whether the agent is running, paused, or degraded.

Usage: elarydil-kvm agent status [OPTIONS]

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm agent status

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## agent enable

```text
Register the agent to start at login.

Usage: elarydil-kvm agent enable [OPTIONS]

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm agent enable

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## agent disable

```text
Remove agent login registration.

Usage: elarydil-kvm agent disable [OPTIONS]

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm agent disable

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## agent pause

```text
Pause automation and cancel pending custom commands.

Usage: elarydil-kvm agent pause [OPTIONS]

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm agent pause

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## agent resume

```text
Resume automation with a fresh USB baseline and startup grace period.

Usage: elarydil-kvm agent resume [OPTIONS]

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm agent resume

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## startup status

```text
Inspect OS login registration for the agent or GUI.

Usage: elarydil-kvm startup status [OPTIONS]

Options:
      --component <component>
          Component

      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm startup status --component agent

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## startup enable

```text
Register the agent or GUI at login for the current user.

Usage: elarydil-kvm startup enable [OPTIONS]

Options:
      --component <component>
          Component

      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm startup enable --component agent

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## startup disable

```text
Remove current-user login registration for the agent or GUI.

Usage: elarydil-kvm startup disable [OPTIONS]

Options:
      --component <component>
          Component

      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm startup disable --component agent

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## startup set

```text
Configure agent and GUI login independently, plus the agent startup delay.

Usage: elarydil-kvm startup set [OPTIONS]

Options:
      --agent <agent>
          Agent

      --json
          Print a versioned JSON response

      --gui <gui>
          Gui

      --home <home>
          Override the private per-user configuration directory

      --delay-ms <delay-ms>
          Delay ms

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm startup set --agent on --gui off --delay-ms 2000

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## gui open

```text
Open the desktop window using the separately installed GUI executable.

Usage: elarydil-kvm gui open [OPTIONS]

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm gui open

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## gui close

```text
Close the desktop interface and tray with acknowledgement. An agent started by the app stops with it unless app.agentStandalone is on; agents started from a terminal or at login keep running.

Usage: elarydil-kvm gui close [OPTIONS]

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm gui close

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## ui mode get

```text
Read the saved desktop mode.

Usage: elarydil-kvm ui mode get [OPTIONS]

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm ui mode get

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## ui mode set

```text
Set display mode: both, window-only, tray-only, menu-bar-only, or headless. Use agent start for a completely invisible daemon.

Usage: elarydil-kvm ui mode set [OPTIONS] [VALUE]

Arguments:
  [VALUE]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm ui mode set headless

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## ui theme get

```text
Read the saved desktop theme.

Usage: elarydil-kvm ui theme get [OPTIONS]

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm ui theme get

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## ui theme set

```text
Set theme to system, light, or dark.

Usage: elarydil-kvm ui theme set [OPTIONS] [VALUE]

Arguments:
  [VALUE]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm ui theme set dark

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## devices list

```text
List live USB models, manufacturers, device types, and tracked selectors.

Usage: elarydil-kvm devices list [OPTIONS]

Options:
      --connected
          Connected

      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

      --tracked
          Tracked

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm devices list

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## devices show

```text
Show USB descriptors and current matching information.

Usage: elarydil-kvm devices show [OPTIONS] [DEVICE]

Arguments:
  [DEVICE]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm devices show <device-id>

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## devices watch

```text
Stream changes to the cached USB inventory as NDJSON; default duration is 30 seconds.

Usage: elarydil-kvm devices watch [OPTIONS]

Options:
      --duration <duration>
          Duration

      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm devices watch

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## devices rescan

```text
Refresh the USB baseline without triggering switching or custom commands.

Usage: elarydil-kvm devices rescan [OPTIONS]

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm devices rescan

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## devices track

```text
Save a live USB device with a stable selector and optional friendly name.

Usage: elarydil-kvm devices track [OPTIONS] [DEVICE]

Arguments:
  [DEVICE]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --name <name>
          Name

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm devices track <device-id>

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## devices alias

```text
Change the friendly name of a tracked USB device.

Usage: elarydil-kvm devices alias [OPTIONS] [DEVICE]

Arguments:
  [DEVICE]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --name <name>
          Name

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm devices alias <device-id> --name "KVM keyboard"

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## devices selectors show

```text
Show alternative selectors for a tracked USB device.

Usage: elarydil-kvm devices selectors show [OPTIONS] [DEVICE]

Arguments:
  [DEVICE]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm devices selectors show <device-id>

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## devices selectors add

```text
Add a serial, instance, topology, or vid-pid selector. VID/PID use hexadecimal values.

Usage: elarydil-kvm devices selectors add [OPTIONS] [DEVICE]

Arguments:
  [DEVICE]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --vid <vid>
          Vid

      --home <home>
          Override the private per-user configuration directory

      --pid <pid>
          Pid

      --serial <serial>
          Serial

      --path <path>
          Path

      --match <match>
          Match

      --allow-ambiguous
          Allow ambiguous

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm devices selectors add <device-id> --match vid-pid --vid 046D --pid C52B

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## devices selectors remove

```text
Remove one alternative selector; configuration must retain a valid device definition.

Usage: elarydil-kvm devices selectors remove [OPTIONS] [DEVICE]

Arguments:
  [DEVICE]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --selector <selector>
          Selector

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm devices selectors remove <device-id> --selector <selector-id>

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## wizard start

```text
Capture USB switching cycles. In a terminal, wizard start guides setup; scripts can use --non-interactive.

Usage: elarydil-kvm wizard start [OPTIONS]

Options:
      --cycles <cycles>
          Cycles

      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

      --yes
          Yes

      --non-interactive
          Non interactive

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm wizard start --cycles 2

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## wizard capture start

```text
Capture USB switching cycles. In a terminal, wizard start guides setup; scripts can use --non-interactive.

Usage: elarydil-kvm wizard capture start [OPTIONS]

Options:
      --cycles <cycles>
          Cycles

      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

      --yes
          Yes

      --non-interactive
          Non interactive

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm wizard capture start --cycles 2

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## wizard capture status

```text
Show the retained USB capture session and candidates.

Usage: elarydil-kvm wizard capture status [OPTIONS]

Options:
      --json
          Print a versioned JSON response

      --session <session>
          Session

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm wizard capture status

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## wizard capture finish

```text
Stop recording and return the USB device candidates.

Usage: elarydil-kvm wizard capture finish [OPTIONS]

Options:
      --json
          Print a versioned JSON response

      --session <session>
          Session

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm wizard capture finish

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## wizard candidates

```text
List candidate USB devices and their matched switching cycles.

Usage: elarydil-kvm wizard candidates [OPTIONS]

Options:
      --json
          Print a versioned JSON response

      --session <session>
          Session

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm wizard candidates

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## wizard apply

```text
Save selected USB candidates and create a validated monitor switching rule.

Usage: elarydil-kvm wizard apply [OPTIONS]

Options:
      --json
          Print a versioned JSON response

      --name <name>
          Name

      --devices <devices>
          Devices

      --home <home>
          Override the private per-user configuration directory

      --mode <mode>
          Mode

      --min-count <min-count>
          Min count

      --on <on>
          On

      --monitor <monitor>
          Monitor

      --input <input>
          Input

      --vcp <vcp>
          Vcp

      --profile <profile>
          Profile

      --session <session>
          Session

      --debounce-ms <debounce-ms>
          Debounce ms

      --rearm-ms <rearm-ms>
          Rearm ms

      --cooldown-ms <cooldown-ms>
          Cooldown ms

      --coincidence-window-ms <coincidence-window-ms>
          Coincidence window ms

      --dry-run
          Dry run

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm wizard apply --devices <device-a>,<device-b> --mode all --monitor <monitor-id> --input 18 --dry-run

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## wizard cancel

```text
Stop the current capture without creating a switching rule.

Usage: elarydil-kvm wizard cancel [OPTIONS]

Options:
      --json
          Print a versioned JSON response

      --session <session>
          Session

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm wizard cancel

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## monitors state

```text
Read cached DDC availability, current input and reversible test state; --watch renews live reads.

Usage: elarydil-kvm monitors state [OPTIONS] [MONITOR]

Arguments:
  [MONITOR]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --watch
          Renew the cached monitor live-read subscription.

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm monitors state <monitor-id> --watch

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## monitors input restore

```text
Return an active input test to its recorded original input immediately.

Usage: elarydil-kvm monitors input restore [OPTIONS] [MONITOR]

Arguments:
  [MONITOR]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm monitors input restore <monitor-id>

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## monitors input watch

```text
Stream cached input state every 300 ms without overlapping hardware reads.

Usage: elarydil-kvm monitors input watch [OPTIONS] [MONITOR]

Arguments:
  [MONITOR]
          Item ID or file path

Options:
      --duration <duration>
          Duration

      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm monitors input watch <monitor-id> --duration 30

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## profiles activate

```text
Activate one profile and disable other profiles.

Usage: elarydil-kvm profiles activate [OPTIONS] [PROFILE]

Arguments:
  [PROFILE]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm profiles activate <profile-id>

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## wizard complete

```text
Atomically create an active profile and connect/disconnect rules from a capture. Input tests are optional.

Usage: elarydil-kvm wizard complete [OPTIONS]

Options:
      --connect-monitor <connect-monitor>
          Target monitor ID when the USB group connects.

      --json
          Print a versioned JSON response

      --disconnect-monitor <disconnect-monitor>
          Target monitor ID when the USB group disconnects.

      --home <home>
          Override the private per-user configuration directory

      --connect-input <connect-input>
          Input value when the USB group connects.

      --disconnect-input <disconnect-input>
          Input value when the USB group disconnects.

      --name <name>
          Name

      --devices <devices>
          Devices

      --connect-devices <connect-devices>
          Comma-separated device IDs that must connect to trigger the connect rule. Defaults to --devices.

      --disconnect-devices <disconnect-devices>
          Comma-separated device IDs that must disconnect to trigger the disconnect rule. Defaults to --devices.

      --mode <mode>
          Mode

      --min-count <min-count>
          Min count

      --session <session>
          Session

      --dry-run
          Dry run

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm wizard complete --devices <device-id> --connect-monitor <monitor-id> --connect-input 17 --disconnect-monitor <monitor-id> --disconnect-input 18

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## wizard watch

```text
Stream live capture observations with disconnect/connect timestamps as they arrive.

Usage: elarydil-kvm wizard watch [OPTIONS]

Options:
      --duration <duration>
          Duration

      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm wizard watch --duration 30

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## monitors list

```text
Enumerate current displays, hardware identities, and DDC backends. No DDC writes are made.

Usage: elarydil-kvm monitors list [OPTIONS]

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm monitors list

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## monitors show

```text
Show a saved monitor and its identity.

Usage: elarydil-kvm monitors show [OPTIONS] [MONITOR]

Arguments:
  [MONITOR]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm monitors show <monitor-id>

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## monitors detect

```text
Enumerate current displays, hardware identities, and DDC backends. No DDC writes are made.

Usage: elarydil-kvm monitors detect [OPTIONS]

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm monitors detect

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## monitors remember

```text
Save an unambiguous live monitor for future switching.

Usage: elarydil-kvm monitors remember [OPTIONS] [MONITOR]

Arguments:
  [MONITOR]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --name <name>
          Name

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm monitors remember <monitor-id>

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## monitors capabilities

```text
Read monitor DDC capabilities and supported input source values.

Usage: elarydil-kvm monitors capabilities [OPTIONS] [MONITOR]

Arguments:
  [MONITOR]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm monitors capabilities <monitor-id>

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## monitors inputs

```text
List saved input aliases and standard, unverified source values.

Usage: elarydil-kvm monitors inputs [OPTIONS] [MONITOR]

Arguments:
  [MONITOR]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm monitors inputs <monitor-id>

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## monitors input get

```text
Read the current VCP input value; this is a separate read, not proof of an earlier write.

Usage: elarydil-kvm monitors input get [OPTIONS] [MONITOR]

Arguments:
  [MONITOR]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --vcp <vcp>
          Vcp

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm monitors input get <monitor-id>

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## monitors input set

```text
Submit a monitor input change. Requires --confirm and a saved input mapping; --force also bypasses the cooldown.

Usage: elarydil-kvm monitors input set [OPTIONS] [MONITOR]

Arguments:
  [MONITOR]
          Item ID or file path

Options:
      --input <input>
          Input

      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

      --vcp <vcp>
          Vcp

      --value <value>
          Value

      --confirm
          Confirm

      --force
          Force

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm monitors input set <monitor-id> --value 18 --confirm

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## monitors input test

```text
Switch to an input after confirmation and return automatically after 15 seconds. The agent owns the timer; use input restore for an early return.

Usage: elarydil-kvm monitors input test [OPTIONS] [MONITOR]

Arguments:
  [MONITOR]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --restore-ms <restore-ms>
          Return to the original input after this many milliseconds (default 15000).

      --home <home>
          Override the private per-user configuration directory

      --input <input>
          Input

      --vcp <vcp>
          Vcp

      --value <value>
          Value

      --confirm
          Confirm

      --force
          Force

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm monitors input test <monitor-id> --value 18 --confirm

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## monitors input alias set

```text
Save a friendly monitor input mapping. Nonstandard VCP codes require --confirm.

Usage: elarydil-kvm monitors input alias set [OPTIONS] [MONITOR]

Arguments:
  [MONITOR]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --vcp <vcp>
          Vcp

      --home <home>
          Override the private per-user configuration directory

      --value <value>
          Value

      --label <label>
          Label

      --confirm
          Confirm

      --verified
          Verified

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm monitors input alias set <monitor-id> --value 18 --label "Work MacBook"

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## monitors input alias remove

```text
Remove one saved input mapping.

Usage: elarydil-kvm monitors input alias remove [OPTIONS] [MONITOR]

Arguments:
  [MONITOR]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --value <value>
          Value

      --home <home>
          Override the private per-user configuration directory

      --reset
          Reset a standard input name while retaining the port and its verification.

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm monitors input alias remove <monitor-id> --value 18

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## profiles list

```text
List profiles and their enabled rules.

Usage: elarydil-kvm profiles list [OPTIONS]

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm profiles list

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## profiles create

```text
Create an empty switching profile.

Usage: elarydil-kvm profiles create [OPTIONS]

Options:
      --json
          Print a versioned JSON response

      --name <name>
          Name

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm profiles create --name "Desk KVM"

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## profiles show

```text
Show a saved profile and every rule.

Usage: elarydil-kvm profiles show [OPTIONS] [PROFILE]

Arguments:
  [PROFILE]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm profiles show <profile-id>

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## profiles rename

```text
Change a profile name.

Usage: elarydil-kvm profiles rename [OPTIONS] [PROFILE]

Arguments:
  [PROFILE]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --name <name>
          Name

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm profiles rename <profile-id> --name "Desk"

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## profiles clone

```text
Copy a profile with new IDs; the copy starts disabled to prevent duplicate commands.

Usage: elarydil-kvm profiles clone [OPTIONS] [PROFILE]

Arguments:
  [PROFILE]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --name <name>
          Name

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm profiles clone <profile-id> --name "Desk copy"

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## profiles enable

```text
Enable all eligible rules in a profile.

Usage: elarydil-kvm profiles enable [OPTIONS] [PROFILE]

Arguments:
  [PROFILE]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm profiles enable <profile-id>

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## profiles disable

```text
Disable a profile and cancel pending commands on configuration reload.

Usage: elarydil-kvm profiles disable [OPTIONS] [PROFILE]

Arguments:
  [PROFILE]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm profiles disable <profile-id>

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## profiles delete

```text
Delete a profile and its rules after confirmation.

Usage: elarydil-kvm profiles delete [OPTIONS] [PROFILE]

Arguments:
  [PROFILE]
          Item ID or file path

Options:
      --confirm
          Confirm

      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm profiles delete <profile-id> --confirm

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## profiles export

```text
Write one profile to a JSON file. Includes private custom command text.

Usage: elarydil-kvm profiles export [OPTIONS] [PROFILE]

Arguments:
  [PROFILE]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --output <output>
          Output

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm profiles export <profile-id> --output profile.json

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## profiles import

```text
Import a validated profile with new IDs. Hardware references must already exist on this host.

Usage: elarydil-kvm profiles import [OPTIONS] [PATH]

Arguments:
  [PATH]
          Item ID or file path

Options:
      --dry-run
          Dry run

      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm profiles import profile.json --dry-run

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## commands list

```text
List queued, started, finished, failed, cancelled, skipped, and dry-run command events. Output is private.

Usage: elarydil-kvm commands list [OPTIONS]

Options:
      --json
          Print a versioned JSON response

      --limit <limit>
          Limit

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm commands list --limit 20

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## commands show

```text
Show the newest state of one command job, including bounded output or a timeout/cancellation error.

Usage: elarydil-kvm commands show [OPTIONS] [JOB]

Arguments:
  [JOB]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm commands show <job-id>

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## rules commands get

```text
Read both saved shell commands, enabled state, shell, directory, and timeout.

Usage: elarydil-kvm rules commands get [OPTIONS] [RULE]

Arguments:
  [RULE]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm rules commands get <rule-id>

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## rules commands set

```text
Save commands without enabling automatic execution. Omitted fields retain their values; use an empty string to clear a command.

Usage: elarydil-kvm rules commands set [OPTIONS] [RULE]

Arguments:
  [RULE]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --on-connect <on-connect>
          Shell command when this computer receives the KVM.

      --home <home>
          Override the private per-user configuration directory

      --on-disconnect <on-disconnect>
          Shell command when the KVM leaves this computer.

      --shell <shell>
          Shell: system, bash, powershell, or cmd.

      --working-dir <working-dir>
          Working directory; empty uses the user home.

      --timeout-ms <timeout-ms>
          Command timeout in milliseconds (100–300000).

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm rules commands set <rule-id> --on-connect 'echo connected' --on-disconnect 'echo disconnected' --shell system

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## rules commands enable

```text
Enable automatic commands for both KVM directions. The profile, rule, and dry-run settings still apply.

Usage: elarydil-kvm rules commands enable [OPTIONS] [RULE]

Arguments:
  [RULE]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm rules commands enable <rule-id>

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## rules commands disable

```text
Disable automatic custom commands and cancel pending command jobs.

Usage: elarydil-kvm rules commands disable [OPTIONS] [RULE]

Arguments:
  [RULE]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm rules commands disable <rule-id>

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## rules commands run

```text
Run a saved command immediately for testing. Requires --confirm; bypasses rule enable/dry-run settings. --wait returns its exit code.

Usage: elarydil-kvm rules commands run [OPTIONS] [RULE]

Arguments:
  [RULE]
          Item ID or file path

Options:
      --event <event>
          Event

      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

      --wait
          Wait for completion and return the command exit code.

      --confirm
          Confirm

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm rules commands run <rule-id> --event connect --confirm --wait

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## rules list

```text
List rules, optionally restricted to a profile.

Usage: elarydil-kvm rules list [OPTIONS]

Options:
      --json
          Print a versioned JSON response

      --profile <profile>
          Profile

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm rules list

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## rules create

```text
Create a monitor switching rule. Start with --dry-run; use saved USB and monitor IDs.

Usage: elarydil-kvm rules create [OPTIONS]

Options:
      --json
          Print a versioned JSON response

      --name <name>
          Name

      --devices <devices>
          Devices

      --home <home>
          Override the private per-user configuration directory

      --mode <mode>
          Mode

      --min-count <min-count>
          Min count

      --on <on>
          On

      --monitor <monitor>
          Monitor

      --input <input>
          Input

      --vcp <vcp>
          Vcp

      --profile <profile>
          Profile

      --debounce-ms <debounce-ms>
          Debounce ms

      --rearm-ms <rearm-ms>
          Rearm ms

      --cooldown-ms <cooldown-ms>
          Cooldown ms

      --coincidence-window-ms <coincidence-window-ms>
          Coincidence window ms

      --dry-run
          Dry run

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm rules create --profile <profile-id> --name "Switch away" --devices <device-id> --mode all --on disconnect --monitor <monitor-id> --input 18 --dry-run

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## rules show

```text
Show a rule, including USB grouping, actions, timing, dry run, and custom commands.

Usage: elarydil-kvm rules show [OPTIONS] [RULE]

Arguments:
  [RULE]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm rules show <rule-id>

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## rules set

```text
Change only supplied rule fields; existing custom commands are preserved.

Usage: elarydil-kvm rules set [OPTIONS] [RULE]

Arguments:
  [RULE]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --name <name>
          Name

      --devices <devices>
          Devices

      --home <home>
          Override the private per-user configuration directory

      --mode <mode>
          Mode

      --min-count <min-count>
          Min count

      --on <on>
          On

      --monitor <monitor>
          Monitor

      --input <input>
          Input

      --vcp <vcp>
          Vcp

      --debounce-ms <debounce-ms>
          Debounce ms

      --rearm-ms <rearm-ms>
          Rearm ms

      --cooldown-ms <cooldown-ms>
          Cooldown ms

      --coincidence-window-ms <coincidence-window-ms>
          Coincidence window ms

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm rules set <rule-id> --mode all

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## rules timing

```text
Set debounce, rearm, cooldown, or temporal coincidence in milliseconds. A coincidence window of zero disables it.

Usage: elarydil-kvm rules timing [OPTIONS] [RULE]

Arguments:
  [RULE]
          Item ID or file path

Options:
      --debounce-ms <debounce-ms>
          Debounce ms

      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

      --rearm-ms <rearm-ms>
          Rearm ms

      --cooldown-ms <cooldown-ms>
          Cooldown ms

      --coincidence-window-ms <coincidence-window-ms>
          Coincidence window ms

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm rules timing <rule-id> --debounce-ms 400 --rearm-ms 500 --cooldown-ms 5000

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## rules add-device

```text
Add one tracked USB device to the group.

Usage: elarydil-kvm rules add-device [OPTIONS] [RULE] [VALUE]

Arguments:
  [RULE]
          Item ID or file path

  [VALUE]
          New value (JSON for config set).

Options:
      --json
          Print a versioned JSON response

      --value <value>
          Value

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm rules add-device <rule-id> <device-id>

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## rules remove-device

```text
Remove one USB device from the group.

Usage: elarydil-kvm rules remove-device [OPTIONS] [RULE] [VALUE]

Arguments:
  [RULE]
          Item ID or file path

  [VALUE]
          New value (JSON for config set).

Options:
      --json
          Print a versioned JSON response

      --value <value>
          Value

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm rules remove-device <rule-id> <device-id>

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## rules add-action

```text
Add a monitor input action.

Usage: elarydil-kvm rules add-action [OPTIONS] [RULE]

Arguments:
  [RULE]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --monitor <monitor>
          Monitor

      --home <home>
          Override the private per-user configuration directory

      --input <input>
          Input

      --vcp <vcp>
          Vcp

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm rules add-action <rule-id> --monitor <monitor-id> --input 18

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## rules remove-action

```text
Remove one monitor input action.

Usage: elarydil-kvm rules remove-action [OPTIONS] [RULE] [VALUE]

Arguments:
  [RULE]
          Item ID or file path

  [VALUE]
          New value (JSON for config set).

Options:
      --json
          Print a versioned JSON response

      --value <value>
          Value

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm rules remove-action <rule-id> <action-id>

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## rules enable

```text
Enable a rule. Profile and command settings still apply.

Usage: elarydil-kvm rules enable [OPTIONS] [RULE]

Arguments:
  [RULE]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm rules enable <rule-id>

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## rules disable

```text
Disable a rule and cancel pending commands on configuration reload.

Usage: elarydil-kvm rules disable [OPTIONS] [RULE]

Arguments:
  [RULE]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm rules disable <rule-id>

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## rules simulate

```text
Simulate USB matching without writing to monitors or executing shell commands.

Usage: elarydil-kvm rules simulate [OPTIONS] [RULE]

Arguments:
  [RULE]
          Item ID or file path

Options:
      --event <event>
          Event

      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm rules simulate <rule-id> --event disconnect

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## rules dry-run

```text
Set on or off. Dry run logs automatic monitor actions and shell commands without executing them.

Usage: elarydil-kvm rules dry-run [OPTIONS] [RULE] [VALUE]

Arguments:
  [RULE]
          Item ID or file path

  [VALUE]
          New value (JSON for config set).

Options:
      --json
          Print a versioned JSON response

      --value <value>
          Value

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm rules dry-run <rule-id> on

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## rules delete

```text
Delete a rule after confirmation.

Usage: elarydil-kvm rules delete [OPTIONS] [RULE]

Arguments:
  [RULE]
          Item ID or file path

Options:
      --confirm
          Confirm

      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm rules delete <rule-id> --confirm

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## config show

```text
Show the full private configuration, including shell commands.

Usage: elarydil-kvm config show [OPTIONS]

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm config show

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## config get

```text
Read an existing setting using its dotted JSON path.

Usage: elarydil-kvm config get [OPTIONS] [KEY]

Arguments:
  [KEY]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm config get app.theme

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## config set

```text
Set an existing dotted setting to a JSON value. Prefer dedicated UI/startup/rules commands for simpler quoting.

Usage: elarydil-kvm config set [OPTIONS] [KEY] [VALUE]

Arguments:
  [KEY]
          Item ID or file path

  [VALUE]
          New value (JSON for config set).

Options:
      --json
          Print a versioned JSON response

      --value <value>
          Value

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm config set safety.globalMonitorCooldownMs 5000

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## config edit

```text
Open a temporary configuration in the EDITOR executable, validate it, and save atomically. Requires a terminal.

Usage: elarydil-kvm config edit [OPTIONS]

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm config edit

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## config validate

```text
Validate the current configuration, or a supplied JSON file, without activation.

Usage: elarydil-kvm config validate [OPTIONS] [PATH]

Arguments:
  [PATH]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm config validate config.json

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## config doctor

```text
Inspect current configuration, USB availability, monitor backends, and permissions.

Usage: elarydil-kvm config doctor [OPTIONS]

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm config doctor

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## config export

```text
Export the full private configuration, including selectors and command text.

Usage: elarydil-kvm config export [OPTIONS] [PATH]

Arguments:
  [PATH]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm config export config.json

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## config import

```text
Validate and atomically replace the configuration, retaining a backup. --dry-run validates only.

Usage: elarydil-kvm config import [OPTIONS] [PATH]

Arguments:
  [PATH]
          Item ID or file path

Options:
      --dry-run
          Dry run

      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm config import config.json --dry-run

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## config reset

```text
Reset to empty defaults after confirmation; stops automatic rules and commands.

Usage: elarydil-kvm config reset [OPTIONS]

Options:
      --confirm
          Confirm

      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm config reset --confirm

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## config schema

```text
Print the configuration JSON Schema, including custom command settings.

Usage: elarydil-kvm config schema [OPTIONS]

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm config schema

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```

## completion

```text
Print shell completion for bash, zsh, fish, powershell, or elvish.

Usage: elarydil-kvm completion [OPTIONS] [SHELL]

Arguments:
  [SHELL]
          Item ID or file path

Options:
      --json
          Print a versioned JSON response

      --home <home>
          Override the private per-user configuration directory

  -h, --help
          Print help (see a summary with '-h')

Examples:
  elarydil-kvm completion bash

Docs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md
```
