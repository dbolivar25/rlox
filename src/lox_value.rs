#[derive(Clone, PartialEq)]
pub enum LoxValue {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

impl LoxValue {
    pub fn is_truthy(&self) -> bool {
        match self {
            LoxValue::Nil => false,
            LoxValue::Boolean(boolean) => *boolean,
            _ => true,
        }
    }

    pub fn is_equal(&self, other: &LoxValue) -> bool {
        match (self, other) {
            (LoxValue::Nil, LoxValue::Nil) => true,
            (LoxValue::Number(number), LoxValue::Number(other_number)) => {
                number == other_number
            }
            (LoxValue::String(string), LoxValue::String(other_string)) => {
                string == other_string
            }
            (LoxValue::Boolean(boolean), LoxValue::Boolean(other_boolean)) => {
                boolean == other_boolean
            }
            _ => false,
        }
    }

    pub fn is_not_equal(&self, other: &LoxValue) -> bool {
        !self.is_equal(other)
    }

    pub fn is_greater(&self, other: &LoxValue) -> bool {
        match (self, other) {
            (LoxValue::Number(number), LoxValue::Number(other_number)) => {
                number > other_number
            }
            _ => false,
        }
    }

    pub fn is_greater_or_equal(&self, other: &LoxValue) -> bool {
        match (self, other) {
            (LoxValue::Number(number), LoxValue::Number(other_number)) => {
                number >= other_number
            }
            _ => false,
        }
    }

    pub fn is_less(&self, other: &LoxValue) -> bool {
        match (self, other) {
            (LoxValue::Number(number), LoxValue::Number(other_number)) => {
                number < other_number
            }
            _ => false,
        }
    }

    pub fn is_less_or_equal(&self, other: &LoxValue) -> bool {
        match (self, other) {
            (LoxValue::Number(number), LoxValue::Number(other_number)) => {
                number <= other_number
            }
            _ => false,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            LoxValue::Number(number) => Some(*number),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<String> {
        match self {
            LoxValue::String(string) => Some(string.clone()),
            _ => None,
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            LoxValue::Boolean(boolean) => Some(*boolean),
            _ => None,
        }
    }

    pub fn as_nil(&self) -> Option<()> {
        match self {
            LoxValue::Nil => Some(()),
            _ => None,
        }
    }    
}

impl std::fmt::Debug for LoxValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxValue::Number(number) => write!(f, "{}", number),
            LoxValue::String(string) => write!(f, "{}", string),
            LoxValue::Boolean(boolean) => write!(f, "{}", boolean),
            LoxValue::Nil => write!(f, "nil"),
        }
    }
}
