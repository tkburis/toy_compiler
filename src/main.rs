mod scanner;
mod token;
mod expr;
mod ast_printer;
mod parser;

use crate::scanner::Scanner;
use crate::parser::Parser;

use std::env;
use std::io::{self, Write};
use std::process;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        eprintln!("Usage: cargo run [-- script]");
        process::exit(64);
    } else if args.len() == 2 {
        run_file(&args[1]);
    } else {
        run_prompt();
    }
}

fn run_file(file_path: &str) {
    let source = fs::read_to_string(file_path).expect("Failed to read file");
    if run(&source).is_err() { process::exit(65) };
}

fn run_prompt() {
    loop {
        print!("> ");
        io::stdout().flush().expect("Flush failed");  // to flush out "> "
        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .expect("Failed to read line");
        _ = run(&line);
    }
}

fn run(source: &str) -> Result<(), ()> {
    let mut scanner = Scanner::new(source.to_owned());
    let tokens: Vec<token::Token> = scanner.scan_tokens()?;

    // Only EOF token.
    if tokens.len() == 1 {
        return Ok(());
    }

    let mut parser = Parser::new(tokens);
    let expression: expr::Expr = parser.parse()?;

    let printer = ast_printer::AstPrinter;
    println!("{}", printer.print(&expression));

    Ok(())
}

fn error_line(line: usize, message: &str) {
    report(line, "", message);
}

fn error_token(token: &token::Token, message: &str) {
    if token.type_ == token::TokenType::Eof {
        report(token.line, " at end", message);
    } else {
        report(token.line, &format!(" at '{}'", token.lexeme)[..], message);
    }
}

fn report(line: usize, loc: &str, message: &str) {
    eprintln!("[line {line}] Error{loc}: {message}");
}

