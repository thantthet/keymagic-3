use clap::Parser;
use std::path::PathBuf;
use kms2km2::convert_kms_to_km2;

#[derive(Parser, Debug)]
#[command(author, version, about = "KeyMagic Script to Binary Converter", long_about = None)]
struct Args {
    /// Input KMS file path
    input: PathBuf,
    
    /// Output KM2 file path (defaults to input with .km2 extension)
    output: Option<PathBuf>,
    
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();
    
    // Determine output path
    let output_path = args.output.unwrap_or_else(|| {
        let mut path = args.input.clone();
        path.set_extension("km2");
        path
    });
    
    if args.verbose {
        println!("Converting {} to {}", args.input.display(), output_path.display());
    }
    
    // Perform conversion
    match convert_kms_to_km2(&args.input, &output_path) {
        Ok(()) => {
            if args.verbose {
                println!("Conversion successful!");
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}