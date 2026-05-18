// ---------------------------------------------------------
//  SA-MP Plugin SDK for Rust
//  lib.rs — Public API surface
// ---------------------------------------------------------
//
//  This crate provides a complete Rust SDK for building
//  SA-MP server plugins. It replaces the original C/C++
//  SDK (amx.h, plugincommon.h, amxplugin.cpp) with safe,
//  idiomatic Rust abstractions.
//
//  # Architecture
//  - `types`     — FFI struct definitions (AMX, AMX_HEADER, etc.)
//  - `consts`    — Protocol constants and flags
//  - `error`     — AMX error code enumeration
//  - `exports`   — Function table dispatch (44 AMX functions)
//  - `helpers`   — Safe high-level wrappers
//  - `plugin`    — SampPlugin trait + define_plugin! macro
//  - `component` — open.mp component mode (feature-gated)
// ---------------------------------------------------------

pub mod consts;
pub mod error;
pub mod exports;
pub mod helpers;
pub mod plugin;
pub mod types;

#[cfg(feature = "component")]
pub mod component;

// ---------------------------------------------------------
//  Convenient Re-exports
// ---------------------------------------------------------

pub use consts::*;
pub use error::AmxError;
pub use exports::log;
pub use helpers::*;
pub use plugin::SampPlugin;
pub use types::*;

#[cfg(feature = "component")]
pub use component::{ComponentVersion, OmpComponent};

