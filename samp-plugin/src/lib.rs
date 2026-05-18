// ---------------------------------------------------------
//  SA-MP Plugin Template
//  lib.rs — Example plugin implementation
// ---------------------------------------------------------
//
//  This is a template showing how to build a SA-MP server
//  plugin using the samp-sdk crate. It demonstrates:
//
//  1. Implementing the SampPlugin trait
//  2. Registering native functions
//  3. Reading parameters from Pawn scripts
//  4. Returning values to Pawn scripts
//
//  Build modes:
//    - Legacy plugin:  cargo build --release
//    - OMP component:  cargo build --release --features component
// ---------------------------------------------------------

use samp_sdk::exports;
use samp_sdk::types::{Amx, AmxNativeInfo, Cell};
#[cfg(not(feature = "component"))]
use samp_sdk::define_plugin;
use samp_sdk::{error::AmxError, get_param_cell, get_string_from_amx, log, SampPlugin};

// ---------------------------------------------------------
//  Plugin Definition
// ---------------------------------------------------------

/// Example SA-MP plugin.
struct MyPlugin;

impl SampPlugin for MyPlugin {
    fn load() -> bool {
        log("  =================================");
        log("  |  MyPlugin v0.1.0 loaded       |");
        log("  |  Built with Rust + samp-sdk   |");
        log("  =================================");
        true
    }

    fn unload() {
        log("  MyPlugin unloaded.");
    }

    fn amx_load(amx: *mut Amx) -> i32 {
        let natives = Self::natives();

        // Filter out natives already registered on this AMX to avoid
        // "already registered" warnings in open.mp when multiple
        // scripts load (main + filterscripts).
        let unregistered: Vec<_> = natives
            .into_iter()
            .filter(|n| {
                let mut idx: i32 = 0;
                let found = unsafe { exports::amx_find_native(amx, n.name, &mut idx) };
                found != 0 // AMX_ERR_NONE = 0 means already found
            })
            .collect();

        if unregistered.is_empty() {
            return AmxError::None as i32;
        }

        unsafe { exports::amx_register(amx, unregistered.as_ptr(), unregistered.len() as i32) }
    }

    fn amx_unload(_amx: *mut Amx) -> i32 {
        AmxError::None as i32
    }

    fn natives() -> Vec<AmxNativeInfo> {
        vec![
            AmxNativeInfo::new(
                b"MyNativeFunction\0".as_ptr().cast(),
                n_my_native_function,
            ),
            AmxNativeInfo::new(
                b"MyStringFunction\0".as_ptr().cast(),
                n_my_string_function,
            ),
        ]
    }
}

// ---------------------------------------------------------
//  open.mp Component Implementation (feature-gated)
// ---------------------------------------------------------

#[cfg(feature = "component")]
impl samp_sdk::OmpComponent for MyPlugin {
    fn uid() -> u64 {
        // Unique component ID — generate yours with a random u64
        0xBA284FB180FCD75A
    }

    fn component_name() -> &'static str {
        "MyPlugin"
    }

    fn component_version() -> samp_sdk::ComponentVersion {
        samp_sdk::ComponentVersion::new(0, 1, 0)
    }
}

// ---------------------------------------------------------
//  Native Function Implementations
// ---------------------------------------------------------

/// Example native: `native MyNativeFunction(playerid);`
///
/// Logs the player ID and returns 1 (success).
///
/// Pawn usage:
/// ```pawn
/// MyNativeFunction(playerid);
/// ```
unsafe extern "C" fn n_my_native_function(_amx: *mut Amx, params: *mut Cell) -> Cell {
    let player_id = unsafe { get_param_cell(params, 1) };
    log(&format!("[MyPlugin] MyNativeFunction(playerid={player_id})"));
    1
}

/// Example native: `native MyStringFunction(const text[]);`
///
/// Reads a string argument from Pawn and logs it.
///
/// Pawn usage:
/// ```pawn
/// MyStringFunction("Hello from Pawn!");
/// ```
unsafe extern "C" fn n_my_string_function(amx: *mut Amx, params: *mut Cell) -> Cell {
    let param = unsafe { get_param_cell(params, 1) };
    match unsafe { get_string_from_amx(amx, param) } {
        Ok(text) => {
            log(&format!("[MyPlugin] MyStringFunction: \"{text}\""));
            1
        }
        Err(err) => {
            log(&format!("[MyPlugin] Error reading string: {err}"));
            0
        }
    }
}

// ---------------------------------------------------------
//  Export Plugin (conditional on build mode)
// ---------------------------------------------------------

// Default: Legacy plugin mode (Supports/Load/Unload/AmxLoad/AmxUnload/ProcessTick)
#[cfg(not(feature = "component"))]
define_plugin!(MyPlugin);

// Component mode: ComponentEntryPoint()
#[cfg(feature = "component")]
samp_sdk::define_component!(MyPlugin);
