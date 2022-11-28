use std::fmt;
use std::convert::From;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen, RightParen, LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus, Semicolon, Slash, Star,

    // One or two character tokens.
    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,

    // Literals.
    Identifier, String_, Number,

    // Keywords.
    And, Class, Else, False, Fun, For, If, Nil, Or,
    Print, Return, Super, This, True, Var, While,

    Eof,
}

// Literal represents `front-end` values that have been manually entered by user.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(f64),
    String_(String),
    Bool(bool),
    Nil,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: String = match self {
            Literal::Number(x) => x.to_string(),
            Literal::String_(x) => x.to_owned(),
            Literal::Bool(x) => x.to_string(),
            Literal::Nil => "nil".to_owned(),
        };
        write!(f, "{}", s)
    }
}

// Value represents values of evaluated expressions within the interpreter.
#[derive(PartialEq, Clone)]
pub enum Value {
    Number(f64),
    String_(String),
    Bool(bool),
    Nil,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: String = match self {
            Value::Number(x) => x.to_string(),
            Value::String_(x) => x.to_owned(),
            Value::Bool(x) => x.to_string(),
            Value::Nil => "nil".to_owned(),
        };
        write!(f, "{}", s)
    }
}

impl From<Literal> for Value {
    fn from(literal: Literal) -> Self {
        match literal {
            Literal::Number(x) => Self::Number(x),
            Literal::String_(x) => Self::String_(x),
            Literal::Bool(x) => Self::Bool(x),
            Literal::Nil => Self::Nil,
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub type_: TokenType,
    pub lexeme: String,
    pub literal: Literal,
    pub line: usize,
}

impl Token {
    pub fn new(type_: TokenType,
               lexeme: &str,
               literal: Literal,
               line: usize) -> Self {
        Self {
            type_,
            lexeme: lexeme.to_owned(),
            literal,
            line,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} {} {:?} L{}", self.type_, self.lexeme, self.literal, self.line)
    }
}

