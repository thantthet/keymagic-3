use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    // Comments and whitespace (skipped)
    #[regex(r"//[^\n]*", logos::skip)]
    #[regex(r"/\*([^*]|\*[^/])*\*/", logos::skip)]
    #[regex(r"[ \t\r\n]+", logos::skip)]
    Comment,

    // Keywords
    #[token("include")]
    Include,
    
    #[token("ANY")]
    Any,
    
    #[token("NULL")]
    #[token("null")]
    Null,

    // Operators
    #[token("=>")]
    Arrow,
    
    #[token("+")]
    Plus,
    
    #[token("&")]
    Ampersand,
    
    #[token("\\")]
    Backslash,
    
    #[token("=")]
    Equals,

    // Delimiters
    #[token("(")]
    LParen,
    
    #[token(")")]
    RParen,
    
    #[token("[")]
    LBracket,
    
    #[token("]")]
    RBracket,
    
    #[token("<")]
    LAngle,
    
    #[token(">")]
    RAngle,

    // Special markers
    #[token("*")]
    Star,
    
    #[token("^")]
    Caret,

    // Variables
    #[regex(r"\$[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Variable(String),

    // Back-references
    #[regex(r"\$[0-9]+", |lex| lex.slice()[1..].parse::<usize>().ok())]
    BackRef(Option<usize>),

    // Unicode literals
    #[regex(r"[Uu][0-9a-fA-F]{4}", |lex| {
        u32::from_str_radix(&lex.slice()[1..], 16).ok()
    })]
    Unicode(Option<u32>),

    // String literals
    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string()
    })]
    #[regex(r#"'([^'\\]|\\.)*'"#, |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string()
    })]
    String(String),

    // Identifiers (for virtual keys, states, etc.)
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    // Options (in comments)
    #[regex(r"@[A-Z_]+", |lex| lex.slice()[1..].to_string())]
    Option(String),

    Error,
}

impl Token {
    pub fn is_option(&self) -> bool {
        matches!(self, Token::Option(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use logos::Logos;

    #[test]
    fn test_basic_tokens() {
        let input = r#"$var => "output" + U1000"#;
        let mut lex = Token::lexer(input);
        
        assert_eq!(lex.next(), Some(Ok(Token::Variable("$var".to_string()))));
        assert_eq!(lex.next(), Some(Ok(Token::Arrow)));
        assert_eq!(lex.next(), Some(Ok(Token::String("output".to_string()))));
        assert_eq!(lex.next(), Some(Ok(Token::Plus)));
        assert_eq!(lex.next(), Some(Ok(Token::Unicode(Some(0x1000)))));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_virtual_keys() {
        let input = "<VK_SHIFT & VK_KEY_A>";
        let mut lex = Token::lexer(input);
        
        assert_eq!(lex.next(), Some(Ok(Token::LAngle)));
        assert_eq!(lex.next(), Some(Ok(Token::Identifier("VK_SHIFT".to_string()))));
        assert_eq!(lex.next(), Some(Ok(Token::Ampersand)));
        assert_eq!(lex.next(), Some(Ok(Token::Identifier("VK_KEY_A".to_string()))));
        assert_eq!(lex.next(), Some(Ok(Token::RAngle)));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_comments_skipped() {
        let input = "// comment\n$var /* block */ => ANY";
        let mut lex = Token::lexer(input);
        
        assert_eq!(lex.next(), Some(Ok(Token::Variable("$var".to_string()))));
        assert_eq!(lex.next(), Some(Ok(Token::Arrow)));
        assert_eq!(lex.next(), Some(Ok(Token::Any)));
        assert_eq!(lex.next(), None);
    }
}