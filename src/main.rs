mod scanner;
mod token;

use crate::scanner::Scanner;
use crate::token::Token;

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
    let tokens: Vec<Token> = scanner.scan_tokens();

    for token in tokens {
        println!("{}", token);
    }
    match scanner.had_error {
        true => Err(()),
        false => Ok(()),
    }
}

fn error(line: usize, message: &str, had_error: &mut bool) {
    report(line, "", message, had_error);
}

fn report(line: usize, loc: &str, message: &str, had_error: &mut bool) {
    eprintln!("[line {line}] Error{loc}: {message}");
    *had_error = true;
}

