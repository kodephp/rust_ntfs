<!-- SPDX-License-Identifier: Apache-2.0 -->

# Third-party licenses

ntfs-mac itself is licensed under the Apache License, Version 2.0, but
each shipped artifact statically links the Rust crates listed below.
This file is the attribution inventory for those crates.

- Generated: 2026-09-13
- Crates: 525 total, 87 CLI, 488 GUI
- Source: `cargo metadata` (honouring `Cargo.lock`) reduced to packages
  reachable over *runtime* edges; `dev`-only edges are excluded, so the
  list matches what ships rather than the whole test matrix

The `CLI` and `GUI` columns mark which artifact links each crate. Crates
marked in neither column are build-time or transitive dependencies of one
of the two and are listed for completeness.

Regenerate with:

```sh
scripts/gen-third-party-licenses.sh
```

Do not edit by hand — the file is generated. CI verifies it is current
with `scripts/gen-third-party-licenses.sh --check`.

## Summary by license

| License | Crates |
| --- | ---: |
| Apache-2.0 | 2 |
| MIT | 118 |
| MIT OR Apache-2.0 | 251 |
| Apache-2.0 OR MIT | 51 |
| BSD-3-Clause | 2 |
| ISC | 1 |
| Zlib | 2 |
| (MIT OR Apache-2.0) AND Unicode-3.0 | 1 |
| 0BSD OR MIT OR Apache-2.0 | 1 |
| Apache-2.0 / MIT | 1 |
| Apache-2.0 AND MIT | 1 |
| Apache-2.0 WITH LLVM-exception | 1 |
| Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | 5 |
| Apache-2.0/MIT | 3 |
| BSD-3-Clause AND MIT | 1 |
| BSD-3-Clause OR Apache-2.0 | 2 |
| BSD-3-Clause OR MIT OR Apache-2.0 | 2 |
| BSD-3-Clause/MIT | 1 |
| CC0-1.0 OR MIT-0 OR Apache-2.0 | 1 |
| MIT OR Apache-2.0 OR LGPL-2.1-or-later | 2 |
| MIT OR Apache-2.0 OR Zlib | 2 |
| MIT OR Zlib OR Apache-2.0 | 2 |
| MIT/Apache-2.0 | 19 |
| MPL-2.0 | 6 |
| Unicode-3.0 | 18 |
| Unlicense OR MIT | 10 |
| Unlicense/MIT | 2 |
| Zlib OR Apache-2.0 OR MIT | 17 |

## Crates

### Apache-2.0

| Crate | Version | CLI | GUI | Repository |
| --- | --- | :---: | :---: | --- |
| sync_wrapper | 1.0.2 |  | ✓ | [https://github.com/Actyx/sync_wrapper](https://github.com/Actyx/sync_wrapper) |
| tao | 0.35.3 |  | ✓ | [https://github.com/tauri-apps/tao](https://github.com/tauri-apps/tao) |

### MIT

| Crate | Version | CLI | GUI | Repository |
| --- | --- | :---: | :---: | --- |
| atk | 0.18.2 |  | ✓ | [https://github.com/gtk-rs/gtk3-rs](https://github.com/gtk-rs/gtk3-rs) |
| atk-sys | 0.18.2 |  | ✓ | [https://github.com/gtk-rs/gtk3-rs](https://github.com/gtk-rs/gtk3-rs) |
| block2 | 0.6.2 |  | ✓ | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| bytes | 1.12.1 |  | ✓ | [https://github.com/tokio-rs/bytes](https://github.com/tokio-rs/bytes) |
| cairo-rs | 0.18.5 |  | ✓ | [https://github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| cairo-sys-rs | 0.18.2 |  | ✓ | [https://github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| cargo_metadata | 0.19.2 |  | ✓ | [https://github.com/oli-obk/cargo_metadata](https://github.com/oli-obk/cargo_metadata) |
| cfb | 0.7.3 |  | ✓ | [https://github.com/mdsteele/rust-cfb](https://github.com/mdsteele/rust-cfb) |
| combine | 4.6.8 |  | ✓ | [https://github.com/Marwes/combine](https://github.com/Marwes/combine) |
| darling | 0.24.1 |  | ✓ | [https://github.com/TedDriggs/darling](https://github.com/TedDriggs/darling) |
| darling_core | 0.24.1 |  | ✓ | [https://github.com/TedDriggs/darling](https://github.com/TedDriggs/darling) |
| darling_macro | 0.24.1 |  | ✓ | [https://github.com/TedDriggs/darling](https://github.com/TedDriggs/darling) |
| derive_more | 2.1.1 |  | ✓ | [https://github.com/JelteF/derive_more](https://github.com/JelteF/derive_more) |
| derive_more-impl | 2.1.1 |  | ✓ | [https://github.com/JelteF/derive_more](https://github.com/JelteF/derive_more) |
| dlopen2 | 0.8.2 |  | ✓ | [https://github.com/OpenByteDev/dlopen2](https://github.com/OpenByteDev/dlopen2) |
| dlopen2_derive | 0.4.3 |  | ✓ | [https://github.com/OpenByteDev/dlopen2](https://github.com/OpenByteDev/dlopen2) |
| dom_query | 0.27.0 |  | ✓ | [https://github.com/niklak/dom_query](https://github.com/niklak/dom_query) |
| embed-resource | 3.0.11 |  | ✓ | [https://github.com/nabijaczleweli/rust-embed-resource](https://github.com/nabijaczleweli/rust-embed-resource) |
| endi | 1.1.1 |  | ✓ | [https://github.com/zeenix/endi](https://github.com/zeenix/endi) |
| gdk | 0.18.2 |  | ✓ | [https://github.com/gtk-rs/gtk3-rs](https://github.com/gtk-rs/gtk3-rs) |
| gdk-pixbuf | 0.18.5 |  | ✓ | [https://github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| gdk-pixbuf-sys | 0.18.0 |  | ✓ | [https://github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| gdk-sys | 0.18.2 |  | ✓ | [https://github.com/gtk-rs/gtk3-rs](https://github.com/gtk-rs/gtk3-rs) |
| gdkwayland-sys | 0.18.2 |  | ✓ | [https://github.com/gtk-rs/gtk3-rs](https://github.com/gtk-rs/gtk3-rs) |
| gdkx11 | 0.18.2 |  | ✓ | [https://github.com/gtk-rs/gtk3-rs](https://github.com/gtk-rs/gtk3-rs) |
| gdkx11-sys | 0.18.2 |  | ✓ | [https://github.com/gtk-rs/gtk3-rs](https://github.com/gtk-rs/gtk3-rs) |
| generic-array | 0.14.7 |  | ✓ | [https://github.com/fizyk20/generic-array.git](https://github.com/fizyk20/generic-array.git) |
| gio | 0.18.4 |  | ✓ | [https://github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| gio-sys | 0.18.1 |  | ✓ | [https://github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| glib | 0.18.5 |  | ✓ | [https://github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| glib-macros | 0.18.5 |  | ✓ | [https://github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| glib-sys | 0.18.1 |  | ✓ | [https://github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| gobject-sys | 0.18.0 |  | ✓ | [https://github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| gtk | 0.18.2 |  | ✓ | [https://github.com/gtk-rs/gtk3-rs](https://github.com/gtk-rs/gtk3-rs) |
| gtk-sys | 0.18.2 |  | ✓ | [https://github.com/gtk-rs/gtk3-rs](https://github.com/gtk-rs/gtk3-rs) |
| gtk3-macros | 0.18.2 |  | ✓ | [https://github.com/gtk-rs/gtk3-rs](https://github.com/gtk-rs/gtk3-rs) |
| http-body | 1.1.0 |  | ✓ | [https://github.com/hyperium/http-body](https://github.com/hyperium/http-body) |
| http-body-util | 0.1.5 |  | ✓ | [https://github.com/hyperium/http-body](https://github.com/hyperium/http-body) |
| hyper | 1.11.1 |  | ✓ | [https://github.com/hyperium/hyper](https://github.com/hyperium/hyper) |
| hyper-util | 0.1.20 |  | ✓ | [https://github.com/hyperium/hyper-util](https://github.com/hyperium/hyper-util) |
| ico | 0.5.0 |  | ✓ | [https://github.com/mdsteele/rust-ico](https://github.com/mdsteele/rust-ico) |
| infer | 0.19.0 |  | ✓ | [https://github.com/bojand/infer](https://github.com/bojand/infer) |
| javascriptcore-rs | 1.1.2 |  | ✓ | [https://github.com/tauri-apps/javascriptcore-rs](https://github.com/tauri-apps/javascriptcore-rs) |
| javascriptcore-rs-sys | 1.1.1 |  | ✓ | [https://github.com/tauri-apps/javascriptcore-rs](https://github.com/tauri-apps/javascriptcore-rs) |
| libredox | 0.1.24 | ✓ | ✓ | [https://gitlab.redox-os.org/redox-os/libredox.git](https://gitlab.redox-os.org/redox-os/libredox.git) |
| matchers | 0.2.0 | ✓ |  | [https://github.com/hawkw/matchers](https://github.com/hawkw/matchers) |
| memoffset | 0.9.1 |  | ✓ | [https://github.com/Gilnaa/memoffset](https://github.com/Gilnaa/memoffset) |
| mio | 1.2.3 |  | ✓ | [https://github.com/tokio-rs/mio](https://github.com/tokio-rs/mio) |
| new_debug_unreachable | 1.0.6 |  | ✓ | [https://github.com/mbrubeck/rust-debug-unreachable](https://github.com/mbrubeck/rust-debug-unreachable) |
| nu-ansi-term | 0.50.3 | ✓ |  | [https://github.com/nushell/nu-ansi-term](https://github.com/nushell/nu-ansi-term) |
| objc2 | 0.6.4 |  | ✓ | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| objc2-encode | 4.1.0 |  | ✓ | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| objc2-foundation | 0.3.2 |  | ✓ | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| pango | 0.18.3 |  | ✓ | [https://github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| pango-sys | 0.18.0 |  | ✓ | [https://github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| phf | 0.13.1 |  | ✓ | [https://github.com/rust-phf/rust-phf](https://github.com/rust-phf/rust-phf) |
| phf_codegen | 0.13.1 |  | ✓ | [https://github.com/rust-phf/rust-phf](https://github.com/rust-phf/rust-phf) |
| phf_generator | 0.13.1 |  | ✓ | [https://github.com/rust-phf/rust-phf](https://github.com/rust-phf/rust-phf) |
| phf_macros | 0.13.1 |  | ✓ | [https://github.com/rust-phf/rust-phf](https://github.com/rust-phf/rust-phf) |
| phf_shared | 0.13.1 |  | ✓ | [https://github.com/rust-phf/rust-phf](https://github.com/rust-phf/rust-phf) |
| plist | 1.10.1 |  | ✓ | [https://github.com/ebarnard/rust-plist/](https://github.com/ebarnard/rust-plist/) |
| precomputed-hash | 0.1.1 |  | ✓ | [https://github.com/emilio/precomputed-hash](https://github.com/emilio/precomputed-hash) |
| quick-xml | 0.42.0 |  | ✓ | [https://github.com/tafia/quick-xml](https://github.com/tafia/quick-xml) |
| redox_syscall | 0.5.18 |  | ✓ | [https://gitlab.redox-os.org/redox-os/syscall](https://gitlab.redox-os.org/redox-os/syscall) |
| redox_users | 0.4.6 | ✓ |  | [https://gitlab.redox-os.org/redox-os/users](https://gitlab.redox-os.org/redox-os/users) |
| redox_users | 0.5.2 |  | ✓ | [https://gitlab.redox-os.org/redox-os/users](https://gitlab.redox-os.org/redox-os/users) |
| rfd | 0.16.0 |  | ✓ | [https://github.com/PolyMeilex/rfd](https://github.com/PolyMeilex/rfd) |
| schemars | 0.8.22 |  | ✓ | [https://github.com/GREsau/schemars](https://github.com/GREsau/schemars) |
| schemars | 0.9.0 |  | ✓ | [https://github.com/GREsau/schemars](https://github.com/GREsau/schemars) |
| schemars | 1.2.2 |  | ✓ | [https://github.com/GREsau/schemars](https://github.com/GREsau/schemars) |
| schemars_derive | 0.8.22 |  | ✓ | [https://github.com/GREsau/schemars](https://github.com/GREsau/schemars) |
| sharded-slab | 0.1.7 | ✓ |  | [https://github.com/hawkw/sharded-slab](https://github.com/hawkw/sharded-slab) |
| simd-adler32 | 0.3.10 |  | ✓ | [https://github.com/mcountryman/simd-adler32](https://github.com/mcountryman/simd-adler32) |
| slab | 0.4.12 |  | ✓ | [https://github.com/tokio-rs/slab](https://github.com/tokio-rs/slab) |
| soup3 | 0.5.0 |  | ✓ | [https://gitlab.gnome.org/World/Rust/soup3-rs](https://gitlab.gnome.org/World/Rust/soup3-rs) |
| soup3-sys | 0.5.0 |  | ✓ | [https://gitlab.gnome.org/World/Rust/soup3-rs](https://gitlab.gnome.org/World/Rust/soup3-rs) |
| strsim | 0.11.1 | ✓ | ✓ | [https://github.com/rapidfuzz/strsim-rs](https://github.com/rapidfuzz/strsim-rs) |
| synstructure | 0.13.2 |  | ✓ | [https://github.com/mystor/synstructure](https://github.com/mystor/synstructure) |
| tauri-winres | 0.3.6 |  | ✓ | [https://github.com/tauri-apps/winres](https://github.com/tauri-apps/winres) |
| tokio | 1.53.1 |  | ✓ | [https://github.com/tokio-rs/tokio](https://github.com/tokio-rs/tokio) |
| tokio-util | 0.7.19 |  | ✓ | [https://github.com/tokio-rs/tokio](https://github.com/tokio-rs/tokio) |
| tower | 0.5.3 |  | ✓ | [https://github.com/tower-rs/tower](https://github.com/tower-rs/tower) |
| tower-http | 0.6.11 |  | ✓ | [https://github.com/tower-rs/tower-http](https://github.com/tower-rs/tower-http) |
| tower-layer | 0.3.3 |  | ✓ | [https://github.com/tower-rs/tower](https://github.com/tower-rs/tower) |
| tower-service | 0.3.3 |  | ✓ | [https://github.com/tower-rs/tower](https://github.com/tower-rs/tower) |
| tracing | 0.1.44 | ✓ | ✓ | [https://github.com/tokio-rs/tracing](https://github.com/tokio-rs/tracing) |
| tracing-appender | 0.2.5 | ✓ |  | [https://github.com/tokio-rs/tracing](https://github.com/tokio-rs/tracing) |
| tracing-attributes | 0.1.31 | ✓ | ✓ | [https://github.com/tokio-rs/tracing](https://github.com/tokio-rs/tracing) |
| tracing-core | 0.1.36 | ✓ | ✓ | [https://github.com/tokio-rs/tracing](https://github.com/tokio-rs/tracing) |
| tracing-log | 0.2.0 | ✓ |  | [https://github.com/tokio-rs/tracing](https://github.com/tokio-rs/tracing) |
| tracing-subscriber | 0.3.23 | ✓ |  | [https://github.com/tokio-rs/tracing](https://github.com/tokio-rs/tracing) |
| try-lock | 0.2.5 |  | ✓ | [https://github.com/seanmonstar/try-lock](https://github.com/seanmonstar/try-lock) |
| uds_windows | 1.2.1 |  | ✓ | [https://github.com/haraldh/rust_uds_windows](https://github.com/haraldh/rust_uds_windows) |
| urlpattern | 0.3.0 |  | ✓ | [https://github.com/denoland/rust-urlpattern](https://github.com/denoland/rust-urlpattern) |
| valuable | 0.1.1 | ✓ | ✓ | [https://github.com/tokio-rs/valuable](https://github.com/tokio-rs/valuable) |
| version-compare | 0.2.1 |  | ✓ | [https://gitlab.com/timvisee/version-compare](https://gitlab.com/timvisee/version-compare) |
| vswhom | 0.1.0 |  | ✓ | [https://github.com/nabijaczleweli/vswhom.rs](https://github.com/nabijaczleweli/vswhom.rs) |
| vswhom-sys | 0.1.3 |  | ✓ | [https://github.com/nabijaczleweli/vswhom-sys.rs](https://github.com/nabijaczleweli/vswhom-sys.rs) |
| want | 0.3.1 |  | ✓ | [https://github.com/seanmonstar/want](https://github.com/seanmonstar/want) |
| webkit2gtk | 2.0.2 |  | ✓ | [https://github.com/tauri-apps/webkit2gtk-rs](https://github.com/tauri-apps/webkit2gtk-rs) |
| webkit2gtk-sys | 2.0.2 |  | ✓ | [https://github.com/tauri-apps/webkit2gtk-rs](https://github.com/tauri-apps/webkit2gtk-rs) |
| webview2-com | 0.38.2 |  | ✓ | [https://github.com/wravery/webview2-rs](https://github.com/wravery/webview2-rs) |
| webview2-com-macros | 0.8.1 |  | ✓ | [https://github.com/wravery/webview2-rs](https://github.com/wravery/webview2-rs) |
| webview2-com-sys | 0.38.2 |  | ✓ | [https://github.com/wravery/webview2-rs](https://github.com/wravery/webview2-rs) |
| winnow | 0.5.40 |  | ✓ | [https://github.com/winnow-rs/winnow](https://github.com/winnow-rs/winnow) |
| winnow | 0.7.15 |  | ✓ | [https://github.com/winnow-rs/winnow](https://github.com/winnow-rs/winnow) |
| winnow | 1.0.4 |  | ✓ | [https://github.com/winnow-rs/winnow](https://github.com/winnow-rs/winnow) |
| winreg | 0.55.0 |  | ✓ | [https://github.com/gentoo90/winreg-rs](https://github.com/gentoo90/winreg-rs) |
| x11 | 2.21.0 |  | ✓ | [https://github.com/AltF02/x11-rs.git](https://github.com/AltF02/x11-rs.git) |
| x11-dl | 2.21.0 |  | ✓ | [https://github.com/AltF02/x11-rs.git](https://github.com/AltF02/x11-rs.git) |
| zbus | 5.19.0 |  | ✓ | [https://github.com/z-galaxy/zbus/](https://github.com/z-galaxy/zbus/) |
| zbus_macros | 5.19.0 |  | ✓ | [https://github.com/z-galaxy/zbus/](https://github.com/z-galaxy/zbus/) |
| zbus_names | 4.3.4 |  | ✓ | [https://github.com/z-galaxy/zbus/](https://github.com/z-galaxy/zbus/) |
| zcheapstr | 1.1.0 |  | ✓ | [https://github.com/z-galaxy/zcheapstr/](https://github.com/z-galaxy/zcheapstr/) |
| zmij | 1.0.23 | ✓ | ✓ | [https://github.com/dtolnay/zmij](https://github.com/dtolnay/zmij) |
| zvariant | 5.15.0 |  | ✓ | [https://github.com/z-galaxy/zbus/](https://github.com/z-galaxy/zbus/) |
| zvariant_derive | 5.15.0 |  | ✓ | [https://github.com/z-galaxy/zbus/](https://github.com/z-galaxy/zbus/) |
| zvariant_utils | 4.2.0 |  | ✓ | [https://github.com/z-galaxy/zbus/](https://github.com/z-galaxy/zbus/) |

### MIT OR Apache-2.0

| Crate | Version | CLI | GUI | Repository |
| --- | --- | :---: | :---: | --- |
| android_system_properties | 0.1.6 |  | ✓ | [https://github.com/nical/android_system_properties](https://github.com/nical/android_system_properties) |
| anstream | 1.0.0 | ✓ |  | [https://github.com/rust-cli/anstyle.git](https://github.com/rust-cli/anstyle.git) |
| anstyle | 1.0.14 | ✓ |  | [https://github.com/rust-cli/anstyle.git](https://github.com/rust-cli/anstyle.git) |
| anstyle-parse | 1.0.0 | ✓ |  | [https://github.com/rust-cli/anstyle.git](https://github.com/rust-cli/anstyle.git) |
| anstyle-query | 1.1.5 | ✓ |  | [https://github.com/rust-cli/anstyle.git](https://github.com/rust-cli/anstyle.git) |
| anstyle-wincon | 3.0.11 | ✓ |  | [https://github.com/rust-cli/anstyle.git](https://github.com/rust-cli/anstyle.git) |
| anyhow | 1.0.104 | ✓ | ✓ | [https://github.com/dtolnay/anyhow](https://github.com/dtolnay/anyhow) |
| async-broadcast | 0.7.2 |  | ✓ | [https://github.com/smol-rs/async-broadcast](https://github.com/smol-rs/async-broadcast) |
| async-recursion | 1.1.1 |  | ✓ | [https://github.com/dcchut/async-recursion](https://github.com/dcchut/async-recursion) |
| async-trait | 0.1.92 |  | ✓ | [https://github.com/dtolnay/async-trait](https://github.com/dtolnay/async-trait) |
| base64 | 0.21.7 |  | ✓ | [https://github.com/marshallpierce/rust-base64](https://github.com/marshallpierce/rust-base64) |
| base64 | 0.22.1 |  | ✓ | [https://github.com/marshallpierce/rust-base64](https://github.com/marshallpierce/rust-base64) |
| base64 | 0.23.1 |  | ✓ | [https://github.com/marshallpierce/rust-base64](https://github.com/marshallpierce/rust-base64) |
| bitflags | 2.13.2 | ✓ | ✓ | [https://github.com/bitflags/bitflags](https://github.com/bitflags/bitflags) |
| block-buffer | 0.10.4 |  | ✓ | [https://github.com/RustCrypto/utils](https://github.com/RustCrypto/utils) |
| bumpalo | 3.20.3 |  | ✓ | [https://github.com/fitzgen/bumpalo](https://github.com/fitzgen/bumpalo) |
| camino | 1.2.5 |  | ✓ | [https://github.com/camino-rs/camino](https://github.com/camino-rs/camino) |
| cargo-platform | 0.1.9 |  | ✓ | [https://github.com/rust-lang/cargo](https://github.com/rust-lang/cargo) |
| cc | 1.4.5 |  | ✓ | [https://github.com/rust-lang/cc-rs](https://github.com/rust-lang/cc-rs) |
| cfg-expr | 0.15.8 |  | ✓ | [https://github.com/EmbarkStudios/cfg-expr](https://github.com/EmbarkStudios/cfg-expr) |
| cfg-if | 1.0.4 | ✓ | ✓ | [https://github.com/rust-lang/cfg-if](https://github.com/rust-lang/cfg-if) |
| chrono | 0.4.45 |  | ✓ | [https://github.com/chronotope/chrono](https://github.com/chronotope/chrono) |
| clap | 4.6.6 | ✓ |  | [https://github.com/clap-rs/clap](https://github.com/clap-rs/clap) |
| clap_builder | 4.6.6 | ✓ |  | [https://github.com/clap-rs/clap](https://github.com/clap-rs/clap) |
| clap_complete | 4.6.9 | ✓ |  | [https://github.com/clap-rs/clap](https://github.com/clap-rs/clap) |
| clap_derive | 4.6.4 | ✓ |  | [https://github.com/clap-rs/clap](https://github.com/clap-rs/clap) |
| clap_lex | 1.1.0 | ✓ |  | [https://github.com/clap-rs/clap](https://github.com/clap-rs/clap) |
| colorchoice | 1.0.5 | ✓ |  | [https://github.com/rust-cli/anstyle.git](https://github.com/rust-cli/anstyle.git) |
| cookie | 0.18.2 |  | ✓ | [https://github.com/SergioBenitez/cookie-rs](https://github.com/SergioBenitez/cookie-rs) |
| core-foundation | 0.10.1 |  | ✓ | [https://github.com/servo/core-foundation-rs](https://github.com/servo/core-foundation-rs) |
| core-foundation-sys | 0.8.7 |  | ✓ | [https://github.com/servo/core-foundation-rs](https://github.com/servo/core-foundation-rs) |
| core-graphics | 0.25.0 |  | ✓ | [https://github.com/servo/core-foundation-rs](https://github.com/servo/core-foundation-rs) |
| core-graphics-types | 0.2.0 |  | ✓ | [https://github.com/servo/core-foundation-rs](https://github.com/servo/core-foundation-rs) |
| cpufeatures | 0.2.17 |  | ✓ | [https://github.com/RustCrypto/utils](https://github.com/RustCrypto/utils) |
| crc32fast | 1.5.1 |  | ✓ | [https://github.com/srijs/rust-crc32fast](https://github.com/srijs/rust-crc32fast) |
| crossbeam-channel | 0.5.17 | ✓ | ✓ | [https://github.com/crossbeam-rs/crossbeam](https://github.com/crossbeam-rs/crossbeam) |
| crossbeam-utils | 0.8.23 | ✓ | ✓ | [https://github.com/crossbeam-rs/crossbeam](https://github.com/crossbeam-rs/crossbeam) |
| crypto-common | 0.1.7 |  | ✓ | [https://github.com/RustCrypto/traits](https://github.com/RustCrypto/traits) |
| defmt | 1.1.1 |  | ✓ | [https://github.com/knurling-rs/defmt](https://github.com/knurling-rs/defmt) |
| defmt-macros | 1.1.1 |  | ✓ | [https://github.com/knurling-rs/defmt](https://github.com/knurling-rs/defmt) |
| defmt-parser | 1.0.0 |  | ✓ | [https://github.com/knurling-rs/defmt](https://github.com/knurling-rs/defmt) |
| deranged | 0.5.8 | ✓ | ✓ | [https://github.com/jhpratt/deranged](https://github.com/jhpratt/deranged) |
| digest | 0.10.7 |  | ✓ | [https://github.com/RustCrypto/traits](https://github.com/RustCrypto/traits) |
| dirs | 5.0.1 | ✓ |  | [https://github.com/soc/dirs-rs](https://github.com/soc/dirs-rs) |
| dirs | 6.0.0 |  | ✓ | [https://github.com/soc/dirs-rs](https://github.com/soc/dirs-rs) |
| dirs-sys | 0.4.1 | ✓ |  | [https://github.com/dirs-dev/dirs-sys-rs](https://github.com/dirs-dev/dirs-sys-rs) |
| dirs-sys | 0.5.0 |  | ✓ | [https://github.com/dirs-dev/dirs-sys-rs](https://github.com/dirs-dev/dirs-sys-rs) |
| displaydoc | 0.2.7 |  | ✓ | [https://github.com/yaahc/displaydoc](https://github.com/yaahc/displaydoc) |
| dtoa | 1.0.11 |  | ✓ | [https://github.com/dtolnay/dtoa](https://github.com/dtolnay/dtoa) |
| dyn-clone | 1.0.20 |  | ✓ | [https://github.com/dtolnay/dyn-clone](https://github.com/dtolnay/dyn-clone) |
| embed_plist | 1.2.2 |  | ✓ | [https://github.com/nvzqz/embed-plist-rs](https://github.com/nvzqz/embed-plist-rs) |
| enumflags2 | 0.7.12 |  | ✓ | [https://github.com/meithecatte/enumflags2](https://github.com/meithecatte/enumflags2) |
| enumflags2_derive | 0.7.12 |  | ✓ | [https://github.com/meithecatte/enumflags2](https://github.com/meithecatte/enumflags2) |
| erased-serde | 0.4.10 |  | ✓ | [https://github.com/dtolnay/erased-serde](https://github.com/dtolnay/erased-serde) |
| errno | 0.3.14 | ✓ | ✓ | [https://github.com/lambda-fairy/rust-errno](https://github.com/lambda-fairy/rust-errno) |
| fdeflate | 0.3.7 |  | ✓ | [https://github.com/image-rs/fdeflate](https://github.com/image-rs/fdeflate) |
| field-offset | 0.3.6 |  | ✓ | [https://github.com/Diggsey/rust-field-offset](https://github.com/Diggsey/rust-field-offset) |
| find-msvc-tools | 0.1.12 |  | ✓ | [https://github.com/rust-lang/cc-rs](https://github.com/rust-lang/cc-rs) |
| flate2 | 1.1.10 |  | ✓ | [https://github.com/rust-lang/flate2-rs](https://github.com/rust-lang/flate2-rs) |
| form_urlencoded | 1.2.2 |  | ✓ | [https://github.com/servo/rust-url](https://github.com/servo/rust-url) |
| futures-channel | 0.3.34 |  | ✓ | [https://github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| futures-core | 0.3.34 |  | ✓ | [https://github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| futures-executor | 0.3.34 |  | ✓ | [https://github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| futures-io | 0.3.34 |  | ✓ | [https://github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| futures-macro | 0.3.34 |  | ✓ | [https://github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| futures-sink | 0.3.34 |  | ✓ | [https://github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| futures-task | 0.3.34 |  | ✓ | [https://github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| futures-util | 0.3.34 |  | ✓ | [https://github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| getrandom | 0.2.17 | ✓ | ✓ | [https://github.com/rust-random/getrandom](https://github.com/rust-random/getrandom) |
| getrandom | 0.3.4 |  | ✓ | [https://github.com/rust-random/getrandom](https://github.com/rust-random/getrandom) |
| getrandom | 0.4.3 |  | ✓ | [https://github.com/rust-random/getrandom](https://github.com/rust-random/getrandom) |
| glob | 0.3.4 |  | ✓ | [https://github.com/rust-lang/glob](https://github.com/rust-lang/glob) |
| hashbrown | 0.12.3 |  | ✓ | [https://github.com/rust-lang/hashbrown](https://github.com/rust-lang/hashbrown) |
| hashbrown | 0.17.1 |  | ✓ | [https://github.com/rust-lang/hashbrown](https://github.com/rust-lang/hashbrown) |
| heck | 0.4.1 |  | ✓ | [https://github.com/withoutboats/heck](https://github.com/withoutboats/heck) |
| heck | 0.5.0 | ✓ | ✓ | [https://github.com/withoutboats/heck](https://github.com/withoutboats/heck) |
| hermit-abi | 0.5.3 |  | ✓ | [https://github.com/hermit-os/hermit-rs](https://github.com/hermit-os/hermit-rs) |
| hex | 0.4.3 |  | ✓ | [https://github.com/KokaKiwi/rust-hex](https://github.com/KokaKiwi/rust-hex) |
| html5ever | 0.38.0 |  | ✓ | [https://github.com/servo/html5ever](https://github.com/servo/html5ever) |
| http | 1.5.0 |  | ✓ | [https://github.com/hyperium/http](https://github.com/hyperium/http) |
| httparse | 1.10.1 |  | ✓ | [https://github.com/seanmonstar/httparse](https://github.com/seanmonstar/httparse) |
| iana-time-zone | 0.1.65 |  | ✓ | [https://github.com/strawlab/iana-time-zone](https://github.com/strawlab/iana-time-zone) |
| iana-time-zone-haiku | 0.1.2 |  | ✓ | [https://github.com/strawlab/iana-time-zone](https://github.com/strawlab/iana-time-zone) |
| idna | 1.1.0 |  | ✓ | [https://github.com/servo/rust-url/](https://github.com/servo/rust-url/) |
| image | 0.25.10 |  | ✓ | [https://github.com/image-rs/image](https://github.com/image-rs/image) |
| ipnet | 2.12.2 |  | ✓ | [https://github.com/krisprice/ipnet](https://github.com/krisprice/ipnet) |
| is_terminal_polyfill | 1.70.2 | ✓ |  | [https://github.com/polyfill-rs/is_terminal_polyfill](https://github.com/polyfill-rs/is_terminal_polyfill) |
| itoa | 1.0.18 | ✓ | ✓ | [https://github.com/dtolnay/itoa](https://github.com/dtolnay/itoa) |
| jni-sys | 0.3.1 |  | ✓ | [https://github.com/jni-rs/jni-sys](https://github.com/jni-rs/jni-sys) |
| jni-sys | 0.4.1 |  | ✓ | [https://github.com/jni-rs/jni-sys](https://github.com/jni-rs/jni-sys) |
| jni-sys-macros | 0.4.1 |  | ✓ | [https://github.com/jni-rs/jni-sys](https://github.com/jni-rs/jni-sys) |
| js-sys | 0.3.105 |  | ✓ | [https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/js-sys](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/js-sys) |
| jsonptr | 0.6.3 |  | ✓ | [https://github.com/chanced/jsonptr](https://github.com/chanced/jsonptr) |
| keyboard-types | 0.7.0 |  | ✓ | [https://github.com/pyfisch/keyboard-types](https://github.com/pyfisch/keyboard-types) |
| lazy_static | 1.5.0 | ✓ |  | [https://github.com/rust-lang-nursery/lazy-static.rs](https://github.com/rust-lang-nursery/lazy-static.rs) |
| libc | 0.2.189 | ✓ | ✓ | [https://github.com/rust-lang/libc](https://github.com/rust-lang/libc) |
| lock_api | 0.4.14 |  | ✓ | [https://github.com/Amanieu/parking_lot](https://github.com/Amanieu/parking_lot) |
| log | 0.4.34 | ✓ | ✓ | [https://github.com/rust-lang/log](https://github.com/rust-lang/log) |
| markup5ever | 0.38.0 |  | ✓ | [https://github.com/servo/html5ever](https://github.com/servo/html5ever) |
| mime | 0.3.17 |  | ✓ | [https://github.com/hyperium/mime](https://github.com/hyperium/mime) |
| ndk | 0.9.0 |  | ✓ | [https://github.com/rust-mobile/ndk](https://github.com/rust-mobile/ndk) |
| ndk-sys | 0.6.0+11769913 |  | ✓ | [https://github.com/rust-mobile/ndk](https://github.com/rust-mobile/ndk) |
| num-conv | 0.2.2 | ✓ | ✓ | [https://github.com/jhpratt/num-conv](https://github.com/jhpratt/num-conv) |
| num-traits | 0.2.19 |  | ✓ | [https://github.com/rust-num/num-traits](https://github.com/rust-num/num-traits) |
| once_cell | 1.21.4 | ✓ | ✓ | [https://github.com/matklad/once_cell](https://github.com/matklad/once_cell) |
| once_cell_polyfill | 1.70.2 | ✓ |  | [https://github.com/polyfill-rs/once_cell_polyfill](https://github.com/polyfill-rs/once_cell_polyfill) |
| ordered-stream | 0.2.0 |  | ✓ | [https://github.com/danieldg/ordered-stream](https://github.com/danieldg/ordered-stream) |
| parking_lot | 0.12.5 |  | ✓ | [https://github.com/Amanieu/parking_lot](https://github.com/Amanieu/parking_lot) |
| parking_lot_core | 0.9.12 |  | ✓ | [https://github.com/Amanieu/parking_lot](https://github.com/Amanieu/parking_lot) |
| percent-encoding | 2.3.2 |  | ✓ | [https://github.com/servo/rust-url/](https://github.com/servo/rust-url/) |
| piper | 0.2.5 |  | ✓ | [https://github.com/smol-rs/piper](https://github.com/smol-rs/piper) |
| pkg-config | 0.3.34 |  | ✓ | [https://github.com/rust-lang/pkg-config-rs](https://github.com/rust-lang/pkg-config-rs) |
| png | 0.17.16 |  | ✓ | [https://github.com/image-rs/image-png](https://github.com/image-rs/image-png) |
| png | 0.18.1 |  | ✓ | [https://github.com/image-rs/image-png](https://github.com/image-rs/image-png) |
| powerfmt | 0.2.0 | ✓ | ✓ | [https://github.com/jhpratt/powerfmt](https://github.com/jhpratt/powerfmt) |
| proc-macro-crate | 1.3.1 |  | ✓ | [https://github.com/bkchr/proc-macro-crate](https://github.com/bkchr/proc-macro-crate) |
| proc-macro-crate | 2.0.2 |  | ✓ | [https://github.com/bkchr/proc-macro-crate](https://github.com/bkchr/proc-macro-crate) |
| proc-macro-crate | 3.5.0 |  | ✓ | [https://github.com/bkchr/proc-macro-crate](https://github.com/bkchr/proc-macro-crate) |
| proc-macro-error | 1.0.4 |  | ✓ | [https://gitlab.com/CreepySkeleton/proc-macro-error](https://gitlab.com/CreepySkeleton/proc-macro-error) |
| proc-macro-error-attr | 1.0.4 |  | ✓ | [https://gitlab.com/CreepySkeleton/proc-macro-error](https://gitlab.com/CreepySkeleton/proc-macro-error) |
| proc-macro2 | 1.0.107 | ✓ | ✓ | [https://github.com/dtolnay/proc-macro2](https://github.com/dtolnay/proc-macro2) |
| quote | 1.0.47 | ✓ | ✓ | [https://github.com/dtolnay/quote](https://github.com/dtolnay/quote) |
| ref-cast | 1.0.27 |  | ✓ | [https://github.com/dtolnay/ref-cast](https://github.com/dtolnay/ref-cast) |
| ref-cast-impl | 1.0.27 |  | ✓ | [https://github.com/dtolnay/ref-cast](https://github.com/dtolnay/ref-cast) |
| regex | 1.13.1 |  | ✓ | [https://github.com/rust-lang/regex](https://github.com/rust-lang/regex) |
| regex-automata | 0.4.18 | ✓ | ✓ | [https://github.com/rust-lang/regex](https://github.com/rust-lang/regex) |
| regex-syntax | 0.8.11 | ✓ | ✓ | [https://github.com/rust-lang/regex](https://github.com/rust-lang/regex) |
| reqwest | 0.13.5 |  | ✓ | [https://github.com/seanmonstar/reqwest](https://github.com/seanmonstar/reqwest) |
| rustc_version | 0.4.1 |  | ✓ | [https://github.com/djc/rustc-version-rs](https://github.com/djc/rustc-version-rs) |
| rustversion | 1.0.23 |  | ✓ | [https://github.com/dtolnay/rustversion](https://github.com/dtolnay/rustversion) |
| scopeguard | 1.2.0 |  | ✓ | [https://github.com/bluss/scopeguard](https://github.com/bluss/scopeguard) |
| semver | 1.0.28 |  | ✓ | [https://github.com/dtolnay/semver](https://github.com/dtolnay/semver) |
| serde | 1.0.229 | ✓ | ✓ | [https://github.com/serde-rs/serde](https://github.com/serde-rs/serde) |
| serde-untagged | 0.1.9 |  | ✓ | [https://github.com/dtolnay/serde-untagged](https://github.com/dtolnay/serde-untagged) |
| serde_core | 1.0.229 | ✓ | ✓ | [https://github.com/serde-rs/serde](https://github.com/serde-rs/serde) |
| serde_derive | 1.0.229 | ✓ | ✓ | [https://github.com/serde-rs/serde](https://github.com/serde-rs/serde) |
| serde_derive_internals | 0.29.1 |  | ✓ | [https://github.com/serde-rs/serde](https://github.com/serde-rs/serde) |
| serde_json | 1.0.151 | ✓ | ✓ | [https://github.com/serde-rs/json](https://github.com/serde-rs/json) |
| serde_repr | 0.1.21 |  | ✓ | [https://github.com/dtolnay/serde-repr](https://github.com/dtolnay/serde-repr) |
| serde_spanned | 0.6.9 |  | ✓ | [https://github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| serde_spanned | 1.1.1 |  | ✓ | [https://github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| serde_with | 3.23.0 |  | ✓ | [https://github.com/jonasbb/serde_with/](https://github.com/jonasbb/serde_with/) |
| serde_with_macros | 3.23.0 |  | ✓ | [https://github.com/jonasbb/serde_with/](https://github.com/jonasbb/serde_with/) |
| serialize-to-javascript | 0.1.2 |  | ✓ | [https://github.com/chippers/serialize-to-javascript](https://github.com/chippers/serialize-to-javascript) |
| serialize-to-javascript-impl | 0.1.2 |  | ✓ | [https://github.com/chippers/serialize-to-javascript](https://github.com/chippers/serialize-to-javascript) |
| servo_arc | 0.4.3 |  | ✓ | [https://github.com/servo/stylo](https://github.com/servo/stylo) |
| sha2 | 0.10.9 |  | ✓ | [https://github.com/RustCrypto/hashes](https://github.com/RustCrypto/hashes) |
| shlex | 2.0.1 |  | ✓ | [https://github.com/comex/rust-shlex](https://github.com/comex/rust-shlex) |
| signal-hook-registry | 1.4.8 |  | ✓ | [https://github.com/vorner/signal-hook](https://github.com/vorner/signal-hook) |
| smallvec | 1.16.1 | ✓ | ✓ | [https://github.com/servo/rust-smallvec](https://github.com/servo/rust-smallvec) |
| socket2 | 0.6.5 |  | ✓ | [https://github.com/rust-lang/socket2](https://github.com/rust-lang/socket2) |
| softbuffer | 0.4.8 |  | ✓ | [https://github.com/rust-windowing/softbuffer](https://github.com/rust-windowing/softbuffer) |
| stable_deref_trait | 1.2.1 |  | ✓ | [https://github.com/storyyeller/stable_deref_trait](https://github.com/storyyeller/stable_deref_trait) |
| string_cache | 0.9.0 |  | ✓ | [https://github.com/servo/string-cache](https://github.com/servo/string-cache) |
| string_cache_codegen | 0.6.1 |  | ✓ | [https://github.com/servo/string-cache](https://github.com/servo/string-cache) |
| swift-rs | 1.0.8 |  | ✓ | [https://github.com/Brendonovich/swift-rs](https://github.com/Brendonovich/swift-rs) |
| syn | 1.0.109 |  | ✓ | [https://github.com/dtolnay/syn](https://github.com/dtolnay/syn) |
| syn | 2.0.119 | ✓ | ✓ | [https://github.com/dtolnay/syn](https://github.com/dtolnay/syn) |
| syn | 3.0.5 | ✓ | ✓ | [https://github.com/dtolnay/syn](https://github.com/dtolnay/syn) |
| system-deps | 6.2.2 |  | ✓ | [https://github.com/gdesmott/system-deps](https://github.com/gdesmott/system-deps) |
| tao-macros | 0.1.4 |  | ✓ | [https://github.com/tauri-apps/tao](https://github.com/tauri-apps/tao) |
| tempfile | 3.27.0 |  | ✓ | [https://github.com/Stebalien/tempfile](https://github.com/Stebalien/tempfile) |
| tendril | 0.5.1 |  | ✓ | [https://github.com/servo/html5ever](https://github.com/servo/html5ever) |
| terminal_size | 0.4.4 | ✓ |  | [https://github.com/eminence/terminal-size](https://github.com/eminence/terminal-size) |
| thiserror | 1.0.69 | ✓ | ✓ | [https://github.com/dtolnay/thiserror](https://github.com/dtolnay/thiserror) |
| thiserror | 2.0.20 | ✓ | ✓ | [https://github.com/dtolnay/thiserror](https://github.com/dtolnay/thiserror) |
| thiserror-impl | 1.0.69 | ✓ | ✓ | [https://github.com/dtolnay/thiserror](https://github.com/dtolnay/thiserror) |
| thiserror-impl | 2.0.20 | ✓ | ✓ | [https://github.com/dtolnay/thiserror](https://github.com/dtolnay/thiserror) |
| thread_local | 1.1.10 | ✓ |  | [https://github.com/Amanieu/thread_local-rs](https://github.com/Amanieu/thread_local-rs) |
| time | 0.3.55 | ✓ | ✓ | [https://github.com/time-rs/time](https://github.com/time-rs/time) |
| time-core | 0.1.9 | ✓ | ✓ | [https://github.com/time-rs/time](https://github.com/time-rs/time) |
| time-macros | 0.2.32 | ✓ | ✓ | [https://github.com/time-rs/time](https://github.com/time-rs/time) |
| toml | 0.8.2 |  | ✓ | [https://github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| toml | 0.9.12+spec-1.1.0 |  | ✓ | [https://github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| toml | 1.1.6+spec-1.1.0 |  | ✓ | [https://github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| toml_datetime | 0.6.3 |  | ✓ | [https://github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| toml_datetime | 0.7.5+spec-1.1.0 |  | ✓ | [https://github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| toml_datetime | 1.1.1+spec-1.1.0 |  | ✓ | [https://github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| toml_edit | 0.19.15 |  | ✓ | [https://github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| toml_edit | 0.20.2 |  | ✓ | [https://github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| toml_edit | 0.25.15+spec-1.1.0 |  | ✓ | [https://github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| toml_parser | 1.1.3+spec-1.1.0 |  | ✓ | [https://github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| toml_writer | 1.1.2+spec-1.1.0 |  | ✓ | [https://github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| tray-icon | 0.24.2 |  | ✓ | [https://github.com/tauri-apps/tray-icon](https://github.com/tauri-apps/tray-icon) |
| typeid | 1.0.3 |  | ✓ | [https://github.com/dtolnay/typeid](https://github.com/dtolnay/typeid) |
| typenum | 1.20.1 |  | ✓ | [https://github.com/paholg/typenum](https://github.com/paholg/typenum) |
| unicode-segmentation | 1.13.3 |  | ✓ | [https://github.com/unicode-rs/unicode-segmentation](https://github.com/unicode-rs/unicode-segmentation) |
| url | 2.5.8 |  | ✓ | [https://github.com/servo/rust-url](https://github.com/servo/rust-url) |
| wasm-bindgen | 0.2.128 |  | ✓ | [https://github.com/wasm-bindgen/wasm-bindgen](https://github.com/wasm-bindgen/wasm-bindgen) |
| wasm-bindgen-futures | 0.4.78 |  | ✓ | [https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/futures](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/futures) |
| wasm-bindgen-macro | 0.2.128 |  | ✓ | [https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/macro](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/macro) |
| wasm-bindgen-macro-support | 0.2.128 |  | ✓ | [https://github.com/wasm-bindgen/wasm-bindgen/tree/main/crates/macro-support](https://github.com/wasm-bindgen/wasm-bindgen/tree/main/crates/macro-support) |
| wasm-bindgen-shared | 0.2.128 |  | ✓ | [https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/shared](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/shared) |
| wasm-streams | 0.5.0 |  | ✓ | [https://github.com/MattiasBuelens/wasm-streams/](https://github.com/MattiasBuelens/wasm-streams/) |
| web-sys | 0.3.105 |  | ✓ | [https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/web-sys](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/web-sys) |
| web_atoms | 0.2.6 |  | ✓ | [https://github.com/servo/html5ever](https://github.com/servo/html5ever) |
| windows | 0.61.3 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-collections | 0.2.0 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-core | 0.61.2 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-core | 0.62.2 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-future | 0.2.1 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-implement | 0.60.2 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-interface | 0.59.3 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-link | 0.1.3 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-link | 0.2.1 | ✓ | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-numerics | 0.2.0 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-result | 0.3.4 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-result | 0.4.1 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-strings | 0.4.2 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-strings | 0.5.1 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-sys | 0.45.0 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-sys | 0.48.0 | ✓ |  | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-sys | 0.59.0 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-sys | 0.60.2 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-sys | 0.61.2 | ✓ | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-targets | 0.42.2 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-targets | 0.48.5 | ✓ |  | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-targets | 0.52.6 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-targets | 0.53.5 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-threading | 0.1.0 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows-version | 0.1.7 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_aarch64_gnullvm | 0.42.2 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_aarch64_gnullvm | 0.48.5 | ✓ |  | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_aarch64_gnullvm | 0.52.6 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_aarch64_gnullvm | 0.53.1 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_aarch64_msvc | 0.42.2 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_aarch64_msvc | 0.48.5 | ✓ |  | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_aarch64_msvc | 0.52.6 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_aarch64_msvc | 0.53.1 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_i686_gnu | 0.42.2 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_i686_gnu | 0.48.5 | ✓ |  | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_i686_gnu | 0.52.6 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_i686_gnu | 0.53.1 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_i686_gnullvm | 0.52.6 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_i686_gnullvm | 0.53.1 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_i686_msvc | 0.42.2 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_i686_msvc | 0.48.5 | ✓ |  | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_i686_msvc | 0.52.6 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_i686_msvc | 0.53.1 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_x86_64_gnu | 0.42.2 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_x86_64_gnu | 0.48.5 | ✓ |  | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_x86_64_gnu | 0.52.6 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_x86_64_gnu | 0.53.1 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_x86_64_gnullvm | 0.42.2 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_x86_64_gnullvm | 0.48.5 | ✓ |  | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_x86_64_gnullvm | 0.52.6 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_x86_64_gnullvm | 0.53.1 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_x86_64_msvc | 0.42.2 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_x86_64_msvc | 0.48.5 | ✓ |  | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_x86_64_msvc | 0.52.6 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| windows_x86_64_msvc | 0.53.1 |  | ✓ | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |

### Apache-2.0 OR MIT

| Crate | Version | CLI | GUI | Repository |
| --- | --- | :---: | :---: | --- |
| async-channel | 2.5.0 |  | ✓ | [https://github.com/smol-rs/async-channel](https://github.com/smol-rs/async-channel) |
| async-executor | 1.14.0 |  | ✓ | [https://github.com/smol-rs/async-executor](https://github.com/smol-rs/async-executor) |
| async-io | 2.6.0 |  | ✓ | [https://github.com/smol-rs/async-io](https://github.com/smol-rs/async-io) |
| async-lock | 3.4.2 |  | ✓ | [https://github.com/smol-rs/async-lock](https://github.com/smol-rs/async-lock) |
| async-process | 2.5.0 |  | ✓ | [https://github.com/smol-rs/async-process](https://github.com/smol-rs/async-process) |
| async-signal | 0.2.14 |  | ✓ | [https://github.com/smol-rs/async-signal](https://github.com/smol-rs/async-signal) |
| async-task | 4.7.1 |  | ✓ | [https://github.com/smol-rs/async-task](https://github.com/smol-rs/async-task) |
| atomic-waker | 1.1.2 |  | ✓ | [https://github.com/smol-rs/atomic-waker](https://github.com/smol-rs/atomic-waker) |
| autocfg | 1.5.1 |  | ✓ | [https://github.com/cuviper/autocfg](https://github.com/cuviper/autocfg) |
| bit-set | 0.8.0 |  | ✓ | [https://github.com/contain-rs/bit-set](https://github.com/contain-rs/bit-set) |
| bit-vec | 0.8.0 |  | ✓ | [https://github.com/contain-rs/bit-vec](https://github.com/contain-rs/bit-vec) |
| blocking | 1.7.0 |  | ✓ | [https://github.com/smol-rs/blocking](https://github.com/smol-rs/blocking) |
| cargo_toml | 0.22.3 |  | ✓ | [https://gitlab.com/lib.rs/cargo_toml](https://gitlab.com/lib.rs/cargo_toml) |
| concurrent-queue | 2.5.0 |  | ✓ | [https://github.com/smol-rs/concurrent-queue](https://github.com/smol-rs/concurrent-queue) |
| ctor | 0.8.0 |  | ✓ | [https://github.com/mmastrac/rust-ctor](https://github.com/mmastrac/rust-ctor) |
| ctor-proc-macro | 0.0.7 |  | ✓ | [https://github.com/mmastrac/rust-ctor](https://github.com/mmastrac/rust-ctor) |
| dtor | 0.3.0 |  | ✓ | [https://github.com/mmastrac/rust-ctor](https://github.com/mmastrac/rust-ctor) |
| dtor-proc-macro | 0.0.6 |  | ✓ | [https://github.com/mmastrac/rust-ctor](https://github.com/mmastrac/rust-ctor) |
| equivalent | 1.0.2 |  | ✓ | [https://github.com/indexmap-rs/equivalent](https://github.com/indexmap-rs/equivalent) |
| event-listener | 5.4.2 |  | ✓ | [https://github.com/smol-rs/event-listener](https://github.com/smol-rs/event-listener) |
| event-listener-strategy | 0.5.4 |  | ✓ | [https://github.com/smol-rs/event-listener-strategy](https://github.com/smol-rs/event-listener-strategy) |
| fastrand | 2.5.0 |  | ✓ | [https://github.com/smol-rs/fastrand](https://github.com/smol-rs/fastrand) |
| futures-lite | 2.6.1 |  | ✓ | [https://github.com/smol-rs/futures-lite](https://github.com/smol-rs/futures-lite) |
| idna_adapter | 1.2.2 |  | ✓ | [https://github.com/hsivonen/idna_adapter](https://github.com/hsivonen/idna_adapter) |
| indexmap | 1.9.3 |  | ✓ | [https://github.com/bluss/indexmap](https://github.com/bluss/indexmap) |
| indexmap | 2.14.2 |  | ✓ | [https://github.com/indexmap-rs/indexmap](https://github.com/indexmap-rs/indexmap) |
| libappindicator | 0.9.0 |  | ✓ | — |
| libappindicator-sys | 0.9.0 |  | ✓ | — |
| muda | 0.19.3 |  | ✓ | [https://github.com/tauri-apps/muda](https://github.com/tauri-apps/muda) |
| parking | 2.2.1 |  | ✓ | [https://github.com/smol-rs/parking](https://github.com/smol-rs/parking) |
| pin-project-lite | 0.2.17 | ✓ | ✓ | [https://github.com/taiki-e/pin-project-lite](https://github.com/taiki-e/pin-project-lite) |
| polling | 3.11.0 |  | ✓ | [https://github.com/smol-rs/polling](https://github.com/smol-rs/polling) |
| portable-atomic | 1.15.0 |  | ✓ | [https://github.com/taiki-e/portable-atomic](https://github.com/taiki-e/portable-atomic) |
| portable-atomic-util | 0.2.8 |  | ✓ | [https://github.com/taiki-e/portable-atomic-util](https://github.com/taiki-e/portable-atomic-util) |
| rustc-hash | 2.1.3 |  | ✓ | [https://github.com/rust-lang/rustc-hash](https://github.com/rust-lang/rustc-hash) |
| tauri | 2.11.5 |  | ✓ | [https://github.com/tauri-apps/tauri](https://github.com/tauri-apps/tauri) |
| tauri-build | 2.6.3 |  | ✓ | [https://github.com/tauri-apps/tauri](https://github.com/tauri-apps/tauri) |
| tauri-codegen | 2.6.3 |  | ✓ | [https://github.com/tauri-apps/tauri](https://github.com/tauri-apps/tauri) |
| tauri-macros | 2.6.3 |  | ✓ | [https://github.com/tauri-apps/tauri](https://github.com/tauri-apps/tauri) |
| tauri-plugin | 2.6.3 |  | ✓ | [https://github.com/tauri-apps/tauri](https://github.com/tauri-apps/tauri) |
| tauri-plugin-dialog | 2.7.3 |  | ✓ | [https://github.com/tauri-apps/plugins-workspace](https://github.com/tauri-apps/plugins-workspace) |
| tauri-plugin-fs | 2.5.2 |  | ✓ | [https://github.com/tauri-apps/plugins-workspace](https://github.com/tauri-apps/plugins-workspace) |
| tauri-plugin-single-instance | 2.4.4 |  | ✓ | [https://github.com/tauri-apps/plugins-workspace](https://github.com/tauri-apps/plugins-workspace) |
| tauri-runtime | 2.11.3 |  | ✓ | [https://github.com/tauri-apps/tauri](https://github.com/tauri-apps/tauri) |
| tauri-runtime-wry | 2.11.4 |  | ✓ | [https://github.com/tauri-apps/tauri](https://github.com/tauri-apps/tauri) |
| tauri-utils | 2.9.3 |  | ✓ | [https://github.com/tauri-apps/tauri](https://github.com/tauri-apps/tauri) |
| utf8_iter | 1.0.4 |  | ✓ | [https://github.com/hsivonen/utf8_iter](https://github.com/hsivonen/utf8_iter) |
| utf8parse | 0.2.2 | ✓ |  | [https://github.com/alacritty/vte](https://github.com/alacritty/vte) |
| uuid | 1.26.1 |  | ✓ | [https://github.com/uuid-rs/uuid](https://github.com/uuid-rs/uuid) |
| window-vibrancy | 0.6.0 |  | ✓ | [https://github.com/tauri-apps/tauri-plugin-vibrancy](https://github.com/tauri-apps/tauri-plugin-vibrancy) |
| wry | 0.55.1 |  | ✓ | [https://github.com/tauri-apps/wry](https://github.com/tauri-apps/wry) |

### BSD-3-Clause

| Crate | Version | CLI | GUI | Repository |
| --- | --- | :---: | :---: | --- |
| alloc-no-stdlib | 2.0.4 |  | ✓ | [https://github.com/dropbox/rust-alloc-no-stdlib](https://github.com/dropbox/rust-alloc-no-stdlib) |
| alloc-stdlib | 0.2.4 |  | ✓ | [https://github.com/dropbox/rust-alloc-no-stdlib](https://github.com/dropbox/rust-alloc-no-stdlib) |

### ISC

| Crate | Version | CLI | GUI | Repository |
| --- | --- | :---: | :---: | --- |
| libloading | 0.7.4 |  | ✓ | [https://github.com/nagisa/rust_libloading/](https://github.com/nagisa/rust_libloading/) |

### Zlib

| Crate | Version | CLI | GUI | Repository |
| --- | --- | :---: | :---: | --- |
| foldhash | 0.2.0 |  | ✓ | [https://github.com/orlp/foldhash](https://github.com/orlp/foldhash) |
| zlib-rs | 0.6.7 |  | ✓ | [https://github.com/trifectatechfoundation/zlib-rs](https://github.com/trifectatechfoundation/zlib-rs) |

### (MIT OR Apache-2.0) AND Unicode-3.0

| Crate | Version | CLI | GUI | Repository |
| --- | --- | :---: | :---: | --- |
| unicode-ident | 1.0.24 | ✓ | ✓ | [https://github.com/dtolnay/unicode-ident](https://github.com/dtolnay/unicode-ident) |

### 0BSD OR MIT OR Apache-2.0

| Crate | Version | CLI | GUI | Repository |
| --- | --- | :---: | :---: | --- |
| adler2 | 2.0.1 |  | ✓ | [https://github.com/oyvindln/adler2](https://github.com/oyvindln/adler2) |

### Apache-2.0 / MIT

| Crate | Version | CLI | GUI | Repository |
| --- | --- | :---: | :---: | --- |
| fnv | 1.0.7 |  | ✓ | [https://github.com/servo/rust-fnv](https://github.com/servo/rust-fnv) |

### Apache-2.0 AND MIT

| Crate | Version | CLI | GUI | Repository |
| --- | --- | :---: | :---: | --- |
| dpi | 0.1.2 |  | ✓ | [https://github.com/rust-windowing/winit](https://github.com/rust-windowing/winit) |

### Apache-2.0 WITH LLVM-exception

| Crate | Version | CLI | GUI | Repository |
| --- | --- | :---: | :---: | --- |
| target-lexicon | 0.12.16 |  | ✓ | [https://github.com/bytecodealliance/target-lexicon](https://github.com/bytecodealliance/target-lexicon) |

### Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT

| Crate | Version | CLI | GUI | Repository |
| --- | --- | :---: | :---: | --- |
| linux-raw-sys | 0.12.1 | ✓ | ✓ | [https://github.com/sunfishcode/linux-raw-sys](https://github.com/sunfishcode/linux-raw-sys) |
| rustix | 1.1.4 | ✓ | ✓ | [https://github.com/bytecodealliance/rustix](https://github.com/bytecodealliance/rustix) |
| wasi | 0.11.1+wasi-snapshot-preview1 | ✓ | ✓ | [https://github.com/bytecodealliance/wasi](https://github.com/bytecodealliance/wasi) |
| wasip2 | 1.0.4+wasi-0.2.12 |  | ✓ | [https://github.com/bytecodealliance/wasi-rs](https://github.com/bytecodealliance/wasi-rs) |
| wit-bindgen | 0.57.1 |  | ✓ | [https://github.com/bytecodealliance/wit-bindgen](https://github.com/bytecodealliance/wit-bindgen) |

### Apache-2.0/MIT

| Crate | Version | CLI | GUI | Repository |
| --- | --- | :---: | :---: | --- |
| cesu8 | 1.1.0 |  | ✓ | [https://github.com/emk/cesu8-rs](https://github.com/emk/cesu8-rs) |
| dbus | 0.9.12 |  | ✓ | [https://github.com/diwic/dbus-rs](https://github.com/diwic/dbus-rs) |
| libdbus-sys | 0.2.7 |  | ✓ | [https://github.com/diwic/dbus-rs](https://github.com/diwic/dbus-rs) |

### BSD-3-Clause AND MIT

| Crate | Version | CLI | GUI | Repository |
| --- | --- | :---: | :---: | --- |
| brotli | 8.0.4 |  | ✓ | [https://github.com/dropbox/rust-brotli](https://github.com/dropbox/rust-brotli) |

### BSD-3-Clause OR Apache-2.0

| Crate | Version | CLI | GUI | Repository |
| --- | --- | :---: | :---: | --- |
| moxcms | 0.8.1 |  | ✓ | [https://github.com/awxkee/moxcms.git](https://github.com/awxkee/moxcms.git) |
| pxfm | 0.1.30 |  | ✓ | [https://github.com/awxkee/pxfm](https://github.com/awxkee/pxfm) |

### BSD-3-Clause OR MIT OR Apache-2.0

| Crate | Version | CLI | GUI | Repository |
| --- | --- | :---: | :---: | --- |
| num_enum | 0.7.6 |  | ✓ | [https://github.com/illicitonion/num_enum](https://github.com/illicitonion/num_enum) |
| num_enum_derive | 0.7.6 |  | ✓ | [https://github.com/illicitonion/num_enum](https://github.com/illicitonion/num_enum) |

### BSD-3-Clause/MIT

| Crate | Version | CLI | GUI | Repository |
| --- | --- | :---: | :---: | --- |
| brotli-decompressor | 5.0.3 |  | ✓ | [https://github.com/dropbox/rust-brotli-decompressor](https://github.com/dropbox/rust-brotli-decompressor) |

### CC0-1.0 OR MIT-0 OR Apache-2.0

| Crate | Version | CLI | GUI | Repository |
| --- | --- | :---: | :---: | --- |
| dunce | 1.0.5 |  | ✓ | [https://gitlab.com/kornelski/dunce](https://gitlab.com/kornelski/dunce) |

### MIT OR Apache-2.0 OR LGPL-2.1-or-later

| Crate | Version | CLI | GUI | Repository |
| --- | --- | :---: | :---: | --- |
| r-efi | 5.3.0 |  | ✓ | [https://github.com/r-efi/r-efi](https://github.com/r-efi/r-efi) |
| r-efi | 6.0.0 |  | ✓ | [https://github.com/r-efi/r-efi](https://github.com/r-efi/r-efi) |

### MIT OR Apache-2.0 OR Zlib

| Crate | Version | CLI | GUI | Repository |
| --- | --- | :---: | :---: | --- |
| raw-window-handle | 0.6.2 |  | ✓ | [https://github.com/rust-windowing/raw-window-handle](https://github.com/rust-windowing/raw-window-handle) |
| tinyvec_macros | 0.1.1 |  | ✓ | [https://github.com/Soveu/tinyvec_macros](https://github.com/Soveu/tinyvec_macros) |

### MIT OR Zlib OR Apache-2.0

| Crate | Version | CLI | GUI | Repository |
| --- | --- | :---: | :---: | --- |
| miniz_oxide | 0.8.9 |  | ✓ | [https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide](https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide) |
| miniz_oxide | 0.9.1 |  | ✓ | [https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide](https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide) |

### MIT/Apache-2.0

| Crate | Version | CLI | GUI | Repository |
| --- | --- | :---: | :---: | --- |
| bitflags | 1.3.2 |  | ✓ | [https://github.com/bitflags/bitflags](https://github.com/bitflags/bitflags) |
| bs58 | 0.5.1 |  | ✓ | [https://github.com/Nullus157/bs58-rs](https://github.com/Nullus157/bs58-rs) |
| foreign-types | 0.5.0 |  | ✓ | [https://github.com/sfackler/foreign-types](https://github.com/sfackler/foreign-types) |
| foreign-types-macros | 0.2.4 |  | ✓ | [https://github.com/sfackler/foreign-types](https://github.com/sfackler/foreign-types) |
| foreign-types-shared | 0.3.1 |  | ✓ | [https://github.com/sfackler/foreign-types](https://github.com/sfackler/foreign-types) |
| ident_case | 1.0.1 |  | ✓ | [https://github.com/TedDriggs/ident_case](https://github.com/TedDriggs/ident_case) |
| jni | 0.21.1 |  | ✓ | [https://github.com/jni-rs/jni-rs](https://github.com/jni-rs/jni-rs) |
| json-patch | 3.0.1 |  | ✓ | [https://github.com/idubrov/json-patch](https://github.com/idubrov/json-patch) |
| siphasher | 1.0.3 |  | ✓ | [https://github.com/jedisct1/rust-siphash](https://github.com/jedisct1/rust-siphash) |
| symlink | 0.1.0 | ✓ |  | [https://gitlab.com/chris-morgan/symlink](https://gitlab.com/chris-morgan/symlink) |
| unic-char-property | 0.9.0 |  | ✓ | [https://github.com/open-i18n/rust-unic/](https://github.com/open-i18n/rust-unic/) |
| unic-char-range | 0.9.0 |  | ✓ | [https://github.com/open-i18n/rust-unic/](https://github.com/open-i18n/rust-unic/) |
| unic-common | 0.9.0 |  | ✓ | [https://github.com/open-i18n/rust-unic/](https://github.com/open-i18n/rust-unic/) |
| unic-ucd-ident | 0.9.0 |  | ✓ | [https://github.com/open-i18n/rust-unic/](https://github.com/open-i18n/rust-unic/) |
| unic-ucd-version | 0.9.0 |  | ✓ | [https://github.com/open-i18n/rust-unic/](https://github.com/open-i18n/rust-unic/) |
| version_check | 0.9.5 |  | ✓ | [https://github.com/SergioBenitez/version_check](https://github.com/SergioBenitez/version_check) |
| winapi | 0.3.9 |  | ✓ | [https://github.com/retep998/winapi-rs](https://github.com/retep998/winapi-rs) |
| winapi-i686-pc-windows-gnu | 0.4.0 |  | ✓ | [https://github.com/retep998/winapi-rs](https://github.com/retep998/winapi-rs) |
| winapi-x86_64-pc-windows-gnu | 0.4.0 |  | ✓ | [https://github.com/retep998/winapi-rs](https://github.com/retep998/winapi-rs) |

### MPL-2.0

| Crate | Version | CLI | GUI | Repository |
| --- | --- | :---: | :---: | --- |
| colored | 3.1.1 | ✓ |  | [https://github.com/mackwic/colored](https://github.com/mackwic/colored) |
| cssparser | 0.36.0 |  | ✓ | [https://github.com/servo/rust-cssparser](https://github.com/servo/rust-cssparser) |
| cssparser-macros | 0.6.1 |  | ✓ | [https://github.com/servo/rust-cssparser](https://github.com/servo/rust-cssparser) |
| dtoa-short | 0.3.5 |  | ✓ | [https://github.com/upsuper/dtoa-short](https://github.com/upsuper/dtoa-short) |
| option-ext | 0.2.0 | ✓ | ✓ | [https://github.com/soc/option-ext.git](https://github.com/soc/option-ext.git) |
| selectors | 0.36.1 |  | ✓ | [https://github.com/servo/stylo](https://github.com/servo/stylo) |

### Unicode-3.0

| Crate | Version | CLI | GUI | Repository |
| --- | --- | :---: | :---: | --- |
| icu_collections | 2.3.0 |  | ✓ | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| icu_locale_core | 2.3.0 |  | ✓ | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| icu_normalizer | 2.3.0 |  | ✓ | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| icu_normalizer_data | 2.3.0 |  | ✓ | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| icu_properties | 2.3.0 |  | ✓ | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| icu_properties_data | 2.3.0 |  | ✓ | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| icu_provider | 2.3.1 |  | ✓ | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| litemap | 0.8.3 |  | ✓ | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| potential_utf | 0.1.6 |  | ✓ | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| tinystr | 0.8.4 |  | ✓ | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| writeable | 0.6.4 |  | ✓ | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| yoke | 0.8.3 |  | ✓ | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| yoke-derive | 0.8.2 |  | ✓ | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| zerofrom | 0.1.8 |  | ✓ | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| zerofrom-derive | 0.1.7 |  | ✓ | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| zerotrie | 0.2.5 |  | ✓ | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| zerovec | 0.11.8 |  | ✓ | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| zerovec-derive | 0.11.6 |  | ✓ | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |

### Unlicense OR MIT

| Crate | Version | CLI | GUI | Repository |
| --- | --- | :---: | :---: | --- |
| aho-corasick | 1.1.5 | ✓ | ✓ | [https://github.com/BurntSushi/aho-corasick](https://github.com/BurntSushi/aho-corasick) |
| byteorder | 1.5.0 |  | ✓ | [https://github.com/BurntSushi/byteorder](https://github.com/BurntSushi/byteorder) |
| byteorder-lite | 0.1.0 |  | ✓ | [https://github.com/image-rs/byteorder-lite](https://github.com/image-rs/byteorder-lite) |
| jiff | 0.2.35 |  | ✓ | [https://github.com/BurntSushi/jiff](https://github.com/BurntSushi/jiff) |
| jiff-core | 0.1.0 |  | ✓ | [https://github.com/BurntSushi/jiff](https://github.com/BurntSushi/jiff) |
| jiff-static | 0.2.35 |  | ✓ | [https://github.com/BurntSushi/jiff](https://github.com/BurntSushi/jiff) |
| jiff-tzdb | 0.1.8 |  | ✓ | [https://github.com/BurntSushi/jiff](https://github.com/BurntSushi/jiff) |
| jiff-tzdb-platform | 0.1.3 |  | ✓ | [https://github.com/BurntSushi/jiff](https://github.com/BurntSushi/jiff) |
| memchr | 2.8.3 | ✓ | ✓ | [https://github.com/BurntSushi/memchr](https://github.com/BurntSushi/memchr) |
| winapi-util | 0.1.11 |  | ✓ | [https://github.com/BurntSushi/winapi-util](https://github.com/BurntSushi/winapi-util) |

### Unlicense/MIT

| Crate | Version | CLI | GUI | Repository |
| --- | --- | :---: | :---: | --- |
| same-file | 1.0.6 |  | ✓ | [https://github.com/BurntSushi/same-file](https://github.com/BurntSushi/same-file) |
| walkdir | 2.5.0 |  | ✓ | [https://github.com/BurntSushi/walkdir](https://github.com/BurntSushi/walkdir) |

### Zlib OR Apache-2.0 OR MIT

| Crate | Version | CLI | GUI | Repository |
| --- | --- | :---: | :---: | --- |
| bytemuck | 1.25.2 |  | ✓ | [https://github.com/Lokathor/bytemuck](https://github.com/Lokathor/bytemuck) |
| dispatch2 | 0.3.1 |  | ✓ | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| objc2-app-kit | 0.3.2 |  | ✓ | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| objc2-cloud-kit | 0.3.2 |  | ✓ | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| objc2-core-data | 0.3.2 |  | ✓ | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| objc2-core-foundation | 0.3.2 |  | ✓ | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| objc2-core-graphics | 0.3.2 |  | ✓ | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| objc2-core-image | 0.3.2 |  | ✓ | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| objc2-core-location | 0.3.2 |  | ✓ | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| objc2-core-text | 0.3.2 |  | ✓ | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| objc2-exception-helper | 0.1.1 |  | ✓ | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| objc2-io-surface | 0.3.2 |  | ✓ | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| objc2-quartz-core | 0.3.2 |  | ✓ | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| objc2-ui-kit | 0.3.2 |  | ✓ | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| objc2-user-notifications | 0.3.2 |  | ✓ | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| objc2-web-kit | 0.3.2 |  | ✓ | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| tinyvec | 1.13.2 |  | ✓ | [https://github.com/Lokathor/tinyvec](https://github.com/Lokathor/tinyvec) |

Total: 525 crates.
