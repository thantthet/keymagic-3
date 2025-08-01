use crate::parser::{Parser, KmsFile};
use keymagic_core::KmsError;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::fs;

/// Processes KMS files with include directives
pub struct IncludeProcessor {
    /// Set of canonical paths already processed to detect circular includes
    processed_files: HashSet<PathBuf>,
    /// Base directory for resolving relative paths
    base_dir: Option<PathBuf>,
}

impl IncludeProcessor {
    pub fn new() -> Self {
        Self {
            processed_files: HashSet::new(),
            base_dir: None,
        }
    }

    pub fn with_base_dir<P: AsRef<Path>>(mut self, base_dir: P) -> Self {
        self.base_dir = Some(base_dir.as_ref().to_path_buf());
        self
    }

    /// Process a KMS file and all its includes
    pub fn process_file(&mut self, file_path: &Path) -> Result<KmsFile, KmsError> {
        let canonical_path = file_path.canonicalize()
            .map_err(|e| KmsError::Io(e))?;
        
        // Check for circular includes
        if self.processed_files.contains(&canonical_path) {
            return Err(KmsError::Parse {
                line: 0,
                message: format!("Circular include detected: {}", file_path.display()),
            });
        }
        
        self.processed_files.insert(canonical_path.clone());
        
        // Read the file
        let mut content = fs::read_to_string(file_path)
            .map_err(|e| KmsError::Io(e))?;
        
        // Strip UTF-8 BOM if present
        if content.starts_with('\u{FEFF}') {
            content = content.trim_start_matches('\u{FEFF}').to_string();
        }
        
        // Parse the file
        let mut parser = Parser::new(&content);
        let mut ast = parser.parse()?;
        
        // Process includes
        if !ast.includes.is_empty() {
            let file_dir = file_path.parent()
                .or(self.base_dir.as_deref())
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf();
            
            ast = self.process_includes(ast, &file_dir)?;
        }
        
        self.processed_files.remove(&canonical_path);
        Ok(ast)
    }

    /// Process a KMS string with base directory for includes
    pub fn process_string(&mut self, content: &str, base_dir: Option<&Path>) -> Result<KmsFile, KmsError> {
        // Parse the content
        let mut parser = Parser::new(content);
        let mut ast = parser.parse()?;
        
        // Process includes if any
        if !ast.includes.is_empty() {
            let dir = base_dir
                .or(self.base_dir.as_deref())
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf();
            
            ast = self.process_includes(ast, &dir)?;
        }
        
        Ok(ast)
    }

    /// Process all includes in an AST
    fn process_includes(&mut self, ast: KmsFile, base_dir: &Path) -> Result<KmsFile, KmsError> {
        // Since includes are processed in-place during parsing, we need a different approach
        // We'll rebuild the AST with includes expanded at their positions
        
        // For now, we'll process includes by reading them when encountered
        // This maintains the order of rules and variables as they appear in the file
        
        // Create a new AST with expanded includes
        let mut result = KmsFile::new();
        result.options = ast.options; // Keep original options
        
        // Process the original file content with includes expanded
        // Since we can't easily track the position of includes relative to rules/variables,
        // we'll need to re-parse with include expansion
        
        // For a proper implementation, we'd need to modify the parser to track
        // the position of includes relative to other elements
        
        // For now, let's append included content at the end
        // Copy original content first
        result.variables = ast.variables;
        result.rules = ast.rules;
        
        // Process each include
        for include_path in ast.includes {
            // Resolve the include path
            let resolved_path = if Path::new(&include_path).is_absolute() {
                PathBuf::from(&include_path)
            } else {
                base_dir.join(&include_path)
            };
            
            // Process the included file
            let included_ast = self.process_file(&resolved_path)?;
            
            // Append included content
            result.variables.extend(included_ast.variables);
            result.rules.extend(included_ast.rules);
        }
        
        Ok(result)
    }

}