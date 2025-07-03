//! Windows Text Services Framework integration for KeyMagic
//! 
//! This crate provides the TSF implementation for Windows.

pub mod ffi;

#[no_mangle]
pub extern "C" fn keymagic_windows_version() -> *const u8 {
    b"0.1.0\0".as_ptr()
}