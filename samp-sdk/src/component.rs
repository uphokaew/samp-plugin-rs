// ---------------------------------------------------------
//  SA-MP Plugin SDK for Rust
//  component.rs — open.mp Native Component Mode (Pure)
// ---------------------------------------------------------
//
//  Allows plugins to be loaded as native open.mp components
//  without any external dependencies (no $CAPI.so required).
//
//  # How It Works
//
//  open.mp loads `.so` files from `components/` and calls
//  `ComponentEntryPoint()` which must return an `IComponent*`.
//  IComponent is a C++ class with virtual methods — the server
//  dispatches lifecycle calls through the C++ vtable.
//
//  We emulate the C++ vtable layout (Itanium ABI, GCC/Clang
//  on Linux, 32-bit i686) by constructing a Rust struct whose
//  memory layout matches a C++ object with virtual functions.
//
//  # Vtable Layout (verified via vtable_probe.cpp)
//
//  Object layout (32-bit):
//    [offset 0] primary vtable pointer (4 bytes)
//    [offset 4] secondary vtable pointer (IUIDProvider, 4 bytes)
//
//  Primary vtable indices:
//    [0]  getExtension         [1]  addExtension
//    [2]  removeExtension(ptr) [3]  removeExtension(uid)
//    [4]  complete destructor  [5]  deleting destructor
//    [6]  supportedVersion     [7]  componentName
//    [8]  componentType        [9]  componentVersion
//    [10] onLoad               [11] onInit
//    [12] onReady              [13] onFree
//    [14] provideConfiguration [15] free
//    [16] reset                [17] getUID
//
//  # Reference
//  - open.mp SDK `component.hpp` — IComponent interface
//  - open.mp SDK `pawn.hpp` — IPawnComponent, PawnEventHandler
//  - `convert-encoding/src/main.cc` — C++ component example
// ---------------------------------------------------------

use core::ffi::{c_char, c_void};

use crate::plugin::SampPlugin;

// ---------------------------------------------------------
//  Types matching open.mp SDK
// ---------------------------------------------------------

/// Semantic version matching open.mp's `SemanticVersion`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ComponentVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
    pub prerel: u16,
}

impl ComponentVersion {
    pub const fn new(major: u8, minor: u8, patch: u8) -> Self {
        Self { major, minor, patch, prerel: 0 }
    }
}

/// open.mp's `StringView` — a pointer + length pair.
#[repr(C)]
pub struct StringView {
    pub data: *const c_char,
    pub length: u32,
}

/// Sync-safe wrapper for vtable arrays (function pointers are immutable).
#[repr(transparent)]
pub struct VTable<const N: usize>(pub [*const c_void; N]);
unsafe impl<const N: usize> Sync for VTable<N> {}

// ---------------------------------------------------------
//  C++ Vtable-Compatible Component Instance
// ---------------------------------------------------------

/// A Rust struct whose memory layout matches a C++ `IComponent`
/// object compiled with GCC/Clang (Itanium ABI, 32-bit i686).
///
/// Layout (verified with robin_hood FlatHashMap):
/// ```text
/// [0]  primary_vtable_ptr   (4 bytes)
/// [4]  miscExtensions       (36 bytes — robin_hood::FlatHashMap)
/// [40] secondary_vtable_ptr (4 bytes — IUIDProvider)
/// [44] ... our data fields ...
/// ```
#[repr(C)]
pub struct RustComponent {
    /// Primary vtable pointer (IExtensible + IComponent methods).
    pub primary_vtable: *const *const c_void,

    /// Padding matching robin_hood::unordered_flat_map<UID, Pair<IExtension*, bool>>
    /// (36 bytes on 32-bit). Must be zero-initialized.
    pub _extensible_padding: [u8; 36],

    /// Secondary vtable pointer (IUIDProvider thunk).
    pub secondary_vtable: *const *const c_void,

    // -- Data fields (our own, not part of IComponent ABI) --

    /// Component UID.
    pub uid: u64,

    /// Null-terminated component name (leaked CString).
    pub name_ptr: *const c_char,
    pub name_len: u32,

    /// Component version.
    pub version: ComponentVersion,

    /// Pointer to ICore (set during onLoad).
    pub core: *mut c_void,

    /// Pointer to IPawnComponent (set during onInit).
    pub pawn: *mut c_void,

    /// Embedded PawnEventHandler — registered with IPawnComponent
    /// to receive onAmxLoad/onAmxUnload callbacks.
    pub pawn_handler: RustPawnHandler,

    /// Plugin data array for legacy AMX initialization.
    pub plugin_data: [*mut c_void; crate::consts::MAX_PLUGIN_DATA],
}

/// C++ vtable-compatible PawnEventHandler.
///
/// Layout: single vtable pointer (4 bytes on 32-bit).
/// vtable[0] = onAmxLoad(IPawnScript&)
/// vtable[1] = onAmxUnload(IPawnScript&)
#[repr(C)]
pub struct RustPawnHandler {
    pub vtable: *const *const c_void,
}

// ---------------------------------------------------------
//  Vtable Index Constants (verified via C++ probe)
// ---------------------------------------------------------

/// IComponentList::queryComponent — vtable index 6.
pub const VTIDX_QUERY_COMPONENT: usize = 6;

/// IPawnComponent::getEventDispatcher — vtable index 18.
pub const VTIDX_GET_EVENT_DISPATCHER: usize = 18;

/// IPawnComponent::getAmxFunctions — vtable index 19.
pub const VTIDX_GET_AMX_FUNCTIONS: usize = 19;

/// IEventDispatcher::addEventHandler — vtable index 0.
pub const VTIDX_ADD_EVENT_HANDLER: usize = 0;

/// IPawnScript::GetAMX — vtable index 57 (0-indexed).
pub const VTIDX_GET_AMX: usize = 57;

// ---------------------------------------------------------
//  Component Trait
// ---------------------------------------------------------

/// Trait for implementing a native open.mp server component.
///
/// Extends `SampPlugin` with component-specific metadata.
/// No external dependencies — works directly with the server.
///
/// # Example
/// ```ignore
/// impl OmpComponent for MyPlugin {
///     fn uid() -> u64 { 0xBA284FB180FCD75A }
///     fn component_name() -> &'static str { "MyPlugin" }
///     fn component_version() -> ComponentVersion {
///         ComponentVersion::new(1, 0, 0)
///     }
/// }
///
/// samp_sdk::define_component!(MyPlugin);
/// ```
pub trait OmpComponent: SampPlugin {
    /// Unique component identifier (must not collide with others).
    fn uid() -> u64;

    /// Human-readable component name shown in server logs.
    fn component_name() -> &'static str;

    /// Component semantic version.
    fn component_version() -> ComponentVersion;
}

// ---------------------------------------------------------
//  Component Entry Macro
// ---------------------------------------------------------

/// Generate a native open.mp `ComponentEntryPoint` export.
///
/// Creates a C++ vtable-compatible struct that the server
/// interacts with through virtual method dispatch. No `$CAPI`
/// or other external components are required.
///
/// # Build
/// ```bash
/// cargo build --release --features component
/// # Deploy: cp target/.../libsamp_plugin.so components/
/// ```
#[macro_export]
macro_rules! define_component {
    ($plugin:ty) => {
        // =======================================================
        //  Primary Vtable Callbacks (IExtensible + IComponent)
        // =======================================================

        // [0] IExtensible::getExtension
        unsafe extern "C" fn __vtable_get_extension(
            _this: *mut $crate::component::RustComponent,
            _id: u64,
        ) -> *mut core::ffi::c_void {
            core::ptr::null_mut()
        }

        // [1] IExtensible::addExtension
        unsafe extern "C" fn __vtable_add_extension(
            _this: *mut $crate::component::RustComponent,
            _ext: *mut core::ffi::c_void,
            _auto_delete: bool,
        ) -> bool {
            false
        }

        // [2] IExtensible::removeExtension(ptr)
        unsafe extern "C" fn __vtable_remove_ext_ptr(
            _this: *mut $crate::component::RustComponent,
            _ext: *mut core::ffi::c_void,
        ) -> bool {
            false
        }

        // [3] IExtensible::removeExtension(uid)
        unsafe extern "C" fn __vtable_remove_ext_uid(
            _this: *mut $crate::component::RustComponent,
            _id: u64,
        ) -> bool {
            false
        }

        // [4] Complete destructor
        unsafe extern "C" fn __vtable_dtor_complete(
            _this: *mut $crate::component::RustComponent,
        ) {}

        // [5] Deleting destructor
        unsafe extern "C" fn __vtable_dtor_deleting(
            _this: *mut $crate::component::RustComponent,
        ) {}

        // [6] IComponent::supportedVersion (final)
        unsafe extern "C" fn __vtable_supported_version(
            _this: *const $crate::component::RustComponent,
        ) -> i32 {
            1 // OMP_VERSION_SUPPORTED
        }

        // [7] IComponent::componentName → returns StringView
        unsafe extern "C" fn __vtable_component_name(
            this: *const $crate::component::RustComponent,
        ) -> $crate::component::StringView {
            unsafe {
                $crate::component::StringView {
                    data: (*this).name_ptr,
                    length: (*this).name_len,
                }
            }
        }

        // [8] IComponent::componentType
        unsafe extern "C" fn __vtable_component_type(
            _this: *const $crate::component::RustComponent,
        ) -> i32 {
            0 // ComponentType::Other
        }

        // [9] IComponent::componentVersion → returns SemanticVersion
        unsafe extern "C" fn __vtable_component_version(
            this: *const $crate::component::RustComponent,
        ) -> $crate::component::ComponentVersion {
            unsafe { (*this).version }
        }

        // [10] IComponent::onLoad(ICore*)
        unsafe extern "C" fn __vtable_on_load(
            this: *mut $crate::component::RustComponent,
            core: *mut core::ffi::c_void,
        ) {
            unsafe { (*this).core = core; }
        }

        // [11] IComponent::onInit(IComponentList*)
        //
        // Query IPawnComponent, register as PawnEventHandler,
        // and get the AMX function table + logprintf.
        unsafe extern "C" fn __vtable_on_init(
            this: *mut $crate::component::RustComponent,
            components: *mut core::ffi::c_void,
        ) {
            if components.is_null() { return; }

            // 1. Query IPawnComponent from IComponentList
            //    IComponentList vtable[6] = queryComponent(UID)
            let cl_vtable = unsafe { *(components as *const *const *const core::ffi::c_void) };
            type QueryFn = unsafe extern "C" fn(*mut core::ffi::c_void, u64) -> *mut core::ffi::c_void;
            let query: QueryFn = unsafe {
                core::mem::transmute(*cl_vtable.add($crate::component::VTIDX_QUERY_COMPONENT))
            };
            let pawn_uid: u64 = 0x78906cd9f19c36a6;
            let pawn_ptr = unsafe { query(components, pawn_uid) };
            if pawn_ptr.is_null() { return; }
            unsafe { (*this).pawn = pawn_ptr; }

            // 2. Register our PawnEventHandler with the Pawn event dispatcher
            //    IPawnComponent vtable[18] = getEventDispatcher() -> &IEventDispatcher
            let pawn_vtable = unsafe { *(pawn_ptr as *const *const *const core::ffi::c_void) };
            type GetDispFn = unsafe extern "C" fn(*mut core::ffi::c_void) -> *mut core::ffi::c_void;
            let get_disp: GetDispFn = unsafe {
                core::mem::transmute(*pawn_vtable.add($crate::component::VTIDX_GET_EVENT_DISPATCHER))
            };
            let dispatcher = unsafe { get_disp(pawn_ptr) };
            if !dispatcher.is_null() {
                // IEventDispatcher vtable[0] = addEventHandler(handler*, priority)
                let disp_vtable = unsafe { *(dispatcher as *const *const *const core::ffi::c_void) };
                type AddHandlerFn = unsafe extern "C" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, i8) -> bool;
                let add_handler: AddHandlerFn = unsafe {
                    core::mem::transmute(*disp_vtable.add($crate::component::VTIDX_ADD_EVENT_HANDLER))
                };
                let handler_ptr = unsafe {
                    &mut (*this).pawn_handler as *mut $crate::component::RustPawnHandler
                        as *mut core::ffi::c_void
                };
                unsafe { add_handler(dispatcher, handler_ptr, 0); }
            }

            // 3. Get AMX function table from IPawnComponent
            //    vtable[19] = getAmxFunctions() -> &StaticArray<void*, 52>
            type GetAmxFn = unsafe extern "C" fn(*const core::ffi::c_void) -> *const core::ffi::c_void;
            let get_amx: GetAmxFn = unsafe {
                core::mem::transmute(*pawn_vtable.add($crate::component::VTIDX_GET_AMX_FUNCTIONS))
            };
            let amx_funcs = unsafe { get_amx(pawn_ptr) };
            if !amx_funcs.is_null() {
                unsafe {
                    (*this).plugin_data[$crate::consts::PLUGIN_DATA_AMX_EXPORTS] = amx_funcs as *mut _;
                    $crate::exports::init_amx_functions(amx_funcs as *mut _);
                }
            }

            // 4. Call plugin load (AMX table is ready)
            <$plugin as $crate::plugin::SampPlugin>::load();
        }

        // [12] IComponent::onReady
        unsafe extern "C" fn __vtable_on_ready(
            _this: *mut $crate::component::RustComponent,
        ) {}

        // [13] IComponent::onFree(IComponent*)
        //
        // Called when ANOTHER component is being freed.
        // Do NOT call unload() here — this fires for every component.
        unsafe extern "C" fn __vtable_on_free(
            _this: *mut $crate::component::RustComponent,
            _component: *mut core::ffi::c_void,
        ) {}

        // [14] IComponent::provideConfiguration
        unsafe extern "C" fn __vtable_provide_config(
            _this: *mut $crate::component::RustComponent,
            _logger: *mut core::ffi::c_void,
            _config: *mut core::ffi::c_void,
            _defaults: bool,
        ) {}

        // [15] IComponent::free — our component is being destroyed
        unsafe extern "C" fn __vtable_free(
            this: *mut $crate::component::RustComponent,
        ) {
            <$plugin as $crate::plugin::SampPlugin>::unload();
            unsafe { drop(Box::from_raw(this)); }
        }

        // [16] IComponent::reset
        unsafe extern "C" fn __vtable_reset(
            _this: *mut $crate::component::RustComponent,
        ) {}

        // [17] IUIDProvider::getUID (merged into primary vtable)
        unsafe extern "C" fn __vtable_get_uid(
            this: *const $crate::component::RustComponent,
        ) -> u64 {
            unsafe { (*this).uid }
        }

        // =======================================================
        //  Secondary Vtable Thunk (IUIDProvider)
        // =======================================================

        // The secondary vtable getUID adjusts `this` pointer
        // by -40 bytes (offset of IUIDProvider within the object)
        // before calling the real getUID.
        unsafe extern "C" fn __sec_vtable_get_uid(
            this_uid: *const core::ffi::c_void,
        ) -> u64 {
            // Adjust this pointer: IUIDProvider is at offset +40,
            // so subtract 40 to get the real RustComponent pointer
            let real_this = (this_uid as *const u8).sub(40)
                as *const $crate::component::RustComponent;
            unsafe { (*real_this).uid }
        }

        // =======================================================
        //  PawnEventHandler Vtable Callbacks
        // =======================================================

        // PawnEventHandler vtable[0] = onAmxLoad(IPawnScript&)
        //
        // Called when a Pawn script is loaded. We get the raw AMX*
        // from IPawnScript::GetAMX() (vtable[57]) and pass it to
        // the plugin's amx_load to register native functions.
        unsafe extern "C" fn __pawn_on_amx_load(
            _this: *mut $crate::component::RustPawnHandler,
            script: *mut core::ffi::c_void,
        ) {
            if script.is_null() { return; }
            // IPawnScript::GetAMX() = vtable[57]
            let script_vtable = unsafe { *(script as *const *const *const core::ffi::c_void) };
            type GetAmxFn = unsafe extern "C" fn(*mut core::ffi::c_void) -> *mut core::ffi::c_void;
            let get_amx: GetAmxFn = unsafe {
                core::mem::transmute(*script_vtable.add($crate::component::VTIDX_GET_AMX))
            };
            let amx = unsafe { get_amx(script) };
            if !amx.is_null() {
                <$plugin as $crate::plugin::SampPlugin>::amx_load(
                    amx as *mut $crate::types::Amx
                );
            }
        }

        // PawnEventHandler vtable[1] = onAmxUnload(IPawnScript&)
        unsafe extern "C" fn __pawn_on_amx_unload(
            _this: *mut $crate::component::RustPawnHandler,
            script: *mut core::ffi::c_void,
        ) {
            if script.is_null() { return; }
            let script_vtable = unsafe { *(script as *const *const *const core::ffi::c_void) };
            type GetAmxFn = unsafe extern "C" fn(*mut core::ffi::c_void) -> *mut core::ffi::c_void;
            let get_amx: GetAmxFn = unsafe {
                core::mem::transmute(*script_vtable.add($crate::component::VTIDX_GET_AMX))
            };
            let amx = unsafe { get_amx(script) };
            if !amx.is_null() {
                <$plugin as $crate::plugin::SampPlugin>::amx_unload(
                    amx as *mut $crate::types::Amx
                );
            }
        }

        // =======================================================
        //  Static Vtables
        // =======================================================

        static PRIMARY_VTABLE: $crate::component::VTable<18> = $crate::component::VTable([
            __vtable_get_extension as *const core::ffi::c_void,      // [0]
            __vtable_add_extension as *const core::ffi::c_void,      // [1]
            __vtable_remove_ext_ptr as *const core::ffi::c_void,     // [2]
            __vtable_remove_ext_uid as *const core::ffi::c_void,     // [3]
            __vtable_dtor_complete as *const core::ffi::c_void,      // [4]
            __vtable_dtor_deleting as *const core::ffi::c_void,      // [5]
            __vtable_supported_version as *const core::ffi::c_void,  // [6]
            __vtable_component_name as *const core::ffi::c_void,     // [7]
            __vtable_component_type as *const core::ffi::c_void,     // [8]
            __vtable_component_version as *const core::ffi::c_void,  // [9]
            __vtable_on_load as *const core::ffi::c_void,            // [10]
            __vtable_on_init as *const core::ffi::c_void,            // [11]
            __vtable_on_ready as *const core::ffi::c_void,           // [12]
            __vtable_on_free as *const core::ffi::c_void,            // [13]
            __vtable_provide_config as *const core::ffi::c_void,     // [14]
            __vtable_free as *const core::ffi::c_void,               // [15]
            __vtable_reset as *const core::ffi::c_void,              // [16]
            __vtable_get_uid as *const core::ffi::c_void,            // [17]
        ]);

        static SECONDARY_VTABLE: $crate::component::VTable<1> = $crate::component::VTable([
            __sec_vtable_get_uid as *const core::ffi::c_void,        // [0]
        ]);

        static PAWN_HANDLER_VTABLE: $crate::component::VTable<2> = $crate::component::VTable([
            __pawn_on_amx_load as *const core::ffi::c_void,          // [0]
            __pawn_on_amx_unload as *const core::ffi::c_void,        // [1]
        ]);

        // =======================================================
        //  ComponentEntryPoint
        // =======================================================

        /// Native open.mp component entry point.
        ///
        /// Returns a vtable-compatible `IComponent*` pointer that
        /// the server dispatches lifecycle calls through.
        #[unsafe(no_mangle)]
        pub extern "C" fn ComponentEntryPoint() -> *mut $crate::component::RustComponent {
            let name_str = <$plugin as $crate::component::OmpComponent>::component_name();
            let name_cstr = std::ffi::CString::new(name_str)
                .expect("component name contains null byte");
            let name_len = name_str.len() as u32;
            let name_ptr = name_cstr.into_raw() as *const core::ffi::c_char;

            let instance = Box::new($crate::component::RustComponent {
                primary_vtable: PRIMARY_VTABLE.0.as_ptr() as *const *const core::ffi::c_void,
                _extensible_padding: [0u8; 36],
                secondary_vtable: SECONDARY_VTABLE.0.as_ptr() as *const *const core::ffi::c_void,
                uid: <$plugin as $crate::component::OmpComponent>::uid(),
                name_ptr,
                name_len,
                version: <$plugin as $crate::component::OmpComponent>::component_version(),
                core: core::ptr::null_mut(),
                pawn: core::ptr::null_mut(),
                pawn_handler: $crate::component::RustPawnHandler {
                    vtable: PAWN_HANDLER_VTABLE.0.as_ptr() as *const *const core::ffi::c_void,
                },
                plugin_data: [core::ptr::null_mut(); $crate::consts::MAX_PLUGIN_DATA],
            });

            Box::into_raw(instance)
        }
    };
}
