pub mod lexer;
pub mod parser;
pub mod binary;

pub use keymagic_core::*;

use std::path::Path;
use std::fs::{File, read_to_string};
use std::io::BufWriter;

pub fn convert_kms_to_km2(input_path: &Path, output_path: &Path) -> std::result::Result<(), KmsError> {
    // Compile KMS file
    let km2 = compile_kms_file(input_path)?;
    
    // Write output
    let file = File::create(output_path)?;
    let writer = BufWriter::new(file);
    let km2_writer = binary::Km2Writer::new(writer);
    km2_writer.write_km2_file(&km2)?;
    
    Ok(())
}

pub fn compile_kms_file(input_path: &Path) -> std::result::Result<Km2File, KmsError> {
    // Read input file
    let input = read_to_string(input_path)?;
    
    // Get the parent directory of the input file for resolving relative paths
    let base_dir = input_path.parent();
    
    compile_kms_with_base_dir(&input, base_dir)
}

pub fn compile_kms(kms_content: &str) -> std::result::Result<Km2File, KmsError> {
    compile_kms_with_base_dir(kms_content, None)
}

pub fn compile_kms_with_base_dir(kms_content: &str, base_dir: Option<&Path>) -> std::result::Result<Km2File, KmsError> {
    // Parse KMS
    let mut parser = parser::Parser::new(kms_content);
    let ast = parser.parse()?;
    
    // Compile to KM2
    let mut compiler = binary::Compiler::new();
    if let Some(dir) = base_dir {
        compiler = compiler.with_base_dir(dir);
    }
    compiler.compile(ast)
}