// ---------------------------------------------------------
//  SA-MP Plugin SDK for Rust
//  types.rs — AMX core type definitions
// ---------------------------------------------------------
//
//  Rust equivalents of the C structures from amx.h:
//  AMX, AMX_HEADER, AMX_NATIVE_INFO, AMX_FUNCSTUB, etc.
//  All structs use #[repr(C, packed)] to match the exact
//  byte-aligned memory layout expected by the SA-MP server.
// ---------------------------------------------------------

use core::ffi::{c_char, c_void};

// ---------------------------------------------------------
//  Primitive Cell Types
// ---------------------------------------------------------

/// AMX signed cell — 32-bit integer matching Pawn's `cell`.
pub type Cell = i32;

/// AMX unsigned cell — 32-bit unsigned matching Pawn's `ucell`.
pub type UCell = u32;

// ---------------------------------------------------------
//  Function Pointer Types
// ---------------------------------------------------------

/// Native function signature: `cell func(AMX *amx, cell *params)`.
pub type AmxNative = unsafe extern "C" fn(amx: *mut Amx, params: *mut Cell) -> Cell;

/// Callback function signature: `int callback(AMX *amx, cell index, cell *result, cell *params)`.
pub type AmxCallback =
    unsafe extern "C" fn(amx: *mut Amx, index: Cell, result: *mut Cell, params: *mut Cell) -> i32;

/// Debug hook function signature: `int debug(AMX *amx)`.
pub type AmxDebug = unsafe extern "C" fn(amx: *mut Amx) -> i32;

/// Server's `logprintf` function signature.
pub type LogprintfFn = unsafe extern "C" fn(format: *const c_char, ...);

// ---------------------------------------------------------
//  AMX — Abstract Machine Instance
// ---------------------------------------------------------

/// Main AMX virtual machine instance.
///
/// This struct is the core runtime state of a Pawn script.
/// Fields are ordered and packed to match the C `struct tagAMX` exactly.
#[repr(C, packed)]
pub struct Amx {
    /// Points to the AMX header plus code (optionally data too).
    pub base: *mut u8,
    /// Points to separate data+stack+heap segment (may be null).
    pub data: *mut u8,
    /// Callback function pointer.
    pub callback: AmxCallback,
    /// Debug hook function pointer.
    pub debug: AmxDebug,
    /// Instruction pointer: relative to `base + header.cod`.
    pub cip: Cell,
    /// Stack frame base: relative to `base + header.dat`.
    pub frm: Cell,
    /// Top of the heap: relative to `base + header.dat`.
    pub hea: Cell,
    /// Bottom of the heap: relative to `base + header.dat`.
    pub hlw: Cell,
    /// Stack pointer: relative to `base + header.dat`.
    pub stk: Cell,
    /// Top of the stack: relative to `base + header.dat`.
    pub stp: Cell,
    /// Current status flags (see `amx_Flags()`).
    pub flags: i32,
    /// User-defined tags for user data slots.
    pub usertags: [i32; AMX_USERNUM],
    /// User-defined data pointers.
    pub userdata: [*mut c_void; AMX_USERNUM],
    /// Native functions can raise an error through this field.
    pub error: i32,
    /// Parameter count for passing arguments.
    pub paramcount: i32,
    /// Primary register (saved during sleep).
    pub pri: Cell,
    /// Alternate register (saved during sleep).
    pub alt: Cell,
    /// Reset value for stack pointer.
    pub reset_stk: Cell,
    /// Reset value for heap pointer.
    pub reset_hea: Cell,
    /// Relocated address/value for the `SYSREQ.D` opcode.
    pub sysreq_d: Cell,
}

// Safety: Amx is a raw FFI struct. The SA-MP server manages its lifecycle
// on the main thread. We store pointers to it but never move it across threads.
unsafe impl Send for Amx {}
unsafe impl Sync for Amx {}

/// Number of user data slots in an AMX instance.
pub const AMX_USERNUM: usize = 4;

/// Maximum name length for file version <= 6.
pub const S_EXPMAX: usize = 19;

/// Maximum name length of a symbol.
pub const S_NAMEMAX: usize = 31;

// ---------------------------------------------------------
//  AMX_HEADER — Binary File / Memory Header
// ---------------------------------------------------------

/// AMX binary header structure.
///
/// This is both the on-disk file format and the in-memory format.
/// The SA-MP server reads this to locate code, data, and tables.
#[repr(C, packed)]
pub struct AmxHeader {
    /// Total size of the AMX file/image.
    pub size: i32,
    /// Magic signature (`0xF1E0` for 32-bit cells).
    pub magic: u16,
    /// File format version.
    pub file_version: i8,
    /// Required AMX runtime version.
    pub amx_version: i8,
    /// AMX flags.
    pub flags: i16,
    /// Size of a definition record.
    pub defsize: i16,
    /// Initial value of COD — code block offset.
    pub cod: i32,
    /// Initial value of DAT — data block offset.
    pub dat: i32,
    /// Initial value of HEA — start of the heap.
    pub hea: i32,
    /// Initial value of STP — stack top.
    pub stp: i32,
    /// Initial value of CIP — instruction pointer.
    pub cip: i32,
    /// Offset to the public functions table.
    pub publics: i32,
    /// Offset to the native functions table.
    pub natives: i32,
    /// Offset to the libraries table.
    pub libraries: i32,
    /// Offset to the public variables table.
    pub pubvars: i32,
    /// Offset to the public tag names table.
    pub tags: i32,
    /// Offset to the name table.
    pub nametable: i32,
}

// ---------------------------------------------------------
//  AMX_NATIVE_INFO — Native Function Registration Entry
// ---------------------------------------------------------

/// Describes a native function to register with the AMX runtime.
///
/// Used with `amx_Register()` to tell the server about plugin-provided
/// native functions that Pawn scripts can call.
#[repr(C)]
pub struct AmxNativeInfo {
    /// Null-terminated function name (must be a static C string).
    pub name: *const c_char,
    /// Function pointer to the native implementation.
    pub func: AmxNative,
}

impl AmxNativeInfo {
    /// Create a new native info entry from a static null-terminated name.
    ///
    /// # Safety
    /// `name` must point to a valid null-terminated C string that lives
    /// for the duration of the plugin (typically a `b"...\0"` literal).
    pub const fn new(name: *const c_char, func: AmxNative) -> Self {
        Self { name, func }
    }

    /// Create a terminator entry (null name + null function).
    ///
    /// The AMX runtime uses this to detect the end of a native list
    /// when `number` is set to `-1` in `amx_Register`.
    #[allow(invalid_value)]
    pub const fn terminator() -> Self {
        Self {
            name: core::ptr::null(),
            // Safety: this is a sentinel value only — the server checks
            // for null and never actually calls this function pointer.
            func: unsafe { core::mem::transmute(0usize) },
        }
    }
}

// ---------------------------------------------------------
//  AMX_FUNCSTUB — Function Stub (name table version <= 6)
// ---------------------------------------------------------

/// Function stub entry with inline name (file version <= 6).
#[repr(C, packed)]
pub struct AmxFuncStub {
    /// Relocated address of the function.
    pub address: UCell,
    /// Null-terminated function name (max 19 chars + null).
    pub name: [c_char; S_EXPMAX + 1],
}

// ---------------------------------------------------------
//  AMX_FUNCSTUBNT — Function Stub (name table version >= 7)
// ---------------------------------------------------------

/// Function stub entry with name table offset (file version >= 7).
#[repr(C, packed)]
pub struct AmxFuncStubNt {
    /// Relocated address of the function.
    pub address: UCell,
    /// Offset into the name table for the function name.
    pub nameofs: u32,
}
