mod ast;
mod interpreter;
mod lexer;
mod value;
mod parser;
mod token;
mod visitor;
mod environment;

use interpreter::*;

use anyhow::Result;
use clap::Parser;
use std::{fs, io::Write};

// argument parser
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, default_value = None)]
    file: Option<String>,
}

#[derive(Debug)]
struct App;

impl App {
    pub fn run_file_interpreter(file: String) -> Result<()> {
        let file_string = fs::read_to_string(file)?;

        Interpreter::new().interpret(file_string);

        Ok(())
    }

    pub fn run_repl_interpreter() -> Result<()> {
        let mut input = String::new();
        let mut interpreter = Interpreter::new();

        loop {
            print!("\n|>  ");
            std::io::stdout().flush()?;
            std::io::stdin().read_line(&mut input)?;

            match input.trim() {
                "" | "q" | "quit" => break,
                input => interpreter.interpret(input.into()),
            }

            input.clear();
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.file {
        Some(file) => App::run_file_interpreter(file)?,
        None => App::run_repl_interpreter()?,
    };

    Ok(())
}
