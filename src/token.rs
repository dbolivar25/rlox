use std::{fmt::Debug, ops::Range};

#[derive(Debug, PartialEq, Clone)]
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
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Error(String),
    Skip,
    Eof,
}

#[derive(Clone)]
pub struct Token {
    pub m_token: TokenType,
    m_col_range: Range<usize>,
    m_line: usize,
}

impl TokenType {
    pub fn new_identifier(lexeme: String) -> TokenType {
        match lexeme.as_str() {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "fun" => TokenType::Fun,
            "for" => TokenType::For,
            "if" => TokenType::If,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "true" => TokenType::True,
            "var" => TokenType::Var,
            "while" => TokenType::While,
            _ => TokenType::Identifier(lexeme),
        }
    }
}

impl Token {
    pub fn new_token(
        token_type: TokenType,
        token_start: usize,
        token_size: usize,
        line_number: usize,
    ) -> Token {
        Token {
            m_token: token_type,
            m_col_range: token_start..token_start + token_size,
            m_line: line_number,
        }
    }
    //
    // pub fn new_literal(
    //     lexeme: String,
    //     token_start: usize,
    //     token_size: usize,
    //     line_number: usize,
    // ) -> Token {
    //     Token {
    //         m_token: TokenType::new_literal(lexeme),
    //         m_token_range: token_start..token_start + token_size,
    //         m_line: line_number,
    //     }
    // }
    //
    pub fn get_token_type(&self) -> &TokenType {
        &self.m_token
    }

    pub fn get_col_range(&self) -> &Range<usize> {
        &self.m_col_range
    }

    pub fn get_line_number(&self) -> usize {
        self.m_line
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self.get_token_type() {
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
                TokenType::Print => "print".to_string(),
                TokenType::Return => "return".to_string(),
                TokenType::Super => "super".to_string(),
                TokenType::This => "this".to_string(),
                TokenType::True => "true".to_string(),
                TokenType::Var => "var".to_string(),
                TokenType::While => "while".to_string(),
                token => format!("{:?}", token),
            }
        )
    }
}
