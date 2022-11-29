use crate::token::Token;

pub enum Error {
    ScanError,
    ParseError,
    // RuntimeError,
    RuntimeError {
        token: Token,
        message: String,
    },
}
