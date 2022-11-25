mod scanner;
mod token;
mod expr;
mod ast_printer;

use crate::scanner::Scanner;
// use crate::token::Token;
// use crate::expr::Expr;  // test

use std::env;
use std::io::{self, Write};
use std::process;
use std::fs;

fn main() {
    // let expr: Expr = Expr::Binary {
    //     left: Box::new(Expr::Unary {
    //         operator: Token {
    //             type_: token::TokenType::Minus,
    //             lexeme: "-".to_owned(),
    //             literal: None,
    //             line: 1,
    //         },
    //         right: Box::new(Expr::Literal {
    //             value: Some(token::Literal::Number(123.0)),
    //         }),
    //     }),
    //     operator: Token {
    //         type_: token::TokenType::Star,
    //         lexeme: "*".to_owned(),
    //         literal: None,
    //         line: 1,
    //     },
    //     right: Box::new(Expr::Grouping {
    //         expression: Box::new(Expr::Literal {
    //             value: Some(token::Literal::Number(45.67)),
    //         })
    //     })
    // };
    // let printer = ast_printer::AstPrinter;
    // println!("{}", printer.print(&expr));
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
        run(&line);
    }
}

fn run(source: &str) -> Result<(), ()> {
    let mut scanner = Scanner::new(source.to_owned());
    let (tokens, res) = scanner.scan_tokens();

    for token in tokens {
        println!("{}", token);
    }

    res
}

fn error(line: usize, message: &str, had_error: &mut bool) {
    report(line, "", message, had_error);
}

fn report(line: usize, loc: &str, message: &str, had_error: &mut bool) {
    eprintln!("[line {line}] Error{loc}: {message}");
    *had_error = true;
}

