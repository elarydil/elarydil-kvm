# Elarydil KVM — first launch on macOS

Elarydil KVM is **ad-hoc signed** and not notarized by Apple. Signature integrity and Gatekeeper trust are
separate checks: Gatekeeper may block a downloaded copy, sometimes with a message
that the app "is damaged".

## Install

1. Open the DMG and drag **Elarydil KVM.app** to **Applications**. When updating,
   replace the previous copy. Eject the DMG.
2. Quit any running Elarydil KVM (menu-bar icon → **Quit Elarydil KVM**).
3. In Terminal, verify and authorize this particular copy:

```sh
codesign --verify --deep --strict "/Applications/Elarydil KVM.app"
xattr -dr com.apple.quarantine "/Applications/Elarydil KVM.app"
open "/Applications/Elarydil KVM.app"
```

`codesign` checks that the app is intact; if it fails, copy the app from the DMG
again. `xattr` removes the download quarantine from this copy only. Global security
settings are not changed. Run these commands only for a build you downloaded from
the project's releases. If you installed the app elsewhere, use its actual path.

## Check the download

The release lists a `SHA256SUMS.txt`. In the folder with the downloaded files:

```sh
shasum -a 256 -c SHA256SUMS.txt
```

CI builds and ad-hoc signs the DMG but does not launch it, and it does not test
physical USB/DDC hardware.

## If the app or its CLI hangs on first launch

The Dock icon keeps bouncing, or `elarydil-kvm --version` prints nothing and never
returns. Elarydil KVM's own code has not started yet in this case.

macOS checks every new, not-notarized executable before it first runs. The
`syspolicyd` system service performs this check and caches the result. If that
service is stuck, the process waits before its first instruction. Removing
quarantine does not help, and an earlier version of the app may keep working
because its check was cached before the service got stuck.

- Restart the service (launchd starts it again immediately), then open the app:

  ```sh
  sudo killall syspolicyd
  ```

- Or restart the Mac.

To confirm this diagnosis, run `sample <pid> 1` on the hung process. If the only
frame is `_dyld_start` and its memory footprint is under 1 MB, the process is
waiting for macOS, not running. A Developer ID signed and notarized release would
avoid this check entirely.

## Why the extra steps

Installing without manual authorization requires an Apple Developer ID certificate
and notarization of each release. An ad-hoc signature does not create that trust.
Signing credentials belong in the repository's GitHub Secrets, never in the source.
