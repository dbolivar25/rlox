use crate::environment::Environment;
use std::fmt::{Debug, Display};

#[derive(Clone)]
pub struct Callable {
    m_env: Option<Environment>,
    m_arity: usize,
    m_call: Box<fn(Vec<Value>) -> Value>,
}

impl Callable {
    pub fn new(
        env: Option<Environment>,
        arity: usize,
        call: Box<fn(Vec<Value>) -> Value>,
    ) -> Callable {
        Callable {
            m_env: env,
            m_arity: arity,
            m_call: call,
        }
    }

    pub fn call(&self, arguments: Vec<Value>) -> Value {
        (self.m_call)(arguments)
    }

    pub fn arity(&self) -> usize {
        self.m_arity
    }
}

impl Debug for Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<callable>")
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
            Value::Callable(_callable) => write!(f, "<function>",),
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
