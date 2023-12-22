use crate::lexer::*;
use crate::parser::*;
use crate::visitor::*;

#[derive(Debug)]
pub struct Interpreter;

impl Interpreter {
    pub fn interpret(input: String) {
        match Lexer::new(&input).tokenize() {
            Ok(tokens) => match Parser::new(tokens).parse() {
                Ok(expr) => {
                    let mut visitor = Evaluator::new();
                    expr.accept(&mut visitor);
                    println!("   {:?}", visitor.get_result());
                }
                Err(err) => {
                    println!("Parser produced {} errors:", err.len());
                    err.iter().for_each(|err| println!("    ERROR: {:?}", &err));
                }
            },
            Err(err) => {
                println!("Lexer produced {} errors:", err.len());
                err.iter().for_each(|err| println!("    ERROR: {:?}", &err));
            }
        }
    }
}
