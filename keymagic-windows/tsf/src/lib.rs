//! Windows Text Services Framework integration for KeyMagic
//! 
//! This crate provides the TSF implementation for Windows.

pub mod ffi;

use std::os::raw::c_void;

// External C++ functions
extern "C" {
    fn InitializeDll(hInstance: *mut c_void) -> i32;
}

// Windows DLL entry point
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn DllMain(
    h_module: *mut c_void,
    reason: u32,
    _reserved: *mut c_void,
) -> i32 {
    const DLL_PROCESS_ATTACH: u32 = 1;
    
    match reason {
        DLL_PROCESS_ATTACH => {
            unsafe {
                InitializeDll(h_module)
            }
        }
        _ => 1,
    }
}

#[no_mangle]
pub extern "C" fn keymagic_windows_version() -> *const u8 {
    b"0.1.0\0".as_ptr()
}