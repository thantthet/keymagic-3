pub mod lexer;
pub mod parser;
pub mod binary;
pub mod include_processor;

pub use keymagic_core::*;

use std::path::Path;
use std::fs::File;
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
    // Use include processor to handle includes
    let mut processor = include_processor::IncludeProcessor::new();
    let ast = processor.process_file(input_path)?;
    
    // Compile to KM2
    let mut compiler = binary::Compiler::new();
    if let Some(dir) = input_path.parent() {
        compiler = compiler.with_base_dir(dir);
    }
    compiler.compile(ast)
}

pub fn compile_kms(kms_content: &str) -> std::result::Result<Km2File, KmsError> {
    compile_kms_with_base_dir(kms_content, None)
}

pub fn compile_kms_with_base_dir(kms_content: &str, base_dir: Option<&Path>) -> std::result::Result<Km2File, KmsError> {
    // Use include processor to handle includes
    let mut processor = include_processor::IncludeProcessor::new();
    let ast = processor.process_string(kms_content, base_dir)?;
    
    // Compile to KM2
    let mut compiler = binary::Compiler::new();
    if let Some(dir) = base_dir {
        compiler = compiler.with_base_dir(dir);
    }
    compiler.compile(ast)
}