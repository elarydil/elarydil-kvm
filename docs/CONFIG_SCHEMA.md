# Configuration and schema

See `config.schema.json` for generated structural schema. The authoritative typed
validator additionally checks cross-references, enum strings, timing and identity
invariants. Every mutation validates the complete result before saving. Schema
version 1 is supported; no older deployed schema exists. Unknown future versions
are rejected rather than silently reinterpreted. Automatic older-version migrations
and automatic corruption repair are not implemented.

Paths: Linux `${XDG_CONFIG_HOME:-~/.config}/elarydil-kvm`; macOS
`~/Library/Application Support/elarydil-kvm`; Windows `%APPDATA%/elarydil-kvm` using the
`directories` crate. All configuration, private IPC metadata, capture and bounded
journal currently share this private directory. Separate OS log/cache directories
are a future refinement. `ELARYDIL_KVM_HOME` overrides all paths; CLI `--home` has the
same effect for that invocation and is inherited by agents it starts.

Files: `config.json`, `config.json.bak`, `config.lock`, `agent.lock`, `token`,
`agent.sock` (Unix), `capture.json`, `rate-state.json`, `events.jsonl`.
The rate file records last attempted wall timestamps keyed by monitor fingerprint;
a fresh agent also has startup grace. A backwards wall jump conservatively blocks
writes until the elapsed safety interval becomes valid again.

## Documented settings registry

`config get KEY` / `config set KEY JSON` address existing typed properties with
`.` notation. Unknown keys and incorrectly typed values fail. For example:

```sh
elarydil-kvm config set app.theme '"dark"'
elarydil-kvm config set safety.globalMonitorCooldownMs 100
```

| Key | Allowed value / behavior |
|---|---|
| app.locale | `"en-US"` only |
| app.theme | `"system"`, `"light"`, `"dark"` |
| app.uiMode | `"tray-only"`, `"menu-bar-only"`, `"window-only"`, `"both"` (default), `"headless"` |
| app.closeWindowAction | `"hide"` (default: keep the app in the menu bar/tray), `"quit-gui"` |
| app.launchAgentAtLogin, app.launchGuiAtLogin | Boolean preferences; use `startup` commands to change OS registration |
| app.agentStandalone | Keep the agent running after the desktop app quits (default `false`: the agent stops with the app that started it; agents started from a terminal or at login are always standalone) |
| app.startupDelayMs | 0–120000; additional startup arming delay |
| app.notifications | `"off"`, `"errors-only"`, `"all"`; preference reserved, OS notification delivery unfinished |
| app.lastPage | Stored string; window/page geometry recovery unfinished |
| app.logRetentionDays | Positive integer; age is applied during size rotation |
| app.logMaxBytes | 4096–104857600 |
| safety.globalMonitorCooldownMs | 3000–120000; default 5000 |
| safety.startupGraceMs, safety.resumeGraceMs | At least 1000 |
| safety.defaultDebounceMs | Unsigned milliseconds; rule-level debounce is authoritative |
| safety.commandTimeoutMs | 100–30000 |
| usbDevices, monitors, profiles | Complete typed arrays, validated atomically |

Prefer dedicated device/profile/rule/input commands to editing arrays. Array
indices in dotted paths can edit an existing property, but final reference and
uniqueness checks still apply. Never use sample IDs against real hardware.

Rules store trigger `connect|disconnect`, mode `all|any|at-least`, minimumCount,
coincidenceWindowMs (`null` disables), debounceMs, rearmMs, cooldownMs, dryRun,
enabled, and ordered actions. Device IDs must exist; duplicate actions for one
monitor are rejected. Each action's nonzero input/VCP pair must be explicitly saved
on the selected monitor. Nonstandard VCP mappings require confirmation when added.

Examples in `examples/*.json` are schema-valid **dry-run templates**, not claims of
connected hardware. Replace sample VID/PID, monitor fingerprint and connector.
Import with `--dry-run`, review, then import. Imports do not silently register OS
startup tasks. For corruption, stop the agent and validate a copy of `.bak` before
restoring it; keep the damaged original for diagnosis. No recovery guesses are sent
to displays.


Each rule has optional `commands`: `enabled` (false), `onConnect`/`onDisconnect`
(empty strings), `shell` (`system`), `workingDirectory` (empty = home), and
`timeoutMs` (10000). Existing configurations load unchanged; saving includes these
defaults. See [command settings and limits](COMMAND_HOOKS.md).


Saved monitors have an optional `hiddenInputs` array (default empty); standard
mappings are merged with explicit aliases, and an input need not be verified to be
referenced. Fresh defaults are the system theme, `uiMode: "both"`,
`closeWindowAction: "hide"`, `app.agentStandalone: false`, debounce 0 ms and a
100 ms global write gap. Rule cooldown accepts 0–120000 ms. Validation errors may
include a `field` path and the exact range. If several profiles are enabled in a
loaded file, the first one stays active. Profile names are made unique on create,
clone, rename and wizard completion.

Unknown fields are refused, so a configuration written by a newer version is not
read by an older one; the app and CLI report this as `config_unreadable` with the
file path.
