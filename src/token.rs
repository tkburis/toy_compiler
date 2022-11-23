use std::fmt;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum Literal {
    Number(f64),
    String_(String),
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: String = match self {
            Literal::Number(x) => x.to_string(),
            Literal::String_(x) => x.to_owned(),
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone)]
pub struct Token {
    pub type_: TokenType,
    pub lexeme: String,
    pub literal: Option<Literal>,
    pub line: usize,
}

impl Token {
    pub fn new(type_: TokenType,
               lexeme: &str,
               literal: Option<Literal>,
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

