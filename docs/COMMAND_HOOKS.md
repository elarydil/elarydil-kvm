# Custom commands

Elarydil KVM can run a command when a rule's selected USB group **connects to this
computer** or **disconnects from it**. These are USB presence transitions; they do
not assert that the monitor successfully changed inputs. Configure each computer
separately. Rule editor → Advanced → Custom commands exposes the same configuration as the CLI.
Commands are saved together with the rule; they reuse its device group and timing.

## Configure from the CLI

```sh
elarydil-kvm agent start
elarydil-kvm rules list --json
# Substitute an actual rule ID. Bash/zsh examples:
elarydil-kvm rules commands set RULE_ID \
  --on-connect 'printf "connected\n" >> "$HOME/kvm-events.log"' \
  --on-disconnect 'printf "disconnected\n" >> "$HOME/kvm-events.log"' \
  --shell system --timeout-ms 10000
elarydil-kvm rules commands get RULE_ID
elarydil-kvm rules commands enable RULE_ID
```

PowerShell example (single quotes keep `$env` references for execution by the agent):

```powershell
elarydil-kvm rules commands set RULE_ID --on-connect 'Add-Content "$env:USERPROFILE\kvm-events.log" "connected"' --on-disconnect 'Add-Content "$env:USERPROFILE\kvm-events.log" "disconnected"' --shell powershell --timeout-ms 10000
elarydil-kvm rules commands enable RULE_ID
```

Setting commands preserves their current enabled state; a new rule defaults to
**disabled**. Both fields are optional, but enabling requires at least one command.
Omitted CLI options preserve their values; pass `--on-connect ''` or
`--on-disconnect ''` to clear a field. Set `--working-dir PATH` for a working
directory; empty uses your home directory. Relative paths are resolved from home.
The chosen directory must exist when the command runs.

`system` means Windows PowerShell on Windows, `/bin/zsh` on macOS, and `/bin/sh`
on Linux. `bash`, `powershell`, and Windows-only `cmd` are also supported. Bash
must be installed and on PATH on Windows; PowerShell requires `pwsh` on Unix.
There is no terminal window, interactive prompt, or administrator elevation.
The agent inherits PATH/environment at its launch; login-launched agents may have
a shorter PATH than an interactive terminal. Use absolute executable paths when needed.

## Automatic execution and tests

The profile, rule, and command settings must all be enabled. Rule dry run logs
`command.dry_run` without execution. The same ALL / ANY / AT LEAST N grouping,
coincidence window, debounce, rearm and cooldown apply in **both directions**,
independently of which direction the monitor action uses. Do not attach the same
command to multiple enabled rules unless you want it to run for each rule.
Cloning a profile creates a disabled copy.

Startup, resume/rescan, and relevant configuration changes establish a new baseline;
existing presence or absence does not execute a command. Pause, stop, USB resync,
matching ambiguity, or relevant configuration changes cancel pending/active jobs.
Changing only appearance does not cancel a command. A long command does not block
USB watching or IPC. One FIFO worker executes at most one command at a time, with
16 pending jobs; jobs waiting over 30 seconds are skipped. Jobs are not replayed
after an agent restart.

Explicit manual tests execute even if the rule/profile is disabled or in dry run:

```sh
elarydil-kvm rules commands run RULE_ID --event connect --confirm --wait --json
elarydil-kvm rules commands run RULE_ID --event disconnect --confirm --json
elarydil-kvm commands show JOB_ID --json
elarydil-kvm commands list --limit 20 --json
elarydil-kvm rules commands disable RULE_ID
```

Without `--wait`, the response gives a job ID immediately. With `--wait`, the CLI
returns the shell exit code (0–255); timeout is 124, cancellation/skipping is 130.
An early setup failure returns 1. Nonzero shell exit codes produce `command.failed`.
See `rules commands run --help` and `commands show --help`.

The shell receives `ELARYDIL_KVM_EVENT` (`connect`/`disconnect`),
`ELARYDIL_KVM_RULE_ID`, `ELARYDIL_KVM_PROFILE_ID`, and `ELARYDIL_KVM_JOB_ID`.
Commands use your account permissions and run exactly as saved. Do not import
configuration containing commands you have not reviewed.

## Limits and output

Timeout is 100–300000 ms, default 10000 ms, including helper and shell startup.
A timeout/cancellation terminates the helper and shell process tree: a Unix process
group or a Windows Job Object. Commands deliberately detaching into another session
or service may outlive that boundary. Successfully launched background applications
may continue after the shell exits; redirect their stdout/stderr explicitly.

Each command field is limited to 16384 UTF-8 bytes. Output is drained continuously,
but only the first 4096 bytes of each stream are retained, with truncation flags.
Results, exit code, duration, and failures are in the private event journal.
Normal events omit command text. Diagnostic export redacts command fields, working
directories, stdout, and stderr; full configuration/profile exports retain them.

The GUI is optional: `elarydil-kvm agent start` runs all automation without a
window, tray, WebView, Node, or Python. See [Headless operation](HEADLESS.md).
