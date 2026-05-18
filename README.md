# samp-sdk-rs

A complete **Rust SDK** for building SA-MP server plugins. Drop-in replacement for the original C/C++ SDK — write your plugins in safe, modern Rust.

Supports **two deployment modes** from a single codebase:

| Mode | Target Server | Deploy To | Build Command |
|------|--------------|-----------|---------------|
| **Legacy Plugin** | SA-MP / open.mp | `plugins/` | `cargo build --release` |
| **Native Component** | open.mp only | `components/` | `cargo build --release --features component` |

> **Zero external dependencies** — no `$CAPI.so`, no C++ bridge. Pure Rust with direct ABI emulation.

---

## Table of Contents

- [Quick Start](#quick-start)
- [Project Structure](#project-structure)
- [Building](#building)
- [Deployment](#deployment)
- [Writing Your Plugin](#writing-your-plugin)
  - [1. Implement SampPlugin](#1-implement-sampplugin)
  - [2. Define Native Functions](#2-define-native-functions)
  - [3. Register Natives](#3-register-natives)
  - [4. Export the Plugin](#4-export-the-plugin)
  - [5. Pawn Include File](#5-pawn-include-file)
- [Component Mode (open.mp)](#component-mode-openmp)
- [SDK Reference](#sdk-reference)
- [Architecture](#architecture)
- [Requirements](#requirements)
- [License](#license)

---

## Quick Start

```bash
# 1. Clone & build (legacy mode)
git clone https://github.com/user/samp-sdk-rs.git
cd samp-sdk-rs
cargo build --release

# 2. Deploy
cp target/i686-unknown-linux-gnu/release/libsamp_plugin.so \
   /path/to/server/plugins/samp_plugin.so

# 3. Configure server (server.cfg or config.json)
# SA-MP:   plugins samp_plugin.so
# open.mp: "legacy_plugins": ["samp_plugin"]

# 4. Copy Pawn include & compile your script
cp samp_plugin.inc /path/to/server/include/
```

---

## Project Structure

```
samp-sdk-rs/
├── Cargo.toml              # Workspace root (resolver = "3")
├── rust-toolchain.toml     # Nightly + i686 target
├── .cargo/config.toml      # Default target: i686-unknown-linux-gnu
│
├── samp-sdk/               # SDK crate (library)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs           # Public API surface
│       ├── types.rs         # FFI structs (AMX, AMX_HEADER, Cell, etc.)
│       ├── consts.rs        # Constants & flags
│       ├── error.rs         # AMX error codes
│       ├── exports.rs       # 44 AMX function thunks + logprintf
│       ├── helpers.rs       # Safe wrappers (get_string, get_param, etc.)
│       ├── plugin.rs        # SampPlugin trait + define_plugin! macro
│       └── component.rs     # [feature: component] open.mp vtable emulation
│
├── samp-plugin/             # Your plugin (cdylib)
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs           # Plugin implementation (your code here!)
│
└── samp_plugin.inc          # Pawn include file for your natives
```

---

## Building

### Prerequisites

```bash
# Install Rust nightly with 32-bit Linux target
rustup toolchain install nightly
rustup target add i686-unknown-linux-gnu

# Install 32-bit C libraries (Ubuntu/Debian)
sudo apt install gcc-multilib
```

### Legacy Plugin (SA-MP + open.mp)

```bash
cargo build --release
```

Output: `target/i686-unknown-linux-gnu/release/libsamp_plugin.so`

### Native Component (open.mp only)

```bash
cargo build --release --features component
```

Output: same path — same binary, different entry point.

> **Note:** The `component` feature changes the exported symbols. Legacy exports `Supports`/`Load`/`Unload`/`AmxLoad`/`AmxUnload`. Component exports `ComponentEntryPoint`.

---

## Deployment

### SA-MP Server

```
server/
├── plugins/
│   └── samp_plugin.so          ← copy libsamp_plugin.so here
├── include/
│   └── samp_plugin.inc         ← copy from project root
└── server.cfg                  ← add: plugins samp_plugin.so
```

### open.mp — Legacy Plugin Mode

```jsonc
// config.json
{
    "pawn": {
        "legacy_plugins": ["samp_plugin"]
    }
}
```

Deploy to `plugins/samp_plugin.so`

### open.mp — Native Component Mode

```jsonc
// config.json — no plugin config needed!
// The server auto-discovers components in the components/ directory
{
    "pawn": {
        "main_scripts": ["main"]
    }
}
```

Deploy to `components/samp_plugin.so`

> **Important:** Don't put the same `.so` in both `plugins/` and `components/` — pick one mode.

---

## Writing Your Plugin

### 1. Implement SampPlugin

The `SampPlugin` trait is the core interface. Implement it on your struct:

```rust
use samp_sdk::{SampPlugin, log};
use samp_sdk::types::{Amx, AmxNativeInfo, Cell};

struct MyPlugin;

impl SampPlugin for MyPlugin {
    /// Called when the plugin loads. Return true to continue.
    fn load() -> bool {
        log("MyPlugin loaded!");
        true
    }

    /// Called when the plugin unloads.
    fn unload() {
        log("MyPlugin unloaded.");
    }

    /// Called when a Pawn script loads — register your natives here.
    fn amx_load(amx: *mut Amx) -> i32 {
        let natives = Self::natives();
        unsafe { samp_sdk::exports::amx_register(amx, natives.as_ptr(), natives.len() as i32) }
    }

    /// Called when a Pawn script unloads.
    fn amx_unload(_amx: *mut Amx) -> i32 {
        0 // AMX_ERR_NONE
    }

    /// Declare your native functions.
    fn natives() -> Vec<AmxNativeInfo> {
        vec![
            AmxNativeInfo::new(b"MyNative\0".as_ptr().cast(), n_my_native),
        ]
    }

    /// Called every server tick (~5ms). Optional.
    fn process_tick() {}
}
```

### 2. Define Native Functions

Native functions have a fixed signature: `unsafe extern "C" fn(*mut Amx, *mut Cell) -> Cell`

```rust
/// native MyNative(playerid);
unsafe extern "C" fn n_my_native(_amx: *mut Amx, params: *mut Cell) -> Cell {
    // Read the first parameter (1-indexed)
    let playerid = unsafe { samp_sdk::get_param_cell(params, 1) };
    log(&format!("Player {} called MyNative", playerid));
    1 // Return value to Pawn
}
```

#### Reading Different Parameter Types

```rust
use samp_sdk::{get_param_cell, get_param_float, get_string_from_amx, set_string_to_amx};

// Integer: native GetValue(index);
let index = unsafe { get_param_cell(params, 1) };

// Float: native SetSpeed(Float:speed);
let speed = unsafe { get_param_float(params, 1) };

// Input string: native ProcessText(const text[]);
let addr = unsafe { get_param_cell(params, 1) };
let text = unsafe { get_string_from_amx(amx, addr) }.unwrap();

// Output string: native GetName(output[], len = sizeof(output));
let out_addr = unsafe { get_param_cell(params, 1) };
let out_len = unsafe { get_param_cell(params, 2) } as usize;
unsafe { set_string_to_amx(amx, out_addr, "Hello", out_len) }.unwrap();

// Reference (output integer): native GetHealth(&health);
let ref_addr = unsafe { get_param_cell(params, 1) };
unsafe { samp_sdk::set_param_ref(amx, ref_addr, 100) }.unwrap();
```

### 3. Register Natives

In `amx_load`, call `amx_register` with your natives list:

```rust
fn amx_load(amx: *mut Amx) -> i32 {
    let natives = Self::natives();
    unsafe { samp_sdk::exports::amx_register(amx, natives.as_ptr(), natives.len() as i32) }
}

fn natives() -> Vec<AmxNativeInfo> {
    vec![
        // Name must be a null-terminated byte string
        AmxNativeInfo::new(b"MyNative\0".as_ptr().cast(), n_my_native),
        AmxNativeInfo::new(b"AnotherNative\0".as_ptr().cast(), n_another),
    ]
}
```

### 4. Export the Plugin

At the bottom of `lib.rs`, use the appropriate macro:

```rust
// Legacy mode (default)
#[cfg(not(feature = "component"))]
samp_sdk::define_plugin!(MyPlugin);

// Component mode (when --features component)
#[cfg(feature = "component")]
samp_sdk::define_component!(MyPlugin);
```

### 5. Pawn Include File

Create a `.inc` file declaring your natives for Pawn scripts:

```pawn
// my_plugin.inc
#if defined _my_plugin_included
    #endinput
#endif
#define _my_plugin_included

native MyNative(playerid);
native AnotherNative(const text[], output[], len = sizeof(output));
```

---

## Component Mode (open.mp)

To enable native component loading on open.mp, implement the `OmpComponent` trait:

```rust
#[cfg(feature = "component")]
impl samp_sdk::OmpComponent for MyPlugin {
    /// Unique component ID (generate a random u64)
    fn uid() -> u64 {
        0xBA284FB180FCD75A
    }

    /// Component name shown in server console
    fn component_name() -> &'static str {
        "MyPlugin"
    }

    /// Semantic version
    fn component_version() -> samp_sdk::ComponentVersion {
        samp_sdk::ComponentVersion::new(0, 1, 0)
    }
}
```

### How It Works

The `define_component!` macro generates a `ComponentEntryPoint` export that returns a vtable-compatible `IComponent*` pointer. The server calls lifecycle methods through this vtable:

```
Server loads .so
  └─ dlsym("ComponentEntryPoint") → IComponent*
       ├─ componentName()     → "MyPlugin"
       ├─ componentVersion()  → 0.1.0
       ├─ getUID()            → 0xBA284FB180FCD75A
       ├─ onLoad(ICore*)      → store core pointer
       ├─ onInit(IComponentList*)
       │    ├─ Query IPawnComponent
       │    ├─ Register PawnEventHandler
       │    ├─ Get AMX function table
       │    └─ Call SampPlugin::load()
       ├─ onAmxLoad(IPawnScript&)  → SampPlugin::amx_load()
       ├─ onAmxUnload(IPawnScript&) → SampPlugin::amx_unload()
       ├─ onReady()
       └─ free()
```

### Legacy vs Component — What Changes?

| Aspect | Legacy | Component |
|--------|--------|-----------|
| Entry point | `Supports` + `Load` + `Unload` | `ComponentEntryPoint` |
| Native registration | Server calls `AmxLoad` | PawnEventHandler `onAmxLoad` |
| AMX functions | From `plugin_data[1]` | From `IPawnComponent::getAmxFunctions()` |
| Logging | `logprintf` from `plugin_data[0]` | `println!` fallback (stdout) |
| Deployment | `plugins/` + config | `components/` (auto-discovered) |
| Your code | **Identical** — same `SampPlugin` impl | **+** `OmpComponent` trait impl |

---

## SDK Reference

### Key Functions

| Function | Description |
|----------|-------------|
| `log(msg)` | Print to server console |
| `get_param_cell(params, index)` | Read integer parameter (1-indexed) |
| `get_param_float(params, index)` | Read float parameter |
| `get_string_from_amx(amx, addr)` | Read string from Pawn |
| `set_string_to_amx(amx, addr, str, max)` | Write string to Pawn |
| `set_param_ref(amx, addr, value)` | Write to reference parameter |
| `exports::amx_register(amx, natives, count)` | Register native functions |

### Key Types

| Type | Description |
|------|-------------|
| `Amx` | AMX virtual machine instance |
| `Cell` | Pawn cell value (`i32`) |
| `AmxNativeInfo` | Native function registration entry |
| `AmxError` | AMX error code enum |
| `ComponentVersion` | Semantic version for components |

---

## Architecture

```
┌─────────────────────────────────────────────────┐
│                  Your Plugin                     │
│            (samp-plugin crate)                   │
│                                                  │
│  ┌─────────────────┐  ┌──────────────────────┐  │
│  │  SampPlugin     │  │  OmpComponent        │  │
│  │  trait impl      │  │  trait impl          │  │
│  │                  │  │  (feature: component)│  │
│  └────────┬─────────┘  └──────────┬───────────┘  │
│           │                       │              │
├───────────┼───────────────────────┼──────────────┤
│           │    samp-sdk crate     │              │
│  ┌────────▼──────────────────────▼────────────┐  │
│  │                                            │  │
│  │  ┌──────────┐  ┌────────┐  ┌───────────┐  │  │
│  │  │ exports  │  │ plugin │  │ component │  │  │
│  │  │ 44 AMX   │  │ macro  │  │ vtable    │  │  │
│  │  │ thunks   │  │        │  │ emulation │  │  │
│  │  └──────────┘  └────────┘  └───────────┘  │  │
│  │                                            │  │
│  │  ┌──────────┐  ┌────────┐  ┌───────────┐  │  │
│  │  │ types    │  │ consts │  │ helpers   │  │  │
│  │  │ FFI      │  │ flags  │  │ safe API  │  │  │
│  │  └──────────┘  └────────┘  └───────────┘  │  │
│  └────────────────────────────────────────────┘  │
│                                                  │
├──────────────────────────────────────────────────┤
│              Server (SA-MP / open.mp)            │
│              32-bit Linux (i686)                 │
└──────────────────────────────────────────────────┘
```

---

## Requirements

| Requirement | Version |
|-------------|---------|
| Rust | Nightly (2024 edition) |
| Target | `i686-unknown-linux-gnu` |
| System libs | `gcc-multilib` (32-bit) |

> The SA-MP server is a **32-bit i686** binary. All plugins must be compiled for the same architecture — this is configured automatically via `.cargo/config.toml` and `rust-toolchain.toml`.

---

## License

MIT
