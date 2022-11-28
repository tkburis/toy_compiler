use crate::expr::{Expr, ExprVisitor};
use crate::token;

pub struct AstPrinter;
impl ExprVisitor<String, ()> for AstPrinter {
    fn visit_binary_expr(&self, left: &Expr, operator: &token::Token, right: &Expr) -> Result<String, ()> {
        Ok(self.parenthesize(operator.lexeme.to_owned(), &[left, right]))
    }
    fn visit_grouping_expr(&self, expression: &Expr) -> Result<String, ()> {
        Ok(self.parenthesize("group".to_owned(), &[expression]))
    }
    fn visit_literal_expr(&self, value: &token::Literal) -> Result<String, ()> {
        Ok(value.to_string())
    }
    fn visit_unary_expr(&self, operator: &token::Token, right: &Expr) -> Result<String, ()> {
        Ok(self.parenthesize(operator.lexeme.to_owned(), &[right]))
    }
    fn visit_variable_expr(&self, name: &token::Token) -> Result<String, ()> {
        Ok(name.to_string())
    }
}

impl AstPrinter {
    pub fn print(&self, expr: &Expr) -> String {
        self.accept_expr(expr).unwrap()
    }
    fn parenthesize(&self, name: String, exprs: &[&Expr]) -> String {
        let mut s: String = String::new();
        s.push('(');
        s.push_str(&name);
        for expr in exprs {
            s.push(' ');
            s.push_str(&self.accept_expr(expr).unwrap());
        }
        s.push(')');
        s
    }
}

