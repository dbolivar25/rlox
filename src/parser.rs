use crate::ast::*;
use crate::token::*;

use anyhow::Result;
use itertools::Itertools;
use std::iter::Peekable;

macro_rules! match_token {
    ($self:ident, [$($token_type:ident $(($($inner:tt)*))? ),*]) => {
        $self.peek_next().is_some_and(|token| match token.get_token_type() {
            $(
                TokenType::$token_type $(($($inner)*))? => true,
            )*
            _ => false,
        })
    };
}

macro_rules! multi_match_token {
    ($self:ident, [$([$($token_type:ident $(($($inner:tt)*))? ),*]),*]) => {{
        let mut iter = $self.m_token_iter.clone().multipeek();
        let mut result = true;

        $(
            result &= iter.peek().map_or(false, |token| {
                match token.get_token_type() {
                    $(
                        // Match for variants with and without data
                        TokenType::$token_type $(($($inner)*))? => true,
                        _ => false,
                    )*
                }
            });
        )*

        result
    }};
}

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
        if multi_match_token!(self, [[Fun], [LeftParen]]) {
            self.take_next();
            self.take_next();

            let mut parameters = Vec::new();
            if !match_token!(self, [RightParen]) {
                loop {
                    if parameters.len() >= 255 {
                        self.m_errors.push(format!(
                            "Cannot have more than 255 parameters\n    => line {} | column {}",
                            self.m_current.as_ref().unwrap().get_line_number(),
                            self.m_current.as_ref().unwrap().get_col_range().start + 1
                        ));
                        return Err(anyhow::anyhow!(""));
                    }

                    parameters.push(self.take_next().unwrap());

                    if !match_token!(self, [Comma]) {
                        break;
                    }

                    self.take_next();
                }
            }

            if match_token!(self, [RightParen]) {
                self.take_next();
            } else {
                self.m_errors.push(format!(
                    "Expected ')' after function parameters\n    => line {} | column {}",
                    self.m_previous.as_ref().unwrap().get_line_number(),
                    self.m_previous.as_ref().unwrap().get_col_range().start + 1
                ));
                return Err(anyhow::anyhow!(""));
            }

            if match_token!(self, [LeftBrace]) {
                let body = self.statement()?;
                let body = match body {
                    Stmt::Block { m_statements } => m_statements,
                    _ => {
                        self.m_errors.push(format!(
                            "Expected block after function declaration\n    => line {} | column {}",
                            self.m_previous.as_ref().unwrap().get_line_number(),
                            self.m_previous.as_ref().unwrap().get_col_range().start + 1,
                        ));
                        return Err(anyhow::anyhow!(""));
                    }
                };
                return Ok(Expr::new_function(parameters, body));
            } else {
                self.m_errors.push(format!(
                    "Expected '{{' after function declaration\n    => line {} | column {}",
                    self.m_previous.as_ref().unwrap().get_line_number(),
                    self.m_previous.as_ref().unwrap().get_col_range().start + 1
                ));
                return Err(anyhow::anyhow!(""));
            }
        }

        if match_token!(self, [False, True, String(_), Number(_), Nil]) {
            return Ok(Expr::new_literal(self.take_next().unwrap()));
        }

        if match_token!(self, [Identifier(_)]) {
            return Ok(Expr::new_variable(self.take_next().unwrap()));
        }

        if match_token!(self, [LeftParen]) {
            self.take_next();
            let expr = self.expression()?;

            while !match_token!(self, [RightParen]) {
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

        if !match_token!(self, [RightParen]) {
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

                if !match_token!(self, [Comma]) {
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
            if match_token!(self, [LeftParen]) {
                self.take_next();
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr> {
        if match_token!(self, [Bang, Minus]) {
            let operator = self.take_next().unwrap();
            let right = self.unary()?;
            return Ok(Expr::new_unary(operator, Box::new(right)));
        }

        self.call()
    }

    fn factor(&mut self) -> Result<Expr> {
        let mut expr = self.unary()?;

        while match_token!(self, [Slash, Star]) {
            let operator = self.take_next().unwrap();
            let right = self.unary()?;
            expr = Expr::new_binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr> {
        let mut expr = self.factor()?;

        while match_token!(self, [Minus, Plus]) {
            let operator = self.take_next().unwrap();
            let right = self.factor()?;
            expr = Expr::new_binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr> {
        let mut expr = self.term()?;

        while match_token!(self, [Greater, GreaterEqual, Less, LessEqual]) {
            let operator = self.take_next().unwrap();
            let right = self.term()?;
            expr = Expr::new_binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr> {
        let mut expr = self.comparison()?;

        while match_token!(self, [BangEqual, EqualEqual]) {
            let operator = self.take_next().unwrap();
            let right = self.comparison()?;
            expr = Expr::new_binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr> {
        let mut expr = self.equality()?;

        while match_token!(self, [And]) {
            let operator = self.take_next().unwrap();
            let right = self.equality()?;
            expr = Expr::new_logical(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr> {
        let mut expr = self.and()?;

        while match_token!(self, [Or]) {
            let operator = self.take_next().unwrap();
            let right = self.and()?;
            expr = Expr::new_logical(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn assignment(&mut self) -> Result<Expr> {
        let expr = self.or()?;

        if match_token!(self, [Equal]) {
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

        if match_token!(self, [LeftBrace]) {
            self.take_next();
            let mut statements = Vec::new();
            while !match_token!(self, [RightBrace]) {
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

        if match_token!(self, [If]) {
            self.take_next();
            if match_token!(self, [LeftParen]) {
                self.take_next();
                let condition = self.expression()?;
                if match_token!(self, [RightParen]) {
                    self.take_next();
                    let then_branch = Box::new(self.statement()?);
                    let else_branch = if match_token!(self, [Else]) {
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

        if match_token!(self, [While]) {
            self.take_next();
            if match_token!(self, [LeftParen]) {
                self.take_next();
                let condition = self.expression()?;
                if match_token!(self, [RightParen]) {
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

        if match_token!(self, [For]) {
            self.take_next();
            if match_token!(self, [LeftParen]) {
                self.take_next();
                let initializer = if match_token!(self, [Semicolon]) {
                    None
                } else if multi_match_token!(self, [[Var], [Identifier(_)], [Equal]]) {
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

                    let initializer = if match_token!(self, [Equal]) {
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

                    Some(Stmt::new_expression(Expr::new_assign(
                        name,
                        Box::new(initializer.unwrap()),
                    )))
                } else {
                    let expr = self.expression()?;
                    if match_token!(self, [Semicolon]) {
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

                let condition = if match_token!(self, [Semicolon]) {
                    None
                } else {
                    Some(self.expression()?)
                };

                if match_token!(self, [Semicolon]) {
                    self.take_next();
                } else {
                    self.m_errors.push(format!(
                        "Expected ';' after for loop condition\n    => line {} | column {}",
                        self.m_previous.as_ref().unwrap().get_line_number(),
                        self.m_previous.as_ref().unwrap().get_col_range().start + 1
                    ));
                    return Err(anyhow::anyhow!(""));
                }

                let increment = if match_token!(self, [RightParen]) {
                    None
                } else {
                    Some(self.expression()?)
                };

                if match_token!(self, [RightParen]) {
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
                    let initializer_definition = match initializer {
                        Stmt::Expression { m_expression } => match m_expression {
                            Expr::Assign {
                                m_token: m_name,
                                m_value,
                            } => Stmt::new_var(m_name, Some(*m_value), vec![*body]),
                            _ => {
                                self.m_errors.push(format!(
                                    "Expected expression after 'var'\n    => line {} | column {}",
                                    self.m_previous.as_ref().unwrap().get_line_number(),
                                    self.m_previous.as_ref().unwrap().get_col_range().start + 1
                                ));
                                return Err(anyhow::anyhow!(""));
                            }
                        },
                        _ => {
                            self.m_errors.push(format!(
                                "Expected expression after 'var'\n    => line {} | column {}",
                                self.m_previous.as_ref().unwrap().get_line_number(),
                                self.m_previous.as_ref().unwrap().get_col_range().start + 1
                            ));
                            return Err(anyhow::anyhow!(""));
                        }
                    };
                    body = Box::new(initializer_definition);
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
        if match_token!(self, [Var]) {
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

            let initializer = if match_token!(self, [Equal]) {
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

            let mut statements = Vec::new();
            while self.peek_next().is_some() && !match_token!(self, [RightBrace, Eof]) {
                statements.push(self.declaration()?);
            }
            // self.take_next();

            return Ok(Stmt::new_var(name, initializer, statements));
        }

        if multi_match_token!(self, [[Fun], [Identifier(_)]]) {
            self.take_next();

            let name = if let TokenType::Identifier(_) = self.peek_next().unwrap().get_token_type()
            {
                self.take_next().unwrap()
            } else {
                self.m_errors.push(format!(
                    "Expected identifier after 'fun'\n    => line {} | column {}",
                    self.m_previous.as_ref().unwrap().get_line_number(),
                    self.m_previous.as_ref().unwrap().get_col_range().start + 1
                ));
                return Err(anyhow::anyhow!(""));
            };

            // if !matches!(name.get_token_type(), TokenType::Identifier(_)) {
            //     self.m_errors.push(format!(
            //         "Expected identifier after 'fun'\n    => line {} | column {}",
            //         name.get_line_number().saturating_sub(1),
            //         name.get_col_range().start + 1
            //     ));
            //     return Err(anyhow::anyhow!(""));
            // }

            if match_token!(self, [LeftParen]) {
                self.take_next();
                let mut parameters = Vec::new();
                if !match_token!(self, [RightParen]) {
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

                        if !match_token!(self, [Comma]) {
                            break;
                        }

                        self.take_next();
                    }
                }

                if match_token!(self, [RightParen]) {
                    self.take_next();
                } else {
                    self.m_errors.push(format!(
                        "Expected ')' after function parameters\n    => line {} | column {}",
                        self.m_previous.as_ref().unwrap().get_line_number(),
                        self.m_previous.as_ref().unwrap().get_col_range().start + 1
                    ));
                    return Err(anyhow::anyhow!(""));
                }

                if match_token!(self, [LeftBrace]) {
                    let body = self.statement()?;
                    let body = match body {
                        Stmt::Block { m_statements } => m_statements,
                        _ => {
                            self.m_errors.push(format!(
                                "Expected block after function declaration\n    => line {} | column {}",
                                self.m_previous.as_ref().unwrap().get_line_number(),
                                self.m_previous.as_ref().unwrap().get_col_range().start + 1,
                            ));
                            return Err(anyhow::anyhow!(""));
                        }
                    };
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

        if match_token!(self, [Return]) {
            self.take_next();
            let keyword = self.m_previous.clone().unwrap();
            let value = if !match_token!(self, [Semicolon]) {
                Some(self.expression()?)
            } else {
                None
            };

            match self.take_next() {
                Some(token) => {
                    if token.get_token_type() == &TokenType::Semicolon {
                    } else {
                        self.m_errors.push(format!(
                            "Expected ';' after return value\n    => line {} | column {}",
                            token.get_line_number().saturating_sub(1),
                            token.get_col_range().start + 1
                        ));
                        return Err(anyhow::anyhow!(""));
                    }
                }
                None => {
                    self.m_errors.push(format!(
                        "Expected ';' after return value\n    => line {} | column {}",
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

            return Ok(Stmt::new_return(keyword, value));
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
