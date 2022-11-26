use crate::expr::{self, ExprVisitor};
use crate::token::{self, TokenType, Value};
use crate::error::Error;

pub struct Interpreter;

impl ExprVisitor<Value, Error> for Interpreter {
    fn visit_literal_expr(&self, value: &token::Literal) -> Result<Value, Error> {
        Ok(Value::from(value.to_owned()))
    }
    fn visit_grouping_expr(&self, expression: &expr::Expr) -> Result<Value, Error> {
        self.evaluate(expression)
    }
    fn visit_unary_expr(&self, operator: &token::Token, right: &expr::Expr) -> Result<Value, Error> {
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

            _ => unreachable!(),
        }
    }
    fn visit_binary_expr(&self, left: &expr::Expr, operator: &token::Token, right: &expr::Expr) -> Result<Value, Error> {
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
                    let (x, y) = (left_eval, right_eval);
                    Ok(Value::String_(format!("{}{}", x, y)))  // TODO: better way to do this?
                }
                // } else {
                //     Err(self.error(operator, "Operands must be two numbers or two strings."))
                    // Err(
                    //     Error::RuntimeError { message: "Operands must be two numbers or two strings.".to_owned(),
                        // token: operator.to_owned() }
                       // )
                // }
            },
            TokenType::BangEqual => {
                Ok(Value::Bool(left_eval != right_eval))
            },
            TokenType::EqualEqual => {
                Ok(Value::Bool(left_eval == right_eval))
            },

            _ => unreachable!(),
        }
    }
}

impl Interpreter {
    pub fn interpret(&self, expression: &expr::Expr) -> Result<Value, Error> {
        self.evaluate(expression)
    }

    fn evaluate(&self, expr: &expr::Expr) -> Result<Value, Error> {
        self.accept(expr)
    }

    fn is_truthy(&self, value: &Value) -> bool {
        match *value {
            Value::Nil => false,
            Value::Bool(x) => x,
            _ => true,
        }
    }

    fn operand_not_number_error(&self, token: &token::Token) -> Error {
        // TODO: why not just report the error directly via crate::error_token?
        // crate::error_token(token, "Operand(s) must be a number.");
        // Error::RuntimeError
        // Error::RuntimeError {
        //     message: "Operand(s) must be a number.".to_owned(),
        //     token: token.to_owned(),
        // }
        self.error(token, "Operand(s) must be a number.")
    }

    fn error(&self, token: &token::Token, message: &str) -> Error {
        crate::error_token(token, message);
        Error::RuntimeError
    }
}

