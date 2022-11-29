use crate::expr::{self, ExprVisitor};
use crate::stmt::{self, StmtVisitor};
use crate::token::{self, TokenType, Value};
use crate::environment::Environment;
use crate::error::Error;

pub struct Interpreter<'a> {
    pub environment: &'a mut Environment,
}

// Expression evaluation.
// Note the return enum is `Value`, which is similar to a `Literal`, but specifically represents
// the values of evaluated expressions.
impl<'a> ExprVisitor<Value, Error> for Interpreter<'a> {
    fn visit_literal_expr(&mut self, value: &token::Literal) -> Result<Value, Error> {
        Ok(Value::from(value.to_owned()))
    }

    fn visit_grouping_expr(&mut self, expression: &expr::Expr) -> Result<Value, Error> {
        self.evaluate(expression)
    }

    fn visit_unary_expr(&mut self, operator: &token::Token, right: &expr::Expr) -> Result<Value, Error> {
        let right_eval: Value = self.evaluate(right)?;

        match operator.type_ {
            TokenType::Bang => {
                Ok(Value::Bool(!self.is_truthy(&right_eval)))
            },
            TokenType::Minus => {
                if let Value::Number(x) = right_eval {
                    Ok(Value::Number(-x))
                } else {
                    Err(self.operand_not_number_error(operator))
                }
            },

            // Note no other operator type is reachable, since the parser builds unary expressions
            // if and only if the operator is either `Bang` or `Minus`.
            _ => unreachable!(),
        }
    }

    fn visit_binary_expr(&mut self, left: &expr::Expr, operator: &token::Token, right: &expr::Expr) -> Result<Value, Error> {
        let left_eval: Value = self.evaluate(left)?;
        let right_eval: Value = self.evaluate(right)?;

        match operator.type_ {
            TokenType::Greater => {
                if let (Value::Number(x), Value::Number(y)) = (left_eval, right_eval) {
                    Ok(Value::Bool(x > y))
                } else {
                    Err(self.operand_not_number_error(operator))
                }
            },
            TokenType::GreaterEqual => {
                if let (Value::Number(x), Value::Number(y)) = (left_eval, right_eval) {
                    Ok(Value::Bool(x >= y))
                } else {
                    Err(self.operand_not_number_error(operator))
                }
            },
            TokenType::Less => {
                if let (Value::Number(x), Value::Number(y)) = (left_eval, right_eval) {
                    Ok(Value::Bool(x < y))
                } else {
                    Err(self.operand_not_number_error(operator))
                }
            },
            TokenType::LessEqual => {
                if let (Value::Number(x), Value::Number(y)) = (left_eval, right_eval) {
                    Ok(Value::Bool(x <= y))
                } else {
                    Err(self.operand_not_number_error(operator))
                }
            },
            TokenType::Minus => {
                if let (Value::Number(x), Value::Number(y)) = (left_eval, right_eval) {
                    Ok(Value::Number(x - y))
                } else {
                    Err(self.operand_not_number_error(operator))
                }
            },
            TokenType::Slash => {
                if let (Value::Number(x), Value::Number(y)) = (left_eval, right_eval) {
                    if y == 0.0 {
                        Err(self.error(operator, "Divide by zero."))
                    } else {
                        Ok(Value::Number(x / y))
                    }
                } else {
                    Err(self.operand_not_number_error(operator))
                }
            },
            TokenType::Star => {
                if let (Value::Number(x), Value::Number(y)) = (left_eval, right_eval) {
                    Ok(Value::Number(x * y))
                } else {
                    Err(self.operand_not_number_error(operator))
                }
            },
            TokenType::Plus => {
                if let (&Value::Number(x), &Value::Number(y)) = (&left_eval, &right_eval) {
                    Ok(Value::Number(x + y))
                } else {
                    // If the values aren't *both* numbers, return the concatenated string
                    // representations of the values.
                    let (x, y) = (left_eval, right_eval);
                    Ok(Value::String_(format!("{}{}", x, y)))
                }
            },

            // My implementation of != and == simply piggybacks Rust's `PartialEq` trait.
            TokenType::BangEqual => {
                Ok(Value::Bool(left_eval != right_eval))
            },
            TokenType::EqualEqual => {
                Ok(Value::Bool(left_eval == right_eval))
            },

            // Note no other operator type is reachable, since the parser builds binary expressions
            // if and only if the operator is one of the above.
            _ => unreachable!(),
        }
    }

    fn visit_variable_expr(&mut self, name: &token::Token) -> Result<Value, Error> {
        self.environment.get(name)?.ok_or_else(|| self.error(name, "Variable not initialized."))
    }

    fn visit_assign_expr(&mut self, name: &token::Token, value: &expr::Expr) -> Result<Value, Error> {
        let value_eval = self.evaluate(value)?;
        self.environment.assign(name, &value_eval)?;
        Ok(value_eval)
    }
}

// Statement execution.
impl<'a> StmtVisitor<(), Error> for Interpreter<'a> {
    fn visit_expression_stmt(&mut self, expression: &expr::Expr) -> Result<(), Error> {
        self.evaluate(expression)?;
        Ok(())
    }

    fn visit_print_stmt(&mut self, expression: &expr::Expr) -> Result<(), Error> {
        let value = self.evaluate(expression)?;
        println!("{}", value.to_string());
        Ok(())
    }

    fn visit_var_stmt(&mut self, name: &token::Token, initializer: &Option<expr::Expr>) -> Result<(), Error> {
        if let Some(x) = initializer {
            let value = self.evaluate(x)?;
            self.environment.define(name.lexeme.to_owned(), Some(&value));
        } else {
            self.environment.define(name.lexeme.to_owned(), None);
        }
        Ok(())
    }
}

impl<'a> Interpreter<'a> {
    pub fn new(environment: &'a mut Environment) -> Self {
        Self {
            environment,
        }
    }
    // Interface. If something went wrong, return a `RuntimeError` object.
    pub fn interpret(&mut self, statements: &Vec<stmt::Stmt>) -> Result<(), Error> {
        for statement in statements {
            if let Err(Error::RuntimeError { token, message }) = self.execute(statement) {
                crate::error_token(&token, &message);
                return Err(Error::RuntimeError { token, message });
            }
        }
        Ok(())
    }

    // Runs `accept` for statements.
    fn execute(&mut self, statement: &stmt::Stmt) -> Result<(), Error> {
        self.accept_stmt(statement)
    }

    // Runs `accept` for expressions.
    fn evaluate(&mut self, expr: &expr::Expr) -> Result<Value, Error> {
        self.accept_expr(expr)
    }

    fn is_truthy(&self, value: &Value) -> bool {
        match *value {
            Value::Nil => false,
            Value::Bool(x) => x,
            _ => true,
        }
    }

    fn operand_not_number_error(&self, token: &token::Token) -> Error {
        self.error(token, "Operand(s) must be a number.")
    }

    // Return a `RuntimeError` object to be bubbled up.
    // Reporting to `crate::error_token` once it has been bubbled up to `interpret()`.
    fn error(&self, token: &token::Token, message: &str) -> Error {
        // crate::error_token(token, message);
        Error::RuntimeError { token: token.to_owned(), message: message.to_owned() }
    }
}

