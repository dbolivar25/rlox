use crate::ast::*;
use crate::environment::*;
use crate::lexer::*;
use crate::parser::*;
use crate::token::*;
use crate::value::*;
use crate::visitor::*;

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Interpreter {
    m_environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let global_env = Environment::new();

        global_env.borrow_mut().define(
            "clock".into(),
            Value::Callable(Callable::NativeFunction(
                None,
                0,
                Box::new(|_| {
                    Value::Number(
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs_f64(),
                    )
                }),
            )),
        );

        global_env.borrow_mut().define(
            "sleep_secs".into(),
            Value::Callable(Callable::NativeFunction(
                None,
                1,
                Box::new(|args| {
                    std::thread::sleep(std::time::Duration::from_secs_f64(
                        args[0].as_number().unwrap(),
                    ));
                    Value::Nil
                }),
            )),
        );

        global_env.borrow_mut().define(
            "sleep_millis".into(),
            Value::Callable(Callable::NativeFunction(
                None,
                1,
                Box::new(|args| {
                    std::thread::sleep(std::time::Duration::from_millis(
                        args[0].as_number().unwrap() as u64,
                    ));
                    Value::Nil
                }),
            )),
        );

        global_env.borrow_mut().define(
            "print".into(),
            Value::Callable(Callable::NativeFunction(
                None,
                1,
                Box::new(|args| {
                    print!("{}", args[0]);
                    Value::Nil
                }),
            )),
        );

        global_env.borrow_mut().define(
            "println".into(),
            Value::Callable(Callable::NativeFunction(
                None,
                1,
                Box::new(|args| {
                    println!("{}", args[0]);
                    Value::Nil
                }),
            )),
        );

        global_env.borrow_mut().define(
            "read_line".into(),
            Value::Callable(Callable::NativeFunction(
                None,
                0,
                Box::new(|_| {
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).unwrap();
                    Value::String(input.trim_end().into())
                }),
            )),
        );

        global_env.borrow_mut().define(
            "dbg".into(),
            Value::Callable(Callable::NativeFunction(
                None,
                1,
                Box::new(|args| {
                    dbg!(&args);
                    println!("value = {:#?}", args[0]);
                    Value::Nil
                }),
            )),
        );

        global_env.borrow_mut().define(
            "test0".into(),
            Value::Callable(Callable::NativeFunction(
                None,
                0,
                Box::new(|_| {
                    println!("testing123 from native print function");
                    Value::Nil
                }),
            )),
        );

        global_env.borrow_mut().define(
            "test2".into(),
            Value::Callable(Callable::Function(
                Some(global_env.clone()),
                vec![],
                0,
                Box::new(Stmt::new_block(vec![
                    Stmt::new_expression(Expr::new_call(
                        Box::new(Expr::new_variable(Token::new_token(
                            TokenType::Identifier("println".into()),
                            0,
                            0,
                            0,
                        ))),
                        Token::new_token(TokenType::LeftParen, 0, 0, 0),
                        vec![Expr::new_literal(Token::new_token(
                            TokenType::String("testing123 from rlox print function".into()),
                            0,
                            0,
                            0,
                        ))],
                    )),
                    Stmt::new_return(
                        Token::new_token(TokenType::Nil, 0, 0, 0),
                        Some(Expr::new_literal(Token::new_token(
                            TokenType::String("HELLO RETURN".into()),
                            0,
                            0,
                            0,
                        ))),
                    ),
                ])),
            )),
        );

        Interpreter {
            m_environment: global_env,
        }
    }

    pub fn interpret(&mut self, input: String) {
        match Lexer::new(&input).tokenize() {
            Ok(tokens) => match Parser::new(tokens).parse() {
                Ok(stmts) => {
                    for stmt in stmts {
                        let mut visitor = StmtEvaluator::new(&self.m_environment);
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
