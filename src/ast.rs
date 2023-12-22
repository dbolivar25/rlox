use crate::token::Token;
use crate::visitor::Visitor;

use paste::paste;

macro_rules! define_ast {
    ($name:ident, $($variant_lowercase:ident: $variant:ident($($field_name:ident: $field:ty),*)),*) => {
        #[derive(Debug)]
        pub enum $name {
            $(
                $variant { $($field_name: $field),* },
            )*
        }

        impl $name {
            pub fn accept(&self, visitor: &mut impl Visitor) {
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
    binary: Binary(m_left: Box<Expr>, m_token: Token, m_right: Box<Expr>),
    grouping: Grouping(m_expression: Box<Expr>),
    literal: Literal(m_token: Token),
    unary: Unary(m_token: Token, m_expression: Box<Expr>)
);
