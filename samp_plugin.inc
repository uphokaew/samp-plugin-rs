// ---------------------------------------------------------
//  samp_plugin.inc — Pawn include file for MyPlugin
// ---------------------------------------------------------
//
//  Include this file in your Pawn scripts to use the
//  native functions provided by the Rust plugin.
//
//  Usage:
//    #include <samp_plugin>
// ---------------------------------------------------------

#if defined _samp_plugin_included
    #endinput
#endif
#define _samp_plugin_included

// ---------------------------------------------------------
//  Native Function Declarations
// ---------------------------------------------------------

/// Call with a player ID. Returns 1 on success.
native MyNativeFunction(playerid);

/// Call with a string. Returns 1 on success.
native MyStringFunction(const text[]);
