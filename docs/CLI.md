# Elarydil KVM CLI guide

The CLI and independent agent do not link a WebView. Keep `elarydil-kvm`,
`elarydil-kvm-agent`, `elarydil-kvm-ddc`, and `elarydil-kvm-command` together;
Apple Silicon bundles also include `m1ddc`. No Rust, Node or Python runtime is
needed by installed executables. Global options: `--json`, `--home PATH`,
`--version`, `--help`. Every command has specific options, an explanation and an
example. [CLI_REFERENCE.md](CLI_REFERENCE.md) is generated from actual help.

## Guided setup

```sh
elarydil-kvm agent start
elarydil-kvm wizard start
```

In a terminal, the wizard captures USB changes, lets you choose devices and the
monitor/input for each direction, then atomically creates one active profile with
connect and disconnect rules. Inputs do not need testing first. Both rules are
live, debounce is 0, and the captured baseline is primed so the first switch can
work immediately. `--dry-run` is an explicit optional choice.

For scripts, use an agent-owned capture and the same completion operation:

```sh
elarydil-kvm wizard start --non-interactive --json
elarydil-kvm wizard watch --duration 30
# Switch away and back; no typing is required while the USB devices are away.
elarydil-kvm wizard capture finish --json
elarydil-kvm wizard candidates --json
elarydil-kvm wizard complete --devices DEVICE_A,DEVICE_B \
  --connect-monitor MONITOR_ID --connect-input 17 \
  --disconnect-monitor MONITOR_ID --disconnect-input 18
```

Use IDs from list/capture results, not the placeholders above. Observations contain
`disconnectedAtMs` and `connectedAtMs` relative to capture start. They record when
the USB watcher observed each event, not a physical USB transfer measurement.
A missing event is null. The GUI refreshes capture state every 50 ms without waiting
for a complete cycle. Results persist when the GUI closes; interrupted captures
are marked inactive after agent restart. Legacy `wizard apply` remains available
for scripts that deliberately create one rule.

Each direction can watch its own devices. `--disconnect-devices` and
`--connect-devices` override `--devices` for that rule. The desktop wizard
preselects, per column, the first device and every device within 100 ms of it:

```sh
elarydil-kvm wizard complete \
  --disconnect-devices KEYBOARD_ID,MOUSE_ID,RECEIVER_ID --connect-devices KEYBOARD_ID,MOUSE_ID \
  --connect-monitor MONITOR_ID --connect-input 17 \
  --disconnect-monitor MONITOR_ID --disconnect-input 18
```

Without a capture session (no `--session`), `wizard complete` takes devices from
the live USB inventory, which is how the GUI's *Choose devices manually* finishes.
While a capture runs, and while the desktop wizard is open, rules do not switch
monitors; `status --json` reports this as `"setupHold": true`. A taken profile
name gets a numbered suffix (`Default profile 2`).

## Monitors and inputs

```sh
elarydil-kvm monitors list --json
elarydil-kvm monitors state MONITOR_ID --watch --json
elarydil-kvm monitors input watch MONITOR_ID --duration 30
elarydil-kvm monitors inputs MONITOR_ID
elarydil-kvm monitors input alias set MONITOR_ID --value 18 --label 'Work MacBook'
elarydil-kvm monitors input alias set MONITOR_ID --value 0x1F --label 'Dock'
elarydil-kvm monitors input alias remove MONITOR_ID --value 18 --reset
elarydil-kvm monitors input alias remove MONITOR_ID --value 0x1F
```

Live observation saves a monitor identity automatically; `monitors remember ID`
is also available. Standard DisplayPort 1/2 and HDMI 1/2 inputs are always available
before testing. `monitors inputs` returns the merged `inputs` list, saved aliases
and standard references. Removing a standard friendly name in the UI (CLI `--reset`) keeps the port, its rules and verification. Hiding a standard input with CLI removal is recorded in `hiddenInputs`;
adding it again removes that hidden marker. An input used by a rule cannot be
removed until the rule is changed. Names are shared by monitor tiles, wizard and
rule editing. Renaming preserves existing verification.

Typical MCCS input values are DisplayPort 1 = 15/0x0F, DisplayPort 2 = 16/0x10,
HDMI 1 = 17/0x11, HDMI 2 = 18/0x12. Actual values depend on the monitor. Read-back
verification is informational: an unverified input is selectable and usable.
Successful reads mark the observed input verified. `--verified` explicitly records
your own confirmation. Built-in Mac displays are excluded from enumeration.

`monitors state --watch` renews a short-lived live-read lease. Reads are serialized,
target 300 ms intervals, and back off on errors. Cached state/USB capture remain
responsive while a read is slow. Watches expire after the viewer closes. Current
input and DDC availability reflect reads, not an unchanging warning label.

```sh
# Optional reversible test, with a fresh read of the original input:
elarydil-kvm monitors input test MONITOR_ID --value 18 --confirm
elarydil-kvm monitors state MONITOR_ID --json
elarydil-kvm monitors input restore MONITOR_ID
# Ordinary switching without a return timer:
elarydil-kvm monitors input set MONITOR_ID --value 18 --confirm
```

Tests return to the recorded original input after 15000 ms; `--restore-ms` accepts
100–60000 ms for scripts/testing. The agent owns the monotonic timer and persists
recovery information. Closing the GUI does not cancel a return. Agent stop attempts
a return before exiting; restart recovers interrupted tests. Normal writes to that
monitor are blocked during an active test. Restoration has priority over periodic
reads and bypasses the ordinary write gap. A lost DDC link can still prevent a
return: failures are reported and retried, and the physical input control remains
the recovery path. Writes are submitted commands, not proof of visual switching.

## Manual rules and profiles

```sh
elarydil-kvm devices list --json
elarydil-kvm devices track DEVICE_A
elarydil-kvm devices track DEVICE_B
elarydil-kvm profiles create --name 'Desk'
elarydil-kvm rules create --profile PROFILE_ID --name 'Switch away' \
  --devices DEVICE_A,DEVICE_B --mode all --on disconnect \
  --monitor MONITOR_ID --input 18
elarydil-kvm profiles activate PROFILE_ID
```

One profile is active in the interface. Activating/enabling one disables the others;
each rule inside it still has an individual enabled state. Existing configurations
with multiple active profiles use the first when loaded. New additional profiles
and cloned profiles start inactive. Configured selectors belonging only to inactive
rules do not inhibit the active rule group.

Rules support `all`, `any`, or `at-least` with `--min-count`. Defaults: debounce 0 ms,
rearm 500 ms, cooldown 500 ms, USB event grouping window 0 (off), dry run off.
`rules timing` changes these values. Debounce/rearm allow 0–60000 ms; cooldown and
event grouping allow 0–120000 ms. Startup/resume grace is a separate baseline period.
The global gap between ordinary writes to one monitor defaults to 100 ms and allows
0–120000 ms. Settings and rule fields show exact validation ranges.

```sh
elarydil-kvm rules timing RULE_ID --debounce-ms 0 --rearm-ms 500 --cooldown-ms 100
elarydil-kvm rules set RULE_ID --mode at-least --min-count 2
elarydil-kvm rules dry-run RULE_ID on
elarydil-kvm rules simulate RULE_ID --json
elarydil-kvm rules dry-run RULE_ID off
```

Simulation has no monitor writes or shell execution. CRUD, selectors, multiple
monitor actions, imports/exports and configuration editing remain available; use
the generated reference for every command. Duplicate monitor actions and dangling
references are rejected. Advanced non-0x60 mappings require explicit confirmation.
VID/PID selector arguments use hexadecimal; input values use decimal or `0x` hex.

## Commands, lifecycle and diagnostics

[Custom commands](COMMAND_HOOKS.md) are configured under each rule’s Advanced
section or `rules commands`. They run in the agent with the GUI closed.

```sh
elarydil-kvm agent start
elarydil-kvm agent pause
elarydil-kvm agent resume
elarydil-kvm agent restart
elarydil-kvm agent stop --wait
elarydil-kvm startup enable --component agent
elarydil-kvm startup disable --component gui
elarydil-kvm gui close
```

`agent start` upgrades a running older agent when it lacks the current API features.
Restart waits for the old instance lock to be released. `agent stop --wait` also
waits for cleanup/input return. GUI/tray actions share the same service calls;
quitting the interface keeps the agent running. [HEADLESS.md](HEADLESS.md) covers
login startup and isolated configuration roots.

`doctor`, `diagnostics export --export PATH`, `events list`, and `logs --follow`
provide diagnostics. Sanitized exports redact paths, serials, command text and
command output; full configuration/profile exports retain private settings.
`config set KEY VALUE` takes a JSON value. `ui theme set system` uses OS appearance.
Use dedicated startup commands to change OS login registration.

Successful JSON: `{"schemaVersion":1,"result":...}`. Errors have `code`, `message`,
optional `detail` and optional `field` identifying an invalid setting. Live streams
emit versioned NDJSON. Exit codes: 0 success; 1 OS/internal; 2 invalid input/config;
3 agent unavailable; 4 monitor missing; 5 unsupported/DDC; 6 permissions/auth;
7 confirmation/cooldown; 8 timeout; 9 ambiguity. Custom command `--wait` returns
its shell exit code, with timeout 124 and cancellation/skipping 130.
