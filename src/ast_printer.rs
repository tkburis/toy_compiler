use crate::expr::{Expr, ExprVisitor};
use crate::token;

pub struct AstPrinter;
impl ExprVisitor<String> for AstPrinter {
    fn visit_binary_expr(&self, left: &Expr, operator: &token::Token, right: &Expr) -> String {
        self.parenthesize(operator.lexeme.to_owned(), &[left, right])
    }
    fn visit_grouping_expr(&self, expression: &Expr) -> String {
        self.parenthesize("group".to_owned(), &[expression])
    }
    fn visit_literal_expr(&self, value: &Option<token::Literal>) -> String {
        match value {
            None => "nil".to_owned(),
            Some(x) => x.to_string(),
        }
    }
    fn visit_unary_expr(&self, operator: &token::Token, right: &Expr) -> String {
        self.parenthesize(operator.lexeme.to_owned(), &[right])
    }
}

impl AstPrinter {
    pub fn print(&self, expr: &Expr) -> String {
        self.accept(expr)
    }
    fn parenthesize(&self, name: String, exprs: &[&Expr]) -> String {
        let mut s: String = String::new();
        s.push('(');
        s.push_str(&name);
        for expr in exprs {
            s.push(' ');
            s.push_str(&self.accept(expr));
        }
        s.push(')');
        s
    }
}

