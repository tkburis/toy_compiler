use crate::token::{Token, TokenType::{*, self}, Literal};
use std::collections::HashMap;
use lazy_static::lazy_static;

lazy_static! {
    static ref KEYWORDS: HashMap<String, TokenType> = {
        let mut m = HashMap::new();
        m.insert("and".to_owned(), And);
        m.insert("class".to_owned(), Class);
        m.insert("else".to_owned(), Else);
        m.insert("false".to_owned(), False);
        m.insert("for".to_owned(), For);
        m.insert("fun".to_owned(), Fun);
        m.insert("if".to_owned(), If);
        m.insert("nil".to_owned(), Nil);
        m.insert("or".to_owned(), Or);
        m.insert("print".to_owned(), Print);
        m.insert("return".to_owned(), Return);
        m.insert("super".to_owned(), Super);
        m.insert("this".to_owned(), This);
        m.insert("true".to_owned(), True);
        m.insert("var".to_owned(), Var);
        m.insert("while".to_owned(), While);
        m
    };
}

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
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

    pub fn scan_tokens(&mut self) -> (Vec<Token>, Result<(), ()>) {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        self.tokens.push(Token::new(Eof, "", None, self.line));
        match self.had_error {
            true => (self.tokens.clone(), Err(())),
            false => (self.tokens.clone(), Ok(())),
        }
    }

    fn scan_token(&mut self) {
        let c: char = self.advance();
        match c {
            '(' => self.add_token(LeftParen),
            ')' => self.add_token(RightParen),
            '{' => self.add_token(LeftBrace),
            '}' => self.add_token(RightBrace),
            ',' => self.add_token(Comma),
            '.' => self.add_token(Dot),
            '-' => self.add_token(Minus),
            '+' => self.add_token(Plus),
            ';' => self.add_token(Semicolon),
            '*' => self.add_token(Star),
            '!' => {
                let t = if self.match_next('=') { BangEqual } else { Bang };
                self.add_token(t);
            },
            '=' => {
                let t = if self.match_next('=') { EqualEqual } else { Equal };
                self.add_token(t);
            },
            '<' => {
                let t = if self.match_next('=') { LessEqual } else { Less };
                self.add_token(t);
            },
            '>' => {
                let t = if self.match_next('=') { GreaterEqual } else { Greater };
                self.add_token(t);
            },
            '/' => {
                if self.match_next('/') {
                    // keep consuming until EOL
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else if self.match_next('*') {
                    while self.peek() != '*' && self.peek_next() != '/' && !self.is_at_end() {
                        if self.peek() == '\n' { self.line += 1; }
                        self.advance();
                    }
                    // consume * then /
                    if !self.is_at_end() { self.advance(); }
                    if !self.is_at_end() { self.advance(); }
                } else {
                    self.add_token(Slash);
                }
            }
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,
            '"' => self.string(),
            '0'..='9' => self.number(),
            'a'..='z' | 'A'..='Z' | '_' => self.identifier(),
            _ => crate::error(self.line, "Unexpected character", &mut self.had_error)
        };
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source.chars().nth(self.current - 1).unwrap()
    }

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

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source.chars().nth(self.current).unwrap()
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source.chars().nth(self.current+1).unwrap()
        }
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_at_end() {
            crate::error(self.line, "Unterminated string", &mut self.had_error);
        } else {
            self.advance();  // closing "
            let s: Literal = Literal::String_(self.source[self.start+1..self.current-1].to_owned());
            self.add_full_token(String_, Some(s));
        }
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            // consume .
            self.advance();
        }
        while self.peek().is_ascii_digit() {
            self.advance();
        }
        let s: Literal = Literal::Number(self.source[self.start..self.current].parse().unwrap());
        self.add_full_token(Number, Some(s))
    }

    fn identifier(&mut self) {
        while self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
            self.advance();
        }
        let s = &self.source[self.start..self.current];
        let type_ = KEYWORDS.get(s).unwrap_or(&Identifier).to_owned();
        self.add_token(type_);
    }

    fn add_token(&mut self, type_: TokenType) {
        self.add_full_token(type_, None);
    }

    fn add_full_token(&mut self, type_: TokenType, literal: Option<Literal>) {
        let token = Token::new(type_, &self.source[self.start..self.current], literal, self.line);
        self.tokens.push(token);
    }
}

