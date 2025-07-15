use std::fs::File;
use std::io::Read;
use byteorder::{LittleEndian, ReadBytesExt};
use keymagic_core::km2::Km2Loader;
use keymagic_core::types::opcodes::*;
use keymagic_core::types::virtual_keys::VirtualKey;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <km2_file>", args[0]);
        std::process::exit(1);
    }

    // First, use the keymagic-core loader to parse the file properly
    let file_data = std::fs::read(&args[1])?;
    
    // Try to load with keymagic-core to validate
    match Km2Loader::load(&file_data) {
        Ok(km2) => {
            println!("Successfully loaded KM2 file with keymagic-core");
            // Copy values from packed struct to avoid alignment issues
            let major_version = km2.header.major_version;
            let minor_version = km2.header.minor_version;
            let string_count = km2.header.string_count;
            let info_count = km2.header.info_count;
            let rule_count = km2.header.rule_count;
            let track_caps = km2.header.layout_options.track_caps;
            let auto_bksp = km2.header.layout_options.auto_bksp;
            let eat = km2.header.layout_options.eat;
            let pos_based = km2.header.layout_options.pos_based;
            let right_alt = km2.header.layout_options.right_alt;
            
            println!("Version: {}.{}", major_version, minor_version);
            println!("Counts: {} strings, {} info, {} rules", 
                     string_count, info_count, rule_count);
            println!("Layout options: track_caps={}, auto_bksp={}, eat={}, pos_based={}, right_alt={}",
                     track_caps, auto_bksp, eat, pos_based, right_alt);
        }
        Err(e) => {
            eprintln!("Warning: keymagic-core failed to load: {:?}", e);
            eprintln!("Continuing with raw dump...");
        }
    }
    
    println!("\n=== RAW DUMP ===");
    
    // Now do the raw dump
    let mut file = File::open(&args[1])?;
    
    // Read header
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)?;
    println!("Magic: {:?} ({})", magic, String::from_utf8_lossy(&magic));
    
    if &magic != b"KMKL" {
        eprintln!("Error: Invalid magic code, not a KM2 file");
        std::process::exit(1);
    }
    
    let major = file.read_u8()?;
    let minor = file.read_u8()?;
    println!("Version: {}.{}", major, minor);
    
    // Handle different versions
    let (string_count, info_count, rule_count, has_right_alt) = match (major, minor) {
        (1, 3) => {
            // v1.3: no info_count
            let string_count = file.read_u16::<LittleEndian>()?;
            let rule_count = file.read_u16::<LittleEndian>()?;
            println!("Counts: {} strings, 0 info (v1.3), {} rules", string_count, rule_count);
            (string_count, 0u16, rule_count, false)
        }
        (1, 4) => {
            // v1.4: has info_count but no right_alt
            let string_count = file.read_u16::<LittleEndian>()?;
            let info_count = file.read_u16::<LittleEndian>()?;
            let rule_count = file.read_u16::<LittleEndian>()?;
            println!("Counts: {} strings, {} info, {} rules", string_count, info_count, rule_count);
            (string_count, info_count, rule_count, false)
        }
        _ => {
            // v1.5 and newer
            let string_count = file.read_u16::<LittleEndian>()?;
            let info_count = file.read_u16::<LittleEndian>()?;
            let rule_count = file.read_u16::<LittleEndian>()?;
            println!("Counts: {} strings, {} info, {} rules", string_count, info_count, rule_count);
            (string_count, info_count, rule_count, true)
        }
    };
    
    // Layout options
    let track_caps = file.read_u8()?;
    let auto_bksp = file.read_u8()?;
    let eat = file.read_u8()?;
    let pos_based = file.read_u8()?;
    
    if has_right_alt {
        let right_alt = file.read_u8()?;
        println!("Options: track_caps={}, auto_bksp={}, eat={}, pos_based={}, right_alt={}", 
                 track_caps, auto_bksp, eat, pos_based, right_alt);
    } else {
        println!("Options: track_caps={}, auto_bksp={}, eat={}, pos_based={}, right_alt=<not present in v{}.{}>", 
                 track_caps, auto_bksp, eat, pos_based, major, minor);
    }
    
    // Skip padding byte if present (only for certain struct sizes)
    if major == 1 && minor >= 5 {
        let _padding = file.read_u8()?; // Padding byte for C++ struct alignment
    }
    
    // Read strings
    println!("\n=== STRINGS ===");
    for i in 0..string_count {
        let len = file.read_u16::<LittleEndian>()?;
        let mut utf16_data = vec![0u16; len as usize];
        for j in 0..len {
            utf16_data[j as usize] = file.read_u16::<LittleEndian>()?;
        }
        let string = String::from_utf16_lossy(&utf16_data);
        println!("String[{}]: len={}, value=\"{}\"", i, len, string);
    }
    
    // Read info (only if version supports it)
    if info_count > 0 {
        println!("\n=== INFO ===");
        for i in 0..info_count {
            let mut id_bytes = [0u8; 4];
            file.read_exact(&mut id_bytes)?;
            let id_str = String::from_utf8_lossy(&id_bytes);
            let id_int = u32::from_le_bytes(id_bytes);
            
            let len = file.read_u16::<LittleEndian>()?;
            let mut data = vec![0u8; len as usize];
            file.read_exact(&mut data)?;
            
            println!("Info[{}]: id='{}' (0x{:08X}), len={}", i, id_str, id_int, len);
            
            // Try to interpret as UTF-8 string
            if let Ok(s) = String::from_utf8(data.clone()) {
                println!("  Data (UTF-8): \"{}\"", s);
            } else {
                // Try UTF-16
                if len % 2 == 0 {
                    let mut utf16_data = Vec::new();
                    for j in (0..len).step_by(2) {
                        let val = u16::from_le_bytes([data[j as usize], data[j as usize + 1]]);
                        utf16_data.push(val);
                    }
                    let s = String::from_utf16_lossy(&utf16_data);
                    println!("  Data (UTF-16): \"{}\"", s);
                } else {
                    println!("  Data (hex): {:02X?}", data);
                }
            }
        }
    }
    
    // Read rules
    println!("\n=== RULES ===");
    for i in 0..rule_count {
        println!("Rule[{}]:", i);
        
        // LHS
        let lhs_len = file.read_u16::<LittleEndian>()?;
        print!("  LHS (len={}): ", lhs_len);
        dump_rule_elements(&mut file, lhs_len)?;
        
        // RHS
        let rhs_len = file.read_u16::<LittleEndian>()?;
        print!("  RHS (len={}): ", rhs_len);
        dump_rule_elements(&mut file, rhs_len)?;
    }
    
    Ok(())
}

fn dump_rule_elements(file: &mut File, len: u16) -> Result<(), Box<dyn std::error::Error>> {
    let mut remaining = len;
    let mut last_was_and = false;
    
    while remaining > 0 {
        let opcode = file.read_u16::<LittleEndian>()?;
        remaining -= 1;
        
        match opcode {
            OP_STRING => {
                let str_len = file.read_u16::<LittleEndian>()?;
                remaining -= 1;
                let mut utf16_data = vec![0u16; str_len as usize];
                for i in 0..str_len {
                    utf16_data[i as usize] = file.read_u16::<LittleEndian>()?;
                    remaining -= 1;
                }
                let s = String::from_utf16_lossy(&utf16_data);
                print!("STRING(\"{}\") ", s);
                last_was_and = false;
            }
            OP_VARIABLE => {
                let idx = file.read_u16::<LittleEndian>()?;
                remaining -= 1;
                print!("VAR({}) ", idx);
                last_was_and = false;
            }
            OP_REFERENCE => {
                let idx = file.read_u16::<LittleEndian>()?;
                remaining -= 1;
                print!("REF({}) ", idx);
                last_was_and = false;
            }
            OP_PREDEFINED => {
                let val = file.read_u16::<LittleEndian>()?;
                remaining -= 1;
                if last_was_and {
                    // This is a virtual key after AND
                    if let Some(vk) = VirtualKey::from_raw(val) {
                        print!("VK({:?}) ", vk);
                    } else {
                        print!("VK({}) ", val);
                    }
                } else {
                    // This is a predefined value (not a virtual key)
                    if val == 1 {
                        print!("PREDEFINED(NULL) ");
                    } else {
                        print!("PREDEFINED({}) ", val);
                    }
                }
                last_was_and = false;
            }
            OP_MODIFIER => {
                let mods = file.read_u16::<LittleEndian>()?;
                remaining -= 1;
                // Check if this is a known flag
                match mods {
                    FLAG_ANYOF => print!("MOD(FLAG_ANYOF) "),
                    FLAG_NANYOF => print!("MOD(FLAG_NANYOF) "),
                    _ => print!("MOD({}) ", mods),
                }
                last_was_and = false;
            }
            OP_AND => {
                print!("AND ");
                last_was_and = true;
            }
            OP_ANY => {
                print!("ANY ");
                last_was_and = false;
            }
            OP_SWITCH => {
                let state_idx = file.read_u16::<LittleEndian>()?;
                remaining -= 1;
                print!("SWITCH({}) ", state_idx);
                last_was_and = false;
            }
            _ => {
                print!("UNKNOWN(0x{:04X}) ", opcode);
                last_was_and = false;
            }
        }
    }
    
    println!();
    Ok(())
}