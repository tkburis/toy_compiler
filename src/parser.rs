use crate::token::{Token, TokenType, Literal};
use crate::expr::Expr;

struct ParseError;

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

    // Return an empty error `Err(())` if error happened, otherwise Ok(Expr).
    pub fn parse(&mut self) -> Result<Expr, ()> {
        match self.expression() {
            Ok(expr) => Ok(expr),
            Err(_) => Err(()),
        }
    }

    // By allowing rules to only match with other rules `below` it, precedence can be controlled.
    // expression -> equality
    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.equality()
    }

    // equality -> comparison ( ( "!=" | "==" ) comparison )*
    fn equality(&mut self) -> Result<Expr, ParseError> {
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
    fn comparison(&mut self) -> Result<Expr, ParseError> {
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
    fn term(&mut self) -> Result<Expr, ParseError> {
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
    fn factor(&mut self) -> Result<Expr, ParseError> {
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
    fn unary(&mut self) -> Result<Expr, ParseError> {
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
    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.match_next(&[TokenType::False]) {
            Ok(Expr::Literal { value: Some(Literal::False) })

        } else if self.match_next(&[TokenType::True]) {
            Ok(Expr::Literal { value: Some(Literal::True) })

        } else if self.match_next(&[TokenType::Nil]) {
            Ok(Expr::Literal { value: Some(Literal::Nil) })

        } else if self.match_next(&[TokenType::Number, TokenType::String_]) {
            Ok(Expr::Literal { value: self.previous().to_owned().literal })

        } else if self.match_next(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            _ = self.match_err(&TokenType::RightParen, "Expect `)` after expression.")?;
            Ok(Expr::Grouping { expression: Box::new(expr) })

        } else {
            // Convert Result<(), ParseError> -> Err(ParseError).
            Err(self.error(self.peek(), "Expected expression.").unwrap_err())
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

    // Check if next token is `token_type`; otherwise, throw an error.
    fn match_err(&mut self, token_type: &TokenType, message: &str) -> Result<Token, ParseError> {
        if self.check(token_type) {
            Ok(self.advance().to_owned())
        } else {
            // Convert Result<(), ParseError> -> Err(ParseError).
            Err(self.error(self.peek(), message).unwrap_err())
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

    // Report error to main function.
    // Also, return ParseError object to be bubbled up.
    fn error(&self, token: &Token, message: &str) -> Result<(), ParseError> {
        crate::error_token(token, message);
        Err(ParseError)
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

