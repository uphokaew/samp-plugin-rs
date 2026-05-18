// ---------------------------------------------------------
//  SA-MP Plugin SDK for Rust
//  error.rs — AMX error code enumeration
// ---------------------------------------------------------
//
//  Type-safe representation of the AMX runtime error codes
//  from the C `enum { AMX_ERR_NONE, ... }` in amx.h.
// ---------------------------------------------------------

use core::fmt;

/// AMX runtime error codes.
///
/// Each variant maps directly to the corresponding `AMX_ERR_*`
/// constant from the C SDK. The discriminant values are identical.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum AmxError {
    /// No error — operation succeeded.
    None = 0,
    /// Forced exit.
    Exit = 1,
    /// Assertion failed.
    Assert = 2,
    /// Stack/heap collision.
    StackErr = 3,
    /// Index out of bounds.
    Bounds = 4,
    /// Invalid memory access.
    MemAccess = 5,
    /// Invalid instruction.
    InvInstr = 6,
    /// Stack underflow.
    StackLow = 7,
    /// Heap underflow.
    HeapLow = 8,
    /// No callback, or invalid callback.
    Callback = 9,
    /// Native function failed.
    Native = 10,
    /// Divide by zero.
    Divide = 11,
    /// Go into sleep mode — code can be restarted.
    Sleep = 12,
    /// Invalid state for this access.
    InvState = 13,
    /// Out of memory.
    Memory = 16,
    /// Invalid file format.
    Format = 17,
    /// File is for a newer version of the AMX.
    Version = 18,
    /// Function not found.
    NotFound = 19,
    /// Invalid index parameter (bad entry point).
    Index = 20,
    /// Debugger cannot run.
    Debug = 21,
    /// AMX not initialized (or doubly initialized).
    Init = 22,
    /// Unable to set user data field (table full).
    UserData = 23,
    /// Cannot initialize the JIT.
    InitJit = 24,
    /// Parameter error.
    Params = 25,
    /// Domain error — expression result out of range.
    Domain = 26,
    /// General error (unknown or unspecific).
    General = 27,
}

impl AmxError {
    /// Convert a raw C error code to an `AmxError`.
    ///
    /// Returns `None` variant for `0` and `General` for any
    /// unrecognized value, ensuring exhaustive coverage.
    pub const fn from_raw(code: i32) -> Self {
        match code {
            0 => Self::None,
            1 => Self::Exit,
            2 => Self::Assert,
            3 => Self::StackErr,
            4 => Self::Bounds,
            5 => Self::MemAccess,
            6 => Self::InvInstr,
            7 => Self::StackLow,
            8 => Self::HeapLow,
            9 => Self::Callback,
            10 => Self::Native,
            11 => Self::Divide,
            12 => Self::Sleep,
            13 => Self::InvState,
            16 => Self::Memory,
            17 => Self::Format,
            18 => Self::Version,
            19 => Self::NotFound,
            20 => Self::Index,
            21 => Self::Debug,
            22 => Self::Init,
            23 => Self::UserData,
            24 => Self::InitJit,
            25 => Self::Params,
            26 => Self::Domain,
            _ => Self::General,
        }
    }

    /// Returns `true` if the error code indicates success.
    pub const fn is_ok(self) -> bool {
        matches!(self, Self::None)
    }

    /// Returns the raw integer error code.
    pub const fn as_raw(self) -> i32 {
        self as i32
    }
}

impl fmt::Display for AmxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            Self::None => "no error",
            Self::Exit => "forced exit",
            Self::Assert => "assertion failed",
            Self::StackErr => "stack/heap collision",
            Self::Bounds => "index out of bounds",
            Self::MemAccess => "invalid memory access",
            Self::InvInstr => "invalid instruction",
            Self::StackLow => "stack underflow",
            Self::HeapLow => "heap underflow",
            Self::Callback => "no or invalid callback",
            Self::Native => "native function failed",
            Self::Divide => "divide by zero",
            Self::Sleep => "sleep mode",
            Self::InvState => "invalid state",
            Self::Memory => "out of memory",
            Self::Format => "invalid file format",
            Self::Version => "version mismatch",
            Self::NotFound => "function not found",
            Self::Index => "invalid index",
            Self::Debug => "debugger error",
            Self::Init => "initialization error",
            Self::UserData => "user data table full",
            Self::InitJit => "JIT initialization failed",
            Self::Params => "parameter error",
            Self::Domain => "domain error",
            Self::General => "general error",
        };
        write!(f, "AMX error: {msg}")
    }
}

impl std::error::Error for AmxError {}
