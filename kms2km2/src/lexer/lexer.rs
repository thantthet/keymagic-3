use logos::{Logos, Lexer as LogosLexer};
use keymagic_core::KmsError;
use super::Token;

pub struct Lexer<'a> {
    inner: LogosLexer<'a, Token>,
    current_line: usize,
    pub input: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            inner: Token::lexer(input),
            current_line: 1,
            input,
        }
    }

    pub fn next_token(&mut self) -> Result<Option<Token>, KmsError> {
        // Update line number based on newlines in skipped content
        let before_pos = self.inner.span().start;
        
        match self.inner.next() {
            Some(Ok(token)) => {
                // Count newlines in the span between tokens
                let span_text = &self.input[before_pos..self.inner.span().start];
                self.current_line += span_text.chars().filter(|&c| c == '\n').count();
                Ok(Some(token))
            }
            Some(Err(_)) => {
                let span = self.inner.span();
                let text = &self.input[span.start..span.end];
                Err(KmsError::Parse {
                    line: self.current_line,
                    message: format!("Unexpected token: '{}'", text),
                })
            }
            None => Ok(None),
        }
    }

    pub fn current_line(&self) -> usize {
        self.current_line
    }

    pub fn peek(&self) -> Option<Token> {
        self.inner.clone().next().and_then(|r| r.ok())
    }

    pub fn collect_all(mut self) -> Result<Vec<Token>, KmsError> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token()? {
            tokens.push(token);
        }
        Ok(tokens)
    }
}

// Special lexer for parsing option comments
pub fn parse_options_from_comment(comment: &str) -> Vec<(String, String)> {
    let mut options = Vec::new();
    
    // Remove comment markers
    let content = comment
        .trim_start_matches("/*")
        .trim_end_matches("*/")
        .trim();
    
    // Parse each line for @OPTION = "VALUE" pattern
    for line in content.lines() {
        let line = line.trim();
        if let Some(at_pos) = line.find('@') {
            let line = &line[at_pos + 1..];
            if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim();
                let value = line[eq_pos + 1..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                options.push((key.to_string(), value.to_string()));
            }
        }
    }
    
    options
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_line_tracking() {
        let input = "line1\nline2\n$var";
        let mut lexer = Lexer::new(input);
        
        let token = lexer.next_token().unwrap();
        assert!(matches!(token, Some(Token::Identifier(_))));
        assert_eq!(lexer.current_line(), 1);
        
        let token = lexer.next_token().unwrap();
        assert!(matches!(token, Some(Token::Identifier(_))));
        assert_eq!(lexer.current_line(), 2);
        
        let token = lexer.next_token().unwrap();
        assert!(matches!(token, Some(Token::Variable(_))));
        assert_eq!(lexer.current_line(), 3);
    }

    #[test]
    fn test_parse_options() {
        let comment = r#"/*
@NAME = "Myanmar Unicode"
@FONTFAMILY = "Myanmar3"
@TRACK_CAPSLOCK = "FALSE"
*/"#;
        
        let options = parse_options_from_comment(comment);
        assert_eq!(options.len(), 3);
        assert_eq!(options[0], ("NAME".to_string(), "Myanmar Unicode".to_string()));
        assert_eq!(options[1], ("FONTFAMILY".to_string(), "Myanmar3".to_string()));
        assert_eq!(options[2], ("TRACK_CAPSLOCK".to_string(), "FALSE".to_string()));
    }
}