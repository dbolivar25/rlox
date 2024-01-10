use crate::value::Value;

use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use anyhow::Result;


#[derive(Debug)]
pub struct Environment {
    m_scope: HashMap<String, Value>,
    m_parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Rc<RefCell<Environment>> {
        Rc::new(RefCell::new(Environment {
            m_scope: HashMap::new(),
            m_parent: None,
        }))
    }

    pub fn new_scope(parent: &Rc<RefCell<Environment>>) -> Rc<RefCell<Environment>> {
        Rc::new(RefCell::new(Environment {
            m_scope: HashMap::new(),
            m_parent: Some(parent.clone()),
        }))
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.m_scope.insert(name, value);
    }

    pub fn assign(&mut self, name: String, value: Value) -> Result<()> {
        match self.m_scope.get_mut(&name) {
            Some(v) => {
                *v = value;
                Ok(())
            },
            None => match &self.m_parent {
                Some(parent) => parent.borrow_mut().assign(name, value),
                None => Err(anyhow::anyhow!("Undefined variable '{}'", name)),
            },
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        match self.m_scope.get(name) {
            Some(value) => Some(value.clone()),
            None => self.m_parent.as_ref().and_then(|parent| parent.borrow().get(name)),
        }
    }
}


