# Architecture

`crates/elarydil-kvm` is one Rust crate with domain/service/platform modules and four
logical executables. Keeping the domain and transports in one crate reduces
coordination overhead for this initial implementation; `cargo build --workspace`
excludes Tauri. `apps/desktop/src-tauri` is an independent native application crate.

```
React → Tauri request ─┐
                      ├→ version-1 local IPC → Service → Engine → Hardware
clap CLI ─────────────┘                           ├→ Store / bounded journal
                                                 └→ agent-owned Capture
```

The GUI calls the CLI application facade, not shell commands. Both call the same
serialized `Service.execute` operations. Offline configuration operations use that
same service against the locked store. Hardware operations require a live agent.
There is no production TCP/HTTP server. The Playwright route emulating Tauri's
transport exists only inside the test runner; its requests reach a real Rust IPC
service with an explicitly virtual hardware adapter.

## Agent lifecycle

The agent is always a separate process, so a busy or crashed window never blocks
USB observation. Its lifetime follows whoever started it. The desktop app exports
`ELARYDIL_KVM_OWNER_PID` before starting (or adopting, via `agent.owner`) an agent;
the agent checks once a second and exits when that process is gone, unless
`app.agentStandalone` is true. Agents started from a terminal or by a login item
have no owner and run until `agent stop`. `status` reports `standalone`.

While the setup wizard is open, the GUI renews a 15-second `wizard.hold` lease
every 5 seconds; `wizard.start` also takes one. During the lease the engine
evaluates nothing, and when it ends the engine is primed from current USB presence,
so switches made during setup are never replayed. Completing or cancelling the
wizard releases the hold.

## Agent and transitions

nusb watches native USB presence events. It never opens a device or claims an
interface. An event triggers inventory refresh; there is no full-inventory timer.
A 50 ms timer advances debounce/rearm deadlines. Enumeration/watcher failures
inhibit automation; watcher creation is retried with bounded 1–30 second backoff.
On reconnection a fresh baseline is established.

Each rule stores its observed previous state, target-edge timestamps, opposite
stability start, pending debounce and last consumed transition. Initial missing
USB is a baseline, not an edge. Startup/resync grace disarms the engine. ANY must
return below its qualifying threshold before rearming; a partially absent group
cannot repeatedly fire ANY. ALL and N-of-M use fresh constituent target edges
for the optional coincidence window. Disabling that window allows steady-state
qualification after a genuine new edge. Decreasing monotonic timestamps are ignored.

Validated decisions are ordered by profile then rule order. Each rule can contain
one action per monitor. Commands run serially in the agent; monitor identity and a
shared fingerprint-based limiter are checked before every dispatch. Failures also
consume a bus attempt, preventing repeated retries. The per-monitor write interval
(`safety.globalMonitorCooldownMs`) defaults to 100 ms and accepts 0–120000 ms; a long
interval can drop a quick switch back. A consumed transition is never retried every tick. Dry run logs a
match and performs no hardware writes. Simulation creates a separate engine.

The engine uses `Instant`. A conservative wall-time gap (>3 seconds between loop
iterations) refreshes inventory and applies resume grace rather than replaying
stale edges. This covers suspend and resume without relying on platform power notifications.

## USB identity and captures

Selectors are alternatives per logical device: serial+VID/PID, instance/topology+
VID/PID, or explicitly chosen VID/PID. Duplicate physical IDs cannot count as
independent devices. Ambiguous broad selectors inhibit matching unless ambiguity
was explicitly allowed. Friendly aliases and selectors persist in typed config.
Capture records baseline devices, away/return cycle counts, currently absent IDs,
and up to 2,000 snapshot records in the private store. It is independent of the
window and persisted on changes. A restarted agent marks an old capture inactive:
it cannot invent evidence for events while it was offline. Correlation without
serial numbers is less certain; repeated identical VID/PID devices require manual
selector review. Full topology correlation and capture continuation after agent
restart are not complete.

## DDC

Windows uses `EnumDisplayMonitors`, GDI display interfaces, registry EDID,
physical monitor enumeration, and DXVA2 Get/SetVCP/capabilities APIs. Every acquired
physical handle is released by RAII. A logical GDI display exposing multiple
physical handles is skipped instead of assigning an index heuristically. No
NirSoft dependency or NVIDIA-only assumption is used.

Apple Silicon uses a bundled, source-vendored MIT m1ddc helper with a small JSON
listing extension. UUID is used explicitly; no default monitor is selected. A
UUID collision blocks writes. The helper relies on private IOAVService interfaces;
connector/OS/adapter compatibility needs physical tests. Intel is unsupported.

Linux reads DRM EDID and resolves its DDC symlink to the exact `/dev/i2c-N` bus.
The native i2c-dev adapter implements MCCS VCP read, write and chunked capabilities
with reply/status/checksum validation. It excludes internal eDP/LVDS displays.
The write helper re-enumerates and compares EDID, connector and bus immediately
before opening the I2C device. Missing DRM access is reported as unavailable.

OS DDC calls run in a separate `elarydil-kvm-ddc` process. The parent drains bounded
stdout concurrently and kills a timed-out helper; on Unix a dedicated process
group also contains its children. This avoids indefinitely freezing the UI or
reusing stale handles after timeout. API success means submitted/unverified;
readback and visual confirmation are separate concepts.

## Configuration, IPC and privacy

JSON schema version 1; unknown fields/versions are rejected. `Config.validate`
checks referenced devices, monitors and saved input mappings, unique IDs,
thresholds, timing ranges and locale/UI settings before activation. A per-file
OS lock serializes writers. Save writes a unique private temp file, fsyncs it,
keeps `.bak`, and replaces atomically (MoveFileEx with replace/write-through on
Windows). Unix also syncs the containing directory. Ordinary edits are acknowledged
and reset rule baselines safely. A one-second metadata check notices external edits.

One agent holds an exclusive per-user lock. Unix uses a private owner-checked
0700 directory and a 0600 filesystem socket. Windows sets a protected user-SID
DACL on the runtime directory and named pipe. Requests contain version, UUID,
private random token, operation and payload; replies echo version/UUID and an
explicit result/error. Input is bounded to 1 MiB. No arbitrary executable operation
is exposed. Same-user hostile clients are outside the isolation boundary.

The journal is bounded to 10,000 in-memory records with configured disk rotation.
USB log events contain internal hashes, not keystrokes or serials. Full config
exports intentionally contain private selectors. Public diagnostic exports should
be reviewed before sharing; no files are uploaded automatically.


Custom commands run in a FIFO worker and separate `elarydil-kvm-command` helper,
with bounded output and process-tree cancellation. Synthetic rules reuse the same
USB evaluator for both directions. See [command execution semantics](COMMAND_HOOKS.md).


The agent serializes hardware DDC through `monitor_control::Controller`. It owns leased
300 ms reads, cached availability/current input, and persisted reversible tests.
A monotonic deadline drives normal return; stored wall timestamps support restart
recovery. Return checks the original monitor identity and has priority over reads.
GUI polling reads cached state, so slow DDC does not block USB observation. The
USB watcher timestamps snapshots before the main IPC queue; wizard observations
therefore retain event ordering while a UI request is being served. Wizard
completion is atomic and primes its explicitly observed baseline without emitting
a synthetic transition. Profile activation is exclusive in the interface.
