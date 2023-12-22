use crate::ast::*;
use crate::token::*;

use anyhow::Result;
use std::iter::Peekable;

pub struct Parser {
    m_token_iter: Peekable<std::vec::IntoIter<Token>>,
    m_current: Option<Token>,
    m_previous: Option<Token>,
    m_errors: Vec<anyhow::Error>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            m_token_iter: tokens.into_iter().peekable(),
            m_current: None,
            m_previous: None,
            m_errors: Vec::new(),
        }
    }

    fn peek_next(&mut self) -> Option<&Token> {
        self.m_token_iter.peek()
    }

    fn take_next(&mut self) -> Option<Token> {
        self.m_previous = self.m_current.clone();
        self.m_current = self.m_token_iter.next();
        self.m_current.clone()
    }

    fn matches(&mut self, types: &[TokenType]) -> bool {
        self.peek_next().is_some_and(|token| {
            types
                .iter()
                .map(|token_type| match token {
                    Token {
                        m_token: TokenType::Number(_),
                        ..
                    } => token_type == &TokenType::Number(0.0),
                    Token {
                        m_token: TokenType::String(_),
                        ..
                    } => token_type == &TokenType::String("".into()),
                    _ => token_type == token.get_token_type(),
                })
                .any(|result| result)
        })
    }

    fn sync(&mut self) {
        while let Some(token) = self.take_next() {
            if token.get_token_type() == &TokenType::Semicolon {
                return;
            }

            match self.peek_next() {
                Some(token) => match token.get_token_type() {
                    TokenType::Return => return,
                    _ => {}
                },
                None => {}
            }
        }
    }

    fn primary(&mut self) -> Result<Expr> {
        if self.matches(&[
            TokenType::False,
            TokenType::True,
            TokenType::Nil,
            TokenType::Number(0.0),
            TokenType::String("".into()),
        ]) {
            return Ok(Expr::new_literal(self.take_next().unwrap()));
        }

        if self.matches(&[TokenType::LeftParen]) {
            self.take_next();
            let expr = self.expression()?;

            while !self.matches(&[TokenType::RightParen]) {
                if let None = self.take_next() {
                    self.m_errors.push(anyhow::anyhow!(
                        "Unterminated grouping, expected ')'\n    => line {} | column {}",
                        self.m_previous.as_ref().unwrap().get_line_number(),
                        self.m_previous.as_ref().unwrap().get_token_range().start + 1
                    ));
                    return Err(anyhow::anyhow!(""));
                }
            }

            self.take_next();

            return Ok(Expr::new_grouping(Box::new(expr)));
        }

        if let Some(token) = self.m_previous.as_ref() {
            self.m_errors.push(anyhow::anyhow!(
                "Invalid operands, expected expression\n    => line {} | column {}",
                token.get_line_number(),
                token.get_token_range().start + 1
            ));
        } else {
            if let Some(token) = self.m_current.as_ref() {
                self.m_errors.push(anyhow::anyhow!(
                    "Invalid operands, expected expression\n    => line {} | column {}",
                    token.get_line_number(),
                    token.get_token_range().start + 1
                ));
            } else {
                self.m_errors
                    .push(anyhow::anyhow!("Invalid operands, expected expression"));
            }
        }

        Err(anyhow::anyhow!(""))
    }

    fn unary(&mut self) -> Result<Expr> {
        if self.matches(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.take_next().unwrap();
            let right = self.unary()?;
            return Ok(Expr::new_unary(operator, Box::new(right)));
        }

        self.primary()
    }

    fn factor(&mut self) -> Result<Expr> {
        let mut expr = self.unary()?;

        while self.matches(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.take_next().unwrap();
            let right = self.unary()?;
            expr = Expr::new_binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr> {
        let mut expr = self.factor()?;

        while self.matches(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.take_next().unwrap();
            let right = self.factor()?;
            expr = Expr::new_binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr> {
        let mut expr = self.term()?;

        while self.matches(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.take_next().unwrap();
            let right = self.term()?;
            expr = Expr::new_binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr> {
        let mut expr = self.comparison()?;

        while self.matches(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.take_next().unwrap();
            let right = self.comparison()?;
            expr = Expr::new_binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn expression(&mut self) -> Result<Expr> {
        self.equality()
    }

    pub fn parse(mut self) -> Result<Expr, Vec<anyhow::Error>> {
        let expr = self.expression();

        if self.m_errors.is_empty() {
            Ok(expr.unwrap())
        } else {
            Err(self.m_errors)
        }
    }
}
