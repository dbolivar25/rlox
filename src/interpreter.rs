use crate::environment::*;
use crate::lexer::*;
use crate::parser::*;
use crate::visitor::*;
use crate::value::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Interpreter {
    m_environment: Option<Environment>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let global_env = Environment::new();
        Interpreter {
            m_environment: Some(global_env),
        }
    }

    pub fn interpret(&mut self ,input: String) {
        match Lexer::new(&input).tokenize() {
            Ok(tokens) => match Parser::new(tokens).parse() {
                Ok(stmts) => {
                    for stmt in stmts.iter() {
                        let mut visitor =
                            StmtEvaluator::new(self.m_environment.take());
                        stmt.accept(&mut visitor);

                        match visitor.get_result() {
                            Ok(result_env) => {
                                self.m_environment = result_env;
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
