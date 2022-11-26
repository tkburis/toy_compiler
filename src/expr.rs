use crate::token;

pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: token::Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: token::Literal,
    },
    Unary {
        operator: token::Token,
        right: Box<Expr>
    },
}

pub trait ExprVisitor<T, E> {
    fn accept(&self, expr: &Expr) -> Result<T, E> {
        match expr {
            Expr::Binary { left, operator, right } => {
                self.visit_binary_expr(left, operator, right)
            },
            Expr::Grouping { expression } => {
                self.visit_grouping_expr(expression)
            },
            Expr::Literal { value } => {
                self.visit_literal_expr(value)
            },
            Expr::Unary { operator, right } => {
                self.visit_unary_expr(operator, right)
            }
        }
    }

    fn visit_binary_expr(&self, left: &Expr, operator: &token::Token, right: &Expr) -> Result<T, E>;
    fn visit_grouping_expr(&self, expression: &Expr) -> Result<T, E>;
    fn visit_literal_expr(&self, value: &token::Literal) -> Result<T, E>;
    fn visit_unary_expr(&self, operator: &token::Token, right: &Expr) -> Result<T, E>;
}

