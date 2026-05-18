// ---------------------------------------------------------
//  SA-MP Plugin SDK for Rust
//  helpers.rs — Safe high-level wrappers for common tasks
// ---------------------------------------------------------
//
//  These functions wrap the raw AMX exports to provide
//  ergonomic, Rust-idiomatic interfaces for the most
//  common plugin operations.
// ---------------------------------------------------------

use crate::error::AmxError;
use crate::exports;
use crate::types::*;

// ---------------------------------------------------------
//  String Operations
// ---------------------------------------------------------

/// Read a string from an AMX parameter into a Rust `String`.
///
/// This is the Rust equivalent of the C `amx_GetCString` helper.
/// It resolves the AMX address, measures the string length,
/// allocates a buffer, and copies the data.
///
/// # Safety
/// `amx` must be a valid AMX pointer. `param` must be a valid
/// cell reference to a string in AMX memory.
pub unsafe fn get_string_from_amx(amx: *mut Amx, param: Cell) -> Result<String, AmxError> {
    // Resolve AMX address to physical pointer.
    let mut ptr: *mut Cell = core::ptr::null_mut();
    let err = unsafe { exports::amx_get_addr(amx, param, &mut ptr) };
    if err != 0 {
        return Err(AmxError::from_raw(err));
    }

    // Measure the string length.
    let mut len: i32 = 0;
    let err = unsafe { exports::amx_str_len(ptr, &mut len) };
    if err != 0 {
        return Err(AmxError::from_raw(err));
    }

    if len <= 0 {
        return Ok(String::new());
    }

    // Allocate buffer and copy the string.
    let size = (len + 1) as usize;
    let mut buffer: Vec<u8> = vec![0u8; size];
    let err = unsafe {
        exports::amx_get_string(buffer.as_mut_ptr().cast(), ptr, 0, size)
    };
    if err != 0 {
        return Err(AmxError::from_raw(err));
    }

    // Find the actual null terminator and truncate.
    let actual_len = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());
    buffer.truncate(actual_len);

    String::from_utf8(buffer).map_err(|_| AmxError::General)
}

/// Write a Rust string into an AMX parameter buffer.
///
/// This is the Rust equivalent of the C `amx_SetCString` helper.
///
/// # Safety
/// `amx` must be a valid AMX pointer. `param` must reference a
/// writable buffer in AMX memory with at least `max_len` cells.
pub unsafe fn set_string_to_amx(
    amx: *mut Amx,
    param: Cell,
    value: &str,
    max_len: usize,
) -> Result<(), AmxError> {
    let mut dest: *mut Cell = core::ptr::null_mut();
    let err = unsafe { exports::amx_get_addr(amx, param, &mut dest) };
    if err != 0 {
        return Err(AmxError::from_raw(err));
    }

    let c_str = std::ffi::CString::new(value).map_err(|_| AmxError::Params)?;
    let err = unsafe { exports::amx_set_string(dest, c_str.as_ptr(), 0, 0, max_len) };
    if err != 0 {
        return Err(AmxError::from_raw(err));
    }

    Ok(())
}

// ---------------------------------------------------------
//  Parameter Access
// ---------------------------------------------------------

/// Get the number of parameters passed to a native function.
///
/// In Pawn, `params[0]` contains the total byte count of arguments.
/// Dividing by `size_of::<Cell>()` gives the argument count.
///
/// # Safety
/// `params` must be a valid pointer from a native function call.
pub unsafe fn get_params_count(params: *const Cell) -> i32 {
    unsafe { *params / core::mem::size_of::<Cell>() as i32 }
}

/// Read a single cell value from native function parameters.
///
/// Parameters are 1-indexed: `index = 1` reads the first argument.
///
/// # Safety
/// `params` must be valid. `index` must be in range `1..=count`.
pub unsafe fn get_param_cell(params: *const Cell, index: i32) -> Cell {
    unsafe { *params.offset(index as isize) }
}

/// Read a float value from native function parameters.
///
/// Pawn stores floats as cell-sized bit patterns. This function
/// reinterprets the bits without changing them (equivalent to
/// the C `amx_ctof` macro).
///
/// # Safety
/// `params` must be valid. `index` must be in range.
pub unsafe fn get_param_float(params: *const Cell, index: i32) -> f32 {
    let cell = unsafe { get_param_cell(params, index) };
    f32::from_bits(cell as u32)
}

/// Convert a float to a cell value for returning to Pawn.
///
/// Equivalent to the C `amx_ftoc` macro.
pub fn float_to_cell(value: f32) -> Cell {
    value.to_bits() as Cell
}

// ---------------------------------------------------------
//  AMX Address Helpers (from amxplugin2.cpp)
// ---------------------------------------------------------

/// Push a physical address onto the AMX stack.
///
/// This performs a reverse relocation of the address and pushes
/// it as a cell value. Equivalent to `amx_PushAddress` from
/// the C SDK's `amxplugin2.cpp`.
///
/// # Safety
/// `amx` must be valid. `address` must point within the AMX
/// data segment.
pub unsafe fn push_address(amx: *mut Amx, address: *mut Cell) -> Result<(), AmxError> {
    let hdr = unsafe { &*((*amx).base as *const AmxHeader) };

    let data = if unsafe { (*amx).data.is_null() } {
        unsafe { (*amx).base.add(hdr.dat as usize) }
    } else {
        unsafe { (*amx).data }
    };

    let xaddr = (address as usize).wrapping_sub(data as usize) as Cell;

    if (xaddr as UCell) >= unsafe { (*amx).stp as UCell } {
        return Err(AmxError::MemAccess);
    }

    let err = unsafe { exports::amx_push(amx, xaddr) };
    if err != 0 {
        return Err(AmxError::from_raw(err));
    }

    Ok(())
}

// ---------------------------------------------------------
//  Native Registration Helpers
// ---------------------------------------------------------

/// Register a single native function with the AMX.
///
/// Convenience wrapper equivalent to the C `amx_RegisterFunc` macro.
///
/// # Safety
/// `amx` must be valid. `name` must be a null-terminated C string.
pub unsafe fn register_native(
    amx: *mut Amx,
    name: *const core::ffi::c_char,
    func: AmxNative,
) -> i32 {
    let info = AmxNativeInfo::new(name, func);
    unsafe { exports::amx_register(amx, &info as *const AmxNativeInfo, 1) }
}

// ---------------------------------------------------------
//  AMX Header Utilities (from amx2.h)
// ---------------------------------------------------------

/// Check if the AMX header uses a name table (file version >= 7).
///
/// Equivalent to the C macro `USENAMETABLE(hdr)`.
pub fn use_name_table(hdr: &AmxHeader) -> bool {
    hdr.defsize as usize == core::mem::size_of::<AmxFuncStubNt>()
}

/// Count the number of entries between two table offsets.
///
/// Equivalent to the C macro `NUMENTRIES(hdr, field, nextfield)`.
/// `field` and `next_field` are the byte offsets from the header.
pub fn num_entries(hdr: &AmxHeader, field: i32, next_field: i32) -> usize {
    if hdr.defsize == 0 {
        return 0;
    }
    ((next_field - field) as usize) / (hdr.defsize as usize)
}

/// Get a function stub entry from a table by index.
///
/// Equivalent to the C macro `GETENTRY(hdr, table, index)`.
///
/// # Safety
/// `hdr` must be a valid AMX header pointer. `table_offset` must
/// be a valid table offset, and `index` must be in range.
pub unsafe fn get_entry(
    hdr: *const AmxHeader,
    table_offset: i32,
    index: usize,
) -> *const AmxFuncStub {
    let base = hdr as *const u8;
    unsafe {
        base.add(table_offset as usize)
            .add(index * (*hdr).defsize as usize) as *const AmxFuncStub
    }
}

/// Get the name of a function stub entry.
///
/// Supports both inline names (file version <= 6) and name table
/// offsets (file version >= 7).
///
/// Equivalent to the C macro `GETENTRYNAME(hdr, entry)`.
///
/// # Safety
/// `hdr` and `entry` must be valid pointers.
pub unsafe fn get_entry_name(
    hdr: *const AmxHeader,
    entry: *const AmxFuncStub,
) -> *const core::ffi::c_char {
    if use_name_table(unsafe { &*hdr }) {
        // Name table mode: entry is actually AmxFuncStubNt
        let nt_entry = entry as *const AmxFuncStubNt;
        let base = hdr as *const u8;
        unsafe { base.add((*nt_entry).nameofs as usize) as *const core::ffi::c_char }
    } else {
        // Inline name mode
        unsafe { (*entry).name.as_ptr() }
    }
}

/// Redirect a native function in the AMX.
///
/// Finds a native function by name in the AMX's native table and
/// replaces its address with `to`. The original address is stored
/// in `store` if provided.
///
/// Equivalent to `amx_Redirect` from amxplugin2.cpp.
///
/// # Safety
/// `amx` must be valid. `from` must be a valid C string.
pub unsafe fn redirect_native(
    amx: *mut Amx,
    from: &str,
    to: UCell,
    store: Option<&mut AmxNative>,
) {
    let hdr = unsafe { &*((*amx).base as *const AmxHeader) };
    let num = num_entries(hdr, hdr.natives, hdr.libraries);

    for idx in 0..num {
        let func = unsafe { get_entry(hdr as *const AmxHeader, hdr.natives, idx) };
        let name_ptr = unsafe { get_entry_name(hdr as *const AmxHeader, func) };

        let name = unsafe { std::ffi::CStr::from_ptr(name_ptr) };
        if let Ok(name_str) = name.to_str() {
            if name_str == from {
                if let Some(store) = store {
                    *store = unsafe { core::mem::transmute((*func).address as usize) };
                }
                // Write the new address
                let func_mut = func as *mut AmxFuncStub;
                unsafe { (*func_mut).address = to };
                return;
            }
        }
    }
}

