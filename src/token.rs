use std::{ops::Range, fmt::{Display, Debug}};

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

    Skip,
    EOF,
}

#[derive(Clone)]
pub struct Token {
    pub m_token: TokenType,
    m_token_range: Range<usize>,
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
            m_token_range: token_start..token_start + token_size,
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

    pub fn get_token_range(&self) -> &Range<usize> {
        &self.m_token_range
    }

    pub fn get_line_number(&self) -> usize {
        self.m_line
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{:?}", self.m_token).as_str())
    }
}
