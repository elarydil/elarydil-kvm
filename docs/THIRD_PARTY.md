# Third-party software

Elarydil KVM is released under the [MIT License](../LICENSE). It is built with the
open-source components below. Their license texts are kept in [licenses/](licenses),
and [DEPENDENCIES.json](DEPENDENCIES.json) lists every locked package with its
version, license and checksum.

## Included in the app

| Component | Used for | License |
|---|---|---|
| [m1ddc](https://github.com/waydabber/m1ddc) by waydabber | DDC/CI on Apple Silicon (bundled helper, source in `vendor/m1ddc`) | MIT — [license](../vendor/m1ddc/LICENSE) |
| [Geist Mono](https://vercel.com/font) via `@fontsource-variable/geist-mono` | Interface font | SIL Open Font License 1.1 — [license](licenses/geist-mono/OFL.txt) |
| [Tauri](https://tauri.app) | Desktop window, menu bar/tray, dialogs | MIT or Apache-2.0 |
| [React](https://react.dev) | Interface | MIT |
| [Lucide](https://lucide.dev) | Icons in the app and the menu-bar menu | ISC — [license](licenses/lucide-react-0.468.0) |
| [nusb](https://github.com/kevinmehall/nusb) | USB device monitoring | MIT or Apache-2.0 |
| Microsoft WebView2 SDK loader | Windows web view | BSD-style — [license](licenses/Microsoft-WebView2-SDK-1.0.3800.47/LICENSE.txt) |

`vendor/m1ddc` is a copy of the upstream project with a JSON display-list output
added; the upstream revision is recorded in `vendor/m1ddc/UPSTREAM.md`.

On Linux, GTK, WebKitGTK and AppIndicator come from the distribution and are not
bundled. On Windows, the WebView2 Runtime is installed by Microsoft's bootstrapper
when needed and is not bundled.

## npm packages in the app

| Package | Version | License |
|---|---|---|
| @fontsource-variable/geist-mono | 5.3.0 | OFL-1.1 |
| @tauri-apps/api | 2.12.4 | Apache-2.0 OR MIT |
| lucide-react | 0.468.0 | ISC |
| react | 19.3.1 | MIT |
| react-dom | 19.3.1 | MIT |
| scheduler | 0.28.0 | MIT |

Build and test tools, not shipped in the app: @babel/code-frame 7.29.7 (MIT), @babel/compat-data 7.29.7 (MIT), @babel/core 7.29.13 (MIT), @babel/generator 7.29.13 (MIT), @babel/helper-compilation-targets 7.29.13 (MIT), @babel/helper-globals 7.29.7 (MIT), @babel/helper-module-imports 7.29.7 (MIT), @babel/helper-module-transforms 7.29.13 (MIT), @babel/helper-plugin-utils 7.29.7 (MIT), @babel/helper-string-parser 7.29.13 (MIT), @babel/helper-validator-identifier 7.29.7 (MIT), @babel/helper-validator-option 7.29.7 (MIT), @babel/helpers 7.29.13 (MIT), @babel/parser 7.29.13 (MIT), @babel/plugin-transform-react-jsx-self 7.29.7 (MIT), @babel/plugin-transform-react-jsx-source 7.29.7 (MIT), @babel/template 7.29.7 (MIT), @babel/traverse 7.29.13 (MIT), @babel/types 7.29.11 (MIT), @esbuild/aix-ppc64 0.25.12 (MIT), @esbuild/android-arm 0.25.12 (MIT), @esbuild/android-arm64 0.25.12 (MIT), @esbuild/android-x64 0.25.12 (MIT), @esbuild/darwin-arm64 0.25.12 (MIT), @esbuild/darwin-x64 0.25.12 (MIT), @esbuild/freebsd-arm64 0.25.12 (MIT), @esbuild/freebsd-x64 0.25.12 (MIT), @esbuild/linux-arm 0.25.12 (MIT), @esbuild/linux-arm64 0.25.12 (MIT), @esbuild/linux-ia32 0.25.12 (MIT), @esbuild/linux-loong64 0.25.12 (MIT), @esbuild/linux-mips64el 0.25.12 (MIT), @esbuild/linux-ppc64 0.25.12 (MIT), @esbuild/linux-riscv64 0.25.12 (MIT), @esbuild/linux-s390x 0.25.12 (MIT), @esbuild/linux-x64 0.25.12 (MIT), @esbuild/netbsd-arm64 0.25.12 (MIT), @esbuild/netbsd-x64 0.25.12 (MIT), @esbuild/openbsd-arm64 0.25.12 (MIT), @esbuild/openbsd-x64 0.25.12 (MIT), @esbuild/openharmony-arm64 0.25.12 (MIT), @esbuild/sunos-x64 0.25.12 (MIT), @esbuild/win32-arm64 0.25.12 (MIT), @esbuild/win32-ia32 0.25.12 (MIT), @esbuild/win32-x64 0.25.12 (MIT), @jridgewell/gen-mapping 0.3.13 (MIT), @jridgewell/remapping 2.3.5 (MIT), @jridgewell/resolve-uri 3.1.2 (MIT), @jridgewell/sourcemap-codec 1.6.0 (MIT), @jridgewell/trace-mapping 0.3.31 (MIT), @playwright/test 1.65.0 (Apache-2.0), @rolldown/pluginutils 1.0.0-beta.27 (MIT), @rollup/rollup-android-arm-eabi 4.68.1 (MIT), @rollup/rollup-android-arm64 4.68.1 (MIT), @rollup/rollup-android-x64 4.68.1 (MIT), @rollup/rollup-darwin-arm64 4.68.1 (MIT), @rollup/rollup-darwin-x64 4.68.1 (MIT), @rollup/rollup-freebsd-arm64 4.68.1 (MIT), @rollup/rollup-freebsd-x64 4.68.1 (MIT), @rollup/rollup-linux-arm-gnueabihf 4.68.1 (MIT), @rollup/rollup-linux-arm-musleabihf 4.68.1 (MIT), @rollup/rollup-linux-arm64-gnu 4.68.1 (MIT), @rollup/rollup-linux-arm64-musl 4.68.1 (MIT), @rollup/rollup-linux-loong64-gnu 4.68.1 (MIT), @rollup/rollup-linux-loong64-musl 4.68.1 (MIT), @rollup/rollup-linux-ppc64-gnu 4.68.1 (MIT), @rollup/rollup-linux-ppc64-musl 4.68.1 (MIT), @rollup/rollup-linux-riscv64-gnu 4.68.1 (MIT), @rollup/rollup-linux-riscv64-musl 4.68.1 (MIT), @rollup/rollup-linux-s390x-gnu 4.68.1 (MIT), @rollup/rollup-linux-x64-gnu 4.68.1 (MIT), @rollup/rollup-linux-x64-musl 4.68.1 (MIT), @rollup/rollup-openbsd-x64 4.68.1 (MIT), @rollup/rollup-openharmony-arm64 4.68.1 (MIT), @rollup/rollup-win32-arm64-msvc 4.68.1 (MIT), @rollup/rollup-win32-ia32-msvc 4.68.1 (MIT), @rollup/rollup-win32-x64-gnu 4.68.1 (MIT), @rollup/rollup-win32-x64-msvc 4.68.1 (MIT), @tauri-apps/cli 2.12.4 (Apache-2.0 OR MIT), @tauri-apps/cli-darwin-arm64 2.12.4 (Apache-2.0 OR MIT), @tauri-apps/cli-darwin-x64 2.12.4 (Apache-2.0 OR MIT), @tauri-apps/cli-linux-arm-gnueabihf 2.12.4 (Apache-2.0 OR MIT), @tauri-apps/cli-linux-arm64-gnu 2.12.4 (Apache-2.0 OR MIT), @tauri-apps/cli-linux-arm64-musl 2.12.4 (Apache-2.0 OR MIT), @tauri-apps/cli-linux-riscv64-gnu 2.12.4 (Apache-2.0 OR MIT), @tauri-apps/cli-linux-x64-gnu 2.12.4 (Apache-2.0 OR MIT), @tauri-apps/cli-linux-x64-musl 2.12.4 (Apache-2.0 OR MIT), @tauri-apps/cli-win32-arm64-msvc 2.12.4 (Apache-2.0 OR MIT), @tauri-apps/cli-win32-ia32-msvc 2.12.4 (Apache-2.0 OR MIT), @tauri-apps/cli-win32-x64-msvc 2.12.4 (Apache-2.0 OR MIT), @types/babel__core 7.20.5 (MIT), @types/babel__generator 7.27.0 (MIT), @types/babel__template 7.4.4 (MIT), @types/babel__traverse 7.28.0 (MIT), @types/estree 1.0.10 (MIT), @types/react 19.3.0 (MIT), @types/react-dom 19.3.0 (MIT), @vitejs/plugin-react 4.7.0 (MIT), baseline-browser-mapping 2.11.35 (Apache-2.0), browserslist 4.29.5 (MIT), caniuse-lite 1.0.30001823 (CC-BY-4.0), convert-source-map 2.0.0 (MIT), csstype 3.2.3 (MIT), debug 4.4.3 (MIT), electron-to-chromium 1.5.478 (ISC), esbuild 0.25.12 (MIT), escalade 3.2.0 (MIT), fdir 6.5.0 (MIT), fsevents 2.3.3 (MIT), gensync 1.0.0-beta.2 (MIT), js-tokens 4.0.0 (MIT), jsesc 3.1.0 (MIT), json5 2.2.3 (MIT), lru-cache 5.1.1 (ISC), ms 2.1.3 (MIT), nanoid 3.3.20 (MIT), node-releases 2.0.63 (MIT), picocolors 1.1.1 (ISC), picomatch 4.0.7 (MIT), playwright 1.65.0 (Apache-2.0), playwright-core 1.65.0 (Apache-2.0), postcss 8.5.32 (MIT), react-refresh 0.17.0 (MIT), rollup 4.68.1 (MIT), semver 6.3.2 (ISC), source-map-js 1.2.2 (BSD-3-Clause), tinyglobby 0.2.17 (MIT), typescript 5.8.3 (Apache-2.0), update-browserslist-db 1.3.4 (MIT), vite 6.4.6 (MIT), yallist 3.1.1 (ISC).

## Rust crates

474 crates from crates.io, including build-time and platform-specific ones, grouped by license.

The MPL-2.0 crates (cssparser 0.37.0, cssparser-macros 0.7.1, dtoa-short 0.3.5, option-ext 0.2.0, selectors 0.38.0) are used unmodified; their source is available from crates.io.

**MIT OR Apache-2.0** (225): android_system_properties 0.1.6, anstream 1.0.0, anstyle 1.0.14, anstyle-parse 1.0.0, anstyle-query 1.1.5, anstyle-wincon 3.0.11, anyhow 1.0.104, async-broadcast 0.7.2, async-recursion 1.2.0, async-trait 0.1.92, base64 0.21.7, base64 0.23.1, bitflags 1.3.2, bitflags 2.13.2, block-buffer 0.10.4, bs58 0.5.1, bumpalo 3.20.3, camino 1.2.6, cargo-platform 0.1.9, cc 1.11.1, cfg-expr 0.15.8, cfg-if 1.0.5, chrono 0.4.45, clap 4.6.7, clap_builder 4.6.7, clap_complete 4.6.11, clap_derive 4.6.7, clap_lex 1.1.1, colorchoice 1.0.5, cookie 0.18.2, core-foundation 0.10.1, core-foundation-sys 0.8.7, core-graphics 0.25.0, core-graphics-types 0.2.0, cpufeatures 0.2.17, crc32fast 1.5.4, crossbeam-channel 0.5.17, crossbeam-utils 0.8.23, crypto-common 0.1.7, deranged 0.5.9, digest 0.10.7, directories 6.0.0, dirs 7.0.0, dirs-sys 0.5.0, displaydoc 0.2.7, dtoa 1.0.11, dyn-clone 1.0.20, embed_plist 1.2.2, enumflags2 0.7.12, enumflags2_derive 0.7.12, erased-serde 0.4.11, errno 0.3.14, fdeflate 0.3.7, field-offset 0.3.6, find-msvc-tools 0.1.15, flate2 1.1.11, foreign-types 0.5.0, foreign-types-macros 0.2.4, foreign-types-shared 0.3.1, form_urlencoded 1.2.2, fs2 0.4.3, futures-channel 0.3.34, futures-core 0.3.34, futures-executor 0.3.34, futures-io 0.3.34, futures-macro 0.3.34, futures-sink 0.3.34, futures-task 0.3.34, futures-util 0.3.34, getrandom 0.3.4, getrandom 0.4.3, glob 0.3.4, hashbrown 0.12.3, hashbrown 0.17.1, heck 0.4.1, heck 0.5.0, hermit-abi 0.5.3, hex 0.4.3, html5ever 0.39.0, http 1.5.0, httparse 1.10.1, iana-time-zone 0.1.65, iana-time-zone-haiku 0.1.2, ident_case 1.0.1, idna 1.1.0, image 0.25.10, io-kit-sys 0.5.0, ipnet 2.12.2, is_terminal_polyfill 1.70.2, itoa 1.0.19, jni 0.21.1, jni-sys 0.3.1, jni-sys 0.4.1, jni-sys-macros 0.4.1, js-sys 0.3.107, json-patch 4.2.0, jsonptr 0.7.1, keyboard-types 0.8.3, libc 0.2.191, lock_api 0.4.14, log 0.4.35, markup5ever 0.39.0, mime 0.3.18, ndk 0.9.0, ndk-context 0.1.1, ndk-sys 0.6.0+11769913, num-conv 0.2.2, num-traits 0.2.19, once_cell 1.21.4, once_cell_polyfill 1.70.2, ordered-stream 0.2.0, parking_lot 0.12.5, parking_lot_core 0.9.12, percent-encoding 2.3.2, piper 0.2.5, pkg-config 0.3.34, png 0.18.1, powerfmt 0.2.1, proc-macro-crate 1.3.1, proc-macro-crate 2.0.2, proc-macro-crate 3.5.0, proc-macro-error 1.0.4, proc-macro-error-attr 1.0.4, proc-macro2 1.0.107, quote 1.0.47, ref-cast 1.0.27, ref-cast-impl 1.0.27, regex 1.13.1, regex-automata 0.4.18, regex-syntax 0.8.11, reqwest 0.13.5, rustc_version 0.4.1, rustversion 1.0.23, scopeguard 1.2.0, semver 1.0.28, serde 1.0.229, serde-untagged 0.1.9, serde_core 1.0.229, serde_derive 1.0.229, serde_derive_internals 0.29.1, serde_derive_internals 0.30.0, serde_json 1.0.154, serde_repr 0.1.21, serde_spanned 0.6.9, serde_spanned 1.1.3, serde_with 3.24.0, serde_with_macros 3.24.0, serialize-to-javascript 0.1.2, serialize-to-javascript-impl 0.1.2, servo_arc 0.4.3, sha2 0.10.9, shlex 2.0.1, signal-hook-registry 1.4.8, siphasher 1.0.4, smallvec 1.16.5, socket2 0.6.5, softbuffer 0.4.8, stable_deref_trait 1.2.1, string_cache 0.9.0, string_cache_codegen 0.6.1, swift-rs 1.0.9, syn 1.0.109, syn 2.0.119, syn 3.0.10, system-deps 6.2.2, tao-macros 0.1.4, tempfile 3.27.0, tendril 0.5.1, thiserror 1.0.69, thiserror 2.0.21, thiserror-impl 1.0.69, thiserror-impl 2.0.21, time 0.3.55, time-core 0.1.9, time-macros 0.2.32, toml 0.8.2, toml 1.2.1+spec-1.1.0, toml_datetime 0.6.3, toml_datetime 1.1.3+spec-1.1.0, toml_edit 0.19.15, toml_edit 0.20.2, toml_edit 0.25.19+spec-1.1.0, toml_parser 1.1.5+spec-1.1.0, toml_writer 1.2.0+spec-1.1.0, tray-icon 0.25.1, typeid 1.0.3, typenum 1.20.1, unicode-segmentation 1.13.3, url 2.5.8, version_check 0.9.5, wasm-bindgen 0.2.130, wasm-bindgen-futures 0.4.80, wasm-bindgen-macro 0.2.130, wasm-bindgen-macro-support 0.2.130, wasm-bindgen-shared 0.2.130, wasm-streams 0.5.0, web-sys 0.3.107, web-time 1.1.0, web_atoms 0.2.6, widestring 1.2.1, winapi 0.3.9, winapi-i686-pc-windows-gnu 0.4.0, winapi-x86_64-pc-windows-gnu 0.4.0, windows 0.62.2, windows-collections 0.3.2, windows-core 0.62.2, windows-future 0.3.2, windows-implement 0.60.2, windows-interface 0.59.3, windows-link 0.2.1, windows-numerics 0.3.1, windows-result 0.4.1, windows-strings 0.5.1, windows-sys 0.45.0, windows-sys 0.61.2, windows-targets 0.42.2, windows-threading 0.2.1, windows-version 0.1.7, windows_aarch64_gnullvm 0.42.2, windows_aarch64_msvc 0.42.2, windows_i686_gnu 0.42.2, windows_i686_msvc 0.42.2, windows_x86_64_gnu 0.42.2, windows_x86_64_gnullvm 0.42.2, windows_x86_64_msvc 0.42.2.

**MIT** (110): atk 0.18.2, atk-sys 0.18.2, block2 0.6.2, bytes 1.12.1, cairo-rs 0.18.5, cairo-sys-rs 0.18.2, cargo_metadata 0.19.2, cfb 0.14.0, combine 4.6.9, darling 0.24.1, darling_core 0.24.1, darling_macro 0.24.1, derive_more 2.1.1, derive_more-impl 2.1.1, dlopen2 0.8.2, dlopen2_derive 0.4.3, dom_query 0.28.0, embed-resource 3.0.12, endi 1.1.1, gdk 0.18.2, gdk-pixbuf 0.18.5, gdk-pixbuf-sys 0.18.0, gdk-sys 0.18.2, gdkwayland-sys 0.18.2, gdkx11 0.18.2, gdkx11-sys 0.18.2, generic-array 0.14.7, gio 0.18.4, gio-sys 0.18.1, glib 0.18.5, glib-macros 0.18.5, glib-sys 0.18.1, gobject-sys 0.18.0, gtk 0.18.2, gtk-sys 0.18.2, gtk3-macros 0.18.2, http-body 1.1.0, http-body-util 0.1.5, hyper 1.12.0, hyper-util 0.1.21, ico 0.6.0, infer 0.22.0, javascriptcore-rs 1.1.2, javascriptcore-rs-sys 1.1.1, libredox 0.1.25, memoffset 0.9.1, mio 1.2.4, new_debug_unreachable 1.0.6, objc2 0.6.5, objc2-encode 4.1.0, objc2-foundation 0.3.2, pango 0.18.3, pango-sys 0.18.0, phf 0.13.1, phf_codegen 0.13.1, phf_generator 0.13.1, phf_macros 0.13.1, phf_shared 0.13.1, plist 1.10.3, precomputed-hash 0.1.1, quick-xml 0.42.0, redox_syscall 0.5.18, redox_users 0.5.3, rfd 0.17.2, schemars 0.8.22, schemars 0.9.0, schemars 1.2.3, schemars_derive 0.8.22, schemars_derive 1.2.3, simd-adler32 0.3.10, slab 0.4.12, soup3 0.5.0, soup3-sys 0.5.0, strsim 0.11.1, synstructure 0.14.0, tauri-winres 0.3.7, tokio 1.54.1, tokio-util 0.7.20, tower 0.5.3, tower-http 0.6.11, tower-layer 0.3.3, tower-service 0.3.3, tracing 0.1.44, tracing-attributes 0.1.31, tracing-core 0.1.36, try-lock 0.2.5, uds_windows 1.2.1, urlpattern 0.6.0, version-compare 0.2.1, vswhom 0.1.0, vswhom-sys 0.1.3, want 0.3.2, webkit2gtk 2.0.2, webkit2gtk-sys 2.0.2, webview2-com 0.39.1, webview2-com-macros 0.8.1, webview2-com-sys 0.39.1, winnow 0.5.40, winnow 1.0.5, winreg 0.56.0, x11 2.21.0, x11-dl 2.21.0, zbus 5.19.0, zbus_macros 5.19.0, zbus_names 4.3.5, zcheapstr 1.2.0, zmij 1.0.23, zvariant 5.15.1, zvariant_derive 5.15.1, zvariant_utils 4.2.0.

**Apache-2.0 OR MIT** (54): async-channel 2.5.0, async-executor 1.14.1, async-io 2.6.0, async-lock 3.5.0, async-process 2.5.0, async-signal 0.2.14, async-task 4.7.1, atomic-waker 1.1.2, autocfg 1.5.1, bit-set 0.8.0, bit-vec 0.8.0, blocking 1.7.0, cargo_toml 1.0.1, cesu8 1.1.0, concurrent-queue 2.5.0, ctor 1.0.13, dbus 0.9.12, equivalent 1.0.3, event-listener 5.4.2, event-listener-strategy 0.5.4, fastrand 2.5.0, fnv 1.0.7, futures-lite 2.6.1, idna_adapter 1.2.2, indexmap 1.9.3, indexmap 2.14.2, libappindicator 0.9.0, libappindicator-sys 0.9.0, libdbus-sys 0.2.7, muda 0.20.0, nusb 0.2.9, parking 2.2.1, pin-project-lite 0.2.17, polling 3.11.0, portable-atomic 1.15.0, portable-atomic-util 0.2.8, rustc-hash 2.1.3, tauri 2.12.5, tauri-build 2.7.4, tauri-codegen 2.7.4, tauri-macros 2.7.4, tauri-plugin 2.7.2, tauri-plugin-dialog 2.8.4, tauri-plugin-fs 2.7.1, tauri-plugin-single-instance 2.5.5, tauri-runtime 2.12.2, tauri-runtime-wry 2.12.3, tauri-utils 2.10.2, utf8_iter 1.1.0, utf8parse 0.2.2, utf_types 1.0.0, uuid 1.29.2, window-vibrancy 0.8.1, wry 0.57.1.

**Unicode-3.0** (18): icu_collections 2.4.0, icu_locale_core 2.4.0, icu_normalizer 2.4.0, icu_normalizer_data 2.4.0, icu_properties 2.4.0, icu_properties_data 2.4.0, icu_provider 2.4.0, litemap 0.8.3, potential_utf 0.1.6, tinystr 0.8.5, writeable 0.6.4, yoke 0.8.4, yoke-derive 0.8.4, zerofrom 0.1.8, zerofrom-derive 0.1.8, zerotrie 0.2.5, zerovec 0.11.9, zerovec-derive 0.11.7.

**Zlib OR Apache-2.0 OR MIT** (17): bytemuck 1.25.2, dispatch2 0.3.1, objc2-app-kit 0.3.2, objc2-cloud-kit 0.3.2, objc2-core-data 0.3.2, objc2-core-foundation 0.3.2, objc2-core-graphics 0.3.2, objc2-core-image 0.3.2, objc2-core-location 0.3.2, objc2-core-text 0.3.2, objc2-exception-helper 0.1.1, objc2-io-surface 0.3.2, objc2-quartz-core 0.3.2, objc2-ui-kit 0.3.2, objc2-user-notifications 0.3.2, objc2-web-kit 0.3.2, tinyvec 1.13.3.

**Unlicense OR MIT** (12): aho-corasick 1.1.5, byteorder 1.5.0, byteorder-lite 0.1.0, jiff 0.2.41, jiff-core 0.1.4, jiff-static 0.2.41, jiff-tzdb 0.1.9, jiff-tzdb-platform 0.1.3, memchr 2.8.3, same-file 1.0.6, walkdir 2.5.0, winapi-util 0.1.11.

**Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT** (5): linux-raw-sys 0.12.1, rustix 1.1.6, wasi 0.11.1+wasi-snapshot-preview1, wasip2 1.0.4+wasi-0.2.12, wit-bindgen 0.57.1.

**MPL-2.0** (5): cssparser 0.37.0, cssparser-macros 0.7.1, dtoa-short 0.3.5, option-ext 0.2.0, selectors 0.38.0.

**0BSD** (2): doctest-file 1.1.1, recvmsg 1.0.0.

**Apache-2.0** (2): sync_wrapper 1.0.2, tao 0.37.2.

**BSD-3-Clause** (2): alloc-no-stdlib 3.0.0, alloc-stdlib 0.3.0.

**BSD-3-Clause OR Apache-2.0** (2): moxcms 0.8.1, pxfm 0.1.30.

**BSD-3-Clause OR MIT OR Apache-2.0** (2): num_enum 0.7.6, num_enum_derive 0.7.6.

**MIT OR Apache-2.0 OR LGPL-2.1-or-later** (2): r-efi 5.3.0, r-efi 6.0.0.

**MIT OR Apache-2.0 OR Zlib** (2): raw-window-handle 0.6.2, static_vcruntime 3.0.0.

**MIT OR Zlib OR Apache-2.0** (2): miniz_oxide 0.8.9, miniz_oxide 0.9.1.

**Zlib** (2): foldhash 0.2.0, zlib-rs 0.6.8.

**(MIT OR Apache-2.0) AND Unicode-3.0** (1): unicode-ident 1.0.27.

**0BSD OR Apache-2.0** (1): interprocess 2.4.8.

**0BSD OR MIT OR Apache-2.0** (1): adler2 2.0.1.

**Apache-2.0 AND MIT** (1): dpi 0.1.2.

**Apache-2.0 WITH LLVM-exception** (1): target-lexicon 0.12.16.

**BSD-2-Clause OR MIT OR Apache-2.0** (1): mach2 0.5.0.

**BSD-3-Clause AND MIT** (1): brotli 9.1.1.

**BSD-3-Clause OR MIT** (1): brotli-decompressor 6.3.1.

**CC0-1.0 OR MIT-0 OR Apache-2.0** (1): dunce 1.0.5.

**ISC** (1): libloading 0.7.4.
