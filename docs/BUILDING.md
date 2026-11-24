# Building and releasing

## Requirements

- Rust stable and Node 22 or later.
- **macOS:** Xcode command-line tools; Apple Silicon. Intel Macs are not supported.
- **Windows:** Visual Studio Build Tools with the C++ workload. The installer
  provisions the Microsoft Edge WebView2 Runtime when it is missing.
- **Linux:** `libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev
  librsvg2-dev libdbus-1-dev` (Debian/Ubuntu names).

Installed binaries need no Rust, Node or Python runtime.

## Build

```sh
cargo build --locked --release              # CLI, agent and helpers (no WebView)
cargo test --workspace --locked
python3 packages/build.py                   # build helpers and stage them for the app
cd apps/desktop && npm ci
npm run tauri build -- --bundles dmg        # macOS
npm run tauri build -- --bundles nsis       # Windows
npm run tauri build -- --bundles deb,rpm    # Linux
```

On macOS, build with `ELARYDIL_KVM_TARGET=aarch64-apple-darwin` and
`MACOSX_DEPLOYMENT_TARGET=13.0`, and pass `--target aarch64-apple-darwin` to
`tauri build`. `packages/build.py` compiles the Rust helpers and the bundled m1ddc
helper (`-arch arm64`), signs them ad hoc on macOS, and writes the CLI-only ZIP to
`dist/`. Tauri then bundles the app with the helpers in its resources and signs the
bundle ad hoc.

Developer ID signing and notarization need an Apple Developer account; store the
certificates and passwords in the repository's GitHub Secrets, never in the source.

## Tests

```sh
cargo test --workspace --locked             # 52 Rust tests
cargo build --locked && python3 tests/cli_smoke.py
python3 tests/catalog_check.py
cargo build --bins --example virtual_agent --locked
cd apps/desktop && npx playwright install chromium && npm test
```

The UI tests drive the real interface through IPC against `virtual_agent`, a test
backend with simulated USB devices and monitors.

## What CI runs

| Event | Workflow | Runners |
|---|---|---|
| Pull request | `build-and-test` | Linux: Rust tests, CLI tests, string catalog check, UI tests |
| Push to `main` | `build-and-test` | Linux: the same checks |
| Push to `main` | `release-installers` | Windows: Rust tests and CLI tests |
| Push to `main` with a new version | `release-installers` | Also the Windows installer, the macOS DMG, the Linux `.deb`/`.rpm`, and publishing the release |

Other branches run nothing until a pull request is opened. A short `plan` job checks
that every file carries the same version and whether `v<version>` is already
released; if it is, the installers are skipped. macOS runner minutes are the most
expensive, so macOS only runs for a new release. To build installers for an existing
version without changing its release, start `release-installers` manually from the
Actions tab with *Build the installers* checked. Build artifacts are kept for 7 days.

## Releasing

1. `python3 packages/bump-version.py 1.0.1` writes the new version into
   `tauri.conf.json` (the source of truth that `packages/version.py` prints), both
   `Cargo.toml` and `Cargo.lock` files, `package.json` / `package-lock.json`, the
   version shown in the app, the dependency inventory and the README, and creates
   `docs/releases/1.0.1.md`.
2. Write the release notes in that file and remove every `TODO`.
   `python3 packages/bump-version.py --check` confirms that everything matches.
3. Commit and merge into `main`.

`release-installers` then builds every installer, waits for the Linux checks and
publishes the `v1.0.1` release with the notes, the installers, the CLI ZIPs,
`MACOS-FIRST-LAUNCH.md` and `SHA256SUMS.txt`. Do not create the tag yourself; the
workflow creates it on the merged commit.
