use crate::parser::{KmsFile, ValueElement, PatternElement, OutputElement, VariableDecl};
use keymagic_core::*;
use std::collections::HashMap;

pub struct Compiler {
    strings: Vec<StringEntry>,
    string_map: HashMap<String, usize>,
    variables: HashMap<String, usize>,
    states: HashMap<String, usize>,
    vk_map: HashMap<&'static str, VirtualKey>,
    next_state_index: usize,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            strings: Vec::new(),
            string_map: HashMap::new(),
            variables: HashMap::new(),
            states: HashMap::new(),
            vk_map: create_vk_map(),
            next_state_index: 0,
        }
    }

    pub fn compile(mut self, ast: KmsFile) -> std::result::Result<Km2File, KmsError> {
        // First, compile all variables
        for var in &ast.variables {
            self.compile_variable(var)?;
        }

        // Scan for states in rules and register them
        for rule in &ast.rules {
            self.scan_for_states(&rule.lhs)?;
            self.scan_for_states_output(&rule.rhs)?;
        }

        // Compile rules
        let mut rules = Vec::new();
        for rule in &ast.rules {
            rules.push(self.compile_rule(rule)?);
        }

        // Create header
        let mut header = FileHeader::new();
        header.string_count = self.strings.len() as u16;
        header.rule_count = rules.len() as u16;

        // Set layout options from AST options
        self.set_layout_options(&mut header.layout_options, &ast.options);

        // Create info entries
        let info = self.create_info_entries(&ast.options);
        header.info_count = info.len() as u16;

        Ok(Km2File {
            header,
            strings: self.strings,
            info,
            rules,
        })
    }

    fn compile_variable(&mut self, var: &VariableDecl) -> std::result::Result<(), KmsError> {
        let var_name = var.name.trim_start_matches('$');
        
        // Compile the variable value to a string
        let value = self.compile_value_elements(&var.value)?;
        
        // Add to strings and map
        let index = self.add_string(value);
        self.variables.insert(var_name.to_string(), index);
        
        Ok(())
    }

    fn compile_value_elements(&mut self, elements: &[ValueElement]) -> std::result::Result<String, KmsError> {
        let mut result = String::new();
        
        for elem in elements {
            match elem {
                ValueElement::String(s) => {
                    result.push_str(&self.process_string_escapes(s)?);
                }
                ValueElement::Unicode(code) => {
                    if let Some(ch) = char::from_u32(*code) {
                        result.push(ch);
                    } else {
                        return Err(KmsError::InvalidUnicode(format!("U{:04X}", code)));
                    }
                }
                ValueElement::Variable(var) => {
                    let var_name = var.trim_start_matches('$');
                    if let Some(&idx) = self.variables.get(var_name) {
                        // For now, we'll inline the variable value
                        // In the actual implementation, we'd use opVARIABLE
                        result.push_str(&self.strings[idx].value);
                    } else {
                        return Err(KmsError::UndefinedVariable(var.clone()));
                    }
                }
            }
        }
        
        Ok(result)
    }

    fn process_string_escapes(&self, s: &str) -> std::result::Result<String, KmsError> {
        let mut result = String::new();
        let mut chars = s.chars();
        
        while let Some(ch) = chars.next() {
            if ch == '\\' {
                match chars.next() {
                    Some('n') => result.push('\n'),
                    Some('r') => result.push('\r'),
                    Some('t') => result.push('\t'),
                    Some('\\') => result.push('\\'),
                    Some('"') => result.push('"'),
                    Some('\'') => result.push('\''),
                    Some('u') => {
                        // Parse \uXXXX
                        let hex: String = chars.by_ref().take(4).collect();
                        if hex.len() == 4 {
                            if let Ok(code) = u32::from_str_radix(&hex, 16) {
                                if let Some(unicode_char) = char::from_u32(code) {
                                    result.push(unicode_char);
                                } else {
                                    return Err(KmsError::InvalidUnicode(format!("\\u{}", hex)));
                                }
                            } else {
                                return Err(KmsError::InvalidUnicode(format!("\\u{}", hex)));
                            }
                        } else {
                            return Err(KmsError::InvalidUnicode("\\u incomplete".to_string()));
                        }
                    }
                    Some('x') => {
                        // Parse \xXX
                        let hex: String = chars.by_ref().take(2).collect();
                        if hex.len() == 2 {
                            if let Ok(code) = u8::from_str_radix(&hex, 16) {
                                result.push(code as char);
                            } else {
                                return Err(KmsError::InvalidUnicode(format!("\\x{}", hex)));
                            }
                        } else {
                            return Err(KmsError::InvalidUnicode("\\x incomplete".to_string()));
                        }
                    }
                    Some(c) => {
                        result.push('\\');
                        result.push(c);
                    }
                    None => result.push('\\'),
                }
            } else {
                result.push(ch);
            }
        }
        
        Ok(result)
    }

    fn compile_rule(&mut self, rule: &crate::parser::RuleDecl) -> std::result::Result<Rule, KmsError> {
        let lhs = self.compile_pattern(&rule.lhs)?;
        let rhs = self.compile_output(&rule.rhs)?;
        
        Ok(Rule { lhs, rhs })
    }

    fn compile_pattern(&mut self, pattern: &[PatternElement]) -> std::result::Result<Vec<BinaryFormatElement>, KmsError> {
        let mut elements = Vec::new();
        
        for elem in pattern {
            match elem {
                PatternElement::String(s) => {
                    let processed = self.process_string_escapes(s)?;
                    elements.push(BinaryFormatElement::String(processed));
                }
                PatternElement::Unicode(code) => {
                    if let Some(ch) = char::from_u32(*code) {
                        elements.push(BinaryFormatElement::String(ch.to_string()));
                    } else {
                        return Err(KmsError::InvalidUnicode(format!("U{:04X}", code)));
                    }
                }
                PatternElement::Variable(var) => {
                    let var_name = var.trim_start_matches('$');
                    if let Some(&idx) = self.variables.get(var_name) {
                        elements.push(BinaryFormatElement::Variable(idx + 1)); // 1-based
                    } else {
                        return Err(KmsError::UndefinedVariable(var.clone()));
                    }
                }
                PatternElement::VariableAnyOf(var) => {
                    let var_name = var.trim_start_matches('$');
                    if let Some(&idx) = self.variables.get(var_name) {
                        // Reference parser uses: opVARIABLE + idx, then opMODIFIER + opANYOF
                        elements.push(BinaryFormatElement::Variable(idx + 1)); // 1-based
                        elements.push(BinaryFormatElement::Modifier(FLAG_ANYOF));
                    } else {
                        return Err(KmsError::UndefinedVariable(var.clone()));
                    }
                }
                PatternElement::VariableNotAnyOf(var) => {
                    let var_name = var.trim_start_matches('$');
                    if let Some(&idx) = self.variables.get(var_name) {
                        // Reference parser uses: opVARIABLE + idx, then opMODIFIER + opNANYOF
                        elements.push(BinaryFormatElement::Variable(idx + 1)); // 1-based
                        elements.push(BinaryFormatElement::Modifier(FLAG_NANYOF));
                    } else {
                        return Err(KmsError::UndefinedVariable(var.clone()));
                    }
                }
                PatternElement::VirtualKey(key) => {
                    // Reference parser always adds opAND for VK in angle brackets
                    elements.push(BinaryFormatElement::And);
                    if let Some(&vk) = self.vk_map.get(key.as_str()) {
                        elements.push(BinaryFormatElement::Predefined(vk as u16));
                    } else {
                        return Err(KmsError::InvalidVirtualKey(key.clone()));
                    }
                }
                PatternElement::VirtualKeyCombo(keys) => {
                    // Reference parser puts opAND first, then all the keys
                    elements.push(BinaryFormatElement::And);
                    for key in keys {
                        if let Some(&vk) = self.vk_map.get(key.as_str()) {
                            elements.push(BinaryFormatElement::Predefined(vk as u16));
                        } else {
                            return Err(KmsError::InvalidVirtualKey(key.clone()));
                        }
                    }
                }
                PatternElement::Any => {
                    elements.push(BinaryFormatElement::Any);
                }
                PatternElement::State(state) => {
                    if let Some(&idx) = self.states.get(state) {
                        elements.push(BinaryFormatElement::Switch(idx));
                    } else {
                        return Err(KmsError::InvalidRule(format!("Unknown state: {}", state)));
                    }
                }
            }
        }
        
        Ok(elements)
    }

    fn compile_output(&mut self, output: &[OutputElement]) -> std::result::Result<Vec<BinaryFormatElement>, KmsError> {
        let mut elements = Vec::new();
        
        for elem in output {
            match elem {
                OutputElement::String(s) => {
                    let processed = self.process_string_escapes(s)?;
                    elements.push(BinaryFormatElement::String(processed));
                }
                OutputElement::Unicode(code) => {
                    if let Some(ch) = char::from_u32(*code) {
                        elements.push(BinaryFormatElement::String(ch.to_string()));
                    } else {
                        return Err(KmsError::InvalidUnicode(format!("U{:04X}", code)));
                    }
                }
                OutputElement::Variable(var) => {
                    let var_name = var.trim_start_matches('$');
                    if let Some(&idx) = self.variables.get(var_name) {
                        elements.push(BinaryFormatElement::Variable(idx + 1)); // 1-based
                    } else {
                        return Err(KmsError::UndefinedVariable(var.clone()));
                    }
                }
                OutputElement::VariableIndexed(var, idx) => {
                    let var_name = var.trim_start_matches('$');
                    if let Some(&var_idx) = self.variables.get(var_name) {
                        elements.push(BinaryFormatElement::Variable(var_idx + 1)); // 1-based
                        elements.push(BinaryFormatElement::Modifier(*idx as u16)); // opMODIFIER + numeric index
                    } else {
                        return Err(KmsError::UndefinedVariable(var.clone()));
                    }
                }
                OutputElement::BackRef(idx) => {
                    elements.push(BinaryFormatElement::Reference(*idx));
                }
                OutputElement::Null => {
                    // NULL is represented as opPREDEFINED(1) = pdNULL
                    elements.push(BinaryFormatElement::Predefined(1));
                }
                OutputElement::State(state) => {
                    if let Some(&idx) = self.states.get(state) {
                        elements.push(BinaryFormatElement::Switch(idx));
                    } else {
                        return Err(KmsError::InvalidRule(format!("Unknown state: {}", state)));
                    }
                }
            }
        }
        
        Ok(elements)
    }

    fn add_string(&mut self, s: String) -> usize {
        if let Some(&idx) = self.string_map.get(&s) {
            idx
        } else {
            let idx = self.strings.len();
            self.strings.push(StringEntry { value: s.clone() });
            self.string_map.insert(s, idx);
            idx
        }
    }

    fn set_layout_options(&self, options: &mut LayoutOptions, ast_options: &HashMap<String, String>) {
        if let Some(v) = ast_options.get("TRACK_CAPSLOCK") {
            options.track_caps = if v.to_uppercase() == "TRUE" { 1 } else { 0 };
        }
        if let Some(v) = ast_options.get("SMART_BACKSPACE") {
            options.auto_bksp = if v.to_uppercase() == "TRUE" { 1 } else { 0 };
        }
        if let Some(v) = ast_options.get("EAT_ALL_UNUSED_KEYS") {
            options.eat = if v.to_uppercase() == "TRUE" { 1 } else { 0 };
        }
        if let Some(v) = ast_options.get("US_LAYOUT_BASED") {
            options.pos_based = if v.to_uppercase() == "TRUE" { 1 } else { 0 };
        }
        if let Some(v) = ast_options.get("TREAT_CTRL_ALT_AS_RALT") {
            options.right_alt = if v.to_uppercase() == "TRUE" { 1 } else { 0 };
        }
    }

    fn create_info_entries(&self, options: &HashMap<String, String>) -> Vec<InfoEntry> {
        let mut entries = Vec::new();
        
        if let Some(name) = options.get("NAME") {
            entries.push(InfoEntry {
                id: *INFO_NAME,
                data: self.string_to_utf8(name),
            });
        }
        
        if let Some(desc) = options.get("DESCRIPTION") {
            entries.push(InfoEntry {
                id: *INFO_DESC,
                data: self.string_to_utf8(desc),
            });
        }
        
        if let Some(font) = options.get("FONTFAMILY") {
            entries.push(InfoEntry {
                id: *INFO_FONT,
                data: self.string_to_utf8(font),
            });
        }
        
        if let Some(hotkey) = options.get("HOTKEY") {
            // Store hotkey as raw UTF-8 string (matching original C++ implementation)
            entries.push(InfoEntry {
                id: *INFO_HTKY,
                data: self.string_to_utf8(hotkey),
            });
        }
        
        // TODO: Handle ICON
        
        entries
    }

    fn string_to_utf8(&self, s: &str) -> Vec<u8> {
        s.as_bytes().to_vec()
    }

    fn scan_for_states(&mut self, pattern: &[PatternElement]) -> std::result::Result<(), KmsError> {
        for elem in pattern {
            if let PatternElement::State(state) = elem {
                self.register_state(state);
            }
        }
        Ok(())
    }

    fn scan_for_states_output(&mut self, output: &[OutputElement]) -> std::result::Result<(), KmsError> {
        for elem in output {
            if let OutputElement::State(state) = elem {
                self.register_state(state);
            }
        }
        Ok(())
    }

    fn register_state(&mut self, state: &str) {
        if !self.states.contains_key(state) {
            // States use their own index counter, not string table indices
            // This ensures each state gets a unique index
            let index = self.next_state_index;
            self.states.insert(state.to_string(), index);
            self.next_state_index += 1;
        }
    }

}