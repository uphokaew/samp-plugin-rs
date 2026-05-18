// ---------------------------------------------------------
//  SA-MP Plugin SDK for Rust
//  exports.rs — AMX function table dispatch & logprintf
// ---------------------------------------------------------

use core::ffi::{c_char, c_void};
use std::sync::OnceLock;

use crate::consts::*;
use crate::types::*;

// ---------------------------------------------------------
//  Global State
// ---------------------------------------------------------

/// Newtype wrapper for raw pointer to make it Send+Sync.
///
/// Safety: The function table is set once during `Load()` on the
/// main server thread and only read afterwards. The server
/// guarantees the table remains valid for the plugin's lifetime.
#[derive(Clone, Copy)]
struct FnTablePtr(*const *const c_void);

unsafe impl Send for FnTablePtr {}
unsafe impl Sync for FnTablePtr {}

static LOGPRINTF: OnceLock<LogprintfFn> = OnceLock::new();
static AMX_FUNCTIONS: OnceLock<FnTablePtr> = OnceLock::new();

// ---------------------------------------------------------
//  Initialization
// ---------------------------------------------------------

/// Store the server's `logprintf` function pointer.
///
/// # Safety
/// Must be called once during `Load()` with `data[PLUGIN_DATA_LOGPRINTF]`.
pub unsafe fn init_logprintf(ptr: *const c_void) {
    let func: LogprintfFn = unsafe { core::mem::transmute(ptr) };
    let _ = LOGPRINTF.set(func);
}

/// Store the AMX function table pointer.
///
/// # Safety
/// Must be called once during `Load()` with `data[PLUGIN_DATA_AMX_EXPORTS]`.
pub unsafe fn init_amx_functions(ptr: *const c_void) {
    let table = FnTablePtr(ptr as *const *const c_void);
    let _ = AMX_FUNCTIONS.set(table);
}

// ---------------------------------------------------------
//  logprintf
// ---------------------------------------------------------

/// Write a message to the SA-MP server console log.
///
/// In legacy mode, uses `logprintf` from the server.
/// In component mode (or if logprintf is not available),
/// falls back to `println!` which the open.mp console captures.
pub fn log(msg: &str) {
    if let Some(logprintf) = LOGPRINTF.get() {
        if let Ok(c_msg) = std::ffi::CString::new(msg) {
            unsafe {
                logprintf(b"%s\0".as_ptr().cast(), c_msg.as_ptr());
            }
        }
    } else {
        // Fallback: write to stdout (open.mp captures this)
        println!("{msg}");
    }
}

// ---------------------------------------------------------
//  Internal Helper
// ---------------------------------------------------------

unsafe fn get_amx_fn(index: usize) -> *const c_void {
    let FnTablePtr(table) = AMX_FUNCTIONS
        .get()
        .copied()
        .expect("AMX functions not initialized");
    unsafe { *table.add(index) }
}

// ---------------------------------------------------------
//  Macro to reduce boilerplate for thunk generation
// ---------------------------------------------------------

macro_rules! amx_thunk {
    (
        $(#[$meta:meta])*
        pub unsafe fn $name:ident($($arg:ident : $ty:ty),* $(,)?) -> $ret:ty;
        index = $idx:expr;
        type Fn = unsafe extern "C" fn($($fty:ty),*) -> $ret2:ty;
    ) => {
        $(#[$meta])*
        pub unsafe fn $name($($arg: $ty),*) -> $ret {
            type ThunkFn = unsafe extern "C" fn($($fty),*) -> $ret2;
            let f: ThunkFn = unsafe { core::mem::transmute(get_amx_fn($idx)) };
            unsafe { f($($arg),*) }
        }
    };
}

// ---------------------------------------------------------
//  AMX Exports — All 44 Functions
// ---------------------------------------------------------

amx_thunk! {
    /// Align a 16-bit value.
    pub unsafe fn amx_align16(v: *mut u16) -> *mut u16;
    index = AMX_EXPORT_ALIGN16;
    type Fn = unsafe extern "C" fn(*mut u16) -> *mut u16;
}

amx_thunk! {
    /// Align a 32-bit value.
    pub unsafe fn amx_align32(v: *mut u32) -> *mut u32;
    index = AMX_EXPORT_ALIGN32;
    type Fn = unsafe extern "C" fn(*mut u32) -> *mut u32;
}

amx_thunk! {
    /// Align a 64-bit value.
    pub unsafe fn amx_align64(v: *mut u64) -> *mut u64;
    index = AMX_EXPORT_ALIGN64;
    type Fn = unsafe extern "C" fn(*mut u64) -> *mut u64;
}

amx_thunk! {
    /// Allocate cells on the AMX heap.
    pub unsafe fn amx_allot(amx: *mut Amx, cells: i32, amx_addr: *mut Cell, phys_addr: *mut *mut Cell) -> i32;
    index = AMX_EXPORT_ALLOT;
    type Fn = unsafe extern "C" fn(*mut Amx, i32, *mut Cell, *mut *mut Cell) -> i32;
}

amx_thunk! {
    /// Invoke the AMX callback function.
    pub unsafe fn amx_callback(amx: *mut Amx, index: Cell, result: *mut Cell, params: *mut Cell) -> i32;
    index = AMX_EXPORT_CALLBACK;
    type Fn = unsafe extern "C" fn(*mut Amx, Cell, *mut Cell, *mut Cell) -> i32;
}

amx_thunk! {
    /// Clean up an AMX instance.
    pub unsafe fn amx_cleanup(amx: *mut Amx) -> i32;
    index = AMX_EXPORT_CLEANUP;
    type Fn = unsafe extern "C" fn(*mut Amx) -> i32;
}

amx_thunk! {
    /// Clone an AMX instance.
    pub unsafe fn amx_clone(amx_clone: *mut Amx, amx_source: *mut Amx, data: *mut c_void) -> i32;
    index = AMX_EXPORT_CLONE;
    type Fn = unsafe extern "C" fn(*mut Amx, *mut Amx, *mut c_void) -> i32;
}

amx_thunk! {
    /// Execute a public function in the AMX.
    pub unsafe fn amx_exec(amx: *mut Amx, retval: *mut Cell, index: i32) -> i32;
    index = AMX_EXPORT_EXEC;
    type Fn = unsafe extern "C" fn(*mut Amx, *mut Cell, i32) -> i32;
}

amx_thunk! {
    /// Find a native function by name.
    pub unsafe fn amx_find_native(amx: *mut Amx, name: *const c_char, index: *mut i32) -> i32;
    index = AMX_EXPORT_FIND_NATIVE;
    type Fn = unsafe extern "C" fn(*mut Amx, *const c_char, *mut i32) -> i32;
}

amx_thunk! {
    /// Find a public function by name.
    pub unsafe fn amx_find_public(amx: *mut Amx, name: *const c_char, index: *mut i32) -> i32;
    index = AMX_EXPORT_FIND_PUBLIC;
    type Fn = unsafe extern "C" fn(*mut Amx, *const c_char, *mut i32) -> i32;
}

amx_thunk! {
    /// Find a public variable by name.
    pub unsafe fn amx_find_pubvar(amx: *mut Amx, name: *const c_char, amx_addr: *mut Cell) -> i32;
    index = AMX_EXPORT_FIND_PUBVAR;
    type Fn = unsafe extern "C" fn(*mut Amx, *const c_char, *mut Cell) -> i32;
}

amx_thunk! {
    /// Find a tag by ID.
    pub unsafe fn amx_find_tag_id(amx: *mut Amx, tag_id: Cell, tagname: *mut c_char) -> i32;
    index = AMX_EXPORT_FIND_TAG_ID;
    type Fn = unsafe extern "C" fn(*mut Amx, Cell, *mut c_char) -> i32;
}

amx_thunk! {
    /// Get AMX flags.
    pub unsafe fn amx_flags(amx: *mut Amx, flags: *mut u16) -> i32;
    index = AMX_EXPORT_FLAGS;
    type Fn = unsafe extern "C" fn(*mut Amx, *mut u16) -> i32;
}

amx_thunk! {
    /// Resolve a cell address to physical memory.
    pub unsafe fn amx_get_addr(amx: *mut Amx, amx_addr: Cell, phys_addr: *mut *mut Cell) -> i32;
    index = AMX_EXPORT_GET_ADDR;
    type Fn = unsafe extern "C" fn(*mut Amx, Cell, *mut *mut Cell) -> i32;
}

amx_thunk! {
    /// Get native function name by index.
    pub unsafe fn amx_get_native(amx: *mut Amx, index: i32, funcname: *mut c_char) -> i32;
    index = AMX_EXPORT_GET_NATIVE;
    type Fn = unsafe extern "C" fn(*mut Amx, i32, *mut c_char) -> i32;
}

amx_thunk! {
    /// Get public function name by index.
    pub unsafe fn amx_get_public(amx: *mut Amx, index: i32, funcname: *mut c_char) -> i32;
    index = AMX_EXPORT_GET_PUBLIC;
    type Fn = unsafe extern "C" fn(*mut Amx, i32, *mut c_char) -> i32;
}

amx_thunk! {
    /// Get public variable by index.
    pub unsafe fn amx_get_pubvar(amx: *mut Amx, index: i32, varname: *mut c_char, amx_addr: *mut Cell) -> i32;
    index = AMX_EXPORT_GET_PUBVAR;
    type Fn = unsafe extern "C" fn(*mut Amx, i32, *mut c_char, *mut Cell) -> i32;
}

amx_thunk! {
    /// Read a string from AMX memory.
    pub unsafe fn amx_get_string(dest: *mut c_char, source: *const Cell, use_wchar: i32, size: usize) -> i32;
    index = AMX_EXPORT_GET_STRING;
    type Fn = unsafe extern "C" fn(*mut c_char, *const Cell, i32, usize) -> i32;
}

amx_thunk! {
    /// Get tag name and ID by index.
    pub unsafe fn amx_get_tag(amx: *mut Amx, index: i32, tagname: *mut c_char, tag_id: *mut Cell) -> i32;
    index = AMX_EXPORT_GET_TAG;
    type Fn = unsafe extern "C" fn(*mut Amx, i32, *mut c_char, *mut Cell) -> i32;
}

amx_thunk! {
    /// Retrieve a user data pointer.
    pub unsafe fn amx_get_user_data(amx: *mut Amx, tag: i32, ptr: *mut *mut c_void) -> i32;
    index = AMX_EXPORT_GET_USER_DATA;
    type Fn = unsafe extern "C" fn(*mut Amx, i32, *mut *mut c_void) -> i32;
}

amx_thunk! {
    /// Initialize an AMX instance.
    pub unsafe fn amx_init(amx: *mut Amx, program: *mut c_void) -> i32;
    index = AMX_EXPORT_INIT;
    type Fn = unsafe extern "C" fn(*mut Amx, *mut c_void) -> i32;
}

amx_thunk! {
    /// Initialize the JIT compiler.
    pub unsafe fn amx_init_jit(amx: *mut Amx, reloc_table: *mut c_void, native_code: *mut c_void) -> i32;
    index = AMX_EXPORT_INIT_JIT;
    type Fn = unsafe extern "C" fn(*mut Amx, *mut c_void, *mut c_void) -> i32;
}

amx_thunk! {
    /// Query memory sizes of an AMX.
    pub unsafe fn amx_mem_info(amx: *mut Amx, codesize: *mut i32, datasize: *mut i32, stackheap: *mut i32) -> i32;
    index = AMX_EXPORT_MEM_INFO;
    type Fn = unsafe extern "C" fn(*mut Amx, *mut i32, *mut i32, *mut i32) -> i32;
}

amx_thunk! {
    /// Get the maximum name length.
    pub unsafe fn amx_name_length(amx: *mut Amx, length: *mut i32) -> i32;
    index = AMX_EXPORT_NAME_LENGTH;
    type Fn = unsafe extern "C" fn(*mut Amx, *mut i32) -> i32;
}

/// Create an `AMX_NATIVE_INFO` entry (server-side).
///
/// # Safety
/// `name` must be null-terminated.
pub unsafe fn amx_native_info(name: *const c_char, func: AmxNative) -> *mut AmxNativeInfo {
    type Fn = unsafe extern "C" fn(*const c_char, AmxNative) -> *mut AmxNativeInfo;
    let f: Fn = unsafe { core::mem::transmute(get_amx_fn(AMX_EXPORT_NATIVE_INFO)) };
    unsafe { f(name, func) }
}

amx_thunk! {
    /// Count registered native functions.
    pub unsafe fn amx_num_natives(amx: *mut Amx, number: *mut i32) -> i32;
    index = AMX_EXPORT_NUM_NATIVES;
    type Fn = unsafe extern "C" fn(*mut Amx, *mut i32) -> i32;
}

amx_thunk! {
    /// Count public functions.
    pub unsafe fn amx_num_publics(amx: *mut Amx, number: *mut i32) -> i32;
    index = AMX_EXPORT_NUM_PUBLICS;
    type Fn = unsafe extern "C" fn(*mut Amx, *mut i32) -> i32;
}

amx_thunk! {
    /// Count public variables.
    pub unsafe fn amx_num_pubvars(amx: *mut Amx, number: *mut i32) -> i32;
    index = AMX_EXPORT_NUM_PUBVARS;
    type Fn = unsafe extern "C" fn(*mut Amx, *mut i32) -> i32;
}

amx_thunk! {
    /// Count tags.
    pub unsafe fn amx_num_tags(amx: *mut Amx, number: *mut i32) -> i32;
    index = AMX_EXPORT_NUM_TAGS;
    type Fn = unsafe extern "C" fn(*mut Amx, *mut i32) -> i32;
}

amx_thunk! {
    /// Push a cell onto the AMX stack.
    pub unsafe fn amx_push(amx: *mut Amx, value: Cell) -> i32;
    index = AMX_EXPORT_PUSH;
    type Fn = unsafe extern "C" fn(*mut Amx, Cell) -> i32;
}

amx_thunk! {
    /// Push an array onto the AMX stack.
    pub unsafe fn amx_push_array(amx: *mut Amx, amx_addr: *mut Cell, phys_addr: *mut *mut Cell, array: *const Cell, numcells: i32) -> i32;
    index = AMX_EXPORT_PUSH_ARRAY;
    type Fn = unsafe extern "C" fn(*mut Amx, *mut Cell, *mut *mut Cell, *const Cell, i32) -> i32;
}

amx_thunk! {
    /// Push a string onto the AMX stack.
    pub unsafe fn amx_push_string(amx: *mut Amx, amx_addr: *mut Cell, phys_addr: *mut *mut Cell, string: *const c_char, pack: i32, use_wchar: i32) -> i32;
    index = AMX_EXPORT_PUSH_STRING;
    type Fn = unsafe extern "C" fn(*mut Amx, *mut Cell, *mut *mut Cell, *const c_char, i32, i32) -> i32;
}

amx_thunk! {
    /// Raise a runtime error.
    pub unsafe fn amx_raise_error(amx: *mut Amx, error: i32) -> i32;
    index = AMX_EXPORT_RAISE_ERROR;
    type Fn = unsafe extern "C" fn(*mut Amx, i32) -> i32;
}

amx_thunk! {
    /// Register native functions with the AMX.
    pub unsafe fn amx_register(amx: *mut Amx, list: *const AmxNativeInfo, number: i32) -> i32;
    index = AMX_EXPORT_REGISTER;
    type Fn = unsafe extern "C" fn(*mut Amx, *const AmxNativeInfo, i32) -> i32;
}

amx_thunk! {
    /// Release heap memory.
    pub unsafe fn amx_release(amx: *mut Amx, amx_addr: Cell) -> i32;
    index = AMX_EXPORT_RELEASE;
    type Fn = unsafe extern "C" fn(*mut Amx, Cell) -> i32;
}

amx_thunk! {
    /// Set the callback function.
    pub unsafe fn amx_set_callback(amx: *mut Amx, callback: AmxCallback) -> i32;
    index = AMX_EXPORT_SET_CALLBACK;
    type Fn = unsafe extern "C" fn(*mut Amx, AmxCallback) -> i32;
}

amx_thunk! {
    /// Set the debug hook.
    pub unsafe fn amx_set_debug_hook(amx: *mut Amx, debug: AmxDebug) -> i32;
    index = AMX_EXPORT_SET_DEBUG_HOOK;
    type Fn = unsafe extern "C" fn(*mut Amx, AmxDebug) -> i32;
}

amx_thunk! {
    /// Write a string into AMX memory.
    pub unsafe fn amx_set_string(dest: *mut Cell, source: *const c_char, pack: i32, use_wchar: i32, size: usize) -> i32;
    index = AMX_EXPORT_SET_STRING;
    type Fn = unsafe extern "C" fn(*mut Cell, *const c_char, i32, i32, usize) -> i32;
}

amx_thunk! {
    /// Store a user data pointer.
    pub unsafe fn amx_set_user_data(amx: *mut Amx, tag: i32, ptr: *mut c_void) -> i32;
    index = AMX_EXPORT_SET_USER_DATA;
    type Fn = unsafe extern "C" fn(*mut Amx, i32, *mut c_void) -> i32;
}

amx_thunk! {
    /// Get string length in cells.
    pub unsafe fn amx_str_len(cstring: *const Cell, length: *mut i32) -> i32;
    index = AMX_EXPORT_STR_LEN;
    type Fn = unsafe extern "C" fn(*const Cell, *mut i32) -> i32;
}

amx_thunk! {
    /// Validate UTF-8 encoding.
    pub unsafe fn amx_utf8_check(string: *const c_char, length: *mut i32) -> i32;
    index = AMX_EXPORT_UTF8_CHECK;
    type Fn = unsafe extern "C" fn(*const c_char, *mut i32) -> i32;
}

amx_thunk! {
    /// Read a single UTF-8 character.
    pub unsafe fn amx_utf8_get(string: *const c_char, endptr: *mut *const c_char, value: *mut Cell) -> i32;
    index = AMX_EXPORT_UTF8_GET;
    type Fn = unsafe extern "C" fn(*const c_char, *mut *const c_char, *mut Cell) -> i32;
}

amx_thunk! {
    /// Get UTF-8 string length.
    pub unsafe fn amx_utf8_len(cstr: *const Cell, length: *mut i32) -> i32;
    index = AMX_EXPORT_UTF8_LEN;
    type Fn = unsafe extern "C" fn(*const Cell, *mut i32) -> i32;
}

amx_thunk! {
    /// Write a single UTF-8 character.
    pub unsafe fn amx_utf8_put(string: *mut c_char, endptr: *mut *mut c_char, maxchars: i32, value: Cell) -> i32;
    index = AMX_EXPORT_UTF8_PUT;
    type Fn = unsafe extern "C" fn(*mut c_char, *mut *mut c_char, i32, Cell) -> i32;
}
