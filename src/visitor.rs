use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::*;
use crate::environment::Environment;
use crate::token::*;
use crate::value::*;

pub trait ExprVisitor {
    fn visit_binary(&mut self, left: &Expr, token: &Token, right: &Expr);
    fn visit_grouping(&mut self, expression: &Expr);
    fn visit_literal(&mut self, token: &Token);
    fn visit_unary(&mut self, token: &Token, expression: &Expr);
    fn visit_variable(&mut self, token: &Token);
    fn visit_assign(&mut self, token: &Token, expression: &Expr);
    fn visit_logical(&mut self, left: &Expr, token: &Token, right: &Expr);
    fn visit_call(&mut self, callee: &Expr, paren: &Token, arguments: &[Expr]);
}

pub struct ExprPrinter {
    m_content: Vec<String>,
}

impl ExprPrinter {
    pub fn new() -> Self {
        Self {
            m_content: Vec::new(),
        }
    }

    pub fn get_result(&self) -> String {
        self.m_content.join(" ")
    }
}

impl ExprVisitor for ExprPrinter {
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

    fn visit_variable(&mut self, token: &Token) {
        self.m_content.push("(".into());
        self.m_content.push(format!("{:?}", token.get_token_type()));
        self.m_content.push(")".into());
    }

    fn visit_assign(&mut self, token: &Token, expression: &Expr) {
        self.m_content.push("(".into());
        self.m_content.push(format!("{:?}", token.get_token_type()));
        expression.accept(self);
        self.m_content.push(")".into());
    }

    fn visit_logical(&mut self, left: &Expr, token: &Token, right: &Expr) {
        self.m_content.push("(".into());
        self.m_content
            .push(format!("{:?}", token.get_token_type(),));
        left.accept(self);
        right.accept(self);
        self.m_content.push(")".into());
    }

    fn visit_call(&mut self, callee: &Expr, paren: &Token, arguments: &[Expr]) {
        self.m_content.push("(".into());
        self.m_content.push("Call".into());
        callee.accept(self);
        self.m_content.push(format!("{:?}", paren.get_token_type()));
        for argument in arguments.iter() {
            argument.accept(self);
        }
        self.m_content.push(")".into());
    }
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
                    token_type => {
                        self.m_errors
                            .push(format!("Invalid binary operator => {}", token_type));
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
                    token_type => {
                        self.m_errors
                            .push(format!("Invalid binary operator => {}", token_type));
                        Value::Nil
                    }
                });
            }
            (Some(right), Some(left)) => {
                self.m_errors.push(format!(
                    "Invalid binary expression => {:?} {:?} {:?}",
                    left, token, right
                ));
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

    fn visit_literal(&mut self, token: &Token) {
        self.m_result.push(match token.get_token_type() {
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
                    token_type => {
                        self.m_errors
                            .push(format!("Invalid unary operator => {}", token_type));
                        Value::Nil
                    }
                });
            }
            Some(Value::Boolean(boolean)) => {
                self.m_result.push(match token.get_token_type() {
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

    fn visit_variable(&mut self, token: &Token) {
        self.m_result.push(match token.get_token_type() {
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

    fn visit_assign(&mut self, token: &Token, expression: &Expr) {
        expression.accept(self);

        if !self.m_errors.is_empty() {
            return;
        }

        match self.m_result.pop() {
            Some(value) => match token.get_token_type() {
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

    fn visit_logical(&mut self, left: &Expr, token: &Token, right: &Expr) {
        left.accept(self);

        if !self.m_errors.is_empty() {
            return;
        }

        match self.m_result.pop() {
            Some(Value::Boolean(left)) => {
                if token.get_token_type() == &TokenType::Or && left {
                    self.m_result.push(Value::Boolean(true));
                    return;
                } else if token.get_token_type() == &TokenType::And && !left {
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
                if token.get_token_type() == &TokenType::Or {
                    self.m_result.push(Value::Boolean(right));
                } else if token.get_token_type() == &TokenType::And {
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

    fn visit_call(&mut self, callee: &Expr, _paren: &Token, arguments: &[Expr]) {
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
        for argument in arguments.iter_mut() {
            argument.accept(self);

            if !self.m_errors.is_empty() {
                return;
            }
        }

        let arguments = self
            .m_result
            .split_off(self.m_result.len() - arguments.len());

        match callee {
            Value::Callable(mut callable) => {
                if callable.arity() != arguments.len() {
                    self.m_errors.push(format!(
                        "Invalid call expression => {:?}{:?}",
                        callable, arguments
                    ));
                    return;
                }

                // dbg!(&self.m_env);

                self.m_result.push(callable.call(arguments));
            }
            callee => {
                self.m_errors
                    .push(format!("Invalid call expression => {:?}", callee));
            }
        }
    }
}

pub trait StmtVisitor {
    fn visit_block(&mut self, statements: &[Stmt]);
    fn visit_expression(&mut self, expression: &Expr);
    fn visit_print(&mut self, expression: &Expr);
    fn visit_var(&mut self, name: &Token, initializer: &Option<Expr>);
    fn visit_while(&mut self, condition: &Expr, body: &Stmt);
    fn visit_if(&mut self, condition: &Expr, then_branch: &Stmt, else_branch: &Option<Box<Stmt>>);
    fn visit_function(&mut self, name: &Token, params: &[Token], body: &[Stmt]);
    // fn visit_return(&mut self, keyword: &Token, value: &Option<Expr>);
    // fn visit_class(&mut self, name: &Token, methods: &[Stmt]);
}

pub struct StmtPrinter {
    m_content: Vec<String>,
    m_errors: Vec<String>,
}

impl StmtPrinter {
    pub fn new() -> Self {
        Self {
            m_content: Vec::new(),
            m_errors: Vec::new(),
        }
    }

    pub fn get_result(&self) -> Result<String, Vec<String>> {
        if self.m_errors.is_empty() {
            Ok(self.m_content.join(" "))
        } else {
            Err(self.m_errors.clone())
        }
    }
}

impl StmtVisitor for StmtPrinter {
    fn visit_block(&mut self, statements: &[Stmt]) {
        self.m_content.push("(".into());
        self.m_content.push("Block".into());
        for stmt in statements.iter() {
            let mut visitor = StmtPrinter::new();
            stmt.accept(&mut visitor);
            match visitor.get_result() {
                Ok(result) => self.m_content.push(result),
                Err(err) => self.m_errors.push(err.join("\n")),
            }
        }
        self.m_content.push(")".into());
    }

    fn visit_expression(&mut self, expression: &Expr) {
        let mut visitor = ExprPrinter::new();
        self.m_content.push("(".into());
        self.m_content.push("Expression".into());
        expression.accept(&mut visitor);
        self.m_content.push(visitor.get_result());
        self.m_content.push(")".into());
    }

    fn visit_print(&mut self, expression: &Expr) {
        let mut visitor = ExprPrinter::new();
        self.m_content.push("(".into());
        self.m_content.push("Print".into());
        expression.accept(&mut visitor);
        self.m_content.push(visitor.get_result());
        self.m_content.push(")".into());
    }

    fn visit_var(&mut self, name: &Token, initializer: &Option<Expr>) {
        let mut visitor = ExprPrinter::new();
        self.m_content.push("(".into());
        self.m_content.push("Var".into());
        self.m_content.push(format!("{:?}", name));
        if let Some(initializer) = initializer {
            initializer.accept(&mut visitor);
        }
        self.m_content.push(visitor.get_result());
        self.m_content.push(")".into());
    }

    fn visit_while(&mut self, condition: &Expr, body: &Stmt) {
        let mut visitor = ExprPrinter::new();
        self.m_content.push("(".into());
        self.m_content.push("While".into());
        condition.accept(&mut visitor);
        self.m_content.push(visitor.get_result());
        let mut visitor = StmtPrinter::new();
        body.accept(&mut visitor);
        self.m_content.push(visitor.get_result().unwrap());
        self.m_content.push(")".into());
    }

    fn visit_if(&mut self, condition: &Expr, then_branch: &Stmt, else_branch: &Option<Box<Stmt>>) {
        let mut visitor = ExprPrinter::new();
        self.m_content.push("(".into());
        self.m_content.push("If".into());
        condition.accept(&mut visitor);
        self.m_content.push(visitor.get_result());
        let mut visitor = StmtPrinter::new();
        then_branch.accept(&mut visitor);
        self.m_content.push(visitor.get_result().unwrap());
        if let Some(else_branch) = else_branch {
            let mut visitor = StmtPrinter::new();
            else_branch.accept(&mut visitor);
            self.m_content.push(visitor.get_result().unwrap());
        }
        self.m_content.push(")".into());
    }

    fn visit_function(&mut self, name: &Token, params: &[Token], body: &[Stmt]) {
        self.m_content.push("(".into());
        self.m_content.push("Function".into());
        self.m_content.push(format!("{:?}", name));
        for param in params.iter() {
            self.m_content.push(format!("{:?}", param));
        }
        for stmt in body.iter() {
            let mut visitor = StmtPrinter::new();
            stmt.accept(&mut visitor);
            match visitor.get_result() {
                Ok(result) => self.m_content.push(result),
                Err(err) => self.m_errors.push(err.join("\n")),
            }
        }
        self.m_content.push(")".into());
    }
}

pub struct StmtEvaluator {
    m_env: Rc<RefCell<Environment>>,
    m_errors: Vec<String>,
}

impl StmtEvaluator {
    pub fn new(env: &Rc<RefCell<Environment>>) -> Self {
        Self {
            m_env: env.clone(),
            m_errors: Vec::new(),
        }
    }

    pub fn get_result(&mut self) -> Result<(), Vec<String>> {
        if self.m_errors.is_empty() {
            Ok(())
        } else {
            Err(self.m_errors.clone())
        }
    }

    fn remap_stmt(stmt: &mut Stmt, mapped_params: &[(Token, Token)]) -> Stmt {
        match stmt {
            Stmt::Block { m_statements } => Stmt::new_block(
                m_statements
                    .iter_mut()
                    .map(|stmt| Self::remap_stmt(stmt, mapped_params))
                    .collect::<Vec<Stmt>>(),
            ),
            Stmt::Expression { m_expression } => {
                Stmt::new_expression(Self::remap_idents(m_expression, mapped_params))
            }
            Stmt::Print { m_expression } => {
                Stmt::new_print(Self::remap_idents(m_expression, mapped_params))
            }
            Stmt::Var {
                m_name,
                m_initializer,
            } => match m_initializer {
                Some(initializer) => Stmt::new_var(
                    m_name.clone(),
                    Some(Self::remap_idents(initializer, mapped_params)),
                ),
                None => stmt.clone(),
            },
            Stmt::While {
                m_condition,
                m_body,
            } => Stmt::new_while(
                Self::remap_idents(m_condition, mapped_params),
                Box::new(Self::remap_stmt(m_body.as_mut(), mapped_params)),
            ),
            Stmt::If {
                m_condition,
                m_then_branch,
                m_else_branch,
            } => Stmt::new_if(
                Self::remap_idents(m_condition, mapped_params),
                Box::new(Self::remap_stmt(m_then_branch.as_mut(), mapped_params)),
                match m_else_branch {
                    Some(else_branch) => Some(Box::new(Self::remap_stmt(
                        else_branch.as_mut(),
                        mapped_params,
                    ))),
                    None => None,
                },
            ),
            Stmt::Function {
                m_name,
                m_params,
                m_body,
            } => Stmt::new_function(
                m_name.clone(),
                m_params.clone(),
                m_body
                    .iter_mut()
                    .map(|stmt| Self::remap_stmt(stmt, mapped_params))
                    .collect::<Vec<Stmt>>(),
            ),
        }
    }

    fn remap_idents(expr: &mut Expr, mapped_params: &[(Token, Token)]) -> Expr {
        match expr {
            Expr::Binary {
                m_left,
                m_token,
                m_right,
            } => Expr::new_binary(
                Box::new(Self::remap_idents(m_left, mapped_params)),
                m_token.clone(),
                Box::new(Self::remap_idents(m_right, mapped_params)),
            ),
            Expr::Grouping { m_expression } => {
                Expr::new_grouping(Box::new(Self::remap_idents(m_expression, mapped_params)))
            }
            Expr::Literal { m_token: _ } => expr.clone(),
            Expr::Unary {
                m_token,
                m_expression,
            } => Expr::new_unary(
                m_token.clone(),
                Box::new(Self::remap_idents(m_expression, mapped_params)),
            ),
            Expr::Variable { m_token } => {
                if let TokenType::Identifier(name) = m_token.get_token_type() {
                    if let Some(mapped_param) = mapped_params.iter().find(|(param, _)| {
                        if let TokenType::Identifier(param) = param.get_token_type() {
                            param == name
                        } else {
                            false
                        }
                    }) {
                        Expr::new_variable(mapped_param.1.clone())
                    } else {
                        expr.clone()
                    }
                } else {
                    expr.clone()
                }
            }
            Expr::Assign { m_token, m_value } => Expr::new_assign(
                m_token.clone(),
                Box::new(Self::remap_idents(m_value, mapped_params)),
            ),
            Expr::Logical {
                m_left,
                m_token,
                m_right,
            } => Expr::new_logical(
                Box::new(Self::remap_idents(m_left, mapped_params)),
                m_token.clone(),
                Box::new(Self::remap_idents(m_right, mapped_params)),
            ),
            Expr::Call {
                m_callee,
                m_paren,
                m_arguments,
            } => Expr::new_call(
                Box::new(Self::remap_idents(m_callee, mapped_params)),
                m_paren.clone(),
                m_arguments
                    .iter_mut()
                    .map(|argument| Self::remap_idents(argument, mapped_params))
                    .collect::<Vec<Expr>>(),
            ),
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
                self.m_errors.push(err.join("\n"));
            }
        }
    }

    fn visit_expression(&mut self, expression: &Expr) {
        let mut visitor = ExprEvaluator::new(&self.m_env);
        expression.accept(&mut visitor);
        match visitor.get_result() {
            Ok(_) => {}
            Err(err) => self.m_errors.push(err.join("\n")),
        }
    }

    fn visit_print(&mut self, expression: &Expr) {
        let mut visitor = ExprEvaluator::new(&self.m_env);
        expression.accept(&mut visitor);
        match visitor.get_result() {
            Ok(result) => {
                println!("{}", result)
            }
            Err(err) => self.m_errors.push(err.join("\n")),
        }
    }

    fn visit_var(&mut self, name: &Token, initializer: &Option<Expr>) {
        let mut visitor = ExprEvaluator::new(&self.m_env);
        if let Some(initializer) = initializer {
            initializer.accept(&mut visitor);
        }
        match visitor.get_result() {
            Ok(result) => {
                if let TokenType::Identifier(name) = name.get_token_type() {
                    self.m_env.borrow_mut().define(name.to_string(), result);
                }
            }
            Err(err) => self.m_errors.push(err.join("\n")),
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
                        self.m_errors.push(err.join("\n"));
                    }
                } else if let Some(else_branch) = else_branch {
                    let mut visitor = StmtEvaluator::new(&inner_scope);
                    else_branch.accept(&mut visitor);
                    if let Err(err) = visitor.get_result() {
                        self.m_errors.push(err.join("\n"));
                    }
                }
            }
            Err(err) => self.m_errors.push(err.join("\n")),
        }
    }

    fn visit_while(&mut self, condition: &Expr, body: &Stmt) {
        while {
            let mut visitor = ExprEvaluator::new(&self.m_env);
            condition.accept(&mut visitor);
            match visitor.get_result() {
                Ok(result) => result.is_truthy(),
                Err(err) => {
                    self.m_errors.push(err.join("\n"));
                    false
                }
            }
        } {
            let inner_scope = Environment::new_scope(&self.m_env);
            let mut visitor = StmtEvaluator::new(&inner_scope);
            body.accept(&mut visitor);
            if let Err(err) = visitor.get_result() {
                self.m_errors.push(err.join("\n"));
            }
        }
    }

    fn visit_function(&mut self, name: &Token, params: &[Token], body: &[Stmt]) {
        // let mut env = self.m_env.take().unwrap().new_scope();
        // params.iter().enumerate().for_each(|(_, param)| {
        // env.define(format!("{}", param).as_str(), Value::Nil);
        // });

        // let mapped_params = params
        //     .into_iter()
        //     .enumerate()
        //     .filter_map(|(i, param)| {
        //         if let TokenType::Identifier(_name) = param.get_token_type() {
        //             Some((
        //                 param.clone(),
        //                 Token::new_token(TokenType::Identifier(format!("arg{}", i)), 0, 0, 0),
        //             ))
        //         } else {
        //             None
        //         }
        //     })
        //     .collect::<Vec<(Token, Token)>>();
        //
        // let body = body
        //     .to_vec()
        //     .into_iter()
        //     .map(|mut stmt| Self::remap_stmt(&mut stmt, &mapped_params))
        //     .collect::<Vec<Stmt>>();

        let callable = Value::Callable(Callable::Function(
            Some(self.m_env.clone()),
            params.to_vec(),
            params.len(),
            Box::new(Stmt::new_block(body.to_vec())),
        ));

        self.m_env
            .borrow_mut()
            .define(format!("{}", name), callable);
    }
}
