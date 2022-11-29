use crate::token::{Value, Token};
use crate::error::Error;

use std::collections::HashMap;

pub struct Environment {
    values: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: &Value) {
        self.values.insert(name, value.clone());
    }

    pub fn get(&self, name: &Token) -> Result<Value, Error> {
        let v = self.values.get(&name.lexeme);
        match v {
            Some(x) => Ok(x.clone()),
            None => {
                crate::error_token(name, &format!("Undefined variable '{}'.", name.lexeme));
                Err(Error::RuntimeError)
            }
        }
    }
}

