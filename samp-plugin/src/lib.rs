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
//  To create your own plugin, modify this file or use it
//  as a reference for your own crate.
// ---------------------------------------------------------

use samp_sdk::exports;
use samp_sdk::types::{Amx, AmxNativeInfo, Cell};
use samp_sdk::{
    define_plugin, error::AmxError, get_param_cell, get_string_from_amx, log, SampPlugin,
};

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
        let result = unsafe { exports::amx_register(amx, natives.as_ptr(), natives.len() as i32) };
        result
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
//  Export Plugin
// ---------------------------------------------------------

define_plugin!(MyPlugin);
