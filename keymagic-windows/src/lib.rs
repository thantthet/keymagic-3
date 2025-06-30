//! Windows Text Services Framework integration for KeyMagic
//! 
//! This crate provides the TSF implementation for Windows.

use keymagic_core::*;

// TODO: Implement Text Services Framework interface
// This will include:
// - ITfTextInputProcessor implementation
// - ITfThreadMgr interaction
// - Key event processing
// - Composition string management

#[no_mangle]
pub extern "C" fn keymagic_windows_version() -> *const u8 {
    b"0.1.0\0".as_ptr()
}