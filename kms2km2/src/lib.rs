pub mod lexer;
pub mod parser;
pub mod binary;

pub use keymagic_core::*;

use std::path::Path;
use std::fs::{File, read_to_string};
use std::io::BufWriter;

pub fn convert_kms_to_km2(input_path: &Path, output_path: &Path) -> Result<(), KmsError> {
    // Compile KMS file
    let km2 = compile_kms_file(input_path)?;
    
    // Write output
    let file = File::create(output_path)?;
    let writer = BufWriter::new(file);
    let km2_writer = binary::Km2Writer::new(writer);
    km2_writer.write_km2_file(&km2)?;
    
    Ok(())
}

pub fn compile_kms_file(input_path: &Path) -> Result<Km2File, KmsError> {
    // Read input file
    let input = read_to_string(input_path)?;
    
    // Parse KMS
    let mut parser = parser::Parser::new(&input);
    let ast = parser.parse()?;
    
    // Compile to KM2
    let compiler = binary::Compiler::new();
    compiler.compile(ast)
}