use std::fs::File;
use std::io::Read;
use byteorder::{LittleEndian, ReadBytesExt};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <km2_file>", args[0]);
        std::process::exit(1);
    }

    let mut file = File::open(&args[1])?;
    
    // Read header
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)?;
    println!("Magic: {:?} ({})", magic, String::from_utf8_lossy(&magic));
    
    let major = file.read_u8()?;
    let minor = file.read_u8()?;
    println!("Version: {}.{}", major, minor);
    
    let string_count = file.read_u16::<LittleEndian>()?;
    let info_count = file.read_u16::<LittleEndian>()?;
    let rule_count = file.read_u16::<LittleEndian>()?;
    println!("Counts: {} strings, {} info, {} rules", string_count, info_count, rule_count);
    
    // Layout options
    let track_caps = file.read_u8()?;
    let auto_bksp = file.read_u8()?;
    let eat = file.read_u8()?;
    let pos_based = file.read_u8()?;
    let right_alt = file.read_u8()?;
    let _padding = file.read_u8()?; // Padding byte for C++ struct alignment
    println!("Options: track_caps={}, auto_bksp={}, eat={}, pos_based={}, right_alt={}", 
             track_caps, auto_bksp, eat, pos_based, right_alt);
    
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
    
    // Read info
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
    
    while remaining > 0 {
        let opcode = file.read_u16::<LittleEndian>()?;
        remaining -= 1;
        
        match opcode {
            0x00F0 => { // opSTRING
                let str_len = file.read_u16::<LittleEndian>()?;
                remaining -= 1;
                let mut utf16_data = vec![0u16; str_len as usize];
                for i in 0..str_len {
                    utf16_data[i as usize] = file.read_u16::<LittleEndian>()?;
                    remaining -= 1;
                }
                let s = String::from_utf16_lossy(&utf16_data);
                print!("STRING(\"{}\") ", s);
            }
            0x00F1 => { // opVARIABLE
                let idx = file.read_u16::<LittleEndian>()?;
                remaining -= 1;
                print!("VAR({}) ", idx);
            }
            0x00F2 => { // opREFERENCE
                let idx = file.read_u16::<LittleEndian>()?;
                remaining -= 1;
                print!("REF({}) ", idx);
            }
            0x00F3 => { // opPREDEFINED
                let vk = file.read_u16::<LittleEndian>()?;
                remaining -= 1;
                print!("VK({}) ", vk);
            }
            0x00F4 => { // opMODIFIER
                let mods = file.read_u16::<LittleEndian>()?;
                remaining -= 1;
                print!("MOD({}) ", mods);
            }
            0x00F5 => { // opANYOF
                let idx = file.read_u16::<LittleEndian>()?;
                remaining -= 1;
                print!("ANYOF({}) ", idx);
            }
            0x00F6 => { // opAND
                print!("AND ");
            }
            0x00F7 => { // opNANYOF
                let idx = file.read_u16::<LittleEndian>()?;
                remaining -= 1;
                print!("NANYOF({}) ", idx);
            }
            0x00F8 => { // opANY
                print!("ANY ");
            }
            0x00F9 => { // opSWITCH
                let str_len = file.read_u16::<LittleEndian>()?;
                remaining -= 1;
                let mut utf16_data = vec![0u16; str_len as usize];
                for i in 0..str_len {
                    utf16_data[i as usize] = file.read_u16::<LittleEndian>()?;
                    remaining -= 1;
                }
                let s = String::from_utf16_lossy(&utf16_data);
                print!("SWITCH(\"{}\") ", s);
            }
            _ => {
                print!("UNKNOWN(0x{:04X}) ", opcode);
            }
        }
    }
    
    println!();
    Ok(())
}