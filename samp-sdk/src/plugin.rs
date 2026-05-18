// ---------------------------------------------------------
//  SA-MP Plugin SDK for Rust
//  plugin.rs — SampPlugin trait & define_plugin! macro
// ---------------------------------------------------------
//
//  This module provides the ergonomic interface for creating
//  SA-MP server plugins in Rust. Users implement the
//  `SampPlugin` trait and invoke `define_plugin!` to generate
//  all required C-ABI exports automatically.
// ---------------------------------------------------------

use crate::types::*;
use crate::consts::*;

// ---------------------------------------------------------
//  Plugin Trait
// ---------------------------------------------------------

/// Trait for implementing a SA-MP server plugin.
///
/// Implement this trait on your plugin struct and then use
/// `define_plugin!(YourStruct)` to generate the required
/// C-ABI exported functions.
///
/// # Example
/// ```ignore
/// struct MyPlugin;
///
/// impl SampPlugin for MyPlugin {
///     fn load() -> bool {
///         samp_sdk::exports::log("Plugin loaded!");
///         true
///     }
///     fn unload() {}
///     fn amx_load(amx: *mut Amx) -> i32 { 0 }
///     fn amx_unload(_amx: *mut Amx) -> i32 { 0 }
///     fn natives() -> Vec<AmxNativeInfo> { vec![] }
/// }
///
/// samp_sdk::define_plugin!(MyPlugin);
/// ```
pub trait SampPlugin: Send + Sync + 'static {
    /// Return the supports flags for this plugin.
    ///
    /// Default: `SUPPORTS_VERSION | SUPPORTS_AMX_NATIVES`.
    /// Override to add `SUPPORTS_PROCESS_TICK` if needed.
    fn supports() -> u32 {
        SUPPORTS_VERSION | SUPPORTS_AMX_NATIVES
    }

    /// Called when the server loads this plugin.
    ///
    /// Return `true` to indicate successful initialization.
    /// The AMX function table and `logprintf` are already
    /// initialized before this is called.
    fn load() -> bool;

    /// Called when the server unloads this plugin.
    fn unload();

    /// Called when a Pawn AMX script is loaded.
    ///
    /// Register your native functions here using `amx_register`.
    fn amx_load(amx: *mut Amx) -> i32;

    /// Called when a Pawn AMX script is unloaded.
    fn amx_unload(amx: *mut Amx) -> i32;

    /// Called every server tick (if `SUPPORTS_PROCESS_TICK` is set).
    ///
    /// Default implementation does nothing.
    fn process_tick() {}

    /// Return the list of native functions this plugin provides.
    ///
    /// These will be registered with each AMX that loads.
    fn natives() -> Vec<AmxNativeInfo>;
}

// ---------------------------------------------------------
//  Plugin Export Macro
// ---------------------------------------------------------

/// Generate the C-ABI exported functions required by SA-MP.
///
/// This macro creates the five (optionally six) `extern "C"`
/// functions that the SA-MP server looks for when loading a
/// plugin shared library:
///
/// - `Supports() -> u32`
/// - `Load(*const *const c_void) -> bool`
/// - `Unload()`
/// - `AmxLoad(*mut Amx) -> i32`
/// - `AmxUnload(*mut Amx) -> i32`
/// - `ProcessTick()` (if `SUPPORTS_PROCESS_TICK` is set)
///
/// # Usage
/// ```ignore
/// samp_sdk::define_plugin!(MyPlugin);
/// ```
#[macro_export]
macro_rules! define_plugin {
    ($plugin:ty) => {
        /// Returns the plugin's capability flags to the server.
        #[unsafe(no_mangle)]
        pub extern "C" fn Supports() -> u32 {
            <$plugin as $crate::plugin::SampPlugin>::supports()
        }

        /// Called by the server to initialize the plugin.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn Load(data: *const *const core::ffi::c_void) -> bool {
            unsafe {
                $crate::exports::init_logprintf(
                    *data.add($crate::consts::PLUGIN_DATA_LOGPRINTF),
                );
                $crate::exports::init_amx_functions(
                    *data.add($crate::consts::PLUGIN_DATA_AMX_EXPORTS),
                );
            }

            <$plugin as $crate::plugin::SampPlugin>::load()
        }

        /// Called by the server when unloading the plugin.
        #[unsafe(no_mangle)]
        pub extern "C" fn Unload() {
            <$plugin as $crate::plugin::SampPlugin>::unload();
        }

        /// Called when a Pawn script (AMX) is loaded.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn AmxLoad(amx: *mut $crate::types::Amx) -> i32 {
            <$plugin as $crate::plugin::SampPlugin>::amx_load(amx)
        }

        /// Called when a Pawn script (AMX) is unloaded.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn AmxUnload(amx: *mut $crate::types::Amx) -> i32 {
            <$plugin as $crate::plugin::SampPlugin>::amx_unload(amx)
        }

        /// Called every server tick (only if `SUPPORTS_PROCESS_TICK` is set).
        #[unsafe(no_mangle)]
        pub extern "C" fn ProcessTick() {
            <$plugin as $crate::plugin::SampPlugin>::process_tick();
        }
    };
}
