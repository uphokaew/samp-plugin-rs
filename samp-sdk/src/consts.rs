// ---------------------------------------------------------
//  SA-MP Plugin SDK for Rust
//  consts.rs — Protocol constants, flags, and enumerations
// ---------------------------------------------------------
//
//  Direct translations of the #define and enum values from
//  plugincommon.h and amx.h. Organized by category.
// ---------------------------------------------------------

// ---------------------------------------------------------
//  Plugin Version
// ---------------------------------------------------------

/// SA-MP plugin interface version.
pub const SAMP_PLUGIN_VERSION: u32 = 0x0200;

// ---------------------------------------------------------
//  Supports Flags
// ---------------------------------------------------------
//
//  Returned by `Supports()` to tell the server what
//  capabilities this plugin provides.

/// Plugin version component (lower 16 bits).
pub const SUPPORTS_VERSION: u32 = SAMP_PLUGIN_VERSION;

/// Mask to extract the version from supports flags.
pub const SUPPORTS_VERSION_MASK: u32 = 0xFFFF;

/// Plugin provides AMX native functions.
pub const SUPPORTS_AMX_NATIVES: u32 = 0x10000;

/// Plugin wants `ProcessTick()` called every server tick.
pub const SUPPORTS_PROCESS_TICK: u32 = 0x20000;

// ---------------------------------------------------------
//  Plugin Data Indices
// ---------------------------------------------------------
//
//  Indices into the `data` array passed to `Load()`.
//  Each index points to a different server subsystem.

/// `void (*logprintf)(char* format, ...)` — server log function.
pub const PLUGIN_DATA_LOGPRINTF: usize = 0x00;

/// `void* AmxFunctionTable[]` — AMX function export table.
pub const PLUGIN_DATA_AMX_EXPORTS: usize = 0x10;

/// `int (*AmxCallPublicFilterScript)(char *szFunctionName)`.
pub const PLUGIN_DATA_CALLPUBLIC_FS: usize = 0x11;

/// `int (*AmxCallPublicGameMode)(char *szFunctionName)`.
pub const PLUGIN_DATA_CALLPUBLIC_GM: usize = 0x12;

/// `CNetGame* GetNetGame()` — server network game instance.
pub const PLUGIN_DATA_NETGAME: usize = 0xE1;

/// `RakServerInterface* PluginGetRakServer()` — RakNet server.
pub const PLUGIN_DATA_RAKSERVER: usize = 0xE2;

/// `bool LoadFilterscriptFromMemory(char*, char*)` — filterscript loader.
pub const PLUGIN_DATA_LOADFSCRIPT: usize = 0xE3;

/// `CConsole* GetConsole()` — server console.
pub const PLUGIN_DATA_CONSOLE: usize = 0xE4;

/// `bool UnloadFilterScript(char*)` — filterscript unloader.
pub const PLUGIN_DATA_UNLOADFSCRIPT: usize = 0xE5;

/// Maximum size of the plugin data array.
pub const MAX_PLUGIN_DATA: usize = 0x100;

// ---------------------------------------------------------
//  AMX Function Export Indices
// ---------------------------------------------------------
//
//  Indices into the AMX function table. Used internally
//  by `AmxExports` to dispatch calls to the server.

/// `amx_Align16` — align a 16-bit value.
pub const AMX_EXPORT_ALIGN16: usize = 0;
/// `amx_Align32` — align a 32-bit value.
pub const AMX_EXPORT_ALIGN32: usize = 1;
/// `amx_Align64` — align a 64-bit value.
pub const AMX_EXPORT_ALIGN64: usize = 2;
/// `amx_Allot` — allocate heap memory.
pub const AMX_EXPORT_ALLOT: usize = 3;
/// `amx_Callback` — invoke a callback.
pub const AMX_EXPORT_CALLBACK: usize = 4;
/// `amx_Cleanup` — clean up an AMX instance.
pub const AMX_EXPORT_CLEANUP: usize = 5;
/// `amx_Clone` — clone an AMX instance.
pub const AMX_EXPORT_CLONE: usize = 6;
/// `amx_Exec` — execute a public function.
pub const AMX_EXPORT_EXEC: usize = 7;
/// `amx_FindNative` — find a native function by name.
pub const AMX_EXPORT_FIND_NATIVE: usize = 8;
/// `amx_FindPublic` — find a public function by name.
pub const AMX_EXPORT_FIND_PUBLIC: usize = 9;
/// `amx_FindPubVar` — find a public variable by name.
pub const AMX_EXPORT_FIND_PUBVAR: usize = 10;
/// `amx_FindTagId` — find a tag by ID.
pub const AMX_EXPORT_FIND_TAG_ID: usize = 11;
/// `amx_Flags` — get AMX flags.
pub const AMX_EXPORT_FLAGS: usize = 12;
/// `amx_GetAddr` — resolve a cell address.
pub const AMX_EXPORT_GET_ADDR: usize = 13;
/// `amx_GetNative` — get native function name by index.
pub const AMX_EXPORT_GET_NATIVE: usize = 14;
/// `amx_GetPublic` — get public function name by index.
pub const AMX_EXPORT_GET_PUBLIC: usize = 15;
/// `amx_GetPubVar` — get public variable by index.
pub const AMX_EXPORT_GET_PUBVAR: usize = 16;
/// `amx_GetString` — read a string from AMX memory.
pub const AMX_EXPORT_GET_STRING: usize = 17;
/// `amx_GetTag` — get tag name by index.
pub const AMX_EXPORT_GET_TAG: usize = 18;
/// `amx_GetUserData` — retrieve user data pointer.
pub const AMX_EXPORT_GET_USER_DATA: usize = 19;
/// `amx_Init` — initialize an AMX instance.
pub const AMX_EXPORT_INIT: usize = 20;
/// `amx_InitJIT` — initialize JIT compiler.
pub const AMX_EXPORT_INIT_JIT: usize = 21;
/// `amx_MemInfo` — query memory sizes.
pub const AMX_EXPORT_MEM_INFO: usize = 22;
/// `amx_NameLength` — get maximum name length.
pub const AMX_EXPORT_NAME_LENGTH: usize = 23;
/// `amx_NativeInfo` — create a native info entry.
pub const AMX_EXPORT_NATIVE_INFO: usize = 24;
/// `amx_NumNatives` — count registered native functions.
pub const AMX_EXPORT_NUM_NATIVES: usize = 25;
/// `amx_NumPublics` — count public functions.
pub const AMX_EXPORT_NUM_PUBLICS: usize = 26;
/// `amx_NumPubVars` — count public variables.
pub const AMX_EXPORT_NUM_PUBVARS: usize = 27;
/// `amx_NumTags` — count tags.
pub const AMX_EXPORT_NUM_TAGS: usize = 28;
/// `amx_Push` — push a cell onto the AMX stack.
pub const AMX_EXPORT_PUSH: usize = 29;
/// `amx_PushArray` — push an array onto the AMX stack.
pub const AMX_EXPORT_PUSH_ARRAY: usize = 30;
/// `amx_PushString` — push a string onto the AMX stack.
pub const AMX_EXPORT_PUSH_STRING: usize = 31;
/// `amx_RaiseError` — raise a runtime error.
pub const AMX_EXPORT_RAISE_ERROR: usize = 32;
/// `amx_Register` — register native functions.
pub const AMX_EXPORT_REGISTER: usize = 33;
/// `amx_Release` — release allocated heap memory.
pub const AMX_EXPORT_RELEASE: usize = 34;
/// `amx_SetCallback` — set the callback function.
pub const AMX_EXPORT_SET_CALLBACK: usize = 35;
/// `amx_SetDebugHook` — set the debug hook.
pub const AMX_EXPORT_SET_DEBUG_HOOK: usize = 36;
/// `amx_SetString` — write a string into AMX memory.
pub const AMX_EXPORT_SET_STRING: usize = 37;
/// `amx_SetUserData` — store user data pointer.
pub const AMX_EXPORT_SET_USER_DATA: usize = 38;
/// `amx_StrLen` — get string length in cells.
pub const AMX_EXPORT_STR_LEN: usize = 39;
/// `amx_UTF8Check` — validate UTF-8 encoding.
pub const AMX_EXPORT_UTF8_CHECK: usize = 40;
/// `amx_UTF8Get` — read a UTF-8 character.
pub const AMX_EXPORT_UTF8_GET: usize = 41;
/// `amx_UTF8Len` — get UTF-8 string length.
pub const AMX_EXPORT_UTF8_LEN: usize = 42;
/// `amx_UTF8Put` — write a UTF-8 character.
pub const AMX_EXPORT_UTF8_PUT: usize = 43;

// ---------------------------------------------------------
//  AMX Flags
// ---------------------------------------------------------

/// Symbolic debug info available.
pub const AMX_FLAG_DEBUG: i32 = 0x02;
/// Compact encoding enabled.
pub const AMX_FLAG_COMPACT: i32 = 0x04;
/// Opcodes are bytes (not cells).
pub const AMX_FLAG_BYTEOPC: i32 = 0x08;
/// No array bounds checking; no STMT opcode.
pub const AMX_FLAG_NOCHECKS: i32 = 0x10;
/// All native functions are registered.
pub const AMX_FLAG_NTVREG: i32 = 0x1000;
/// Abstract machine is JIT compiled.
pub const AMX_FLAG_JITC: i32 = 0x2000;
/// Busy browsing.
pub const AMX_FLAG_BROWSE: i32 = 0x4000;
/// Jump/call addresses relocated.
pub const AMX_FLAG_RELOC: i32 = 0x8000_u32 as i32;

// ---------------------------------------------------------
//  AMX Magic Numbers
// ---------------------------------------------------------

/// Magic signature for 32-bit cell AMX files.
pub const AMX_MAGIC: u16 = 0xF1E0;

// ---------------------------------------------------------
//  AMX Exec Indices
// ---------------------------------------------------------

/// Execute from the program entry point (main).
pub const AMX_EXEC_MAIN: i32 = -1;
/// Continue execution from the last address.
pub const AMX_EXEC_CONT: i32 = -2;

// ---------------------------------------------------------
//  AMX File Version
// ---------------------------------------------------------

/// Current file format version.
pub const CUR_FILE_VERSION: i32 = 8;
/// Lowest supported file format version.
pub const MIN_FILE_VERSION: i32 = 6;
/// Minimum AMX version for current file format.
pub const MIN_AMX_VERSION: i32 = 8;

// ---------------------------------------------------------
//  AMX Cell Constants
// ---------------------------------------------------------

/// Maximum value for unpacked cells.
pub const UNPACKEDMAX: Cell = ((1i32) << 24) - 1;

/// Unlimited size sentinel (matching C's `(~1u >> 1)`).
pub const UNLIMITED: u32 = !1u32 >> 1;

/// Compact margin size for memory operations.
pub const AMX_COMPACTMARGIN: i32 = 64;

use crate::types::Cell;
