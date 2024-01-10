use crate::token::Token;
use crate::visitor::*;

use paste::paste;

macro_rules! define_ast {
    ($name:ident, $visitor:ident, $($variant_lowercase:ident: $variant:ident($($field_name:ident: $field:ty),*)),*,) => {
        #[derive(Debug, Clone)]
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

define_ast!(
    Stmt,
    StmtVisitor,
    block: Block(m_statements: Vec<Stmt>),
    expression: Expression(m_expression: Expr),
    print: Print(m_expression: Expr),
    var: Var(m_name: Token, m_initializer: Option<Expr>),
    r#while: While(m_condition: Expr, m_body: Box<Stmt>),
    r#if: If(m_condition: Expr, m_then_branch: Box<Stmt>, m_else_branch: Option<Box<Stmt>>),
    function: Function(m_name: Token, m_params: Vec<Token>, m_body: Vec<Stmt>),
    // r#return: Return(m_keyword: Token, m_value: Option<Expr>),
    // class: Class(m_name: Token, m_methods: Vec<Stmt>),
);
