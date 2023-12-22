use itertools::Itertools;

use crate::ast::Expr;
use crate::lox_value::LoxValue;
use crate::token::*;

pub trait Visitor {
    fn visit_binary(&mut self, left: &Expr, token: &Token, right: &Expr);
    fn visit_grouping(&mut self, expression: &Expr);
    fn visit_literal(&mut self, token: &Token);
    fn visit_unary(&mut self, token: &Token, expression: &Expr);
}

pub struct Printer {
    m_content: Vec<String>,
}

impl Printer {
    pub fn new() -> Self {
        Self {
            m_content: Vec::new(),
        }
    }
}

impl Visitor for Printer {
    fn visit_binary(&mut self, left: &Expr, token: &Token, right: &Expr) {
        self.m_content.push("(".into());
        self.m_content
            .push(format!("{:?}", token.get_token_type(),));
        left.accept(self);
        right.accept(self);
        self.m_content.push(")".into());
    }

    fn visit_grouping(&mut self, expression: &Expr) {
        self.m_content.push("(".into());
        self.m_content.push("Grouping".into());
        expression.accept(self);
        self.m_content.push(")".into());
    }

    fn visit_literal(&mut self, token: &Token) {
        self.m_content.push("(".into());
        match token.get_token_type() {
            crate::token::TokenType::Number(number) => {
                self.m_content.push(format!("{:?}", number));
            }
            token => {
                self.m_content.push(format!("{:?}", token));
            }
        }
        self.m_content.push(")".into());
    }

    fn visit_unary(&mut self, token: &Token, expression: &Expr) {
        self.m_content.push("(".into());
        self.m_content.push(format!("{:?}", token.get_token_type()));
        expression.accept(self);
        self.m_content.push(")".into());
    }
}

pub struct Evaluator {
    m_result: Vec<LoxValue>,
}

impl Evaluator {
    pub fn new() -> Self {
        Self {
            m_result: Vec::new(),
        }
    }

    pub fn get_result(&self) -> LoxValue {
        self.m_result.last().unwrap().clone()
    }
}

impl Visitor for Evaluator {
    fn visit_binary(&mut self, left: &Expr, token: &Token, right: &Expr) {
        left.accept(self);
        right.accept(self);

        let right = self.m_result.pop().unwrap();
        let left = self.m_result.pop().unwrap();

        match token.get_token_type() {
            crate::token::TokenType::Plus => match left.as_number() {
                Some(left_number) => match right.as_number() {
                    Some(right_number) => {
                        self.m_result
                            .push(LoxValue::Number(left_number + right_number));
                    }
                    None => {
                        self.m_result.push(LoxValue::String(format!(
                            "{}{}",
                            left_number,
                            right.as_string().unwrap()
                        )));
                    }
                },
                None => {
                    self.m_result.push(LoxValue::String(format!(
                        "{}{}",
                        left.as_string().unwrap(),
                        right.as_string().unwrap()
                    )));
                }
            },
            crate::token::TokenType::Minus => {
                self.m_result.push(LoxValue::Number(
                    left.as_number().unwrap() - right.as_number().unwrap(),
                ));
            }
            crate::token::TokenType::Star => {
                self.m_result.push(LoxValue::Number(
                    left.as_number().unwrap() * right.as_number().unwrap(),
                ));
            }
            crate::token::TokenType::Slash => {
                self.m_result.push(LoxValue::Number(
                    left.as_number().unwrap() / right.as_number().unwrap(),
                ));
            }
            crate::token::TokenType::Greater => {
                self.m_result.push(LoxValue::Boolean(
                    left.as_number().unwrap() > right.as_number().unwrap(),
                ));
            }
            crate::token::TokenType::GreaterEqual => {
                self.m_result.push(LoxValue::Boolean(
                    left.as_number().unwrap() >= right.as_number().unwrap(),
                ));
            }
            crate::token::TokenType::Less => {
                self.m_result.push(LoxValue::Boolean(
                    left.as_number().unwrap() < right.as_number().unwrap(),
                ));
            }
            crate::token::TokenType::LessEqual => {
                self.m_result.push(LoxValue::Boolean(
                    left.as_number().unwrap() <= right.as_number().unwrap(),
                ));
            }
            crate::token::TokenType::EqualEqual => {
                self.m_result.push(LoxValue::Boolean(left == right));
            }
            crate::token::TokenType::BangEqual => {
                self.m_result.push(LoxValue::Boolean(left != right));
            }
            _ => {}
        }
    }

    fn visit_grouping(&mut self, expression: &Expr) {
        expression.accept(self);
    }

    fn visit_unary(&mut self, token: &Token, expression: &Expr) {
        expression.accept(self);

        let value = self.m_result.pop().unwrap();

        match token.get_token_type() {
            crate::token::TokenType::Minus => {
                self.m_result
                    .push(LoxValue::Number(-value.as_number().unwrap()));
            }
            crate::token::TokenType::Bang => {
                self.m_result.push(LoxValue::Boolean(!value.is_truthy()));
            }
            _ => {}
        }
    }

    fn visit_literal(&mut self, token: &Token) {
        match token.get_token_type() {
            crate::token::TokenType::Number(number) => {
                self.m_result.push(LoxValue::Number(*number));
            }
            crate::token::TokenType::True => {
                self.m_result.push(LoxValue::Boolean(true));
            }
            crate::token::TokenType::False => {
                self.m_result.push(LoxValue::Boolean(false));
            }
            crate::token::TokenType::Nil => {
                self.m_result.push(LoxValue::Nil);
            }
            crate::token::TokenType::String(string) => {
                self.m_result.push(LoxValue::String(string.clone()));
            }
            _ => {}
        }
    }
}
