use crate::token::{Value, Token};
use crate::error::Error;

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Environment {
    // Store the `parent` environment.
    // Note the decision here to store the owned `Environment`, as opposed to a reference. This is
    // for my convenience, as storing mutable references would involve lifetimes, and I'm not yet
    // mentally prepared for that. (I'm not sure if it's even possible...!)
    enclosing: Option<Box<Environment>>,

    // Uninitialized identifiers will have value `None`.
    values: HashMap<String, Option<Value>>,

    // Store the modified values of the `parent` environment.
    // This is because `enclosing` is a clone of the `parent` environment. Hence, changes made by
    // `assign` in this environment will not `be saved` once the environments have been swapped
    // back in the interpreter. See `update()`.
    parent_modified: HashMap<String, Value>,
}

impl Environment {
    pub fn new(enclosing: Option<Environment>) -> Self {
        Self {
            enclosing: enclosing.map(Box::new),
            values: HashMap::new(),
            parent_modified: HashMap::new(),
        }
    }

    // Define a new identifier. Can be `None` (uninitialized).
    pub fn define(&mut self, name: String, value: Option<&Value>) {
        self.values.insert(name, value.cloned());
    }

    // Get the value assigned to `name`. Return the `Option<>` - the calling function will have to
    // deal with uninitialized identifiers themself. If not found, return RuntimeError.
    pub fn get(&self, name: &Token) -> Result<Option<Value>, Error> {
        let v = self.values.get(&name.lexeme);
        match v {
            Some(x) => Ok(x.clone()),
            None => {
                // If the variable is not found in this scope, maybe it is found in the enclosing
                // scope? Recursively search enclosing scopes for the variable.
                if let Some(enclosing) = &self.enclosing {
                    Ok(enclosing.get(name)?)
                } else {
                    // Variable not found and this scope is the outermost.
                    Err(self.undefined_variable_error(name))
                }
            }
        }
    }

    // Assign value to `name`.
    // Note here `value` is *not* `Option<Value>`.
    pub fn assign(&mut self, name: &Token, value: &Value) -> Result<(), Error> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), Some(value.to_owned()));
            Ok(())
        } else {
            // See above.
            if let Some(enclosing) = &mut self.enclosing {
                enclosing.assign(name, value)?;
                // Keep track of changes made to the `parent` environment, so they can be `saved`.
                self.parent_modified.insert(name.lexeme.clone(), value.to_owned());
                Ok(())
            } else {
                Err(self.undefined_variable_error(name))
            }
        }
    }

    // Given a `child` environment, `save` the changes made from the `child` environment.
    pub fn update(&mut self, update_from: &mut Environment) {
        for (key, value) in &update_from.parent_modified {
            self.assign_string(key, value);
        }
    }

    // Similar to `assign()`, but takes a `String` for a `name` as opposed to a `Token`.
    // This is because `parent_modified` cannot store `Token`s as they are not hashable.
    fn assign_string(&mut self, name: &String, value: &Value) {
        if self.values.contains_key(name) {
            self.values.insert(name.to_owned(), Some(value.to_owned()));
        } else {
            if let Some(enclosing) = &mut self.enclosing {
                enclosing.assign_string(name, value);
                self.parent_modified.insert(name.clone(), value.to_owned());
            } else {
                // Note this is should not be possible, since all variables in `parent_modified` are
                // there because the variable has been found in the `parent` environment.
                // However, there may be some edge cases if it is possible to `drop` a variable.
                panic!("Variable {name} not found when calling `assign_string`");
            }
        }
    }

    // Helper function to return a RuntimeError for undefined variables.
    fn undefined_variable_error(&self, token: &Token) -> Error {
        Error::RuntimeError {
            token: token.to_owned(),
            message: format!("Undefined variable '{}'.", token.lexeme)
        }
    }
}

