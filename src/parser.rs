use crate::ast::*;
use crate::token::*;

use anyhow::Result;
use std::iter::Peekable;

#[derive(Debug)]
pub struct Parser {
    m_token_iter: Peekable<std::vec::IntoIter<Token>>,
    m_current: Option<Token>,
    m_previous: Option<Token>,
    m_errors: Vec<String>,
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
        self.m_previous = self.m_current.take();
        self.m_current = self.m_token_iter.next();
        self.m_current.clone()
    }

    fn matches(&mut self, types: &[TokenType]) -> bool {
        self.peek_next().is_some_and(|token| {
            types
                .iter()
                .any(|token_type| match (token.get_token_type(), token_type) {
                    (TokenType::Number(_), TokenType::Number(_)) => true,
                    (TokenType::String(_), TokenType::String(_)) => true,
                    (TokenType::Identifier(_), TokenType::Identifier(_)) => true,
                    (lhs, rhs) => lhs == rhs,
                })
        })
    }

    fn sync(&mut self) {
        while let Some(token) = self.take_next() {
            if token.get_token_type() == &TokenType::Semicolon {
                return;
            }

            if self
                .peek_next()
                .is_some_and(|token| &TokenType::Return == token.get_token_type())
            {
                return;
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

        if self.matches(&[TokenType::Identifier("".into())]) {
            return Ok(Expr::new_variable(self.take_next().unwrap()));
        }

        if self.matches(&[TokenType::LeftParen]) {
            self.take_next();
            let expr = self.expression()?;

            while !self.matches(&[TokenType::RightParen]) {
                if self.take_next().is_none() {
                    self.m_errors.push(format!(
                        "Unterminated grouping, expected ')'\n    => line {} | column {}",
                        self.m_previous.as_ref().unwrap().get_line_number(),
                        self.m_previous.as_ref().unwrap().get_col_range().start + 1
                    ));
                    return Err(anyhow::anyhow!(""));
                }
            }

            self.take_next();

            return Ok(Expr::new_grouping(Box::new(expr)));
        }

        if let Some(token) = self.m_previous.as_ref() {
            self.m_errors.push(format!(
                "Invalid operands, expected expression\n    => line {} | column {}",
                token.get_line_number(),
                token.get_col_range().start + 1
            ));
        } else if let Some(token) = self.m_current.as_ref() {
            self.m_errors.push(format!(
                "Invalid operands, expected expression\n    => line {} | column {}",
                token.get_line_number(),
                token.get_col_range().start + 1
            ));
        } else {
            self.m_errors
                .push("Invalid operands, expected expression".to_string());
        }

        Err(anyhow::anyhow!(""))
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr> {
        let mut arguments = Vec::new();

        if !self.matches(&[TokenType::RightParen]) {
            loop {
                if arguments.len() >= 255 {
                    self.m_errors.push(format!(
                        "Cannot have more than 255 arguments\n    => line {} | column {}",
                        self.m_previous.as_ref().unwrap().get_line_number(),
                        self.m_previous.as_ref().unwrap().get_col_range().start + 1
                    ));
                    return Err(anyhow::anyhow!(""));
                }

                arguments.push(self.expression()?);

                if !self.matches(&[TokenType::Comma]) {
                    break;
                }

                self.take_next();
            }
        }

        match self.take_next() {
            Some(token) => {
                if token.get_token_type() == &TokenType::RightParen {
                    Ok(Expr::new_call(
                        Box::new(callee),
                        token,
                        arguments.into_iter().collect(),
                    ))
                } else {
                    self.m_errors.push(format!(
                        "Expected ')' after arguments\n    => line {} | column {}",
                        token.get_line_number().saturating_sub(1),
                        token.get_col_range().start + 1
                    ));
                    Err(anyhow::anyhow!(""))
                }
            }
            None => {
                self.m_errors.push(format!(
                    "Expected ')' after arguments\n    => line {} | column {}",
                    self.m_previous
                        .as_ref()
                        .unwrap()
                        .get_line_number()
                        .saturating_sub(1),
                    self.m_previous.as_ref().unwrap().get_col_range().start + 1
                ));
                Err(anyhow::anyhow!(""))
            }
        }
    }

    fn call(&mut self) -> Result<Expr> {
        let mut expr = self.primary()?;

        loop {
            if self.matches(&[TokenType::LeftParen]) {
                self.take_next();
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr> {
        if self.matches(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.take_next().unwrap();
            let right = self.unary()?;
            return Ok(Expr::new_unary(operator, Box::new(right)));
        }

        self.call()
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

    fn and(&mut self) -> Result<Expr> {
        let mut expr = self.equality()?;

        while self.matches(&[TokenType::And]) {
            let operator = self.take_next().unwrap();
            let right = self.equality()?;
            expr = Expr::new_logical(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr> {
        let mut expr = self.and()?;

        while self.matches(&[TokenType::Or]) {
            let operator = self.take_next().unwrap();
            let right = self.and()?;
            expr = Expr::new_logical(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn assignment(&mut self) -> Result<Expr> {
        let expr = self.or()?;

        if self.matches(&[TokenType::Equal]) {
            let equals = self.take_next().unwrap();
            let value = self.assignment()?;

            if let Expr::Variable { m_token } = expr {
                return Ok(Expr::new_assign(m_token, Box::new(value)));
            }

            self.m_errors.push(format!(
                "Invalid assignment target\n    => line {} | column {}",
                equals.get_line_number(),
                equals.get_col_range().start + 1
            ));
            return Err(anyhow::anyhow!(""));
        }

        Ok(expr)
    }

    fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn statement(&mut self) -> Result<Stmt> {
        // if self.matches(&[TokenType::Print]) {
        //     self.take_next();
        //     let expr = self.expression()?;
        //     match self.take_next() {
        //         Some(token) => {
        //             if token.get_token_type() == &TokenType::Semicolon {
        //             } else {
        //                 self.m_errors.push(format!(
        //                     "Expected ';' after expression\n    => line {} | column {}",
        //                     token.get_line_number().saturating_sub(1),
        //                     token.get_col_range().start + 1
        //                 ));
        //                 self.sync();
        //             }
        //         }
        //         None => {
        //             self.m_errors.push(format!(
        //                 "Expected ';' after expression\n    => line {} | column {}",
        //                 self.m_previous
        //                     .as_ref()
        //                     .unwrap()
        //                     .get_line_number()
        //                     .saturating_sub(1),
        //                 self.m_previous.as_ref().unwrap().get_col_range().start + 1
        //             ));
        //             self.sync();
        //         }
        //     }
        //     return Ok(Stmt::new_print(expr));
        // }

        if self.matches(&[TokenType::LeftBrace]) {
            self.take_next();
            let mut statements = Vec::new();
            while !self.matches(&[TokenType::RightBrace]) {
                if self.peek_next().is_none() {
                    self.m_errors.push(format!(
                        "Unterminated block, expected '}}'\n    => line {} | column {}",
                        self.m_previous.as_ref().unwrap().get_line_number(),
                        self.m_previous.as_ref().unwrap().get_col_range().start + 1
                    ));
                    return Err(anyhow::anyhow!(""));
                }
                statements.push(self.declaration()?);
            }
            self.take_next();
            return Ok(Stmt::new_block(statements));
        }

        if self.matches(&[TokenType::If]) {
            self.take_next();
            if self.matches(&[TokenType::LeftParen]) {
                self.take_next();
                let condition = self.expression()?;
                if self.matches(&[TokenType::RightParen]) {
                    self.take_next();
                    let then_branch = Box::new(self.statement()?);
                    let else_branch = if self.matches(&[TokenType::Else]) {
                        self.take_next();
                        Some(Box::new(self.statement()?))
                    } else {
                        None
                    };
                    return Ok(Stmt::new_if(condition, then_branch, else_branch));
                } else {
                    self.m_errors.push(format!(
                        "Expected ')' after if condition\n    => line {} | column {}",
                        self.m_previous.as_ref().unwrap().get_line_number(),
                        self.m_previous.as_ref().unwrap().get_col_range().start + 1
                    ));
                    return Err(anyhow::anyhow!(""));
                }
            } else {
                self.m_errors.push(format!(
                    "Expected '(' after 'if'\n    => line {} | column {}",
                    self.m_previous.as_ref().unwrap().get_line_number(),
                    self.m_previous.as_ref().unwrap().get_col_range().start + 1
                ));
                return Err(anyhow::anyhow!(""));
            }
        }

        if self.matches(&[TokenType::While]) {
            self.take_next();
            if self.matches(&[TokenType::LeftParen]) {
                self.take_next();
                let condition = self.expression()?;
                if self.matches(&[TokenType::RightParen]) {
                    self.take_next();
                    let body = Box::new(self.statement()?);
                    return Ok(Stmt::new_while(condition, body));
                } else {
                    self.m_errors.push(format!(
                        "Expected ')' after while condition\n    => line {} | column {}",
                        self.m_previous.as_ref().unwrap().get_line_number(),
                        self.m_previous.as_ref().unwrap().get_col_range().start + 1
                    ));
                    return Err(anyhow::anyhow!(""));
                }
            } else {
                self.m_errors.push(format!(
                    "Expected '(' after 'while'\n    => line {} | column {}",
                    self.m_previous.as_ref().unwrap().get_line_number(),
                    self.m_previous.as_ref().unwrap().get_col_range().start + 1
                ));
                return Err(anyhow::anyhow!(""));
            }
        }

        if self.matches(&[TokenType::For]) {
            self.take_next();
            if self.matches(&[TokenType::LeftParen]) {
                self.take_next();
                let initializer = if self.matches(&[TokenType::Semicolon]) {
                    None
                } else if self.matches(&[TokenType::Var]) {
                    Some(self.declaration()?)
                } else {
                    let expr = self.expression()?;
                    if self.matches(&[TokenType::Semicolon]) {
                        self.take_next();
                    } else {
                        self.m_errors.push(format!(
                            "Expected ';' after for loop initializer\n    => line {} | column {}",
                            self.m_previous.as_ref().unwrap().get_line_number(),
                            self.m_previous.as_ref().unwrap().get_col_range().start + 1
                        ));
                        return Err(anyhow::anyhow!(""));
                    }

                    Some(Stmt::new_expression(expr))
                };

                let condition = if self.matches(&[TokenType::Semicolon]) {
                    None
                } else {
                    Some(self.expression()?)
                };

                if self.matches(&[TokenType::Semicolon]) {
                    self.take_next();
                } else {
                    self.m_errors.push(format!(
                        "Expected ';' after for loop condition\n    => line {} | column {}",
                        self.m_previous.as_ref().unwrap().get_line_number(),
                        self.m_previous.as_ref().unwrap().get_col_range().start + 1
                    ));
                    return Err(anyhow::anyhow!(""));
                }

                let increment = if self.matches(&[TokenType::RightParen]) {
                    None
                } else {
                    Some(self.expression()?)
                };

                if self.matches(&[TokenType::RightParen]) {
                    self.take_next();
                } else {
                    self.m_errors.push(format!(
                        "Expected ')' after for loop increment\n    => line {} | column {}",
                        self.m_previous.as_ref().unwrap().get_line_number(),
                        self.m_previous.as_ref().unwrap().get_col_range().start + 1
                    ));
                    return Err(anyhow::anyhow!(""));
                }

                let mut body = Box::new(self.statement()?);

                if let Some(increment) = increment {
                    body = Box::new(Stmt::new_block(vec![
                        *body,
                        Stmt::new_expression(increment),
                    ]));
                }

                if let Some(condition) = condition {
                    body = Box::new(Stmt::new_while(condition, body));
                }

                if let Some(initializer) = initializer {
                    body = Box::new(Stmt::new_block(vec![initializer, *body]));
                }

                return Ok(*body);
            } else {
                self.m_errors.push(format!(
                    "Expected '(' after 'for'\n    => line {} | column {}",
                    self.m_previous.as_ref().unwrap().get_line_number(),
                    self.m_previous.as_ref().unwrap().get_col_range().start + 1
                ));
                return Err(anyhow::anyhow!(""));
            }
        }

        let expr = self.expression()?;
        match self.take_next() {
            Some(token) => {
                if token.get_token_type() == &TokenType::Semicolon {
                } else {
                    self.m_errors.push(format!(
                        "Expected ';' after expression\n    => line {} | column {}",
                        token.get_line_number().saturating_sub(1),
                        token.get_col_range().start + 1
                    ));
                    return Err(anyhow::anyhow!(""));
                }
            }
            None => {
                self.m_errors.push(format!(
                    "Expected ';' after expression\n    => line {} | column {}",
                    self.m_previous
                        .as_ref()
                        .unwrap()
                        .get_line_number()
                        .saturating_sub(1),
                    self.m_previous.as_ref().unwrap().get_col_range().start + 1
                ));
                return Err(anyhow::anyhow!(""));
            }
        }

        Ok(Stmt::new_expression(expr))
    }

    fn declaration(&mut self) -> Result<Stmt> {
        if self.matches(&[TokenType::Var]) {
            self.take_next();
            let name = self.take_next().unwrap();

            if !matches!(name.get_token_type(), TokenType::Identifier(_)) {
                self.m_errors.push(format!(
                    "Expected identifier after 'var'\n    => line {} | column {}",
                    name.get_line_number().saturating_sub(1),
                    name.get_col_range().start + 1
                ));
                return Err(anyhow::anyhow!(""));
            }

            let initializer = if self.matches(&[TokenType::Equal]) {
                self.take_next();
                Some(self.expression()?)
            } else {
                None
            };

            match self.take_next() {
                Some(token) => {
                    if token.get_token_type() == &TokenType::Semicolon {
                    } else {
                        self.m_errors.push(format!(
                            "Expected ';' after variable declaration\n    => line {} | column {}",
                            token.get_line_number().saturating_sub(1),
                            token.get_col_range().start + 1
                        ));
                        return Err(anyhow::anyhow!(""));
                    }
                }
                None => {
                    self.m_errors.push(format!(
                        "Expected ';' after variable declaration\n    => line {} | column {}",
                        self.m_previous
                            .as_ref()
                            .unwrap()
                            .get_line_number()
                            .saturating_sub(1),
                        self.m_previous.as_ref().unwrap().get_col_range().start + 1
                    ));
                    return Err(anyhow::anyhow!(""));
                }
            }

            return Ok(Stmt::new_var(name, initializer));
        }

        if self.matches(&[TokenType::Fun]) {
            self.take_next();
            let name = self.take_next().unwrap();

            if !matches!(name.get_token_type(), TokenType::Identifier(_)) {
                self.m_errors.push(format!(
                    "Expected identifier after 'fun'\n    => line {} | column {}",
                    name.get_line_number().saturating_sub(1),
                    name.get_col_range().start + 1
                ));
                return Err(anyhow::anyhow!(""));
            }

            if self.matches(&[TokenType::LeftParen]) {
                self.take_next();
                let mut parameters = Vec::new();
                if !self.matches(&[TokenType::RightParen]) {
                    loop {
                        if parameters.len() >= 255 {
                            self.m_errors.push(format!(
                                "Cannot have more than 255 parameters\n    => line {} | column {}",
                                self.m_previous.as_ref().unwrap().get_line_number(),
                                self.m_previous.as_ref().unwrap().get_col_range().start + 1
                            ));
                            return Err(anyhow::anyhow!(""));
                        }

                        parameters.push(self.take_next().unwrap());

                        if !self.matches(&[TokenType::Comma]) {
                            break;
                        }

                        self.take_next();
                    }
                }

                if self.matches(&[TokenType::RightParen]) {
                    self.take_next();
                } else {
                    self.m_errors.push(format!(
                        "Expected ')' after function parameters\n    => line {} | column {}",
                        self.m_previous.as_ref().unwrap().get_line_number(),
                        self.m_previous.as_ref().unwrap().get_col_range().start + 1
                    ));
                    return Err(anyhow::anyhow!(""));
                }

                if self.matches(&[TokenType::LeftBrace]) {
                    let body = self.statement()?;
                    let body = vec![body];
                    return Ok(Stmt::new_function(name, parameters, body));
                } else {
                    self.m_errors.push(format!(
                        "Expected '{{' after function declaration\n    => line {} | column {}",
                        self.m_previous.as_ref().unwrap().get_line_number(),
                        self.m_previous.as_ref().unwrap().get_col_range().start + 1
                    ));
                    return Err(anyhow::anyhow!(""));
                }
            } else {
                self.m_errors.push(format!(
                    "Expected '(' after function name\n    => line {} | column {}",
                    name.get_line_number().saturating_sub(1),
                    name.get_col_range().start
                ));
                return Err(anyhow::anyhow!(""));
            }
        }

        self.statement()
    }

    pub fn parse(mut self) -> Result<Vec<Stmt>, Vec<String>> {
        let mut statements = Vec::new();
        while self
            .peek_next()
            .is_some_and(|token| token.get_token_type() != &TokenType::Eof)
        {
            if let Ok(stmt) = self.declaration() {
                statements.push(stmt);
            } else {
                self.sync();
            }
        }

        // dbg!(&statements);

        if self.m_errors.is_empty() {
            Ok(statements)
        } else {
            Err(self.m_errors)
        }
    }
}
