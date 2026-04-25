# Rust Libraries

Shared Rust crates for cross-platform functionality.

## Purpose

This directory contains Rust libraries that provide core functionality shared across platforms (desktop, mobile, server, WASM). These crates are designed for maximum portability and performance.

## Structure

```
src/lib/rust/
├── README.md           # This file
├── core-types/         # Shared type definitions
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
├── core-logic/         # Business logic (platform-agnostic)
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
├── platform-bindings/  # FFI bindings for mobile/WASM
│   ├── Cargo.toml
│   └── src/
│       ├── ios.rs
│       ├── android.rs
│       └── wasm.rs
└── storage/            # Storage abstraction layer
    └── ...
```

## Creating a New Crate

1. Create the crate directory:
   ```bash
   mkdir -p src/lib/rust/my-crate
   cd src/lib/rust/my-crate
   cargo init --lib --name my-crate
   ```

2. Add to the root `Cargo.toml` workspace:
   ```toml
   [workspace]
   members = [
       "src/lib/rust/my-crate",
       # ... other members
   ]
   ```

3. Configure `Cargo.toml` for cross-platform:
   ```toml
   [package]
   name = "my-crate"
   version = "0.1.0"
   edition = "2021"

   [lib]
   crate-type = ["lib", "cdylib", "staticlib"]

   [features]
   default = ["desktop"]
   desktop = []
   mobile = []
   wasm = []
   server = []
   ```

## Cross-Platform Targets

Use cargo aliases defined in `.cargo/config.toml`:

```bash
# Desktop (native)
cargo desktop-check
cargo desktop-build

# iOS
cargo mobile-check
cargo mobile-build

# WASM
cargo web-check
cargo web-build

# Server
cargo server-check
cargo server-build
```

## TypeScript Bindings

Use `ts-rs` crate for automatic TypeScript type generation:

```rust
use ts_rs::TS;

#[derive(TS)]
#[ts(export)]
struct MyType {
    field: String,
}
```

Types are exported to `src/lib/typescript/shared/src/types/` via `TS_RS_EXPORT_DIR`.

## Best Practices

1. **Feature Flags**: Use features to conditionally compile platform-specific code
2. **No Unsafe**: Avoid `unsafe` unless absolutely necessary
3. **Error Handling**: Use `thiserror` for library errors, `anyhow` for applications
4. **Documentation**: Document public APIs with `///` comments
5. **Testing**: Write unit tests with `#[cfg(test)]` module
