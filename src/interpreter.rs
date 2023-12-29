use crate::environment::*;
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
                    let mut env: Option<Environment> = None;
                    for stmt in stmts.iter() {
                        let mut visitor =
                            StmtEvaluator::new(env.take().unwrap_or(Environment::new()));
                        stmt.accept(&mut visitor);

                        match visitor.get_result() {
                            Ok(result_env) => {
                                env = Some(result_env);
                            }
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
