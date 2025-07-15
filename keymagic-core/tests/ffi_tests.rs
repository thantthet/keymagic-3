//! FFI tests for keymagic-core

use keymagic_core::ffi::*;
use std::ffi::{CStr, CString};
use std::ptr;

mod common;
use common::*;

#[test]
fn test_engine_lifecycle() {
    unsafe {
        let engine = keymagic_engine_new();
        assert!(!engine.is_null());
        keymagic_engine_free(engine);
    }
}

#[test]
fn test_version() {
    unsafe {
        let version = keymagic_get_version();
        assert!(!version.is_null());
        let version_str = CStr::from_ptr(version).to_str().unwrap();
        assert!(!version_str.is_empty());
    }
}

#[test]
fn test_process_key_test_does_not_modify_engine() {
    unsafe {
        let engine = keymagic_engine_new();
        assert!(!engine.is_null());

        // Load a basic keyboard
        let km2_data = create_basic_km2();
        let binary = create_km2_binary(&km2_data).unwrap();
        
        let result = keymagic_engine_load_keyboard_from_memory(
            engine,
            binary.as_ptr(),
            binary.len(),
        );
        assert_eq!(result, KeyMagicResult::Success);

        // Initial state should be empty
        let initial_composition = keymagic_engine_get_composition(engine);
        assert!(!initial_composition.is_null());
        let initial_str = CStr::from_ptr(initial_composition).to_str().unwrap();
        assert_eq!(initial_str, "");
        keymagic_free_string(initial_composition);

        // Process a key normally to set some state
        let mut normal_output = ProcessKeyOutput {
            action_type: 0,
            text: ptr::null_mut(),
            delete_count: 0,
            composing_text: ptr::null_mut(),
            is_processed: 0,
        };
        
        let result = keymagic_engine_process_key(
            engine,
            97, // 'a' key code
            b'a' as i8,
            0, 0, 0, 0,
            &mut normal_output
        );
        assert_eq!(result, KeyMagicResult::Success);

        // Check that engine state was modified
        let after_composition = keymagic_engine_get_composition(engine);
        assert!(!after_composition.is_null());
        let after_str = CStr::from_ptr(after_composition).to_str().unwrap();
        assert_eq!(after_str, "a");
        keymagic_free_string(after_composition);

        // Now test in test mode - should not modify engine state
        let mut test_output = ProcessKeyOutput {
            action_type: 0,
            text: ptr::null_mut(),
            delete_count: 0,
            composing_text: ptr::null_mut(),
            is_processed: 0,
        };
        
        let test_result = keymagic_engine_process_key_test(
            engine,
            98, // 'b' key code
            b'b' as i8,
            0, 0, 0, 0,
            &mut test_output
        );
        assert_eq!(test_result, KeyMagicResult::Success);

        // Check that test mode produced expected output
        assert!(!test_output.composing_text.is_null());
        let test_composition_str = CStr::from_ptr(test_output.composing_text).to_str().unwrap();
        assert_eq!(test_composition_str, "ab"); // Should show what would happen

        // But engine state should remain unchanged
        let final_composition = keymagic_engine_get_composition(engine);
        assert!(!final_composition.is_null());
        let final_str = CStr::from_ptr(final_composition).to_str().unwrap();
        assert_eq!(final_str, "a"); // Should still be "a", not "ab"

        // Cleanup
        keymagic_free_string(test_output.composing_text);
        keymagic_free_string(final_composition);
        if !normal_output.text.is_null() {
            keymagic_free_string(normal_output.text);
        }
        if !normal_output.composing_text.is_null() {
            keymagic_free_string(normal_output.composing_text);
        }
        keymagic_engine_free(engine);
    }
}

#[test]
fn test_process_key_test_win_vk() {
    unsafe {
        let engine = keymagic_engine_new();
        assert!(!engine.is_null());

        // Load a basic keyboard
        let km2_data = create_basic_km2();
        let binary = create_km2_binary(&km2_data).unwrap();
        
        let result = keymagic_engine_load_keyboard_from_memory(
            engine,
            binary.as_ptr(),
            binary.len(),
        );
        assert_eq!(result, KeyMagicResult::Success);

        // Test in test mode with Windows VK codes
        let mut test_output = ProcessKeyOutput {
            action_type: 0,
            text: ptr::null_mut(),
            delete_count: 0,
            composing_text: ptr::null_mut(),
            is_processed: 0,
        };
        
        let test_result = keymagic_engine_process_key_test_win(
            engine,
            0x41, // VK_A
            b'a' as i8,
            0, 0, 0, 0,
            &mut test_output
        );
        assert_eq!(test_result, KeyMagicResult::Success);

        // Check that test mode produced expected output
        assert!(!test_output.composing_text.is_null());
        let test_composition_str = CStr::from_ptr(test_output.composing_text).to_str().unwrap();
        assert_eq!(test_composition_str, "a");

        // Engine state should remain empty
        let final_composition = keymagic_engine_get_composition(engine);
        assert!(!final_composition.is_null());
        let final_str = CStr::from_ptr(final_composition).to_str().unwrap();
        assert_eq!(final_str, ""); // Should still be empty

        // Cleanup
        keymagic_free_string(test_output.composing_text);
        keymagic_free_string(final_composition);
        keymagic_engine_free(engine);
    }
}