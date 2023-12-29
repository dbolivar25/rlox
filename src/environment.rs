use crate::lox_value::Value;

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Environment {
    m_scopes: Vec<HashMap<String, Value>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            m_scopes: vec![HashMap::new()],
        }
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.m_scopes.last_mut().unwrap().insert(name.into(), value);
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.m_scopes.iter().rev().find_map(|scope| scope.get(name))
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Value> {
        self.m_scopes
            .iter_mut()
            .rev()
            .find_map(|scope| scope.get_mut(name))
    }
    
    pub fn new_scope(&mut self) {
        self.m_scopes.push(HashMap::new());
    }

    pub fn drop_scope(&mut self) {
        self.m_scopes.pop();
    }
}
