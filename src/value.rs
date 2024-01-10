use crate::environment::Environment;
use crate::token::Token;
use crate::visitor::ErrorValue;
use crate::{ast::*, visitor::StmtEvaluator};

use anyhow::Result;
use itertools::Itertools;
use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::rc::Rc;

#[derive(Clone)]
pub enum Callable {
    NativeFunction(
        Option<Rc<RefCell<Environment>>>,
        usize,
        Box<fn(Vec<Value>) -> Value>,
    ),
    Function(
        Option<Rc<RefCell<Environment>>>,
        Vec<Token>,
        usize,
        Box<Stmt>,
    ),
}

impl Callable {
    pub fn call(&self, arguments: Vec<(Option<String>, Value)>) -> Result<Value, Vec<String>> {
        match self {
            Callable::NativeFunction(_env, _arity, call) => {
                Ok(call(arguments.into_iter().map(|(_, v)| v).collect()))
            }
            Callable::Function(env, params, _arity, stmt) => {
                let inner_scope = Environment::new_scope(env.as_ref().unwrap());

                for (param, (_ident, argument)) in params.iter().zip(arguments.iter()) {
                    // match _ident {
                    //     Some(ident) => inner_scope
                    //         .borrow_mut()
                    //         .assign(ident.clone(), argument.clone())
                    //         .unwrap(),
                    //     None => inner_scope
                    //         .borrow_mut()
                    //         .define(format!("{}", param), argument.clone()),
                    // }

                    inner_scope
                        .borrow_mut()
                        .define(format!("{}", param), argument.clone())
                }
                // dbg!(&inner_scope);

                let mut visitor = StmtEvaluator::new(&inner_scope);
                stmt.accept(&mut visitor);

                match visitor.get_result() {
                    Ok(()) => Ok(Value::Nil),
                    Err(value) => match value.last() {
                        Some(ErrorValue::Return(value)) => Ok(value.clone()),
                        _ => Err(value
                            .into_iter()
                            .map(|e| match e {
                                ErrorValue::Return(_) => unreachable!(),
                                ErrorValue::Error(message) => message,
                            })
                            .collect()),
                    },
                }
            }
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            Callable::NativeFunction(_env, arity, _call) => *arity,
            Callable::Function(_ident, _env, arity, _stmt) => *arity,
        }
    }
}

impl Debug for Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Callable::NativeFunction(_env, _arity, _call) => write!(f, "<native function>"),
            Callable::Function(_env, _param, _arity, _stmt) => write!(f, "fun ({}) {:?}", _param.iter().join(", "), _stmt),
        }
    }
}

#[derive(Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Callable(Callable),
    Nil,
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::Boolean(boolean) => *boolean,
            _ => true,
        }
    }

    pub fn is_equal(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Number(number), Value::Number(other_number)) => {
                0.0000000001 > (number - other_number).abs()
            }
            (Value::String(string), Value::String(other_string)) => string == other_string,
            (Value::Boolean(boolean), Value::Boolean(other_boolean)) => boolean == other_boolean,
            (Value::Callable(_callable), Value::Callable(_other_callable)) => false,
            (Value::Nil, _) => false,
            (_, Value::Nil) => false,
            _ => false,
        }
    }

    pub fn is_not_equal(&self, other: &Value) -> bool {
        !self.is_equal(other)
    }

    pub fn is_greater(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Number(number), Value::Number(other_number)) => number > other_number,
            _ => false,
        }
    }

    pub fn is_greater_or_equal(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Number(number), Value::Number(other_number)) => number >= other_number,
            _ => false,
        }
    }

    pub fn is_less(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Number(number), Value::Number(other_number)) => number < other_number,
            _ => false,
        }
    }

    pub fn is_less_or_equal(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Number(number), Value::Number(other_number)) => number <= other_number,
            _ => false,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            Value::Number(number) => Some(*number),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<String> {
        match self {
            Value::String(string) => Some(string.clone()),
            _ => None,
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Value::Boolean(boolean) => Some(*boolean),
            _ => None,
        }
    }

    pub fn as_some(&self) -> Option<()> {
        match self {
            Value::Nil => None,
            _ => Some(()),
        }
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(number) => write!(f, "{}", number),
            Value::String(string) => write!(f, "\"{}\"", string),
            Value::Boolean(boolean) => write!(f, "{}", boolean),
            Value::Callable(callable) => write!(f, "{:?}", callable),
            Value::Nil => write!(f, "nil"),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(string) => write!(f, "{}", string),
            other => write!(f, "{:?}", other),
        }
    }
}
