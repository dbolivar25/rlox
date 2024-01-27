use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while, take_while1},
    character::complete::{char, digit1, multispace0, multispace1},
    combinator::{cut, eof, map, map_res, opt, peek},
    error::{convert_error, VerboseError},
    multi::{many0, many_till},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};

use crate::ast_v2::*;
use crate::token_v2::*;

type Token = TokenType;

pub(crate) fn parse_identifier<'a>(
    input: &'a str,
) -> IResult<&'a str, Token, VerboseError<&'a str>> {
    pair(
        take_while1(|c: char| c.is_alphabetic() || c == '_'),
        take_while(|c: char| c.is_alphanumeric() || c == '_'),
    )(input)
    .map(|(input, (ident0, ident1))| {
        (
            input,
            Token::new_identifier(format!("{}{}", ident0, ident1).as_str()),
        )
    })
}

pub(crate) fn parse_string<'a>(input: &'a str) -> IResult<&'a str, Token, VerboseError<&'a str>> {
    delimited(char('\"'), take_until("\""), cut(char('\"')))(input)
        .map(|(input, string)| (input, Token::String(string.to_string())))
}

pub(crate) fn parse_number<'a>(input: &'a str) -> IResult<&'a str, Token, VerboseError<&'a str>> {
    map_res(
        pair(digit1, opt(preceded(char('.'), cut(digit1)))),
        |(int, dec): (&str, Option<&str>)| {
            let mut num = int.to_string();
            if let Some(dec) = dec {
                num.push('.');
                num.push_str(dec);
            }
            num.parse::<f64>()
        },
    )(input)
    .map(|(input, number)| (input, Token::Number(number)))
}

pub(crate) fn parse_anonymous_function<'a>(
    input: &'a str,
) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    map(
        tuple((
            preceded(multispace0, tag("fun")),
            preceded(multispace0, cut(char('('))),
            preceded(multispace0, opt(parse_parameters)),
            preceded(multispace0, cut(char(')'))),
            preceded(multispace0, cut(parse_block)),
        )),
        |(_, _, params, _, body)| Expr::new_function(params.unwrap_or(vec![]), Box::new(body)),
    )(input)
}

pub(crate) fn parse_arguments<'a>(
    input: &'a str,
) -> IResult<&'a str, Vec<Expr>, VerboseError<&'a str>> {
    let (input, (first, (tail, _))) = pair(
        opt(parse_expression),
        many_till(
            preceded(
                preceded(multispace0, char(',')),
                preceded(multispace0, parse_expression),
            ),
            peek(preceded(multispace0, char(')'))),
        ),
    )(input)?;
    Ok((
        input,
        first.map_or(vec![], |expr| {
            let mut args = vec![expr];
            args.extend(tail);
            args
        }),
    ))
}

pub(crate) fn parse_parameters<'a>(
    input: &'a str,
) -> IResult<&'a str, Vec<Token>, VerboseError<&'a str>> {
    pair(
        parse_identifier,
        many_till(
            preceded(
                preceded(multispace0, char(',')),
                preceded(multispace0, parse_identifier),
            ),
            peek(preceded(multispace0, char(')'))),
        ),
    )(input)
    .map(|(input, (first, (tail, _)))| {
        (
            input,
            vec![first].into_iter().chain(tail.into_iter()).collect(),
        )
    })
}

pub(crate) fn parse_comment<'a>(input: &'a str) -> IResult<&'a str, Stmt, VerboseError<&'a str>> {
    map(
        preceded(
            multispace0,
            preceded(
                tag("//"),
                terminated(take_while(|c: char| c != '\n'), char('\n')),
            ),
        ),
        |_| Stmt::new_expression(Expr::new_literal(Token::Nil)),
    )(input)
}

pub(crate) fn parse_primary<'a>(input: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    alt((
        map(tag("true"), |_| Expr::new_literal(Token::True)),
        map(tag("false"), |_| Expr::new_literal(Token::False)),
        map(tag("nil"), |_| Expr::new_literal(Token::Nil)),
        map(parse_number, Expr::new_literal),
        map(parse_string, Expr::new_literal),
        parse_anonymous_function,
        map(parse_identifier, Expr::new_variable),
        map(delimited(char('('), parse_expression, cut(char(')'))), |expr| {
            Expr::new_grouping(Box::new(expr))
        }),
    ))(input)
}

pub(crate) fn parse_call<'a>(input: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    pair(
        preceded(multispace0, parse_primary),
        many0(delimited(
            preceded(multispace0, char('(')),
            parse_arguments,
            preceded(multispace0, cut(char(')'))),
        )),
    )(input)
    .map(|(input, (callee, arguments))| {
        (
            input,
            arguments.into_iter().fold(callee, |callee, arguments| {
                Expr::new_call(Box::new(callee), arguments)
            }),
        )
    })
}

pub(crate) fn parse_unary<'a>(input: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    alt((
        map(
            pair(
                alt((char('!'), char('-'))),
                preceded(multispace0, parse_unary),
            ),
            |(op, expr)| match op {
                '!' => Expr::new_unary(Token::Bang, Box::new(expr)),
                '-' => Expr::new_unary(Token::Minus, Box::new(expr)),
                _ => unreachable!(),
            },
        ),
        parse_call,
    ))(input)
}

pub(crate) fn parse_factor<'a>(input: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    pair(
        parse_unary,
        many0(map(
            pair(
                preceded(multispace0, alt((char('*'), char('/')))),
                preceded(multispace0, cut(parse_unary)),
            ),
            |(op, expr)| match op {
                '*' => (Token::Star, expr),
                '/' => (Token::Slash, expr),
                _ => unreachable!(),
            },
        )),
    )(input)
    .map(|(input, (first, tail))| {
        (
            input,
            tail.into_iter().fold(first, |left, (op, right)| {
                Expr::new_binary(Box::new(left), op, Box::new(right))
            }),
        )
    })
}

pub(crate) fn parse_term<'a>(input: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    pair(
        parse_factor,
        many0(map(
            pair(
                preceded(multispace0, alt((char('+'), char('-')))),
                preceded(multispace0, cut(parse_factor)),
            ),
            |(op, expr)| match op {
                '+' => (Token::Plus, expr),
                '-' => (Token::Minus, expr),
                _ => unreachable!(),
            },
        )),
    )(input)
    .map(|(input, (first, tail))| {
        (
            input,
            tail.into_iter().fold(first, |left, (op, right)| {
                Expr::new_binary(Box::new(left), op, Box::new(right))
            }),
        )
    })
}

pub(crate) fn parse_comparison<'a>(
    input: &'a str,
) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    pair(
        parse_term,
        many0(map(
            pair(
                preceded(multispace0, alt((tag(">="), tag(">"), tag("<="), tag("<")))),
                preceded(multispace0, cut(parse_term)),
            ),
            |(op, expr)| match op {
                ">" => (Token::Greater, expr),
                ">=" => (Token::GreaterEqual, expr),
                "<" => (Token::Less, expr),
                "<=" => (Token::LessEqual, expr),
                _ => unreachable!(),
            },
        )),
    )(input)
    .map(|(input, (first, tail))| {
        (
            input,
            tail.into_iter().fold(first, |left, (op, right)| {
                Expr::new_binary(Box::new(left), op, Box::new(right))
            }),
        )
    })
}

pub(crate) fn parse_equality<'a>(input: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    pair(
        parse_comparison,
        many0(map(
            pair(
                preceded(multispace0, alt((tag("!="), tag("==")))),
                preceded(multispace0, cut(parse_comparison)),
            ),
            |(op, expr)| match op {
                "!=" => (Token::BangEqual, expr),
                "==" => (Token::EqualEqual, expr),
                _ => unreachable!(),
            },
        )),
    )(input)
    .map(|(input, (first, tail))| {
        (
            input,
            tail.into_iter().fold(first, |left, (op, right)| {
                Expr::new_binary(Box::new(left), op, Box::new(right))
            }),
        )
    })
}

pub(crate) fn parse_logic_and<'a>(input: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    pair(
        parse_equality,
        many0(map(
            pair(
                preceded(multispace0, tag("and")),
                preceded(multispace0, cut(parse_equality)),
            ),
            |(_, expr)| expr,
        )),
    )(input)
    .map(|(input, (first, tail))| {
        (
            input,
            tail.into_iter().fold(first, |left, right| {
                Expr::new_logical(Box::new(left), Token::And, Box::new(right))
            }),
        )
    })
}

pub(crate) fn parse_logic_or<'a>(input: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    pair(
        parse_logic_and,
        many0(map(
            pair(
                preceded(multispace0, tag("or")),
                preceded(multispace0, cut(parse_logic_and)),
            ),
            |(_, expr)| expr,
        )),
    )(input)
    .map(|(input, (first, tail))| {
        (
            input,
            tail.into_iter().fold(first, |left, right| {
                Expr::new_logical(Box::new(left), Token::Or, Box::new(right))
            }),
        )
    })
}

pub(crate) fn parse_assignment<'a>(
    input: &'a str,
) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    alt((
        map(
            separated_pair(
                parse_identifier,
                preceded(multispace0, char('=')),
                preceded(multispace0, cut(parse_assignment)),
            ),
            |(name, value)| Expr::new_assign(name, Box::new(value)),
        ),
        parse_logic_or,
    ))(input)
}

pub(crate) fn parse_expression<'a>(
    input: &'a str,
) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    parse_assignment(input)
}

pub(crate) fn parse_block<'a>(input: &'a str) -> IResult<&'a str, Stmt, VerboseError<&'a str>> {
    preceded(
        preceded(multispace0, char('{')),
        many_till(
            preceded(multispace0, parse_declaration),
            preceded(multispace0, cut(char('}'))),
        ),
    )(input)
    .map(|(input, (stmts, _))| (input, Stmt::new_block(stmts)))
}

pub(crate) fn parse_while<'a>(input: &'a str) -> IResult<&'a str, Stmt, VerboseError<&'a str>> {
    map(
        tuple((
            preceded(multispace0, tag("while")),
            preceded(multispace1, cut(char('('))),
            preceded(multispace0, cut(parse_expression)),
            preceded(multispace0, cut(char(')'))),
            preceded(multispace0, cut(parse_block)),
        )),
        |(_, _, condition, _, body)| Stmt::new_while(condition, Box::new(body)),
    )(input)
}

pub(crate) fn parse_return<'a>(input: &'a str) -> IResult<&'a str, Stmt, VerboseError<&'a str>> {
    map(
        delimited(
            preceded(multispace0, tag("return")),
            opt(preceded(multispace1, parse_expression)),
            preceded(multispace0, cut(char(';'))),
        ),
        Stmt::new_return,
    )(input)
}

pub(crate) fn parse_if<'a>(input: &'a str) -> IResult<&'a str, Stmt, VerboseError<&'a str>> {
    map(
        tuple((
            preceded(multispace0, tag("if")),
            preceded(multispace1, cut(char('('))),
            preceded(multispace0, cut(parse_expression)),
            preceded(multispace0, cut(char(')'))),
            preceded(multispace0, cut(parse_block)),
            opt(preceded(
                preceded(multispace0, tag("else")),
                cut(parse_block),
            )),
        )),
        |(_, _, condition, _, then_branch, else_branch)| {
            Stmt::new_if(condition, Box::new(then_branch), else_branch.map(Box::new))
        },
    )(input)
}

pub(crate) fn parse_for<'a>(input: &'a str) -> IResult<&'a str, Stmt, VerboseError<&'a str>> {
    map(
        tuple((
            preceded(multispace0, tag("for")),
            preceded(multispace1, cut(char('('))),
            preceded(
                multispace0,
                alt((
                    parse_declaration,
                    parse_expression_stmt,
                    map(cut(char(';')), |_| {
                        Stmt::new_expression(Expr::new_literal(Token::Nil))
                    }),
                )),
            ),
            opt(preceded(multispace0, parse_expression)),
            preceded(multispace0, cut(char(';'))),
            opt(preceded(multispace0, parse_expression)),
            preceded(multispace0, cut(char(')'))),
            preceded(multispace0, cut(parse_block)),
        )),
        |(_, _, initializer, condition, _, increment, _, body)| {
            Stmt::new_block(vec![
                initializer,
                Stmt::new_while(
                    condition.unwrap_or(Expr::new_literal(Token::True)),
                    Box::new(Stmt::new_block(vec![
                        body,
                        Stmt::new_expression(increment.unwrap_or(Expr::new_literal(Token::Nil))),
                    ])),
                ),
            ])
        },
    )(input)
}

pub(crate) fn parse_expression_stmt<'a>(
    input: &'a str,
) -> IResult<&'a str, Stmt, VerboseError<&'a str>> {
    map(
        terminated(parse_expression, preceded(multispace0, cut(char(';')))),
        Stmt::new_expression,
    )(input)
}

pub(crate) fn parse_statement<'a>(input: &'a str) -> IResult<&'a str, Stmt, VerboseError<&'a str>> {
    alt((
        parse_block,
        parse_while,
        parse_return,
        parse_if,
        parse_for,
        parse_expression_stmt,
    ))(input)
}

pub(crate) fn parse_var<'a>(input: &'a str) -> IResult<&'a str, Stmt, VerboseError<&'a str>> {
    map(
        tuple((
            preceded(multispace0, tag("let")),
            preceded(multispace1, parse_identifier),
            opt(preceded(
                preceded(multispace0, char('=')),
                preceded(multispace0, cut(parse_expression)),
            )),
            preceded(multispace0, cut(char(';'))),
        )),
        |(_, name, initializer, _)| Stmt::new_var(name, initializer),
    )(input)
}

pub(crate) fn parse_function<'a>(input: &'a str) -> IResult<&'a str, Stmt, VerboseError<&'a str>> {
    map(
        tuple((
            preceded(multispace0, tag("fun")),
            preceded(multispace1, parse_identifier),
            preceded(multispace0, cut(char('('))),
            preceded(multispace0, opt(parse_parameters)),
            preceded(multispace0, cut(char(')'))),
            preceded(multispace0, cut(parse_block)),
        )),
        |(_, name, _, params, _, body)| {
            Stmt::new_function(name, params.unwrap_or(vec![]), Box::new(body))
        },
    )(input)
}

pub(crate) fn parse_declaration<'a>(
    input: &'a str,
) -> IResult<&'a str, Stmt, VerboseError<&'a str>> {
    delimited(
        multispace0,
        alt((parse_var, parse_function, parse_statement, parse_comment)),
        multispace0,
    )(input)
}

pub(crate) fn parse_program<'a>(input: &'a str) -> Result<Vec<Stmt>, ()> {
    let output = many_till(parse_declaration, preceded(multispace0, eof))(input);

    match output {
        Ok((_, (stmts, _))) => Ok(stmts),
        Err(err) => {
            println!("ERROR: ");
            err.map(|err| println!("{}", convert_error(input, err)));
            Err(())
        }
    }
}
