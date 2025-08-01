use crate::lexer::{Lexer, Token, parse_options_from_comment};
use keymagic_core::KmsError;
use super::ast::*;
use std::collections::HashMap;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Option<Token>,
    peek: Option<Token>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer::new(input);
        // Don't silently ignore lexer errors - we'll handle them in parse()
        let current = match lexer.next_token() {
            Ok(token) => token,
            Err(_) => None, // We'll check for errors properly in parse()
        };
        let peek = lexer.peek();
        
        Self {
            lexer,
            current,
            peek,
        }
    }

    pub fn parse(&mut self) -> Result<KmsFile, KmsError> {
        // Check if we had an error getting the first token
        // This happens when Parser::new encountered a lexer error
        if self.current.is_none() && !self.lexer.input.is_empty() {
            // Try to get the actual error by attempting to read the first token again
            let mut temp_lexer = Lexer::new(self.lexer.input);
            if let Err(e) = temp_lexer.next_token() {
                return Err(e);
            }
        }
        
        let mut ast = KmsFile::new();
        
        // First pass: collect options from the raw input
        ast.options = self.extract_options_from_input();
        
        // Parse the file
        while let Some(token) = &self.current {
            match token {
                Token::Include => self.parse_include(&mut ast)?,
                Token::Variable(_) => {
                    // Check if this is a variable declaration or part of a rule
                    if self.peek == Some(Token::Equals) {
                        let var = self.parse_variable_decl()?;
                        ast.variables.push(var);
                    } else {
                        // It's part of a rule
                        let rule = self.parse_rule()?;
                        ast.rules.push(rule);
                    }
                }
                _ => {
                    // Try to parse as a rule
                    let rule = self.parse_rule()?;
                    ast.rules.push(rule);
                }
            }
        }
        
        Ok(ast)
    }

    fn extract_options_from_input(&self) -> HashMap<String, String> {
        // This is a hack to extract options from comments
        // In a real implementation, we'd modify the lexer to preserve option comments
        let mut options = HashMap::new();
        
        // First check for multi-line comments /* */
        if let Some(start) = self.lexer.input.find("/*") {
            if let Some(end) = self.lexer.input[start..].find("*/") {
                let comment = &self.lexer.input[start..start + end + 2];
                for (key, value) in parse_options_from_comment(comment) {
                    options.insert(key, value);
                }
            }
        }
        
        // Also check for single-line comments //
        for line in self.lexer.input.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") && trimmed.contains('@') {
                // Pass the line with // prefix to the parser
                for (key, value) in parse_options_from_comment(trimmed) {
                    options.insert(key, value);
                }
            }
        }
        
        options
    }

    fn advance(&mut self) -> Result<(), KmsError> {
        self.current = self.lexer.next_token()?;
        self.peek = self.lexer.peek();
        Ok(())
    }

    fn expect(&mut self, expected: Token) -> Result<(), KmsError> {
        if self.current.as_ref() != Some(&expected) {
            return Err(KmsError::Parse {
                line: self.lexer.current_line(),
                message: format!("Expected {:?}, found {:?}", expected, self.current),
            });
        }
        self.advance()
    }

    fn parse_include(&mut self, ast: &mut KmsFile) -> Result<(), KmsError> {
        self.expect(Token::Include)?;
        self.expect(Token::LParen)?;
        
        if let Some(Token::String(path)) = &self.current {
            ast.includes.push(path.clone());
            self.advance()?;
        } else {
            return Err(KmsError::Parse {
                line: self.lexer.current_line(),
                message: "Expected string literal after 'include('".to_string(),
            });
        }
        
        self.expect(Token::RParen)?;
        Ok(())
    }

    fn parse_variable_decl(&mut self) -> Result<VariableDecl, KmsError> {
        let name = if let Some(Token::Variable(n)) = &self.current {
            n.clone()
        } else {
            return Err(KmsError::Parse {
                line: self.lexer.current_line(),
                message: "Expected variable name".to_string(),
            });
        };
        
        self.advance()?;
        self.expect(Token::Equals)?;
        
        let value = self.parse_value_expr()?;
        
        Ok(VariableDecl { name, value })
    }

    fn parse_value_expr(&mut self) -> Result<Vec<ValueElement>, KmsError> {
        let mut elements = Vec::new();
        
        loop {
            match &self.current {
                Some(Token::String(s)) => {
                    elements.push(ValueElement::String(s.clone()));
                    self.advance()?;
                }
                Some(Token::Unicode(Some(code))) => {
                    elements.push(ValueElement::Unicode(*code));
                    self.advance()?;
                }
                Some(Token::Variable(v)) => {
                    elements.push(ValueElement::Variable(v.clone()));
                    self.advance()?;
                }
                _ => break,
            }
            
            // Check for concatenation
            if self.current.as_ref() == Some(&Token::Plus) {
                self.advance()?;
                // Check for line continuation
                if self.current.as_ref() == Some(&Token::Backslash) {
                    self.advance()?;
                }
            } else {
                break;
            }
        }
        
        if elements.is_empty() {
            return Err(KmsError::Parse {
                line: self.lexer.current_line(),
                message: "Expected value expression".to_string(),
            });
        }
        
        Ok(elements)
    }

    fn parse_rule(&mut self) -> Result<RuleDecl, KmsError> {
        let lhs = self.parse_pattern()?;
        self.expect(Token::Arrow)?;
        let rhs = self.parse_output()?;
        
        Ok(RuleDecl { lhs, rhs })
    }

    fn parse_pattern(&mut self) -> Result<Vec<PatternElement>, KmsError> {
        let mut elements = Vec::new();
        
        loop {
            match &self.current {
                Some(Token::String(s)) => {
                    elements.push(PatternElement::String(s.clone()));
                    self.advance()?;
                }
                Some(Token::Unicode(Some(code))) => {
                    elements.push(PatternElement::Unicode(*code));
                    self.advance()?;
                }
                Some(Token::Variable(v)) => {
                    let var_name = v.clone();
                    self.advance()?;
                    // Check for [*] or [^]
                    if self.current.as_ref() == Some(&Token::LBracket) {
                        self.advance()?;
                        match &self.current {
                            Some(Token::Star) => {
                                elements.push(PatternElement::VariableAnyOf(var_name));
                                self.advance()?;
                            }
                            Some(Token::Caret) => {
                                elements.push(PatternElement::VariableNotAnyOf(var_name));
                                self.advance()?;
                            }
                            _ => {
                                return Err(KmsError::Parse {
                                    line: self.lexer.current_line(),
                                    message: "Expected * or ^ in variable bracket".to_string(),
                                });
                            }
                        }
                        self.expect(Token::RBracket)?;
                    } else {
                        elements.push(PatternElement::Variable(var_name));
                    }
                }
                Some(Token::Any) => {
                    elements.push(PatternElement::Any);
                    self.advance()?;
                }
                Some(Token::LAngle) => {
                    elements.push(self.parse_virtual_key()?);
                }
                Some(Token::LParen) => {
                    elements.push(self.parse_state_pattern()?);
                }
                _ => break,
            }
            
            // Check for concatenation
            if self.current.as_ref() == Some(&Token::Plus) {
                self.advance()?;
            } else if matches!(&self.current, Some(Token::Arrow)) {
                break;
            }
        }
        
        if elements.is_empty() {
            return Err(KmsError::Parse {
                line: self.lexer.current_line(),
                message: "Expected pattern".to_string(),
            });
        }
        
        Ok(elements)
    }

    fn parse_virtual_key(&mut self) -> Result<PatternElement, KmsError> {
        self.expect(Token::LAngle)?;
        
        let mut keys = Vec::new();
        
        loop {
            if let Some(Token::Identifier(key)) = &self.current {
                keys.push(key.clone());
                self.advance()?;
                
                if self.current.as_ref() == Some(&Token::Ampersand) {
                    self.advance()?;
                } else {
                    break;
                }
            } else {
                return Err(KmsError::Parse {
                    line: self.lexer.current_line(),
                    message: "Expected virtual key identifier".to_string(),
                });
            }
        }
        
        self.expect(Token::RAngle)?;
        
        Ok(PatternElement::VirtualKeyCombo(keys))
    }

    fn parse_state_pattern(&mut self) -> Result<PatternElement, KmsError> {
        self.expect(Token::LParen)?;
        
        let state = if let Some(Token::String(s)) = &self.current {
            s.clone()
        } else {
            return Err(KmsError::Parse {
                line: self.lexer.current_line(),
                message: "Expected state name string".to_string(),
            });
        };
        
        self.advance()?;
        self.expect(Token::RParen)?;
        
        Ok(PatternElement::State(state))
    }

    fn parse_output(&mut self) -> Result<Vec<OutputElement>, KmsError> {
        let mut elements = Vec::new();
        
        loop {
            match &self.current {
                Some(Token::String(s)) => {
                    elements.push(OutputElement::String(s.clone()));
                    self.advance()?;
                }
                Some(Token::Unicode(Some(code))) => {
                    elements.push(OutputElement::Unicode(*code));
                    self.advance()?;
                }
                Some(Token::Variable(v)) => {
                    let var_name = v.clone();
                    self.advance()?;
                    // Check for [$n] indexing
                    if self.current.as_ref() == Some(&Token::LBracket) {
                        self.advance()?;
                        if let Some(Token::BackRef(Some(idx))) = &self.current {
                            let idx_val = *idx;
                            self.advance()?;
                            self.expect(Token::RBracket)?;
                            elements.push(OutputElement::VariableIndexed(var_name, idx_val));
                        } else {
                            return Err(KmsError::Parse {
                                line: self.lexer.current_line(),
                                message: "Expected back-reference in variable bracket".to_string(),
                            });
                        }
                    } else {
                        elements.push(OutputElement::Variable(var_name));
                    }
                }
                Some(Token::BackRef(Some(idx))) => {
                    elements.push(OutputElement::BackRef(*idx));
                    self.advance()?;
                }
                Some(Token::Null) => {
                    elements.push(OutputElement::Null);
                    self.advance()?;
                }
                Some(Token::LParen) => {
                    elements.push(self.parse_state_output()?);
                }
                _ => break,
            }
            
            // Check for concatenation
            if self.current.as_ref() == Some(&Token::Plus) {
                self.advance()?;
            } else {
                break;
            }
        }
        
        if elements.is_empty() {
            return Err(KmsError::Parse {
                line: self.lexer.current_line(),
                message: "Expected output expression".to_string(),
            });
        }
        
        Ok(elements)
    }

    fn parse_state_output(&mut self) -> Result<OutputElement, KmsError> {
        self.expect(Token::LParen)?;
        
        let state = if let Some(Token::String(s)) = &self.current {
            s.clone()
        } else {
            return Err(KmsError::Parse {
                line: self.lexer.current_line(),
                message: "Expected state name string".to_string(),
            });
        };
        
        self.advance()?;
        self.expect(Token::RParen)?;
        
        Ok(OutputElement::State(state))
    }
}