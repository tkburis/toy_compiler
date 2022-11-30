use crate::token;

pub enum Expr {
    // Assignment is an expression since it returns a value, so that expressions like `a = b = 2`
    // are possible.
    Assign {
        name: token::Token,
        value: Box<Expr>,
    },
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
    Logical {
        left: Box<Expr>,
        operator: token::Token,
        right: Box<Expr>,
    },
    Unary {
        operator: token::Token,
        right: Box<Expr>,
    },
    Variable {
        name: token::Token,
    },
}

pub trait ExprVisitor<T, E> {
    fn accept_expr(&mut self, expr: &Expr) -> Result<T, E> {
        match expr {
            Expr::Assign { name, value } => {
                self.visit_assign_expr(name, value)
            },
            Expr::Binary { left, operator, right } => {
                self.visit_binary_expr(left, operator, right)
            },
            Expr::Grouping { expression } => {
                self.visit_grouping_expr(expression)
            },
            Expr::Literal { value } => {
                self.visit_literal_expr(value)
            },
            Expr::Logical { left, operator, right } => {
                self.visit_logical_expr(left, operator, right)
            },
            Expr::Unary { operator, right } => {
                self.visit_unary_expr(operator, right)
            },
            Expr::Variable { name } => {
                self.visit_variable_expr(name)
            },
        }
    }

    fn visit_assign_expr(&mut self, name: &token::Token, value: &Expr) -> Result<T, E>;
    fn visit_binary_expr(&mut self, left: &Expr, operator: &token::Token, right: &Expr) -> Result<T, E>;
    fn visit_grouping_expr(&mut self, expression: &Expr) -> Result<T, E>;
    fn visit_literal_expr(&mut self, value: &token::Literal) -> Result<T, E>;
    fn visit_logical_expr(&mut self, left: &Expr, operator: &token::Token, right: &Expr) -> Result<T, E>;
    fn visit_unary_expr(&mut self, operator: &token::Token, right: &Expr) -> Result<T, E>;
    fn visit_variable_expr(&mut self, name: &token::Token) -> Result<T, E>;
}

