use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

use crate::ast_v2::*;
use crate::environment::Environment;
use crate::token_v2::*;
use crate::value::*;

pub trait ExprVisitor {
    fn visit_binary(&mut self, left: &Expr, token: &TokenType, right: &Expr);
    fn visit_grouping(&mut self, expression: &Expr);
    fn visit_literal(&mut self, token: &TokenType);
    fn visit_unary(&mut self, token: &TokenType, expression: &Expr);
    fn visit_variable(&mut self, token: &TokenType);
    fn visit_assign(&mut self, token: &TokenType, expression: &Expr);
    fn visit_logical(&mut self, left: &Expr, token: &TokenType, right: &Expr);
    fn visit_call(&mut self, callee: &Expr, arguments: &[Expr]);
    fn visit_function(&mut self, params: &[TokenType], body: &Stmt);
}

pub struct ExprEvaluator {
    m_env: Rc<RefCell<Environment>>,
    m_result: Vec<Value>,
    m_errors: Vec<String>,
}

impl ExprEvaluator {
    pub fn new(env: &Rc<RefCell<Environment>>) -> Self {
        Self {
            m_env: env.clone(),
            m_result: Vec::new(),
            m_errors: Vec::new(),
        }
    }

    pub fn get_result(&self) -> Result<Value, Vec<String>> {
        if self.m_errors.is_empty() {
            match self.m_result.last() {
                Some(result) => Ok(result.clone()),
                None => Ok(Value::Nil),
            }
        } else {
            Err(self.m_errors.clone())
        }
    }
}

impl ExprVisitor for ExprEvaluator {
    fn visit_binary(&mut self, left: &Expr, token: &TokenType, right: &Expr) {
        left.accept(self);
        right.accept(self);

        if !self.m_errors.is_empty() {
            return;
        }

        match (self.m_result.pop(), self.m_result.pop()) {
            (Some(Value::Number(right)), Some(Value::Number(left))) => {
                self.m_result.push(match token {
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
                    token_type => {
                        self.m_errors
                            .push(format!("Invalid binary operator => {}", token_type));
                        Value::Nil
                    }
                });
            }
            (Some(Value::String(right)), Some(Value::String(left))) => {
                self.m_result.push(match token {
                    TokenType::Plus => Value::String(format!("{}{}", left, right)),
                    TokenType::Greater => Value::Boolean(left > right),
                    TokenType::GreaterEqual => Value::Boolean(left >= right),
                    TokenType::Less => Value::Boolean(left < right),
                    TokenType::LessEqual => Value::Boolean(left <= right),
                    TokenType::BangEqual => Value::Boolean(left != right),
                    TokenType::EqualEqual => Value::Boolean(left == right),
                    token_type => {
                        self.m_errors
                            .push(format!("Invalid binary operator => {}", token_type));
                        Value::Nil
                    }
                });
            }
            (Some(right), Some(left)) => {
                self.m_result.push(match token {
                    TokenType::BangEqual => Value::Boolean(!left.is_equal(&right)),
                    TokenType::EqualEqual => Value::Boolean(left.is_equal(&right)),
                    token_type => {
                        self.m_errors
                            .push(format!("Invalid binary operator => {}", token_type));
                        Value::Nil
                    }
                });
            }
            (right, left) => self.m_errors.push(format!(
                "Invalid binary expression => {:?} {:?} {:?}",
                left, token, right
            )),
        }
    }

    fn visit_grouping(&mut self, expression: &Expr) {
        expression.accept(self);
    }

    fn visit_literal(&mut self, token: &TokenType) {
        self.m_result.push(match token {
            TokenType::Number(number) => Value::Number(*number),
            TokenType::String(string) => Value::String(string.clone()),
            TokenType::True => Value::Boolean(true),
            TokenType::False => Value::Boolean(false),
            TokenType::Nil => Value::Nil,
            token => {
                self.m_errors
                    .push(format!("Invalid literal expression => {:?}", token));
                Value::Nil
            }
        });
    }

    fn visit_unary(&mut self, token: &TokenType, expression: &Expr) {
        expression.accept(self);

        if !self.m_errors.is_empty() {
            return;
        }

        match self.m_result.pop() {
            Some(Value::Number(number)) => {
                self.m_result.push(match token {
                    TokenType::Minus => Value::Number(-number),
                    TokenType::Bang => {
                        Value::Boolean(!Value::Number(number).is_equal(&Value::Number(0.0)))
                    }
                    token_type => {
                        self.m_errors
                            .push(format!("Invalid unary operator => {}", token_type));
                        Value::Nil
                    }
                });
            }
            Some(Value::Boolean(boolean)) => {
                self.m_result.push(match token {
                    TokenType::Bang => Value::Boolean(!boolean),
                    token_type => {
                        self.m_errors
                            .push(format!("Invalid unary operator => {}", token_type));
                        Value::Nil
                    }
                });
            }
            Some(value) => {
                self.m_errors.push(format!(
                    "Invalid unary expression => {:?} {:?}",
                    token, value
                ));
            }
            None => {
                self.m_errors.push(format!(
                    "Invalid unary expression => {:?} {:?}",
                    token, self.m_result
                ));
            }
        }
    }

    fn visit_variable(&mut self, token: &TokenType) {
        self.m_result.push(match token {
            TokenType::Identifier(identifier) => match identifier.as_str() {
                "true" => Value::Boolean(true),
                "false" => Value::Boolean(false),
                "nil" => Value::Nil,
                identifier => match self.m_env.borrow().get(identifier) {
                    Some(value) => value.clone(),
                    None => {
                        self.m_errors
                            .push(format!("Undefined variable => {:?}", token));
                        Value::Nil
                    }
                },
            },
            token => {
                self.m_errors
                    .push(format!("Invalid variable expression => {:?}", token));
                Value::Nil
            }
        });
    }

    fn visit_assign(&mut self, token: &TokenType, expression: &Expr) {
        expression.accept(self);

        if !self.m_errors.is_empty() {
            return;
        }

        match self.m_result.pop() {
            Some(value) => match token {
                TokenType::Identifier(identifier) => {
                    if let Err(err) = self
                        .m_env
                        .borrow_mut()
                        .assign(identifier.to_string(), value.clone())
                    {
                        self.m_errors.push(format!("{}", err));
                    }

                    self.m_result.push(value);
                }
                token => {
                    self.m_errors
                        .push(format!("Invalid assign expression => {:?}", token));
                }
            },
            None => {
                self.m_errors.push(format!(
                    "Invalid assign expression => {:?} {:?}",
                    token, self.m_result
                ));
            }
        }
    }

    fn visit_logical(&mut self, left: &Expr, token: &TokenType, right: &Expr) {
        left.accept(self);

        if !self.m_errors.is_empty() {
            return;
        }

        match self.m_result.pop() {
            Some(Value::Boolean(left)) => {
                if token == &TokenType::Or && left {
                    self.m_result.push(Value::Boolean(true));
                    return;
                } else if token == &TokenType::And && !left {
                    self.m_result.push(Value::Boolean(false));
                    return;
                }
            }
            Some(left) => {
                self.m_errors.push(format!(
                    "Invalid logical expression => {:?} {:?}",
                    token, left
                ));
                return;
            }
            None => {
                self.m_errors.push(format!(
                    "Invalid logical expression => {:?} {:?}",
                    token, self.m_result
                ));
                return;
            }
        }

        right.accept(self);

        if !self.m_errors.is_empty() {
            return;
        }

        match self.m_result.pop() {
            Some(Value::Boolean(right)) => {
                if matches!(token, TokenType::Or | TokenType::And) {
                    self.m_result.push(Value::Boolean(right));
                }
            }
            Some(right) => {
                self.m_errors.push(format!(
                    "Invalid logical expression => {:?} {:?}",
                    token, right
                ));
            }
            None => {
                self.m_errors.push(format!(
                    "Invalid logical expression => {:?} {:?}",
                    token, self.m_result
                ));
            }
        }
    }

    fn visit_call(&mut self, callee: &Expr, arguments: &[Expr]) {
        callee.accept(self);

        if !self.m_errors.is_empty() {
            return;
        }

        let callee = match self.m_result.pop() {
            Some(callee) => callee,
            None => {
                self.m_errors
                    .push(format!("Invalid call expression => {:?}", callee));
                return;
            }
        };

        let mut arguments = arguments.to_vec();
        let mut idents = Vec::new();
        for argument in arguments.iter_mut() {
            let ident = match argument {
                Expr::Variable { m_token } => Some(format!("{}", m_token)),
                _ => None,
            };

            argument.accept(self);

            if !self.m_errors.is_empty() {
                return;
            }

            idents.push(ident);
        }

        let arguments = self
            .m_result
            .split_off(self.m_result.len() - arguments.len());

        match callee {
            Value::Callable(callable) => {
                if callable.arity() != arguments.len() {
                    self.m_errors.push(format!(
                        "Invalid call expression => {:?}{:?}",
                        callable, arguments
                    ));
                    return;
                }

                let arguments = arguments
                    .into_iter()
                    .zip(idents)
                    .map(|(value, _ident)| (None, value))
                    .collect();

                match callable.call(arguments) {
                    Ok(result) => self.m_result.push(result),
                    Err(err) => self.m_errors.extend(err),
                }
            }
            callee => {
                self.m_errors
                    .push(format!("Invalid call expression => {:?}", callee));
            }
        }
    }

    fn visit_function(&mut self, params: &[TokenType], body: &Stmt) {
        let callable = Value::Callable(Callable::Function(
            Some(self.m_env.clone()),
            params.to_vec(),
            params.len(),
            Box::new(body.clone()),
        ));

        self.m_result.push(callable);
    }
}

pub trait StmtVisitor {
    fn visit_block(&mut self, statements: &[Stmt]);
    fn visit_expression(&mut self, expression: &Expr);
    fn visit_var(&mut self, name: &TokenType, initializer: &Option<Expr>);
    fn visit_while(&mut self, condition: &Expr, body: &Stmt);
    fn visit_if(&mut self, condition: &Expr, then_branch: &Stmt, else_branch: &Option<Box<Stmt>>);
    fn visit_function(&mut self, name: &TokenType, params: &[TokenType], body: &Stmt);
    fn visit_return(&mut self, value: &Option<Expr>);
    // fn visit_class(&mut self, name: &Token, methods: &[Stmt]);
}

#[derive(Debug, Clone)]
pub enum ErrorValue {
    Error(String),
    Return(Value),
}

impl Display for ErrorValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorValue::Error(message) => write!(f, "{}", message),
            ErrorValue::Return(value) => write!(f, "{}", value),
        }
    }
}

pub struct StmtEvaluator {
    m_env: Rc<RefCell<Environment>>,
    m_errors: Vec<ErrorValue>,
}

impl StmtEvaluator {
    pub fn new(env: &Rc<RefCell<Environment>>) -> Self {
        Self {
            m_env: env.clone(),
            m_errors: Vec::new(),
        }
    }

    pub fn get_result(&mut self) -> Result<(), Vec<ErrorValue>> {
        if self.m_errors.is_empty() {
            Ok(())
        } else {
            Err(self.m_errors.clone())
        }
    }
}

impl StmtVisitor for StmtEvaluator {
    fn visit_block(&mut self, statements: &[Stmt]) {
        let block_scope = Environment::new_scope(&self.m_env);
        for stmt in statements.iter() {
            let mut visitor = StmtEvaluator::new(&block_scope);
            stmt.accept(&mut visitor);
            if let Err(err) = visitor.get_result() {
                self.m_errors.extend(err);
            }
        }
    }

    fn visit_expression(&mut self, expression: &Expr) {
        let mut visitor = ExprEvaluator::new(&self.m_env);
        expression.accept(&mut visitor);
        if let Err(err) = visitor.get_result() {
            self.m_errors.extend(err.into_iter().map(ErrorValue::Error));
        }
    }

    fn visit_var(&mut self, name: &TokenType, initializer: &Option<Expr>) {
        let mut visitor = ExprEvaluator::new(&self.m_env);
        if let Some(initializer) = initializer {
            initializer.accept(&mut visitor);
        }
        match visitor.get_result() {
            Ok(result) => {
                if let TokenType::Identifier(name) = name {
                    // let inner_scope = Environment::new_scope(&self.m_env);
                    self.m_env.borrow_mut().define(name.to_string(), result);
                    // for stmt in statements.iter() {
                    //     let mut visitor = StmtEvaluator::new(&inner_scope);
                    //     stmt.accept(&mut visitor);
                    //     if let Err(err) = visitor.get_result() {
                    //         self.m_errors.extend(err);
                    //     }
                    // }
                }
            }
            Err(err) => self.m_errors.extend(err.into_iter().map(ErrorValue::Error)),
        }
    }

    fn visit_if(&mut self, condition: &Expr, then_branch: &Stmt, else_branch: &Option<Box<Stmt>>) {
        let mut visitor = ExprEvaluator::new(&self.m_env);
        condition.accept(&mut visitor);
        match visitor.get_result() {
            Ok(result) => {
                let inner_scope = Environment::new_scope(&self.m_env);
                if result.is_truthy() {
                    let mut visitor = StmtEvaluator::new(&inner_scope);
                    then_branch.accept(&mut visitor);
                    if let Err(err) = visitor.get_result() {
                        self.m_errors.extend(err)
                    }
                } else if let Some(else_branch) = else_branch {
                    let mut visitor = StmtEvaluator::new(&inner_scope);
                    else_branch.accept(&mut visitor);
                    if let Err(err) = visitor.get_result() {
                        self.m_errors.extend(err)
                    }
                }
            }
            Err(err) => self.m_errors.extend(err.into_iter().map(ErrorValue::Error)),
        }
    }

    fn visit_while(&mut self, condition: &Expr, body: &Stmt) {
        while {
            let mut visitor = ExprEvaluator::new(&self.m_env);
            condition.accept(&mut visitor);
            match visitor.get_result() {
                Ok(result) => result.is_truthy(),
                Err(err) => {
                    self.m_errors.extend(err.into_iter().map(ErrorValue::Error));
                    false
                }
            }
        } {
            let inner_scope = Environment::new_scope(&self.m_env);
            let mut visitor = StmtEvaluator::new(&inner_scope);
            body.accept(&mut visitor);
            if let Err(err) = visitor.get_result() {
                self.m_errors.extend(err)
            }
        }
    }

    fn visit_function(&mut self, name: &TokenType, params: &[TokenType], body: &Stmt) {
        let callable = Value::Callable(Callable::Function(
            Some(self.m_env.clone()),
            params.to_vec(),
            params.len(),
            Box::new(body.clone()),
        ));

        // println!("{:?}", callable);
        self.m_env
            .borrow_mut()
            .define(format!("{}", name), callable.clone());
    }

    fn visit_return(&mut self, value: &Option<Expr>) {
        let mut visitor = ExprEvaluator::new(&self.m_env);
        if let Some(value) = value {
            value.accept(&mut visitor);
        }

        match visitor.get_result() {
            Ok(result) => {
                self.m_errors.push(ErrorValue::Return(result));
            }
            Err(err) => self.m_errors.extend(err.into_iter().map(ErrorValue::Error)),
        }
    }
}
