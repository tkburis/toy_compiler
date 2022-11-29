use crate::expr::{Expr, ExprVisitor};
use crate::token;

pub struct AstPrinter;
impl ExprVisitor<String, ()> for AstPrinter {
    fn visit_binary_expr(&mut self, left: &Expr, operator: &token::Token, right: &Expr) -> Result<String, ()> {
        Ok(self.parenthesize(operator.lexeme.to_owned(), &[left, right]))
    }
    fn visit_grouping_expr(&mut self, expression: &Expr) -> Result<String, ()> {
        Ok(self.parenthesize("group".to_owned(), &[expression]))
    }
    fn visit_literal_expr(&mut self, value: &token::Literal) -> Result<String, ()> {
        Ok(value.to_string())
    }
    fn visit_unary_expr(&mut self, operator: &token::Token, right: &Expr) -> Result<String, ()> {
        Ok(self.parenthesize(operator.lexeme.to_owned(), &[right]))
    }
    fn visit_variable_expr(&mut self, name: &token::Token) -> Result<String, ()> {
        Ok(name.to_string())
    }
    fn visit_assign_expr(&mut self, name: &token::Token, value: &Expr) -> Result<String, ()> {
        Ok(self.parenthesize(name.lexeme.clone(), &[value]))
    }
}

impl AstPrinter {
    pub fn print(&mut self, expr: &Expr) -> String {
        self.accept_expr(expr).unwrap()
    }

    fn parenthesize(&mut self, name: String, exprs: &[&Expr]) -> String {
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

