use crate::token::*;

use anyhow::Result;
use itertools::*;

#[derive(Debug)]
pub struct Lexer<'a> {
    m_chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    m_line_number: usize,
    m_tokens: Vec<Token>,
    m_errors: Vec<String>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            m_chars: input.char_indices().peekable(),
            m_line_number: 1,
            m_tokens: Vec::new(),
            m_errors: Vec::new(),
        }
    }

    fn lex_token(&mut self) -> Result<Token, String> {
        return match self.m_chars.next() {
            Some((index, char)) => match char {
                '\n' => {
                    self.m_line_number += 1;
                    Ok(Token::new_token(
                        TokenType::Skip,
                        index,
                        1,
                        self.m_line_number,
                    ))
                }
                ' ' | '\r' | '\t' => Ok(Token::new_token(
                    TokenType::Skip,
                    index,
                    1,
                    self.m_line_number,
                )),
                '(' => Ok(Token::new_token(
                    TokenType::LeftParen,
                    index,
                    1,
                    self.m_line_number,
                )),
                ')' => Ok(Token::new_token(
                    TokenType::RightParen,
                    index,
                    1,
                    self.m_line_number,
                )),
                '{' => Ok(Token::new_token(
                    TokenType::LeftBrace,
                    index,
                    1,
                    self.m_line_number,
                )),
                '}' => Ok(Token::new_token(
                    TokenType::RightBrace,
                    index,
                    1,
                    self.m_line_number,
                )),
                ',' => Ok(Token::new_token(
                    TokenType::Comma,
                    index,
                    1,
                    self.m_line_number,
                )),
                '.' => Ok(Token::new_token(
                    TokenType::Dot,
                    index,
                    1,
                    self.m_line_number,
                )),
                '-' => Ok(Token::new_token(
                    TokenType::Minus,
                    index,
                    1,
                    self.m_line_number,
                )),
                '+' => Ok(Token::new_token(
                    TokenType::Plus,
                    index,
                    1,
                    self.m_line_number,
                )),
                ';' => Ok(Token::new_token(
                    TokenType::Semicolon,
                    index,
                    1,
                    self.m_line_number,
                )),
                '*' => Ok(Token::new_token(
                    TokenType::Star,
                    index,
                    1,
                    self.m_line_number,
                )),
                '!' => match self.m_chars.peek() {
                    Some((_, '=')) => {
                        self.m_chars.next();
                        Ok(Token::new_token(
                            TokenType::BangEqual,
                            index,
                            2,
                            self.m_line_number,
                        ))
                    }
                    _ => Ok(Token::new_token(
                        TokenType::Bang,
                        index,
                        1,
                        self.m_line_number,
                    )),
                },
                '=' => match self.m_chars.peek() {
                    Some((_, '=')) => {
                        self.m_chars.next();
                        Ok(Token::new_token(
                            TokenType::EqualEqual,
                            index,
                            2,
                            self.m_line_number,
                        ))
                    }
                    _ => Ok(Token::new_token(
                        TokenType::Equal,
                        index,
                        1,
                        self.m_line_number,
                    )),
                },
                '<' => match self.m_chars.peek() {
                    Some((_, '=')) => {
                        self.m_chars.next();
                        Ok(Token::new_token(
                            TokenType::LessEqual,
                            index,
                            2,
                            self.m_line_number,
                        ))
                    }
                    _ => Ok(Token::new_token(
                        TokenType::Less,
                        index,
                        1,
                        self.m_line_number,
                    )),
                },
                '>' => match self.m_chars.peek() {
                    Some((_, '=')) => {
                        self.m_chars.next();
                        Ok(Token::new_token(
                            TokenType::GreaterEqual,
                            index,
                            2,
                            self.m_line_number,
                        ))
                    }
                    _ => Ok(Token::new_token(
                        TokenType::Greater,
                        index,
                        1,
                        self.m_line_number,
                    )),
                },
                '/' => match self.m_chars.peek() {
                    Some((_, '/')) => {
                        self.m_chars.next();
                        for (_, char) in self.m_chars.by_ref() {
                            if char == '\n' {
                                self.m_line_number += 1;
                                break;
                            }
                        }
                        Ok(Token::new_token(
                            TokenType::Skip,
                            index,
                            1,
                            self.m_line_number,
                        ))
                    }
                    _ => Ok(Token::new_token(
                        TokenType::Slash,
                        index,
                        1,
                        self.m_line_number,
                    )),
                },
                '"' => {
                    let mut lexeme = String::new();
                    loop {
                        match self.m_chars.next() {
                            Some((_, '"')) => break,
                            Some((_, '\n')) => self.m_line_number += 1,
                            Some((_, char)) => lexeme.push(char),
                            None => {
                                return Err(format!(
                                    "Unterminated string \"{}\"\n           => line {} | column {}",
                                    lexeme,
                                    self.m_line_number,
                                    index + 1 + lexeme.len()
                                ))
                            }
                        }
                    }

                    Ok(Token::new_token(
                        TokenType::String(lexeme.clone()),
                        index,
                        lexeme.len() + 2,
                        self.m_line_number,
                    ))
                }
                char if char.is_ascii_digit() => {
                    let mut lexeme = String::new();
                    lexeme.push(char);

                    while let Some((_, char)) = self.m_chars.peek() {
                        if char.is_ascii_digit() {
                            lexeme.push(*char);
                            self.m_chars.next();
                        } else {
                            break;
                        }
                    }

                    if let Some((_, '.')) = self.m_chars.peek() {
                        if let Some(past_point) = self.m_chars.clone().multipeek().nth(1) {
                            if past_point.1.is_ascii_digit() {
                                lexeme.push('.');
                                self.m_chars.next();

                                while let Some((_, char)) = self.m_chars.peek() {
                                    if char.is_ascii_digit() {
                                        lexeme.push(*char);
                                        self.m_chars.next();
                                    } else {
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    let parsed_float: f64 = lexeme.parse().unwrap();

                    Ok(Token::new_token(
                        TokenType::Number(parsed_float),
                        index,
                        lexeme.len(),
                        self.m_line_number,
                    ))
                }
                char if char.is_ascii_alphabetic() || char == '_' => {
                    let mut lexeme = String::new();
                    lexeme.push(char);

                    while let Some((_, char)) = self.m_chars.peek() {
                        if char.is_ascii_alphanumeric() || char == &'_' {
                            lexeme.push(*char);
                            self.m_chars.next();
                        } else {
                            break;
                        }
                    }

                    Ok(Token::new_token(
                        TokenType::new_identifier(lexeme.clone()),
                        index,
                        lexeme.len(),
                        self.m_line_number,
                    ))
                }
                bad_char => Err(format!(
                    "Unexpected character '{}'\n           => line {} | column {}",
                    bad_char,
                    self.m_line_number,
                    index + 1
                )),
            },
            None => Err("Unexpected end of input".to_string()),
        };
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, Vec<String>> {
        while self.m_chars.peek().is_some() {
            match self.lex_token() {
                Ok(token) if token.get_token_type() == &TokenType::Skip => {}
                Ok(token) => self.m_tokens.push(token),
                Err(err) => self.m_errors.push(err),
            }
        }

        if self.m_errors.is_empty() {
            self.m_tokens.push(Token::new_token(
                TokenType::Eof,
                0,
                0,
                self.m_line_number + 1,
            ));

            Ok(self.m_tokens.clone())
        } else {
            Err(self.m_errors.clone())
        }
    }
}
