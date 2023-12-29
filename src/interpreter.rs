use crate::lexer::*;
use crate::parser::*;
use crate::visitor::*;

#[derive(Debug)]
pub struct Interpreter;

impl Interpreter {
    pub fn interpret(input: String) {
        match Lexer::new(&input).tokenize() {
            Ok(tokens) => match Parser::new(tokens).parse() {
                Ok(stmts) => {
                    for stmt in stmts.iter() {
                        let mut visitor = StmtEvaluator::new();
                        stmt.accept(&mut visitor);

                        match visitor.get_result() {
                            Ok(_) => {}
                            Err(err) => {
                                println!(
                                    "Runtime produced {} {}:",
                                    err.len(),
                                    if err.len() == 1 { "error" } else { "errors" }
                                );
                                err.iter().for_each(|err| println!("    ERROR: {}", &err));
                            }
                        }
                    }
                }
                Err(err) => {
                    println!(
                        "Parser produced {} {}:",
                        err.len(),
                        if err.len() == 1 { "error" } else { "errors" },
                    );
                    err.iter().for_each(|err| println!("    ERROR: {}", &err));
                }
            },
            Err(err) => {
                println!(
                    "Lexer produced {} {}:",
                    err.len(),
                    if err.len() == 1 { "error" } else { "errors" },
                );
                err.iter().for_each(|err| println!("    ERROR: {}", &err));
            }
        }
    }
}
