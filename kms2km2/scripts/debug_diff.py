#!/usr/bin/env python3
# debug_diff.py - Compare two KM2 files and show differences
# Part of kms2km2 - KeyMagic Script to Binary Converter
# Copyright (C) 2024 KeyMagic Contributors
# Licensed under GPL-2.0

import struct
import sys

def read_rule(f):
    """Read a single rule and return (lhs_opcodes, rhs_opcodes)"""
    # Read LHS
    lhs_len = struct.unpack('<H', f.read(2))[0]
    lhs_data = f.read(lhs_len * 2)
    
    # Read RHS  
    rhs_len = struct.unpack('<H', f.read(2))[0]
    rhs_data = f.read(rhs_len * 2)
    
    return lhs_data, rhs_data

def decode_opcodes(data):
    """Decode opcodes from binary data"""
    opcodes = []
    i = 0
    while i < len(data):
        opcode = struct.unpack('<H', data[i:i+2])[0]
        i += 2
        
        if opcode == 0xF0:  # STRING
            opcodes.append(f"STRING(")
            if i < len(data):
                str_len = struct.unpack('<H', data[i:i+2])[0]
                i += 2
                chars = []
                for _ in range(str_len):
                    if i < len(data):
                        ch = struct.unpack('<H', data[i:i+2])[0]
                        chars.append(chr(ch) if ch < 128 else f"U+{ch:04X}")
                        i += 2
                opcodes[-1] += ''.join(chars) + ')'
        elif opcode == 0xF1:  # VARIABLE
            if i < len(data):
                idx = struct.unpack('<H', data[i:i+2])[0]
                i += 2
                opcodes.append(f"VAR({idx})")
        elif opcode == 0xF2:  # REFERENCE
            if i < len(data):
                idx = struct.unpack('<H', data[i:i+2])[0]
                i += 2
                opcodes.append(f"REF({idx})")
        elif opcode == 0xF3:  # PREDEFINED
            if i < len(data):
                vk = struct.unpack('<H', data[i:i+2])[0]
                i += 2
                opcodes.append(f"VK({vk})")
        elif opcode == 0xF4:  # MODIFIER
            if i < len(data):
                mod = struct.unpack('<H', data[i:i+2])[0]
                i += 2
                opcodes.append(f"MOD({mod:04X})")
        elif opcode == 0xF5:  # ANYOF
            if i < len(data):
                idx = struct.unpack('<H', data[i:i+2])[0]
                i += 2
                opcodes.append(f"ANYOF({idx})")
        elif opcode == 0xF6:  # AND
            opcodes.append("AND")
        elif opcode == 0xF7:  # NANYOF
            if i < len(data):
                idx = struct.unpack('<H', data[i:i+2])[0]
                i += 2
                opcodes.append(f"NANYOF({idx})")
        elif opcode == 0xF8:  # ANY
            opcodes.append("ANY")
        elif opcode == 0xF9:  # SWITCH
            if i < len(data):
                idx = struct.unpack('<H', data[i:i+2])[0]
                i += 2
                opcodes.append(f"SWITCH({idx})")
        else:
            opcodes.append(f"UNKNOWN({opcode:04X})")
    
    return ' '.join(opcodes)

def compare_files(ref_file, our_file):
    with open(ref_file, 'rb') as ref, open(our_file, 'rb') as our:
        # Skip headers (18 bytes)
        ref.seek(18)
        our.seek(18)
        
        # Read string counts from header
        ref.seek(6)
        ref_str_count = struct.unpack('<H', ref.read(2))[0]
        our.seek(6)
        our_str_count = struct.unpack('<H', our.read(2))[0]
        
        # Skip to after header
        ref.seek(18)
        our.seek(18)
        
        # Skip strings section
        print(f"Reference has {ref_str_count} strings, ours has {our_str_count} strings")
        
        for i in range(ref_str_count):
            str_len = struct.unpack('<H', ref.read(2))[0]
            ref.read(str_len * 2)
            
        for i in range(our_str_count):
            str_len = struct.unpack('<H', our.read(2))[0]
            our.read(str_len * 2)
        
        # Read info counts
        ref.seek(8)
        ref_info_count = struct.unpack('<H', ref.read(2))[0]
        our.seek(8) 
        our_info_count = struct.unpack('<H', our.read(2))[0]
        
        # Skip to rules by reading through info section
        ref_pos = 18
        for i in range(ref_str_count):
            ref.seek(ref_pos)
            str_len = struct.unpack('<H', ref.read(2))[0]
            ref_pos += 2 + str_len * 2
        
        our_pos = 18
        for i in range(our_str_count):
            our.seek(our_pos)
            str_len = struct.unpack('<H', our.read(2))[0]
            our_pos += 2 + str_len * 2
            
        # Skip info sections
        for i in range(ref_info_count):
            ref.seek(ref_pos)
            ref.read(4)  # ID
            info_len = struct.unpack('<H', ref.read(2))[0]
            ref_pos += 6 + info_len
            
        for i in range(our_info_count):
            our.seek(our_pos)
            our.read(4)  # ID
            info_len = struct.unpack('<H', our.read(2))[0]
            our_pos += 6 + info_len
        
        # Now at rules section
        ref.seek(ref_pos)
        our.seek(our_pos)
        
        print(f"\nComparing rules starting at ref:{ref_pos:04X} our:{our_pos:04X}")
        
        # Read rule count
        ref.seek(10)
        rule_count = struct.unpack('<H', ref.read(2))[0]
        
        ref.seek(ref_pos)
        our.seek(our_pos)
        
        # Compare rules
        for i in range(rule_count):
            print(f"\n=== Rule {i} ===")
            print(f"Ref position: {ref.tell():04X}, Our position: {our.tell():04X}")
            
            try:
                ref_lhs, ref_rhs = read_rule(ref)
                our_lhs, our_rhs = read_rule(our)
                
                if ref_lhs != our_lhs:
                    print("LHS DIFFERS:")
                    print(f"  Ref: {decode_opcodes(ref_lhs)}")
                    print(f"  Our: {decode_opcodes(our_lhs)}")
                else:
                    print(f"LHS matches: {decode_opcodes(ref_lhs)}")
                    
                if ref_rhs != our_rhs:
                    print("RHS DIFFERS:")
                    print(f"  Ref: {decode_opcodes(ref_rhs)}")
                    print(f"  Our: {decode_opcodes(our_rhs)}")
                else:
                    print(f"RHS matches: {decode_opcodes(ref_rhs)}")
                    
            except Exception as e:
                print(f"Error reading rule: {e}")
                break

if __name__ == '__main__':
    if len(sys.argv) != 3:
        print("Usage: debug_diff.py reference.km2 ours.km2")
        sys.exit(1)
    
    compare_files(sys.argv[1], sys.argv[2])