# GUI / CLI parity

The GUI bridge invokes the shared `cli::dispatch` facade and agent service. All
persistent controls in the app have the CLI equivalents below. Some settings are
available only from the CLI or configuration files: advanced device selectors,
ordering several monitor actions in one rule, and profile import.

| GUI control/action | CLI equivalent |
|---|---|
| Dashboard status / refresh | `status --json` |
| Start / pause / resume / restart / stop agent | `agent start|pause|resume|restart|stop` |
| Profile enabled toggle | `profiles enable|disable PROFILE_ID` |
| Device inventory / refresh | `devices list --json`, `devices rescan` |
| Multi-select and track devices | `devices track DEVICE_ID` for each chosen ID |
| Device friendly name | `devices alias DEVICE_ID --name NAME` |
| Select physical monitor | `monitors remember MONITOR_ID` |
| Read source / capabilities | `monitors input get ID`, `monitors capabilities ID` |
| Reference/raw source choice and alias | `monitors input alias set ID --value VALUE --label LABEL` |
| Confirmed DDC test | `monitors input test ID --value VALUE --confirm` |
| Create rule (including manual choice) | `profiles create --name NAME`, `rules create ...` |
| Edit trigger, device group, ALL/ANY/N, monitor/input | `rules set RULE_ID ...` |
| Debounce / rearm / coincidence / cooldown | `rules timing RULE_ID ...` |
| Dry run / enabled | `rules dry-run RULE_ID on|off`, `rules enable|disable RULE_ID` |
| Rule **Test** (real 15-second reversible switch) | `monitors input test MONITOR_ID --value INPUT --confirm` |
| Rule simulation (no hardware writes) | `rules simulate RULE_ID --json` |
| Clone / delete / export profile (native save dialog) | `profiles clone|delete|export PROFILE_ID ...`; clone names are made unique |
| Start / finish / cancel wizard capture | `wizard start`, `wizard capture finish`, `wizard cancel` |
| Per-direction trigger devices, manual device choice | `wizard complete --disconnect-devices ... --connect-devices ...` |
| Capture results / single-rule continuation | `wizard candidates --json`, `wizard apply ...` |
| Live events / search / download | `events list --json`, `logs --follow`, filter JSON for text search |
| Sanitized diagnostics download | `doctor --export PATH` / `diagnostics export --export PATH` |
| Theme / presentation mode | `ui theme set VALUE`, `ui mode set VALUE` |
| Close action | `config set app.closeWindowAction '"hide"'` or `'"quit-gui"'` |
| Open at login (app) / Start agent at login | `startup enable|disable --component gui|agent` |
| Keep agent running when the app quits | `config set app.agentStandalone true|false` |
| Global cooldown / startup / resume grace | `config set safety.KEY MILLISECONDS` |
| Additional startup delay | `startup set --delay-ms MILLISECONDS` |
| Default debounce / DDC timeout | `config set safety.defaultDebounceMs VALUE`, `config set safety.commandTimeoutMs VALUE` |
| Full configuration download | `config export PATH` |
| Quit interface (an app-owned agent stops with it) | `gui close` |
| Connect/disconnect shell commands, shell/directory/timeout | `rules commands get|set|enable|disable RULE_ID ...` |
| Manual command tests and results | `rules commands run RULE_ID --event connect|disconnect --confirm [--wait]`, `commands show JOB_ID` |

Unknown settings are rejected by the typed schema. Hardware actions share the same
identity check and per-monitor cooldown; GUI confirmation is not the only guard.
Virtual UI tests perform manual setup, persist theme, capture cycles, and confirm
one simulated DDC write through the same service used by the CLI.


Standard/custom input tiles map to `monitors inputs` and input
alias set/remove. Live current input is `monitors state --watch` / `monitors input
watch`. Timed tests use `monitors input test --confirm`; early return is `monitors
input restore`. Live capture timing is `wizard watch`; atomic two-rule setup is
`wizard complete`. The active profile selector is `profiles activate`. Rule
Advanced contains shell commands; every option remains configurable in the CLI.

The wizard holds automation while open and completes with
separate disconnect/connect devices (`wizard complete --disconnect-devices
--connect-devices`), including without a capture. The agent's lifecycle follows
the app unless `app.agentStandalone` is on; `status --json` reports `standalone`
and `setupHold`.
