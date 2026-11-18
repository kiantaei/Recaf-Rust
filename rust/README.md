# Recaf-Rust — Rust workspace

Rust components for the Recaf-Rust project, managed with Cargo and rendered
with [GPUI](https://crates.io/crates/gpui). The Java core of Recaf
(`recaf-core`, `recaf-ui`) is **not** modified by this workspace.

## Crates

| Crate | Kind | Purpose |
| --- | --- | --- |
| `recaf-classfile` | library | Dependency-free JVM class file parser: constant pool (all 17 tag types of the JVM spec), class file versions, access flags, interfaces, field/method summaries and attribute names. Strict parsing with byte-offset errors, no panics on malformed input. |
| `recaf-gpui` | binary (`recaf-rust`) | GPUI desktop class file inspector: dark IDE-style window with class metadata and a scrollable, color-coded constant pool view. Parses an embedded demo class or any `.class` file passed as the first CLI argument. |

## Build & test

```sh
cargo build --workspace          # debug build
cargo test --workspace           # all tests (parser + embedded sample)
cargo build --release            # optimized build -> target/release/recaf-rust
```

See the repository `README.md` ("Rust Components") for the required Linux
system packages.

## Run

```sh
cargo run -p recaf-gpui                      # embedded demo class file
cargo run -p recaf-gpui -- path/X.class      # any real .class file
```

## Notes

- `patches/xattr` is a local fork of `xattr 0.2.3` wired in via
  `[patch.crates-io]`: upstream 0.2.3 fails to compile against current libc
  releases on Linux (the BSD-only `ENOATTR` errno constant was removed there;
  `ENODATA` is its Linux equivalent). It can be dropped once a fixed 0.2.x
  release of xattr exists, or the dependency chain is updated.
- `rust-toolchain.toml` pins the workspace to stable Rust.
