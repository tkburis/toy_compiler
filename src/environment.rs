use crate::token::{Value, Token};
use crate::error::Error;

use std::collections::HashMap;

pub struct Environment {
    values: HashMap<String, Option<Value>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Option<&Value>) {
        self.values.insert(name, value.cloned());
    }

    pub fn get(&self, name: &Token) -> Result<Option<Value>, Error> {
        let v = self.values.get(&name.lexeme);
        match v {
            Some(x) => Ok(x.clone()),
            None => {
                Err(self.undefined_variable_error(name))
            }
        }
    }

    // Note here `value` is *not* an `Option<Value>`.
    pub fn assign(&mut self, name: &Token, value: &Value) -> Result<(), Error> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), Some(value.to_owned()));
            Ok(())
        } else {
            Err(self.undefined_variable_error(name))
        }
    }

    fn undefined_variable_error(&self, token: &Token) -> Error {
        Error::RuntimeError { token: token.to_owned(), message: format!("Undefined variable '{}'.", token.lexeme) }
    }
}

