//! DLL entry points (`DllMain`, the COM server exports), the module handle and
//! the module lock count.

#![allow(non_snake_case)]

use std::ffi::{c_void, OsString};
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicI32, Ordering};

use windows::core::{Interface, BOOL, GUID, HRESULT};
use windows::Win32::Foundation::{
    CLASS_E_CLASSNOTAVAILABLE, E_POINTER, HMODULE, MAX_PATH, S_FALSE, S_OK,
};
use windows::Win32::System::Com::IClassFactory;
use windows::Win32::System::LibraryLoader::GetModuleFileNameW;
use windows::Win32::System::Ole::SELFREG_E_CLASS;
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;

use crate::factory::ClassFactory;
use crate::guids::GUID_TEXT_SERVICE;

/// The DLL's own module handle, captured in `DllMain`.
static mut DLL_INSTANCE: HMODULE = HMODULE(std::ptr::null_mut());

/// Module lock count; drives `DllCanUnloadNow`.
static LOCK_COUNT: AtomicI32 = AtomicI32::new(0);

pub(crate) fn lock_module() {
    LOCK_COUNT.fetch_add(1, Ordering::SeqCst);
}

pub(crate) fn unlock_module() {
    // Never let an imbalanced unlock drive the count negative.
    let _ = LOCK_COUNT.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| n.checked_sub(1).filter(|&m| m >= 0));
}

pub(crate) fn dll_instance() -> HMODULE {
    // SAFETY: written once on DLL_PROCESS_ATTACH, read-only afterwards.
    unsafe { DLL_INSTANCE }
}

/// Directory containing the DLL. The `dict` folder lives next to it.
pub(crate) fn dll_dir() -> Option<PathBuf> {
    let mut buf = [0u16; MAX_PATH as usize];
    // SAFETY: buf is a valid writable buffer.
    let len = unsafe { GetModuleFileNameW(Some(dll_instance()), &mut buf) } as usize;
    if len == 0 || len >= buf.len() {
        return None;
    }
    let path = PathBuf::from(OsString::from_wide(&buf[..len]));
    path.parent().map(|p| p.to_path_buf())
}

/// Start loading the engine with the current configuration.
pub(crate) fn load_engine() {
    let Some(dir) = dll_dir() else { return };
    let options = crate::config::current().engine_options();
    crate::engine_host::HOST.start_load(dir.join("dict"), crate::config::Config::user_dir(), options);
}

/// Give the engine the current settings' options, reading the dictionaries
/// again only if another thread holds it.
pub(crate) fn apply_engine_options() {
    let options = crate::config::current().engine_options();
    if !crate::engine_host::HOST.apply_options(&options) {
        load_engine();
    }
}

#[no_mangle]
pub extern "system" fn DllMain(hinst: HMODULE, reason: u32, _reserved: *mut c_void) -> BOOL {
    if reason == DLL_PROCESS_ATTACH {
        // SAFETY: single-threaded loader callback; set once.
        unsafe {
            DLL_INSTANCE = hinst;
        }
    }
    BOOL(1)
}

/// Standard COM in-proc-server class-object entry point.
///
/// # Safety
/// Called by COM with valid `rclsid`, `riid` and `ppv` pointers.
#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if ppv.is_null() || rclsid.is_null() || riid.is_null() {
        return E_POINTER;
    }
    unsafe {
        *ppv = std::ptr::null_mut();
        if *rclsid != GUID_TEXT_SERVICE {
            return CLASS_E_CLASSNOTAVAILABLE;
        }
        let factory: IClassFactory = ClassFactory::new().into();
        factory.query(riid, ppv)
    }
}

/// # Safety
/// COM entry point; only reads the module lock count.
#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    if LOCK_COUNT.load(Ordering::SeqCst) == 0 {
        S_OK
    } else {
        S_FALSE
    }
}

/// Registers the COM server, the TSF profile and the TSF categories.
///
/// # Safety
/// `regsvr32` entry point; needs elevation.
#[no_mangle]
pub unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    match crate::registry::register_all() {
        Ok(()) => S_OK,
        Err(_) => {
            let _ = crate::registry::unregister_all();
            SELFREG_E_CLASS
        }
    }
}

/// # Safety
/// `regsvr32` entry point; needs elevation.
#[no_mangle]
pub unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    let _ = crate::registry::unregister_all();
    S_OK
}
