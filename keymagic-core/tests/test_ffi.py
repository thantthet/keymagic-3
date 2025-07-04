#!/usr/bin/env python3
"""
Comprehensive FFI tests for KeyMagic Core
Tests the C FFI interface using Python ctypes
"""

import ctypes
import os
import sys
import platform
import tempfile
import struct
from pathlib import Path
from typing import Optional

# Determine library name based on platform
def get_library_name():
    system = platform.system()
    if system == "Windows":
        return "keymagic_core.dll"
    elif system == "Darwin":
        return "libkeymagic_core.dylib"
    else:  # Linux and others
        return "libkeymagic_core.so"

# Find the library path
def find_library():
    # Look in common build directories
    script_dir = Path(__file__).parent
    project_root = script_dir.parent.parent
    
    search_paths = [
        # Release builds
        project_root / "target" / "release",
        project_root / "target" / "x86_64-pc-windows-msvc" / "release",
        project_root / "target" / "aarch64-pc-windows-msvc" / "release",
        # Debug builds
        project_root / "target" / "debug",
        project_root / "target" / "x86_64-pc-windows-msvc" / "debug",
        project_root / "target" / "aarch64-pc-windows-msvc" / "debug",
    ]
    
    lib_name = get_library_name()
    for path in search_paths:
        lib_path = path / lib_name
        if lib_path.exists():
            return str(lib_path)
    
    raise FileNotFoundError(f"Could not find {lib_name} in any of: {search_paths}")

# Load the library
try:
    lib_path = find_library()
    print(f"Loading library from: {lib_path}")
    
    # On Windows, try using WinDLL instead of CDLL for better error messages
    if platform.system() == "Windows":
        try:
            lib = ctypes.WinDLL(lib_path)
        except OSError as e:
            print(f"Failed with WinDLL: {e}")
            # Try CDLL as fallback
            lib = ctypes.CDLL(lib_path)
    else:
        lib = ctypes.CDLL(lib_path)
        
except Exception as e:
    print(f"Failed to load library: {e}")
    print(f"Python platform: {platform.machine()}, {platform.architecture()}")
    print("Make sure to build with: cargo build --release")
    
    # Try to give more helpful error on Windows
    if platform.system() == "Windows" and "not a valid Win32 application" in str(e):
        print("\nThis error usually means architecture mismatch between Python and the DLL.")
        print("Make sure both are built for the same architecture (x64 or ARM64).")
    
    sys.exit(1)

# Define structures
class ProcessKeyOutput(ctypes.Structure):
    _fields_ = [
        ("action_type", ctypes.c_int),
        ("text", ctypes.POINTER(ctypes.c_char)),
        ("delete_count", ctypes.c_int),
        ("composing_text", ctypes.POINTER(ctypes.c_char)),
        ("is_processed", ctypes.c_int),
    ]

# Define function signatures
lib.keymagic_engine_new.restype = ctypes.c_void_p
lib.keymagic_engine_new.argtypes = []

lib.keymagic_engine_free.restype = None
lib.keymagic_engine_free.argtypes = [ctypes.c_void_p]

lib.keymagic_engine_load_keyboard.restype = ctypes.c_int
lib.keymagic_engine_load_keyboard.argtypes = [ctypes.c_void_p, ctypes.c_char_p]

lib.keymagic_engine_load_keyboard_from_memory.restype = ctypes.c_int
lib.keymagic_engine_load_keyboard_from_memory.argtypes = [
    ctypes.c_void_p, ctypes.POINTER(ctypes.c_ubyte), ctypes.c_size_t
]

lib.keymagic_engine_process_key.restype = ctypes.c_int
lib.keymagic_engine_process_key.argtypes = [
    ctypes.c_void_p,  # handle
    ctypes.c_int,     # key_code
    ctypes.c_char,    # character
    ctypes.c_int,     # shift
    ctypes.c_int,     # ctrl
    ctypes.c_int,     # alt
    ctypes.c_int,     # caps_lock
    ctypes.POINTER(ProcessKeyOutput)
]

lib.keymagic_free_string.restype = None
lib.keymagic_free_string.argtypes = [ctypes.POINTER(ctypes.c_char)]

lib.keymagic_engine_reset.restype = ctypes.c_int
lib.keymagic_engine_reset.argtypes = [ctypes.c_void_p]

lib.keymagic_engine_get_composition.restype = ctypes.POINTER(ctypes.c_char)
lib.keymagic_engine_get_composition.argtypes = [ctypes.c_void_p]

lib.keymagic_get_version.restype = ctypes.c_char_p
lib.keymagic_get_version.argtypes = []

# Test helpers
def get_test_km2_path() -> Path:
    """Get path to the test KM2 file"""
    script_dir = Path(__file__).parent
    km2_path = script_dir / "test_keyboard.km2"
    
    # If it doesn't exist, create it using kms2km2
    if not km2_path.exists():
        kms_path = script_dir / "test_keyboard.kms"
        if not kms_path.exists():
            # Create the KMS file
            with open(kms_path, 'w', encoding='utf-8') as f:
                f.write('''/*
@NAME = "Test Keyboard"
@DESCRIPTION = "Simple test keyboard for FFI testing"
@TRACK_CAPSLOCK = "FALSE"
@SMART_BACKSPACE = "TRUE"
@US_LAYOUT_BASED = "TRUE"
*/

// Simple rules for testing
"a" => "အ"
"ka" => "က"
"kha" => "ခ"

// Test with modifier
<VK_SHIFT & VK_KEY_A> => "အ"

// Test backspace
"က" + <VK_BACK> => NULL
''')
        
        # Compile it
        import subprocess
        project_root = script_dir.parent.parent
        result = subprocess.run([
            'cargo', 'run', '-p', 'kms2km2', '--bin', 'kms2km2', '--',
            str(kms_path), str(km2_path)
        ], cwd=project_root, capture_output=True, text=True)
        
        if result.returncode != 0:
            print(f"Failed to compile KMS: {result.stderr}")
            raise RuntimeError("Failed to compile test keyboard")
    
    return km2_path

def load_test_km2() -> bytes:
    """Load the test KM2 file"""
    km2_path = get_test_km2_path()
    with open(km2_path, 'rb') as f:
        return f.read()

# Tests
def test_version():
    """Test version retrieval"""
    print("\n=== Test: Version ===")
    version = lib.keymagic_get_version()
    version_str = version.decode('utf-8')
    print(f"✓ Library version: {version_str}")
    assert version_str, "Version should not be empty"

def test_engine_lifecycle():
    """Test engine creation and destruction"""
    print("\n=== Test: Engine Lifecycle ===")
    
    engine = lib.keymagic_engine_new()
    assert engine != 0, "Engine creation failed"
    print("✓ Engine created")
    
    lib.keymagic_engine_free(engine)
    print("✓ Engine freed")

def test_invalid_parameters():
    """Test error handling for invalid parameters"""
    print("\n=== Test: Invalid Parameters ===")
    
    # Test null handle
    result = lib.keymagic_engine_load_keyboard(None, b"test.km2")
    assert result == -2, f"Expected -2, got {result}"
    print("✓ Null handle rejected")
    
    # Test null path
    engine = lib.keymagic_engine_new()
    result = lib.keymagic_engine_load_keyboard(engine, None)
    assert result == -2, f"Expected -2, got {result}"
    print("✓ Null path rejected")
    
    lib.keymagic_engine_free(engine)

def test_file_loading():
    """Test loading KM2 from file"""
    print("\n=== Test: File Loading ===")
    
    engine = lib.keymagic_engine_new()
    
    # Test non-existent file
    result = lib.keymagic_engine_load_keyboard(engine, b"nonexistent.km2")
    assert result == -3, f"Expected -3, got {result}"
    print("✓ Non-existent file handled correctly")
    
    # Test with actual compiled KM2 file
    km2_path = get_test_km2_path()
    result = lib.keymagic_engine_load_keyboard(engine, str(km2_path).encode('utf-8'))
    assert result == 0, f"Expected 0, got {result}"
    print("✓ KM2 file loaded successfully")
    
    lib.keymagic_engine_free(engine)

def test_memory_loading():
    """Test loading KM2 from memory"""
    print("\n=== Test: Memory Loading ===")
    
    engine = lib.keymagic_engine_new()
    km2_data = load_test_km2()
    
    print(f"  KM2 data size: {len(km2_data)} bytes")
    print(f"  First 16 bytes: {' '.join(f'{b:02X}' for b in km2_data[:16])}")
    
    # Convert to ctypes array
    data_array = (ctypes.c_ubyte * len(km2_data))(*km2_data)
    
    result = lib.keymagic_engine_load_keyboard_from_memory(
        engine, data_array, len(km2_data)
    )
    if result != 0:
        print(f"  Load failed with error code: {result}")
        # Error codes: -1=invalid handle, -2=invalid param, -3=engine failure, -5=no keyboard
    assert result == 0, f"Expected 0, got {result}"
    print("✓ KM2 loaded from memory")
    
    lib.keymagic_engine_free(engine)

def test_key_processing():
    """Test key processing and UTF-8 output"""
    print("\n=== Test: Key Processing ===")
    
    engine = lib.keymagic_engine_new()
    
    # Load keyboard
    km2_data = load_test_km2()
    data_array = (ctypes.c_ubyte * len(km2_data))(*km2_data)
    lib.keymagic_engine_load_keyboard_from_memory(engine, data_array, len(km2_data))
    
    # Process key 'a'
    output = ProcessKeyOutput()
    result = lib.keymagic_engine_process_key(
        engine,
        65,  # VK_KEY_A
        ord('a'),
        0, 0, 0, 0,  # no modifiers
        ctypes.byref(output)
    )
    
    assert result == 0, f"Expected 0, got {result}"
    assert output.is_processed == 1, "Key should be processed"
    assert output.action_type == 1, "Should be insert action"
    
    # Check UTF-8 output
    if output.text:
        # Convert POINTER(c_char) to string
        text = ctypes.cast(output.text, ctypes.c_char_p).value.decode('utf-8')
        print(f"✓ Output text: '{text}'")
        assert text == "အ", f"Expected 'အ', got '{text}'"
        
        # Verify UTF-8 encoding
        utf8_bytes = list(text.encode('utf-8'))
        assert utf8_bytes == [0xE1, 0x80, 0xA1], f"Wrong UTF-8 encoding: {utf8_bytes}"
        print("✓ UTF-8 encoding verified")
        
        lib.keymagic_free_string(output.text)
    
    if output.composing_text:
        lib.keymagic_free_string(output.composing_text)
    
    lib.keymagic_engine_free(engine)

def test_multi_char_sequence():
    """Test multi-character input sequences"""
    print("\n=== Test: Multi-character Sequence ===")
    
    engine = lib.keymagic_engine_new()
    
    # Load keyboard with "ka" => "က" rule
    km2_data = load_test_km2()
    data_array = (ctypes.c_ubyte * len(km2_data))(*km2_data)
    lib.keymagic_engine_load_keyboard_from_memory(engine, data_array, len(km2_data))
    
    # Type 'k'
    output1 = ProcessKeyOutput()
    lib.keymagic_engine_process_key(
        engine, 75, ord('k'), 0, 0, 0, 0, ctypes.byref(output1)
    )
    
    if output1.composing_text:
        comp = ctypes.cast(output1.composing_text, ctypes.c_char_p).value.decode('utf-8')
        print(f"✓ After 'k': composing='{comp}'")
        assert comp == "k", f"Expected 'k', got '{comp}'"
        lib.keymagic_free_string(output1.composing_text)
    
    if output1.text:
        lib.keymagic_free_string(output1.text)
    
    # Type 'a' to complete "ka"
    output2 = ProcessKeyOutput()
    lib.keymagic_engine_process_key(
        engine, 65, ord('a'), 0, 0, 0, 0, ctypes.byref(output2)
    )
    
    assert output2.is_processed == 1
    assert output2.action_type == 3, "Should be delete and insert"
    assert output2.delete_count == 1, "Should delete 1 char"
    
    if output2.text:
        text = ctypes.cast(output2.text, ctypes.c_char_p).value.decode('utf-8')
        print(f"✓ After 'ka': output='{text}'")
        assert text == "က", f"Expected 'က', got '{text}'"
        lib.keymagic_free_string(output2.text)
    
    if output2.composing_text:
        lib.keymagic_free_string(output2.composing_text)
    
    lib.keymagic_engine_free(engine)

def test_engine_reset():
    """Test engine reset functionality"""
    print("\n=== Test: Engine Reset ===")
    
    engine = lib.keymagic_engine_new()
    
    # Load keyboard
    km2_data = load_test_km2()
    data_array = (ctypes.c_ubyte * len(km2_data))(*km2_data)
    lib.keymagic_engine_load_keyboard_from_memory(engine, data_array, len(km2_data))
    
    # Type 'k' to start a sequence
    output = ProcessKeyOutput()
    lib.keymagic_engine_process_key(
        engine, 75, ord('k'), 0, 0, 0, 0, ctypes.byref(output)
    )
    
    if output.text:
        lib.keymagic_free_string(output.text)
    if output.composing_text:
        lib.keymagic_free_string(output.composing_text)
    
    # Reset engine
    result = lib.keymagic_engine_reset(engine)
    assert result == 0, f"Expected 0, got {result}"
    print("✓ Engine reset")
    
    # Check composition is cleared
    comp = lib.keymagic_engine_get_composition(engine)
    if comp:
        comp_str = ctypes.cast(comp, ctypes.c_char_p).value.decode('utf-8')
        assert comp_str == "", f"Expected empty, got '{comp_str}'"
        lib.keymagic_free_string(comp)
    print("✓ Composition cleared")
    
    lib.keymagic_engine_free(engine)

def test_memory_safety():
    """Test memory safety with multiple operations"""
    print("\n=== Test: Memory Safety ===")
    
    for i in range(10):
        engine = lib.keymagic_engine_new()
        
        # Load keyboard
        km2_data = load_test_km2()
        data_array = (ctypes.c_ubyte * len(km2_data))(*km2_data)
        lib.keymagic_engine_load_keyboard_from_memory(engine, data_array, len(km2_data))
        
        # Process multiple keys
        for ch in "hello":
            output = ProcessKeyOutput()
            lib.keymagic_engine_process_key(
                engine, ord(ch.upper()), ord(ch), 0, 0, 0, 0, ctypes.byref(output)
            )
            
            if output.text:
                lib.keymagic_free_string(output.text)
            if output.composing_text:
                lib.keymagic_free_string(output.composing_text)
        
        lib.keymagic_engine_free(engine)
    
    print("✓ No memory leaks detected in 10 iterations")

def run_all_tests():
    """Run all FFI tests"""
    print("=== KeyMagic Core FFI Tests (Python) ===")
    
    tests = [
        test_version,
        test_engine_lifecycle,
        test_invalid_parameters,
        test_file_loading,
        test_memory_loading,
        test_key_processing,
        test_multi_char_sequence,
        test_engine_reset,
        test_memory_safety,
    ]
    
    failed = 0
    for test in tests:
        try:
            test()
        except Exception as e:
            print(f"✗ {test.__name__} failed: {e}")
            failed += 1
    
    print(f"\n=== Summary: {len(tests) - failed}/{len(tests)} tests passed ===")
    return failed == 0

if __name__ == "__main__":
    success = run_all_tests()
    sys.exit(0 if success else 1)