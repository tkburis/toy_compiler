use crate::token::{Token, TokenType, Literal};
use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::error::Error;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,  // point to the *next* token to be parsed
}


impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
        }
    }

    // Interface.
    // program -> declaration* EOF
    pub fn parse(&mut self) -> Result<Vec<Stmt>, Error> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            if let Some(x) = self.declaration_wrapper() {
                statements.push(x);
            }
        }
        Ok(statements)
    }

    // Convert `Result<Stmt, Error>` to `Option<Stmt>`, and call `synchronize()` if something went
    // wrong. This is to allow `parse()` to collect as many statements as possible into the AST by
    // omitting invalid statements (`None` variant).
    fn declaration_wrapper(&mut self) -> Option<Stmt> {
        let res = self.declaration();
        if res.is_err() {
            self.synchronize();
        }
        res.ok()
    }

    // Statements.

    // declaration -> var_declaration | statement
    fn declaration(&mut self) -> Result<Stmt, Error> {
        if self.match_next(&[TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    // var_declaration -> "var" identifier ( "=" expression )? ";"
    fn var_declaration(&mut self) -> Result<Stmt, Error> {
        let name = self.match_err(&TokenType::Identifier, "Expected variable name.")?;

        let initializer = match self.match_next(&[TokenType::Equal]) {
            true => Some(self.expression()?),
            false => None,
        };

        self.match_err(&TokenType::Semicolon, "Expected ';' after variable declaration.")?;
        Ok(Stmt::Var { name, initializer })
    }

    // statement -> for_statement
    //              | if_statement
    //              | print_statement
    //              | while_statement
    //              | block
    //              | expression_statement
    fn statement(&mut self) -> Result<Stmt, Error> {
        if self.match_next(&[TokenType::For]) {
            self.for_statement()

        } else if self.match_next(&[TokenType::If]) {
            self.if_statement()

        } else if self.match_next(&[TokenType::Print]) {
            self.print_statement()

        } else if self.match_next(&[TokenType::While]) {
            self.while_statement()

        } else if self.match_next(&[TokenType::LeftBrace]) {
            Ok(Stmt::Block { statements: self.block()? })

        } else {
            self.expression_statement()
        }
    }

    // `Desugar` the `for` statement into a `while` loop.
    // for_statement -> "for" "(" ( var_declaration | expression_statement | ";" ) expression? ";"
    // expression? ";" ")" statement
    fn for_statement(&mut self) -> Result<Stmt, Error> {
        self.match_err(&TokenType::LeftParen, "Expect `(` after `for`.")?;

        let initializer: Option<Stmt>;
        if self.match_next(&[TokenType::Semicolon]) {
            initializer = None;
        } else if self.match_next(&[TokenType::Var]) {
            initializer = Some(self.var_declaration()?);
        } else {
            initializer = Some(self.expression_statement()?);
        }
        // Note the `;` has already been consumed by either `var_declaration` or
        // `expression_statement` already.

        // TODO: better way to do this?
        let mut condition = Expr::Literal { value: Literal::Bool(true) };
        if !self.check(&TokenType::Semicolon) {
            condition = self.expression()?;
        }
        self.match_err(&TokenType::Semicolon, "Expected `;` after `for` condition.")?;

        let mut increment: Option<Expr> = None;
        if !self.check(&TokenType::RightParen) {
            increment = Some(self.expression()?);
        }
        self.match_err(&TokenType::RightParen, "Expected `)` after `for` clause.")?;

        // Now we convert it to:
        //  {
        //      `initializer`
        //      while (`condition`) {
        //          `body`
        //          `increment`
        //      }
        //  }

        let mut body = self.statement()?;
        if let Some(inc) = increment {
            // As the increment will be executed after every iteration, we add it to the body of
            // the loop.
            body = Stmt::Block {
                statements: vec![body, Stmt::Expression { expression: inc }]
            };
        }

        body = Stmt::While { condition, body: Box::new(body) };

        if let Some(init) = initializer {
            body = Stmt::Block {
                statements: vec![init, body]
            };
        }

        Ok(body)
    }

    fn if_statement(&mut self) -> Result<Stmt, Error> {
        self.match_err(&TokenType::LeftParen, "Expected `(` after `if`.")?;
        let condition = self.expression()?;
        self.match_err(&TokenType::RightParen, "Expected ')' after condition.")?;

        let then_branch = self.statement()?;

        // Note `else_branch` is greedily added, so it will be attached to the nearest `if`
        // statement.
        // if (first) if (second) something(); else something_else();
        // Here, `else` is attached to the `if` with the statement `second`.
        let else_branch = match self.match_next(&[TokenType::Else]) {
            true => Some(self.statement()?),
            false => None,
        };

        Ok(Stmt::If { condition,
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new)
        })
    }

    // print_statement -> "print" expression ";"
    fn print_statement(&mut self) -> Result<Stmt, Error> {
        let value = self.expression()?;
        self.match_err(&TokenType::Semicolon, "Expected `;` after value.")?;
        Ok(Stmt::Print { expression: value })
    }

    // while_statement -> "while" "(" expression ")" statement
    fn while_statement(&mut self) -> Result<Stmt, Error> {
        self.match_err(&TokenType::LeftParen, "Expected `(` after `while`.")?;
        let condition = self.expression()?;
        self.match_err(&TokenType::RightParen, "Expected ')' after condition.")?;

        let body = self.statement()?;

        Ok(Stmt::While { condition, body: Box::new(body) })
    }

    // block -> "{" declaration* "}"
    fn block(&mut self) -> Result<Vec<Stmt>, Error> {
        let mut statements = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.match_err(&TokenType::RightBrace, "Expected `}` after block.")?;
        Ok(statements)
    }

    // expression_statement -> expression ";"
    // These are for expressions with side effects such as function calls.
    fn expression_statement(&mut self) -> Result<Stmt, Error> {
        let expr = self.expression()?;
        self.match_err(&TokenType::Semicolon, "Expected `;` after expression.")?;
        Ok(Stmt::Expression { expression: expr })
    }

    // Expressions.

    // By allowing rules to only match with other rules `below` it, precedence can be controlled.
    // expression -> assignment
    fn expression(&mut self) -> Result<Expr, Error> {
        self.assignment()
    }

    // assignment -> (identifier "=" assignment) | logic_or
    fn assignment(&mut self) -> Result<Expr, Error> {
        // We let `self.equality()` collect the identifier.
        let expr = self.logic_or()?;

        if self.match_next(&[TokenType::Equal]) {
            let equals = self.previous().to_owned();
            let value = self.assignment()?;

            // Test if what is collected can be used as a variable.
            // Doing it this way allows identifiers like `Point(x+2, 0.0).y` since it itself is an
            // expression.
            if let Expr::Variable { name } = expr {
                return Ok(Expr::Assign { name, value: Box::new(value) });
            } else {
                // Note we don't bubble up error because we don't need to go into panic mode and
                // synchronize. We accept their mistake by reporting the error and move on.
                self.error(&equals, "Invalid assignment target.");
            }
        }

        Ok(expr)
    }

    // logic_or -> logic_and ("or" logic_and)*
    fn logic_or(&mut self) -> Result<Expr, Error> {
        let mut expr = self.logic_and()?;

        while self.match_next(&[TokenType::Or]) {
            let operator = self.previous().to_owned();
            let right = self.logic_and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right)
            };
        }

        Ok(expr)
    }

    // logic_and -> equality ("and" equality)*
    fn logic_and(&mut self) -> Result<Expr, Error> {
        let mut expr = self.equality()?;

        while self.match_next(&[TokenType::And]) {
            let operator = self.previous().to_owned();
            let right = self.equality()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right)
            };
        }

        Ok(expr)
    }

    // equality -> comparison ( ( "!=" | "==" ) comparison )*
    fn equality(&mut self) -> Result<Expr, Error> {
        let mut expr = self.comparison()?;

        while self.match_next(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().to_owned();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    // comparison -> term ( ( ">" | ">=" | "<" | "<=" ) term )*
    fn comparison(&mut self) -> Result<Expr, Error> {
        let mut expr = self.term()?;

        while self.match_next(&[TokenType::Greater, TokenType::GreaterEqual, TokenType::Less, TokenType::LessEqual]) {
            let operator = self.previous().to_owned();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    // term -> factor ( ( "-" | "+" ) factor )*
    fn term(&mut self) -> Result<Expr, Error> {
        let mut expr = self.factor()?;

        while self.match_next(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous().to_owned();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    // factor -> unary ( ( "/" | "*" ) unary )*
    fn factor(&mut self) -> Result<Expr, Error> {
        let mut expr = self.unary()?;

        while self.match_next(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous().to_owned();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    // unary -> ( ( "!" | "-" ) unary ) | primary
    fn unary(&mut self) -> Result<Expr, Error> {
        if self.match_next(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().to_owned();
            let right = self.unary()?;
            Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            })
        } else {
            self.primary()
        }
    }

    // primary -> literal | "(" expression ")"
    fn primary(&mut self) -> Result<Expr, Error> {
        if self.match_next(&[TokenType::False]) {
            Ok(Expr::Literal { value: Literal::Bool(false) })

        } else if self.match_next(&[TokenType::True]) {
            Ok(Expr::Literal { value: Literal::Bool(true) })

        } else if self.match_next(&[TokenType::Nil]) {
            Ok(Expr::Literal { value: Literal::Nil })

        } else if self.match_next(&[TokenType::Number, TokenType::String_]) {
            Ok(Expr::Literal { value: self.previous().to_owned().literal })

        } else if self.match_next(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.match_err(&TokenType::RightParen, "Expected `)` after expression.")?;
            Ok(Expr::Grouping { expression: Box::new(expr) })

        } else if self.match_next(&[TokenType::Identifier]) {
            Ok(Expr::Variable { name: self.previous().to_owned() })

        } else {
            Err(self.error(self.peek(), "Expected expression."))
        }
    }

    // Return `true` if one of `token_types` matches the next token type.
    fn match_next(&mut self, token_types: &[TokenType]) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    // Check if next token is `token_type`; otherwise, return an error.
    fn match_err(&mut self, token_type: &TokenType, message: &str) -> Result<Token, Error> {
        if self.check(token_type) {
            Ok(self.advance().to_owned())
        } else {
            Err(self.error(self.peek(), message))
        }
    }

    // Return whether next token is `token_type`.
    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().type_ == *token_type
        }
    }

    // Return current token.
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() { self.current += 1; }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        return self.peek().type_ == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap()
    }

    fn previous(&self) -> &Token {
        self.tokens.get(self.current - 1)
            .expect("Could not get previous token: at beginning of file")
    }

    // Report error to main function to be printed.
    // Also, return `Error::ParseError` variant to be bubbled up.
    fn error(&self, token: &Token, message: &str) -> Error {
        crate::error_token(token, message);
        Error::ParseError
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().type_ == TokenType::Semicolon {
                return;
            }

            match self.peek().type_ {
                TokenType::Class |
                TokenType::Fun |
                TokenType::Var |
                TokenType::For |
                TokenType::If |
                TokenType::While |
                TokenType::Print |
                TokenType::Return => { return; },

                _ => (),
            }

            self.advance();
        }
    }
}

