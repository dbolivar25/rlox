use itertools::Itertools;
use std::fmt::Debug;

use crate::token::Token;
use crate::visitor::*;

use paste::paste;

macro_rules! define_ast {
    ($name:ident, $visitor:ident, $($variant_lowercase:ident: $variant:ident($($field_name:ident: $field:ty),*)),*,) => {
        #[derive(Clone)]
        pub enum $name {
            $(
                $variant { $($field_name: $field),* },
            )*
        }

        impl $name {
            pub fn accept(&self, visitor: &mut impl $visitor) {
                match self {
                    $(
                        $name::$variant { $($field_name),* } => {
                            paste! {
                                visitor.[<visit_ $variant_lowercase>]($($field_name),*);
                            }
                        }
                    )*
                }
            }

            $(
                paste! {
                    pub fn [<new_ $variant_lowercase>]($($field_name: $field),*) -> $name {
                        return $name::$variant { $($field_name),* };
                    }
                }
            )*
        }

    }
}

define_ast!(
    Expr,
    ExprVisitor,
    binary: Binary(m_left: Box<Expr>, m_token: Token, m_right: Box<Expr>),
    grouping: Grouping(m_expression: Box<Expr>),
    literal: Literal(m_token: Token),
    unary: Unary(m_token: Token, m_expression: Box<Expr>),
    variable: Variable(m_token: Token),
    assign: Assign(m_token: Token, m_value: Box<Expr>),
    logical: Logical(m_left: Box<Expr>, m_token: Token, m_right: Box<Expr>),
    call: Call(m_callee: Box<Expr>, m_paren: Token, m_arguments: Vec<Expr>),
);

impl Debug for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Binary {
                m_left,
                m_token,
                m_right,
            } => write!(f, "{:?} {:?} {:?}", m_left, m_token, m_right),
            Expr::Grouping { m_expression } => write!(f, "{:?}", m_expression),
            Expr::Literal { m_token } => write!(f, "{}", m_token),
            Expr::Unary {
                m_token,
                m_expression,
            } => write!(f, "{:?} {:?}", m_token, m_expression),
            Expr::Variable { m_token } => write!(f, "{}", m_token),
            Expr::Assign { m_token, m_value } => write!(f, "{} = {:?}", m_token, m_value),
            Expr::Logical {
                m_left,
                m_token,
                m_right,
            } => write!(f, "{:?} {:?} {:?}", m_left, m_token, m_right),
            Expr::Call {
                m_callee,
                m_paren: _,
                m_arguments,
            } => write!(
                f,
                "{:?}({})",
                m_callee,
                m_arguments
                    .iter()
                    .map(|e| format!("{:?}", e))
                    .join(", ")
            ),
        }
    }
}

define_ast!(
    Stmt,
    StmtVisitor,
    block: Block(m_statements: Vec<Stmt>),
    expression: Expression(m_expression: Expr),
    var: Var(m_name: Token, m_initializer: Option<Expr>),
    r#while: While(m_condition: Expr, m_body: Box<Stmt>),
    r#if: If(m_condition: Expr, m_then_branch: Box<Stmt>, m_else_branch: Option<Box<Stmt>>),
    function: Function(m_name: Token, m_params: Vec<Token>, m_body: Vec<Stmt>),
    r#return: Return(m_keyword: Token, m_value: Option<Expr>),
    // class: Class(m_name: Token, m_methods: Vec<Stmt>),
);

impl Debug for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stmt::Block { m_statements } => {
                let mut s = String::new();
                for (i, stmt) in m_statements.iter().enumerate() {
                    if i == 0 && m_statements.len() == i + 1 {
                        s.push_str(&format!("{:?}", stmt));
                    } else if i == 0 {
                        s.push_str(&format!("{:?}", stmt));
                    } else if m_statements.len() == i + 1 {
                        s.push_str(&format!("{:?}", stmt));
                    } else {
                        s.push_str(&format!("{:?}", stmt));
                    }
                }

                write!(f, "{{ {}}} ", s)
            }
            Stmt::Expression { m_expression } => write!(f, "{:?}; ", m_expression),
            Stmt::Var {
                m_name,
                m_initializer,
            } => match m_initializer {
                Some(expr) => write!(f, "let {} = {:?}; ", m_name, expr),
                None => write!(f, "let {}; ", m_name),
            },
            Stmt::While {
                m_condition,
                m_body,
            } => write!(f, "while {:?} {:?} ", m_condition, m_body),
            Stmt::If {
                m_condition,
                m_then_branch,
                m_else_branch,
            } => match m_else_branch {
                Some(else_branch) => write!(
                    f,
                    "if {:?} {:?} else {:?} ",
                    m_condition, m_then_branch, else_branch
                ),
                None => write!(f, "if {:?} {:?} ", m_condition, m_then_branch),
            },
            Stmt::Function {
                m_name,
                m_params,
                m_body,
            } => {
                let mut s = String::new();
                for (i, param) in m_params.iter().enumerate() {
                    if i == 0 {
                        s.push_str(&format!("{:?}", param));
                    } else {
                        s.push_str(&format!(", {:?}", param));
                    }
                }

                write!(
                    f,
                    "fun {}({}) {{ {}}} ",
                    m_name,
                    s,
                    m_body
                        .iter()
                        .enumerate()
                        .map(|(i, stmt)| if i == 0 && m_body.len() == i + 1 {
                            format!("{:?}", stmt)
                        } else if i == 0 {
                            format!("{:?}", stmt)
                        } else if m_body.len() == i + 1 {
                            format!("{:?}", stmt)
                        } else {
                            format!("{:?}", stmt)
                        })
                        .join("")
                )
            }
            Stmt::Return {
                m_keyword: _,
                m_value,
            } => match m_value {
                Some(expr) => write!(f, "return {:?}; ", expr),
                None => write!(f, "return; "),
            },
            // Stmt::Class { m_name, m_methods } => {
            //     let mut s = String::new();
            //     for method in m_methods {
            //         s.push_str(&format!("{:?}\n", method));
            //     }
            //     write!(f, "(class {:?} {:?})", m_name, s)
            // }
        }
    }
}
