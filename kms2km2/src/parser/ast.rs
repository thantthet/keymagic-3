use std::collections::HashMap;

// AST nodes for KMS parsing
#[derive(Debug)]
pub struct KmsFile {
    pub options: HashMap<String, String>,
    pub variables: Vec<VariableDecl>,
    pub rules: Vec<RuleDecl>,
    pub includes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct VariableDecl {
    pub name: String,
    pub value: Vec<ValueElement>,
}

#[derive(Debug, Clone)]
pub struct RuleDecl {
    pub lhs: Vec<PatternElement>,
    pub rhs: Vec<OutputElement>,
}

#[derive(Debug, Clone)]
pub enum ValueElement {
    String(String),
    Unicode(u32),
    Variable(String),
}

#[derive(Debug, Clone)]
pub enum PatternElement {
    String(String),
    Unicode(u32),
    Variable(String),
    VariableAnyOf(String),      // $var[*]
    VariableNotAnyOf(String),   // $var[^]
    VirtualKey(String),
    VirtualKeyCombo(Vec<String>), // <VK_SHIFT & VK_A>
    Any,
    State(String),              // ('state_name')
}

#[derive(Debug, Clone)]
pub enum OutputElement {
    String(String),
    Unicode(u32),
    Variable(String),
    VariableIndexed(String, usize), // $var[$1]
    BackRef(usize),             // $1, $2, etc.
    Null,
    State(String),              // ('state_name')
}

impl KmsFile {
    pub fn new() -> Self {
        Self {
            options: HashMap::new(),
            variables: Vec::new(),
            rules: Vec::new(),
            includes: Vec::new(),
        }
    }
}