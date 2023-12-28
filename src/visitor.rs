use crate::ast::*;
use crate::lox_value::Value;
use crate::token::*;

pub trait ExprVisitor {
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

    pub fn get_result(&self) -> String {
        self.m_content.join(" ")
    }
}

impl ExprVisitor for Printer {
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
    m_result: Vec<Value>,
    m_errors: Vec<String>,
}

impl Evaluator {
    pub fn new() -> Self {
        Self {
            m_result: Vec::new(),
            m_errors: Vec::new(),
        }
    }

    pub fn get_result(&self) -> Result<Value, Vec<String>> {
        if self.m_errors.is_empty() {
            match self.m_result.last() {
                Some(result) => Ok(result.clone()),
                None => Err(vec!["No result".into()]),
            }
        } else {
            Err(self.m_errors.clone())
        }
    }
}

impl ExprVisitor for Evaluator {
    fn visit_binary(&mut self, left: &Expr, token: &Token, right: &Expr) {
        left.accept(self);
        right.accept(self);

        if !self.m_errors.is_empty() {
            return;
        }

        match (self.m_result.pop(), self.m_result.pop()) {
            (Some(Value::Number(right)), Some(Value::Number(left))) => {
                self.m_result.push(match token.get_token_type() {
                    TokenType::Minus => Value::Number(left - right),
                    TokenType::Plus => Value::Number(left + right),
                    TokenType::Slash => Value::Number(left / right),
                    TokenType::Star => Value::Number(left * right),
                    TokenType::Greater => Value::Boolean(left > right),
                    TokenType::GreaterEqual => Value::Boolean(left >= right),
                    TokenType::Less => Value::Boolean(left < right),
                    TokenType::LessEqual => Value::Boolean(left <= right),
                    TokenType::BangEqual => Value::Boolean(left != right),
                    TokenType::EqualEqual => Value::Boolean(left == right),
                    token => {
                        self.m_errors
                            .push(format!("Invalid binary operator \"{:?}\"", token));
                        Value::Nil
                    }
                });
            }
            (Some(Value::String(right)), Some(Value::String(left))) => {
                self.m_result.push(match token.get_token_type() {
                    TokenType::Plus => Value::String(format!("{}{}", left, right)),
                    TokenType::Greater => Value::Boolean(left > right),
                    TokenType::GreaterEqual => Value::Boolean(left >= right),
                    TokenType::Less => Value::Boolean(left < right),
                    TokenType::LessEqual => Value::Boolean(left <= right),
                    TokenType::BangEqual => Value::Boolean(left != right),
                    TokenType::EqualEqual => Value::Boolean(left == right),
                    token => {
                        self.m_errors
                            .push(format!("Invalid binary operator \"{:?}\"", token));
                        Value::Nil
                    }
                });
            }
            (Some(right), Some(left)) => {
                self.m_errors.push(format!(
                    "Invalid binary expression \"{:?} {:?} {:?}\"",
                    left, token, right
                ));
            }
            (right, left) => self.m_errors.push(format!(
                "Invalid binary expression \"{:?} {:?} {:?}\"",
                left, token, right
            )),
        }
    }

    fn visit_grouping(&mut self, expression: &Expr) {
        expression.accept(self);
    }

    fn visit_literal(&mut self, token: &Token) {
        self.m_result.push(match token.get_token_type() {
            TokenType::Number(number) => Value::Number(*number),
            TokenType::String(string) => Value::String(string.clone()),
            TokenType::True => Value::Boolean(true),
            TokenType::False => Value::Boolean(false),
            TokenType::Nil => Value::Nil,
            token => {
                self.m_errors
                    .push(format!("Invalid literal expression \"{:?}\"", token));
                Value::Nil
            }
        });
    }

    fn visit_unary(&mut self, token: &Token, expression: &Expr) {
        expression.accept(self);

        if !self.m_errors.is_empty() {
            return;
        }

        match self.m_result.pop() {
            Some(Value::Number(number)) => {
                self.m_result.push(match token.get_token_type() {
                    TokenType::Minus => Value::Number(-number),
                    TokenType::Bang => {
                        Value::Boolean(!Value::Number(number).is_equal(&Value::Number(0.0)))
                    }
                    token => {
                        self.m_errors
                            .push(format!("Invalid unary operator \"{:?}\"", token));
                        Value::Nil
                    }
                });
            }
            Some(Value::Boolean(boolean)) => {
                self.m_result.push(match token.get_token_type() {
                    TokenType::Bang => Value::Boolean(!boolean),
                    token => {
                        self.m_errors
                            .push(format!("Invalid unary operator \"{:?}\"", token));
                        Value::Nil
                    }
                });
            }
            Some(value) => {
                self.m_errors.push(format!(
                    "Invalid unary expression \"{:?} {:?}\"",
                    token, value
                ));
            }
            None => {
                self.m_errors.push(format!(
                    "Invalid unary expression \"{:?} {:?}\"",
                    token, self.m_result
                ));
            }
        }
    }
}

pub trait StmtVisitor {
    fn visit_block(&mut self, statements: &[Stmt]);
    fn visit_expression(&mut self, expression: &Expr);
    fn visit_print(&mut self, expression: &Expr);
    fn visit_var(&mut self, name: &Token, initializer: &Expr);
    fn visit_while(&mut self, condition: &Expr, body: &Stmt);
    fn visit_if(&mut self, condition: &Expr, then_branch: &Stmt, else_branch: &Option<Box<Stmt>>);
    fn visit_function(&mut self, name: &Token, params: &[Token], body: &[Stmt]);
    fn visit_return(&mut self, keyword: &Token, value: &Option<Expr>);
    fn visit_class(&mut self, name: &Token, methods: &[Stmt]);
}
