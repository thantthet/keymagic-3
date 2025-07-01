use keymagic_core::{km2::Km2Loader, Metadata};
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <km2-file>", args[0]);
        std::process::exit(1);
    }

    let km2_path = &args[1];
    let km2_bytes = fs::read(km2_path).expect("Failed to read KM2 file");
    
    match Km2Loader::load(&km2_bytes) {
        Ok(km2) => {
            // Get the metadata object for convenient access
            let metadata = km2.metadata();
            
            println!("Keyboard Information:");
            println!("====================");
            
            if let Some(name) = metadata.name() {
                println!("Name: {}", name);
            }
            
            if let Some(desc) = metadata.description() {
                println!("Description: {}", desc);
            }
            
            if let Some(font) = metadata.font_family() {
                println!("Font Family: {}", font);
            }
            
            if let Some(hotkey) = metadata.hotkey() {
                println!("Hotkey: {}", hotkey);
            }
            
            if let Some(icon) = metadata.icon() {
                println!("Icon: {} bytes", icon.len());
            }
            
            println!("\nLayout Options:");
            println!("===============");
            let opts = &km2.header.layout_options;
            println!("Track Caps Lock: {}", opts.track_caps == 1);
            println!("Smart Backspace: {}", opts.auto_bksp == 1);
            println!("Eat Unused Keys: {}", opts.eat == 1);
            println!("US Layout Based: {}", opts.pos_based == 1);
            println!("Treat Ctrl+Alt as RAlt: {}", opts.right_alt == 1);
            
            println!("\nStatistics:");
            println!("===========");
            println!("Rules: {}", km2.rules.len());
            println!("Variables: {}", km2.strings.len());
            println!("Metadata Entries: {}", metadata.len());
            
            // Demonstrate using the Metadata struct independently
            print_all_metadata(&metadata);
            
            // Example of checking for specific metadata
            println!("\nMetadata Checks:");
            println!("================");
            println!("Has name: {}", metadata.has(b"eman"));
            println!("Has description: {}", metadata.has(b"csed"));
            println!("Has font: {}", metadata.has(b"tnof"));
            println!("Has icon: {}", metadata.has(b"noci"));
            println!("Has hotkey: {}", metadata.has(b"ykth"));
        }
        Err(e) => {
            eprintln!("Failed to load KM2 file: {:?}", e);
            std::process::exit(1);
        }
    }
}

// Function that works with just the Metadata struct
fn print_all_metadata(metadata: &Metadata) {
    println!("\nAll Metadata Entries:");
    println!("====================");
    for (id, data) in metadata.iter() {
        let id_str = String::from_utf8_lossy(id);
        // Try to interpret as text first
        if let Ok(text) = std::str::from_utf8(data) {
            println!("{}: \"{}\"", id_str, text);
        } else {
            // Binary data
            println!("{}: <binary data, {} bytes>", id_str, data.len());
        }
    }
}

