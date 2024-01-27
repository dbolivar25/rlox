use std::fmt::{Debug, Display};

#[derive(Clone, PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier(String),
    String(String),
    Number(f64),

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    // Prnt,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    // Error(String),
    Skip,
    Eof,
}

impl TokenType {
    pub fn new_identifier(name: &str) -> TokenType {
        match name {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "if" => TokenType::If,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            // "print" => Token::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "true" => TokenType::True,
            "let" => TokenType::Var,
            "while" => TokenType::While,
            _ => TokenType::Identifier(name.to_string()),
        }
    }
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TokenType::Skip => "skip".to_string(),
                TokenType::Eof => "eof".to_string(),
                TokenType::LeftParen => "(".to_string(),
                TokenType::RightParen => ")".to_string(),
                TokenType::LeftBrace => "{".to_string(),
                TokenType::RightBrace => "}".to_string(),
                TokenType::Comma => ",".to_string(),
                TokenType::Plus => "+".to_string(),
                TokenType::Minus => "-".to_string(),
                TokenType::Semicolon => ";".to_string(),
                TokenType::Slash => "/".to_string(),
                TokenType::Star => "*".to_string(),
                TokenType::Bang => "!".to_string(),
                TokenType::BangEqual => "!=".to_string(),
                TokenType::EqualEqual => "==".to_string(),
                TokenType::Greater => ">".to_string(),
                TokenType::GreaterEqual => ">=".to_string(),
                TokenType::Less => "<".to_string(),
                TokenType::LessEqual => "<=".to_string(),
                TokenType::Equal => "=".to_string(),
                TokenType::And => "and".to_string(),
                TokenType::Class => "class".to_string(),
                TokenType::Else => "else".to_string(),
                TokenType::False => "false".to_string(),
                TokenType::Fun => "fun".to_string(),
                TokenType::Dot => ".".to_string(),
                TokenType::For => "for".to_string(),
                TokenType::If => "if".to_string(),
                TokenType::Nil => "nil".to_string(),
                TokenType::Or => "or".to_string(),
                // Token::Print => "print".to_string(),
                TokenType::Return => "return".to_string(),
                TokenType::Super => "super".to_string(),
                TokenType::This => "this".to_string(),
                TokenType::True => "true".to_string(),
                TokenType::Var => "let".to_string(),
                TokenType::While => "while".to_string(),
                TokenType::Identifier(identifier) => identifier.to_string(),
                TokenType::String(string) => string.to_string(),
                TokenType::Number(number) => number.to_string(),
                // TokenType::Error(error) => error.to_string(),
            }
        )
    }
}

impl Debug for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::String(string) => write!(f, "\"{}\"", string),
            other => write!(f, "{}", other),
        }
    }
}
