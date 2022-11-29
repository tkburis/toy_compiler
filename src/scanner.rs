use crate::token::{Token, TokenType, Literal};
use crate::error::Error;

use std::collections::HashMap;

use lazy_static::lazy_static;

lazy_static! {
    static ref KEYWORDS: HashMap<String, TokenType> = {
        let mut m = HashMap::new();
        m.insert("and".to_owned(), TokenType::And);
        m.insert("class".to_owned(), TokenType::Class);
        m.insert("else".to_owned(), TokenType::Else);
        m.insert("false".to_owned(), TokenType::False);
        m.insert("for".to_owned(), TokenType::For);
        m.insert("fun".to_owned(), TokenType::Fun);
        m.insert("if".to_owned(), TokenType::If);
        m.insert("nil".to_owned(), TokenType::Nil);
        m.insert("or".to_owned(), TokenType::Or);
        m.insert("print".to_owned(), TokenType::Print);
        m.insert("return".to_owned(), TokenType::Return);
        m.insert("super".to_owned(), TokenType::Super);
        m.insert("this".to_owned(), TokenType::This);
        m.insert("true".to_owned(), TokenType::True);
        m.insert("var".to_owned(), TokenType::Var);
        m.insert("while".to_owned(), TokenType::While);
        m
    };
}

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,  // point to the start of the current token
    current: usize,  // point to the *next* character to be scanned
    line: usize,
    had_error: bool,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            had_error: false,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, Error> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        self.tokens.push(Token::new(TokenType::Eof, "", Literal::Nil, self.line));
        match self.had_error {
            true => Err(Error::ScanError),
            false => Ok(self.tokens.to_owned()),
        }
    }

    fn scan_token(&mut self) {
        let c: char = self.advance();
        match c {
            // 1-character tokens
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),

            // 2-character tokens
            '!' => {
                let t = if self.match_next('=') { TokenType::BangEqual } else { TokenType::Bang };
                self.add_token(t);
            },
            '=' => {
                let t = if self.match_next('=') { TokenType::EqualEqual } else { TokenType::Equal };
                self.add_token(t);
            },
            '<' => {
                let t = if self.match_next('=') { TokenType::LessEqual } else { TokenType::Less };
                self.add_token(t);
            },
            '>' => {
                let t = if self.match_next('=') { TokenType::GreaterEqual } else { TokenType::Greater };
                self.add_token(t);
            },
            '/' => {
                if self.match_next('/') {
                    // `//` style comments
                    // keep consuming until EOL
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else if self.match_next('*') {
                    // `/* ... */` style comments
                    while self.peek() != '*' && self.peek_next() != '/' && !self.is_at_end() {
                        if self.peek() == '\n' { self.line += 1; }
                        self.advance();
                    }

                    // consume `*` then `/`
                    self.advance();
                    self.advance();
                } else {
                    self.add_token(TokenType::Slash);
                }
            }

            // ignore
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,

            // literals and identifier
            '"' => self.string(),
            '0'..='9' => self.number(),
            'a'..='z' | 'A'..='Z' | '_' => self.identifier(),

            _ => self.error("Unexpected character"),
        };
    }

    fn error(&mut self, message: &str) {
        crate::error_line(self.line, message);
        self.had_error = true;
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    // TODO: refactor to make more Rust-ic by returning Option<char> instead
    // Return the current character and increment current pointer.
    fn advance(&mut self) -> char {
        if !self.is_at_end() { self.current += 1; }
        self.source.chars().nth(self.current - 1).unwrap()
    }

    // Return whether or not next character is `expected`. If so, consume it.
    fn match_next(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.chars().nth(self.current).unwrap() != expected {
            return false;
        }
        self.current += 1;
        true
    }

    // TODO: refactor to make more Rust-ic by returning Option<char> instead
    // Return next character (the one pointed at by `current`).
    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source.chars().nth(self.current).unwrap()
        }
    }

    // TODO: refactor to make more Rust-ic by returning Option<char> instead
    // Return character after next.
    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source.chars().nth(self.current+1).unwrap()
        }
    }

    // Process string.
    fn string(&mut self) {
        // Keep consuming until `"`.
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            self.error("Unterminated string");
        } else {
            self.advance();  // closing `"`
            let s: Literal = Literal::String_(self.source[self.start+1..self.current-1].to_owned());
            self.add_full_token(TokenType::String_, s);
        }
    }

    // Process number.
    fn number(&mut self) {
        // Keep consuming digits.
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        // Consume decimal point only if the character after is a digit.
        // `123.` will give Number(123), Dot.
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();  // consume `.`
        }

        // Consume the fractional part.
        while self.peek().is_ascii_digit() {
            self.advance();
        }
        let s: Literal = Literal::Number(self.source[self.start..self.current].parse().unwrap());
        self.add_full_token(TokenType::Number, s)
    }

    // Process identifier.
    fn identifier(&mut self) {
        // Allow alphanumeric and `_` in identifier.
        while self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        let s = &self.source[self.start..self.current];

        // Check if `s` is a keyword. If so, add that; otherwise, add `TokenType::Identifier`.
        let type_ = KEYWORDS.get(s).unwrap_or(&TokenType::Identifier).to_owned();
        self.add_token(type_);
    }

    // Add a non-literal token.
    fn add_token(&mut self, type_: TokenType) {
        self.add_full_token(type_, Literal::Nil);
    }

    // Add a token with a literal.
    fn add_full_token(&mut self, type_: TokenType, literal: Literal) {
        let lexeme = &self.source[self.start..self.current];
        let token = Token::new(type_, lexeme, literal, self.line);
        self.tokens.push(token);
    }
}

