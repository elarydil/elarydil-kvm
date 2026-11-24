# Troubleshooting

- **macOS reports “Elarydil KVM.app is damaged”:** the app is not notarized by
  Apple. Install the app from its DMG and follow the scoped
  verification and first-launch steps in [MACOS_FIRST_LAUNCH.md](MACOS_FIRST_LAUNCH.md).
  If `codesign --verify --deep --strict` fails, replace the app from the DMG.
- **The Dock icon bounces forever, or even `elarydil-kvm --version` hangs:** macOS
  has not let the new executable start yet; its `syspolicyd` check is stuck. Run
  `sudo killall syspolicyd` (launchd restarts it) or restart the Mac. Details and
  how to confirm it are in [MACOS_FIRST_LAUNCH.md](MACOS_FIRST_LAUNCH.md).
- **The background agent does not start:** the app shows a window with the reason.
  Every failed start, from the app or `elarydil-kvm agent start`, is appended to
  `/tmp/elarydil-kvm-<uid>/agent-start.log` (`%TEMP%\elarydil-kvm\agent-start.log` on
  Windows) with the error, the configuration path and the agent's own error output.
  *The settings file could not be read* usually means an older version is opening
  settings written by a newer one: install the newer version again.
- **Switching stops when the app quits:** this is the default; the agent belongs to
  the app. Keep the app in the menu bar (closing the window does that), or enable
  Settings → *Keep agent running when the app quits*. See [HEADLESS.md](HEADLESS.md).
- **Switching back right after a switch is ignored:** the *Minimum interval between
  monitor writes* blocks further writes to the same monitor for its duration. Older
  installs may still have 5000 ms; 100 ms (the current default) is enough to stop
  two rules from fighting. Set it in Settings or with
  `elarydil-kvm config set safety.globalMonitorCooldownMs 100`.
- **Monitors switch while running the setup wizard:** they should not; the wizard
  holds automation while it is open (`status --json` shows `"setupHold": true`).
  The hold is renewed by the open window and lapses within 15 seconds otherwise.

- **Agent unavailable:** `elarydil-kvm agent start`, then `elarydil-kvm doctor --json`.
  The CLI, agent and DDC helper must be installed together. `--home` must point to
  the same private directory for both clients. A second agent is prevented by a lock.
- **Visible display, unavailable DDC:** enable DDC/CI in the monitor OSD. Read input
  before trying a write. Test a direct supported connector without the dock. Video
  working does not establish DDC access. Do not scan arbitrary VCP values.
- **Command submitted:** the OS accepted the API call; this does not establish that
  the panel actually switched. Read back if possible and confirm visually yourself.
- **Picture disappears:** use the monitor's physical input button, or restore the
  physical KVM/video route. A software revert is never guaranteed.
- **Missing/ambiguous monitor:** reselect an exact EDID + connector. No command is
  redirected to the primary screen. Identical monitors without unique serials need
  explicit connector selection. Replacing a monitor with an identical EDID cannot
  be distinguished physically; review the saved selector after hardware changes.
- **Windows MST/multiple physical handles:** one GDI logical display exposing more
  than one physical handle is conservatively omitted. Better handle-to-EDID mapping
  for this topology is an open task; never use index fallback.
- **Apple Silicon:** private IOAVService support varies by macOS, GPU, port, display
  and adapter. Direct HDMI and USB-C/DP must be verified separately. The helper is
  bundled by the build script; do not install Homebrew. Intel Mac DDC is unsupported.
- **Linux permissions:** check `/sys/class/drm` and the selected `/dev/i2c-N`, and
  ask an administrator whether `i2c-dev` is loaded. The optional reviewed group rule
  is in `packages/60-elarydil-kvm-i2c.rules`. Create the i2c group, add the intended user,
  install the rule through normal administration, and log out/in. Do not chmod all
  I2C devices world-writable. Containers and remote virtual desktops often expose
  neither DRM nor USB; the agent should report degraded/unavailable.
- **No capture candidates:** some KVMs emulate permanently connected peripherals.
  Try a hub/receiver that really disappears or choose a manual selector. Use two
  cycles. Identical devices without serials need topology/manual review.
- **No trigger:** initial absence is not an event. Return the group to its opposite
  stable state, wait rearm, then switch. Check dry run, profile/rule enable flags,
  debounce, coincidence window, source mapping, startup/resume grace and cooldown.
  N-of-M counts logical physical devices, not HID interfaces.
- **Two hosts fighting:** there is no network coordination. Rate limits apply on
  each host only. Prefer connect → that host's input and avoid ping-pong rules.
- **No tray icon:** desktop/AppIndicator support varies. Use window-only mode.
  `elarydil-kvm gui open` is the recovery command.
- **Autostart:** registrations are per-user, not privileged services. Linux needs
  a functioning `systemctl --user`. macOS may block LaunchAgents in Login Items;
  review System Settings.
- **Linux app does not open:** install the package with `apt` or `dnf` so that
  WebKitGTK and AppIndicator are installed with it.

Stop automation before changing hardware identities. Start with dry run and review
recent events. Diagnostic exports are local files; inspect them before sharing.

A Debian/Ubuntu permission setup can use the following commands
from this source checkout. Replace the target login with the actual desktop user;
review the udev rule before installing it. The app never changes these permissions
itself.

```sh
sudo modprobe i2c-dev
getent group i2c || sudo groupadd --system i2c
sudo install -m 0644 packages/60-elarydil-kvm-i2c.rules /etc/udev/rules.d/60-elarydil-kvm-i2c.rules
sudo usermod -aG i2c TARGET_LOGIN
sudo udevadm control --reload-rules
sudo udevadm trigger --subsystem-match=i2c-dev
# Log the target user out and in before retesting.
```
