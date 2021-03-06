use codespan_reporting::diagnostic::{Diagnostic, Label};
use serde_json::json;

use crate::{
    lexer::{Token, TokenKind},
    span::{FileId, Span, Spanned},
    typechecker::Type,
};

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(Span),
    ExpectedIdentifier(Span),
    ExpectedToken(TokenKind, Span),
    UnexpectedEndOfInput(Span),
}

impl ParseError {
    pub fn report(&self) -> Diagnostic<usize> {
        use ParseError::*;
        match *self {
            UnexpectedToken(span) => Diagnostic::error()
                .with_message("unexpected token encountered")
                .with_labels(vec![Label::primary(span.source.0, span)]),
            ExpectedIdentifier(span) => Diagnostic::error()
                .with_message("expected identifier")
                .with_labels(vec![Label::primary(span.source.0, span)]),
            ExpectedToken(ref kind, span) => Diagnostic::error()
                .with_message(format!("expected token {}", kind.human_name()))
                .with_labels(vec![Label::primary(span.source.0, span)]),
            UnexpectedEndOfInput(span) => Diagnostic::error()
                .with_message("unexpected end of input")
                .with_labels(vec![Label::primary(span.source.0, span)]),
        }
    }
}

impl ParseError {
    pub fn json(&self) -> serde_json::Value {
        use ParseError::*;
        match *self {
            UnexpectedToken(span) => json!({
                "message": "unexpected token encountered",
                "span": span.json(),
            }),
            ExpectedIdentifier(span) => json!({
                "message": "expected identifier",
                "span": span.json(),
            }),
            ExpectedToken(ref kind, span) => json!({
                "message": format!("expected token {}", kind.human_name()),
                "span": span.json(),
            }),
            UnexpectedEndOfInput(span) => json!({
                "message": "reached unexpected end of input",
                "span": span.json(),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Restriction {
    None,
    NoStructLiteral,
}

#[derive(Debug, Clone)]
pub struct ParsedFunctionCall {
    pub name: String,
    pub name_span: Span,
    pub args: Vec<ParsedExpression>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ParsedStructLiteral {
    pub name: String,
    pub name_span: Span,
    pub fields: Vec<(String, Span, ParsedExpression)>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ParsedArrayLiteral {
    pub elements: Vec<ParsedExpression>,
}

#[derive(Debug, Clone)]
pub enum Literal {
    String(String, Span),
    Int(i32, Span),
    Bool(bool, Span),
    Struct(ParsedStructLiteral, Span),
    Array(ParsedArrayLiteral, Span),
}

#[derive(Debug, Clone, Copy)]
pub enum CompareOperation {
    Equality,
    GreaterThan,
    GreaterThanEqual,
    LessThan,
    LessThanEqual,
}

#[derive(Debug, Clone, Copy)]
pub enum MathOperation {
    Addition,
    Subtraction,
    Multiplication,
    Division,
}

#[derive(Debug, Clone)]
pub struct ParsedFieldAccess {
    pub object: Box<ParsedExpression>,
    pub object_span: Span,
    pub field_name: String,
    pub field_name_span: Span,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ParsedArrayIndex {
    pub index: Box<ParsedExpression>,
    pub array: Box<ParsedExpression>,
}

#[derive(Debug, Clone)]
pub struct ParsedPointerTo {
    pub pointer_span: Span,
    pub inner: Box<ParsedExpression>,
    pub is_mut: bool,
}

#[derive(Debug, Clone)]
pub struct ParsedDeref {
    pub star_span: Span,
    pub inner: Box<ParsedExpression>,
}

#[derive(Debug, Clone)]
pub enum ParsedExpression {
    Literal(Literal),
    FunctionCall(ParsedFunctionCall),
    Variable(String, Span),
    CompareOp(
        Box<ParsedExpression>,
        Box<ParsedExpression>,
        CompareOperation,
    ),
    MathOp(Box<ParsedExpression>, Box<ParsedExpression>, MathOperation),
    FieldAccess(ParsedFieldAccess),
    ArrayIndex(ParsedArrayIndex),
    Assignment(Box<ParsedExpression>, Box<ParsedExpression>),
    PointerTo(ParsedPointerTo),
    Deref(ParsedDeref),
}

impl Spanned for ParsedExpression {
    fn span(&self) -> Span {
        match self {
            Self::Literal(l) => match l {
                Literal::String(_, span) => *span,
                Literal::Int(_, span) => *span,
                Literal::Bool(_, span) => *span,
                Literal::Struct(_, span) => *span,
                Literal::Array(_, span) => *span,
            },
            Self::FunctionCall(f) => f.span,
            Self::Variable(_, span) => *span,
            Self::CompareOp(lhs, rhs, _) => lhs.span().to(rhs.span()),
            Self::MathOp(lhs, rhs, _) => lhs.span().to(rhs.span()),
            Self::FieldAccess(field_access) => field_access.span,
            Self::ArrayIndex(array_index) => array_index.array.span().to(array_index.index.span()),
            Self::Assignment(lhs, rhs) => lhs.span().to(rhs.span()),
            Self::PointerTo(pointer_to) => pointer_to.pointer_span.to(pointer_to.inner.span()),
            Self::Deref(deref) => deref.star_span.to(deref.inner.span()),
        }
    }
}

#[derive(Debug)]
pub struct ParsedWhileLoop {
    pub condition: ParsedExpression,
    pub body: ParsedBlock,
}

#[derive(Debug)]
pub struct ParsedIfElse {
    pub condition: ParsedExpression,
    pub if_body: ParsedBlock,
    pub else_body: Option<ParsedBlock>,
}

#[derive(Debug)]
pub struct ParsedForInLoop {
    pub elem_var_name: String,
    pub elem_var_name_span: Span,
    pub index_var: Option<(String, Span)>,
    pub iterable_value: ParsedExpression,
    pub body: ParsedBlock,
}

#[derive(Debug)]
pub struct ParsedLetAssign {
    pub name: String,
    pub name_span: Span,
    pub value: ParsedExpression,
    pub is_mut: bool,
}

#[derive(Debug)]
pub enum ParsedStatement {
    Expression(ParsedExpression),
    LetAssign(ParsedLetAssign),
    WhileLoop(ParsedWhileLoop),
    IfElse(ParsedIfElse),
    ForInLoop(ParsedForInLoop),
    Return(ParsedExpression),
}

#[derive(Debug)]
pub struct ParsedBlock {
    pub statements: Vec<ParsedStatement>,
}

#[derive(Debug)]
pub struct ParsedFunction {
    pub name: String,
    pub name_span: Span,
    pub parameters: Vec<FunctionParameter>,
    pub body: ParsedBlock,
    pub return_type: Type,
    pub return_type_span: Span,
}

#[derive(Debug, Clone)]
pub struct FunctionParameter {
    pub name: String,
    pub name_span: Span,
    pub ttype: Type,
    pub type_span: Span,
}

#[derive(Debug)]
pub struct ParsedExternFunction {
    pub name: String,
    pub name_span: Span,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: Type,
    pub return_type_span: Span,
}

#[derive(Debug)]
pub enum ParsedStruct {
    Opaque(String, Span),
    Transparent(String, Span, Vec<(String, Type)>),
}

#[derive(Debug)]
pub struct ParsedProgram {
    pub structs: Vec<ParsedStruct>,
    pub extern_functions: Vec<ParsedExternFunction>,
    pub functions: Vec<ParsedFunction>,
}

macro_rules! expect {
    ($errors:expr, $tokens:expr, $idx:expr, $($kind:tt)+) => {{
        if matches!($tokens.get(*$idx)?, &Token { kind: $($kind)+, .. }) {
            *$idx += 1;
        } else {
            $errors.push(ParseError::ExpectedToken($($kind)+, $tokens.get(*$idx)?.span));
        }
    }};
}

macro_rules! recover_at_token {
    ($errors:expr, $tokens:expr, $idx:expr, $($expected_kind:tt)+) => {{
        while *$idx < $tokens.len()
            && !matches!(
                $tokens.get(*$idx)?,
                &Token {
                    kind: $($expected_kind)+,
                    ..
                }
            )
        {
            $errors.push(ParseError::ExpectedToken(
                $($expected_kind)+,
                $tokens.get(*$idx)?.span,
            ));
            *$idx += 1;
        }
        expect!($errors, $tokens, $idx, $($expected_kind)+);
    }};
}

pub fn parse_program(tokens: &[Token], idx: &mut usize) -> (ParsedProgram, Vec<ParseError>) {
    let mut errors = vec![];
    let mut program = ParsedProgram {
        structs: vec![],
        extern_functions: vec![],
        functions: vec![],
    };

    while *idx < tokens.len() {
        let reached_unexpected_eoi = (|| {
            let token = &tokens[*idx];
            match token {
                Token {
                    kind: TokenKind::Opaque,
                    ..
                } => {
                    let (r#struct, mut errs) = parse_opaque_struct(tokens, idx)?;
                    program.structs.push(r#struct);
                    errors.append(&mut errs);
                }
                Token {
                    kind: TokenKind::Struct,
                    ..
                } => {
                    let (r#struct, mut errs) = parse_struct(tokens, idx)?;
                    program.structs.push(r#struct);
                    errors.append(&mut errs);
                }
                Token {
                    kind: TokenKind::Fn,
                    ..
                } => {
                    let (fun, mut errs) = parse_function(tokens, idx)?;
                    program.functions.push(fun);
                    errors.append(&mut errs);
                }
                Token {
                    kind: TokenKind::Extern,
                    ..
                } => {
                    let (fun, mut errs) = parse_extern_function(tokens, idx)?;
                    program.extern_functions.push(fun);
                    errors.append(&mut errs);
                }
                _ => {
                    errors.push(ParseError::UnexpectedToken(token.span));
                    *idx += 1;
                }
            }
            Some(())
        })()
        .is_none();

        if reached_unexpected_eoi {
            let last_span = tokens.last().unwrap().span;
            errors.push(ParseError::UnexpectedEndOfInput(last_span));
            break;
        };
    }

    (program, errors)
}

fn parse_struct(tokens: &[Token], idx: &mut usize) -> Option<(ParsedStruct, Vec<ParseError>)> {
    let mut errors = vec![];

    expect!(&mut errors, tokens, idx, TokenKind::Struct);

    let (name, name_span, mut errs) = parse_name(tokens, idx)?;
    errors.append(&mut errs);

    expect!(&mut errors, tokens, idx, TokenKind::OBrace);

    let mut fields = vec![];
    while *idx < tokens.len()
        && !matches!(
            tokens.get(*idx)?,
            &Token {
                kind: TokenKind::CBrace,
                ..
            }
        )
    {
        let (field, mut errs) = parse_parameter(tokens, idx)?;
        fields.push((field.name, field.ttype));
        errors.append(&mut errs);

        if matches!(
            tokens.get(*idx)?,
            &Token {
                kind: TokenKind::Comma,
                ..
            }
        ) {
            *idx += 1;
        } else {
            break;
        }
    }

    recover_at_token!(&mut errors, tokens, idx, TokenKind::CBrace);

    Some((
        ParsedStruct::Transparent(name, name_span, fields.into_iter().collect()),
        errors,
    ))
}

fn parse_opaque_struct(
    tokens: &[Token],
    idx: &mut usize,
) -> Option<(ParsedStruct, Vec<ParseError>)> {
    let mut errors = vec![];

    expect!(&mut errors, tokens, idx, TokenKind::Opaque);
    expect!(&mut errors, tokens, idx, TokenKind::Struct);

    let (name, name_span, mut errs) = parse_name(tokens, idx)?;
    errors.append(&mut errs);

    expect!(&mut errors, tokens, idx, TokenKind::SemiColon);

    Some((ParsedStruct::Opaque(name, name_span), errors))
}

fn parse_extern_function(
    tokens: &[Token],
    idx: &mut usize,
) -> Option<(ParsedExternFunction, Vec<ParseError>)> {
    let mut errors = vec![];

    *idx += 1; // Consume `extern` keyword

    expect!(&mut errors, tokens, idx, TokenKind::Fn);

    let (name, name_span, mut errs) = parse_name(tokens, idx)?;
    errors.append(&mut errs);

    expect!(&mut errors, tokens, idx, TokenKind::OParen);

    let mut parameters = vec![];
    while *idx < tokens.len()
        && !matches!(
            tokens.get(*idx)?,
            &Token {
                kind: TokenKind::CParen,
                ..
            }
        )
    {
        let (param, mut errs) = parse_parameter(tokens, idx)?;
        parameters.push(param);
        errors.append(&mut errs);

        if matches!(
            tokens.get(*idx)?,
            &Token {
                kind: TokenKind::Comma,
                ..
            }
        ) {
            *idx += 1;
        } else {
            break;
        }
    }

    expect!(&mut errors, tokens, idx, TokenKind::CParen);

    let (return_type, return_type_span) = if let Some(Token {
        kind: TokenKind::Colon,
        ..
    }) = tokens.get(*idx)
    {
        expect!(&mut errors, tokens, idx, TokenKind::Colon);
        let (return_type, return_type_span, mut errs) = parse_type(tokens, idx)?;
        errors.append(&mut errs);

        (return_type, return_type_span)
    } else {
        (Type::Unit, Span::new(FileId(0), 0, 0))
    };

    // Semicolon should be the very next token, but if there was a parse error before
    // that might not be the case.
    // Looking for the next semicolon allows for recovery from an invalid state
    recover_at_token!(&mut errors, tokens, idx, TokenKind::SemiColon);

    let fun = ParsedExternFunction {
        name,
        name_span,
        parameters,
        return_type,
        return_type_span,
    };

    Some((fun, errors))
}

fn parse_function(tokens: &[Token], idx: &mut usize) -> Option<(ParsedFunction, Vec<ParseError>)> {
    let mut errors = vec![];

    expect!(&mut errors, tokens, idx, TokenKind::Fn);

    let (name, name_span, mut errs) = parse_name(tokens, idx)?;
    errors.append(&mut errs);

    expect!(&mut errors, tokens, idx, TokenKind::OParen);

    let mut parameters = vec![];
    while *idx < tokens.len()
        && !matches!(
            tokens.get(*idx)?,
            &Token {
                kind: TokenKind::CParen,
                ..
            }
        )
    {
        let (param, mut errs) = parse_parameter(tokens, idx)?;
        parameters.push(param);
        errors.append(&mut errs);

        if matches!(
            tokens.get(*idx)?,
            &Token {
                kind: TokenKind::Comma,
                ..
            }
        ) {
            *idx += 1;
        } else {
            break;
        }
    }

    expect!(&mut errors, tokens, idx, TokenKind::CParen);

    let (return_type, return_type_span) = if let Some(Token {
        kind: TokenKind::Colon,
        ..
    }) = tokens.get(*idx)
    {
        expect!(&mut errors, tokens, idx, TokenKind::Colon);
        let (return_type, return_type_span, mut errs) = parse_type(tokens, idx)?;
        errors.append(&mut errs);

        (return_type, return_type_span)
    } else {
        (Type::Unit, Span::new(FileId(0), 0, 0))
    };

    let (body, mut errs) = parse_block(tokens, idx)?;
    errors.append(&mut errs);

    let fun = ParsedFunction {
        name,
        name_span,
        body,
        parameters,
        return_type,
        return_type_span,
    };

    Some((fun, errors))
}

fn parse_type(tokens: &[Token], idx: &mut usize) -> Option<(Type, Span, Vec<ParseError>)> {
    let mut errors = vec![];

    let is_pointer = matches!(
        tokens.get(*idx)?,
        Token {
            kind: TokenKind::RightArrow,
            ..
        }
    );
    if is_pointer {
        *idx += 1;
    }
    let is_mut_pointer = is_pointer
        && matches!(
            tokens.get(*idx)?,
            Token {
                kind: TokenKind::Mut,
                ..
            }
        );
    if is_mut_pointer {
        *idx += 1;
    }

    let (ttype, type_span) = if let tok @ &Token {
        kind: TokenKind::Ident(ref name),
        ..
    } = tokens.get(*idx)?
    {
        *idx += 1;
        (Type::from_string(name), tok.span)
    } else {
        let span = tokens.get(*idx)?.span;
        errors.push(ParseError::ExpectedIdentifier(span));
        (Type::Unit, span)
    };

    let ttype = if is_pointer {
        Type::Pointer(Box::new(ttype), is_mut_pointer)
    } else {
        ttype
    };

    Some((ttype, type_span, errors))
}

fn parse_parameter(
    tokens: &[Token],
    idx: &mut usize,
) -> Option<(FunctionParameter, Vec<ParseError>)> {
    let mut errors = vec![];

    let (name, name_span, mut errs) = parse_name(tokens, idx)?;
    errors.append(&mut errs);

    expect!(&mut errors, tokens, idx, TokenKind::Colon);

    let (ttype, type_span, mut errs) = parse_type(tokens, idx)?;
    errors.append(&mut errs);

    Some((
        FunctionParameter {
            name,
            name_span,
            ttype,
            type_span,
        },
        errors,
    ))
}

fn parse_block(tokens: &[Token], idx: &mut usize) -> Option<(ParsedBlock, Vec<ParseError>)> {
    let mut errors = vec![];

    expect!(&mut errors, tokens, idx, TokenKind::OBrace);

    let mut statements = vec![];
    while *idx < tokens.len()
        && !matches!(
            tokens.get(*idx)?,
            &Token {
                kind: TokenKind::CBrace,
                ..
            }
        )
    {
        let (stmt, mut errs) = parse_statement(tokens, idx)?;
        statements.push(stmt);
        errors.append(&mut errs);
    }

    expect!(&mut errors, tokens, idx, TokenKind::CBrace);

    Some((ParsedBlock { statements }, errors))
}

fn parse_statement(
    tokens: &[Token],
    idx: &mut usize,
) -> Option<(ParsedStatement, Vec<ParseError>)> {
    let (statement, mut errors, needs_semi) = match tokens.get(*idx)? {
        Token {
            kind: TokenKind::Let,
            ..
        } => {
            let mut errors = vec![];

            *idx += 1; // Consume `let` token

            let is_mut = if let Token {
                kind: TokenKind::Mut,
                ..
            } = tokens.get(*idx)?
            {
                *idx += 1; // Consume `mut` token
                true
            } else {
                false
            };

            let (name, name_span) = if let tok @ &Token {
                kind: TokenKind::Ident(ref name),
                ..
            } = &tokens.get(*idx)?
            {
                *idx += 1;
                (name.clone(), tok.span)
            } else {
                errors.push(ParseError::ExpectedIdentifier(tokens.get(*idx)?.span));
                (String::new(), tokens.get(*idx)?.span)
            };

            expect!(&mut errors, tokens, idx, TokenKind::Equal);

            let (value, mut errs) = parse_expression(tokens, idx, Restriction::None)?;
            errors.append(&mut errs);

            (
                ParsedStatement::LetAssign(ParsedLetAssign {
                    name,
                    name_span,
                    value,
                    is_mut,
                }),
                errors,
                true,
            )
        }
        Token {
            kind: TokenKind::While,
            ..
        } => {
            let (stmt, errors) = parse_while_loop(tokens, idx)?;
            (ParsedStatement::WhileLoop(stmt), errors, false)
        }
        Token {
            kind: TokenKind::If,
            ..
        } => {
            let (if_else, errors) = parse_if_else(tokens, idx)?;
            (ParsedStatement::IfElse(if_else), errors, false)
        }
        Token {
            kind: TokenKind::For,
            ..
        } => {
            let (for_in, errors) = parse_for_in_loop(tokens, idx)?;
            (ParsedStatement::ForInLoop(for_in), errors, false)
        }
        Token {
            kind: TokenKind::Return,
            ..
        } => {
            *idx += 1; // Consume `return` token
            let (return_value, errors) = parse_expression(tokens, idx, Restriction::None)?;
            (ParsedStatement::Return(return_value), errors, true)
        }
        _ => {
            let (expr, errors) = parse_expression(tokens, idx, Restriction::None)?;
            (ParsedStatement::Expression(expr), errors, true)
        }
    };

    if needs_semi {
        // Semicolon should be the very next token, but if there was a parse error before
        // that might not be the case.
        // Looking for the next semicolon allows for recovery from an invalid state
        recover_at_token!(&mut errors, tokens, idx, TokenKind::SemiColon);
    }

    Some((statement, errors))
}

fn parse_for_in_loop(
    tokens: &[Token],
    idx: &mut usize,
) -> Option<(ParsedForInLoop, Vec<ParseError>)> {
    let mut errors = vec![];

    expect!(&mut errors, tokens, idx, TokenKind::For);

    let ((elem_var_name, elem_var_name_span), index_var) = if matches!(
        tokens.get(*idx + 1),
        Some(Token {
            kind: TokenKind::Comma,
            ..
        })
    ) {
        let (index_var_name, index_var_name_span, mut errs) = parse_name(tokens, idx)?;
        errors.append(&mut errs);

        expect!(&mut errors, tokens, idx, TokenKind::Comma);

        let (elem_var_name, elem_var_name_span, mut errs) = parse_name(tokens, idx)?;
        errors.append(&mut errs);

        (
            (elem_var_name, elem_var_name_span),
            Some((index_var_name, index_var_name_span)),
        )
    } else {
        let (elem_var_name, elem_var_name_span, mut errs) = parse_name(tokens, idx)?;
        errors.append(&mut errs);

        ((elem_var_name, elem_var_name_span), None)
    };

    expect!(&mut errors, tokens, idx, TokenKind::In);

    let (iterable_value, mut errs) = parse_expression(tokens, idx, Restriction::NoStructLiteral)?;
    errors.append(&mut errs);

    let (body, mut errs) = parse_block(tokens, idx)?;
    errors.append(&mut errs);

    Some((
        ParsedForInLoop {
            elem_var_name,
            elem_var_name_span,
            index_var,
            iterable_value,
            body,
        },
        errors,
    ))
}

fn parse_while_loop(
    tokens: &[Token],
    idx: &mut usize,
) -> Option<(ParsedWhileLoop, Vec<ParseError>)> {
    let mut errors = vec![];

    expect!(&mut errors, tokens, idx, TokenKind::While);

    let (condition, mut errs) = parse_expression(tokens, idx, Restriction::NoStructLiteral)?;
    errors.append(&mut errs);

    let (body, mut errs) = parse_block(tokens, idx)?;
    errors.append(&mut errs);

    Some((ParsedWhileLoop { condition, body }, errors))
}

fn parse_if_else(tokens: &[Token], idx: &mut usize) -> Option<(ParsedIfElse, Vec<ParseError>)> {
    let mut errors = vec![];

    expect!(&mut errors, tokens, idx, TokenKind::If);

    let (condition, mut errs) = parse_expression(tokens, idx, Restriction::NoStructLiteral)?;
    errors.append(&mut errs);

    let (if_body, mut errs) = parse_block(tokens, idx)?;
    errors.append(&mut errs);

    let else_body = if matches!(
        tokens.get(*idx),
        Some(Token {
            kind: TokenKind::Else,
            ..
        })
    ) {
        expect!(&mut errors, tokens, idx, TokenKind::Else);

        let (else_body, mut errs) = parse_block(tokens, idx)?;
        errors.append(&mut errs);

        Some(else_body)
    } else {
        None
    };

    Some((
        ParsedIfElse {
            condition,
            if_body,
            else_body,
        },
        errors,
    ))
}

fn parse_expression(
    tokens: &[Token],
    idx: &mut usize,
    restriction: Restriction,
) -> Option<(ParsedExpression, Vec<ParseError>)> {
    let (expr, mut errors) = parse_assignment(tokens, idx, restriction)?;
    let expr = if let Some(
        tok @ Token {
            kind:
                TokenKind::EqualEqual
                | TokenKind::GreaterThan
                | TokenKind::GreaterThanEqual
                | TokenKind::LessThan
                | TokenKind::LessThanEqual,
            ..
        },
    ) = tokens.get(*idx)
    {
        *idx += 1; // Consume operator token

        let op = match tok.kind {
            TokenKind::EqualEqual => CompareOperation::Equality,
            TokenKind::GreaterThan => CompareOperation::GreaterThan,
            TokenKind::GreaterThanEqual => CompareOperation::GreaterThanEqual,
            TokenKind::LessThan => CompareOperation::LessThan,
            TokenKind::LessThanEqual => CompareOperation::LessThanEqual,
            _ => unreachable!(),
        };

        let (rhs, mut errs) = parse_expression(tokens, idx, restriction)?;
        errors.append(&mut errs);

        ParsedExpression::CompareOp(Box::new(expr), Box::new(rhs), op)
    } else {
        expr
    };
    Some((expr, errors))
}

fn parse_assignment(
    tokens: &[Token],
    idx: &mut usize,
    restriction: Restriction,
) -> Option<(ParsedExpression, Vec<ParseError>)> {
    let (expr, mut errors) = parse_math(tokens, idx, restriction)?;
    let expr = if let Some(Token {
        kind: TokenKind::Equal,
        ..
    }) = tokens.get(*idx)
    {
        *idx += 1; // Consume operator token

        let (rhs, mut errs) = parse_math(tokens, idx, restriction)?;
        errors.append(&mut errs);

        ParsedExpression::Assignment(Box::new(expr), Box::new(rhs))
    } else {
        expr
    };
    Some((expr, errors))
}

fn parse_math(
    tokens: &[Token],
    idx: &mut usize,
    restriction: Restriction,
) -> Option<(ParsedExpression, Vec<ParseError>)> {
    let (expr, mut errors) = parse_factor(tokens, idx, restriction)?;
    let expr = if let Some(
        tok @ Token {
            kind: TokenKind::Plus | TokenKind::Minus,
            ..
        },
    ) = tokens.get(*idx)
    {
        *idx += 1; // Consume operator token

        let op = match tok.kind {
            TokenKind::Plus => MathOperation::Addition,
            TokenKind::Minus => MathOperation::Subtraction,
            _ => unreachable!(),
        };

        let (rhs, mut errs) = parse_math(tokens, idx, restriction)?;
        errors.append(&mut errs);

        ParsedExpression::MathOp(Box::new(expr), Box::new(rhs), op)
    } else {
        expr
    };
    Some((expr, errors))
}

fn parse_factor(
    tokens: &[Token],
    idx: &mut usize,
    restriction: Restriction,
) -> Option<(ParsedExpression, Vec<ParseError>)> {
    let (expr, mut errors) = parse_term(tokens, idx, restriction)?;
    let expr = if let Some(
        tok @ Token {
            kind: TokenKind::Star | TokenKind::Slash,
            ..
        },
    ) = tokens.get(*idx)
    {
        *idx += 1; // Consume operator token

        let op = match tok.kind {
            TokenKind::Star => MathOperation::Multiplication,
            TokenKind::Slash => MathOperation::Division,
            _ => unreachable!(),
        };

        let (rhs, mut errs) = parse_factor(tokens, idx, restriction)?;
        errors.append(&mut errs);

        ParsedExpression::MathOp(Box::new(expr), Box::new(rhs), op)
    } else {
        expr
    };
    Some((expr, errors))
}

fn parse_term(
    tokens: &[Token],
    idx: &mut usize,
    restriction: Restriction,
) -> Option<(ParsedExpression, Vec<ParseError>)> {
    let mut errors = vec![];
    let (expr, mut errors) = loop {
        break match tokens.get(*idx)? {
            pointer_tok @ Token {
                kind: TokenKind::RightArrow,
                ..
            } => {
                *idx += 1; // Consume `->` token
                let mut_span = if let Token {
                    kind: TokenKind::Mut,
                    span,
                } = tokens.get(*idx)?
                {
                    *idx += 1; // Consume `mut` token
                    Some(*span)
                } else {
                    None
                };
                let (expr, mut errs) = parse_term(tokens, idx, restriction)?;
                errors.append(&mut errs);

                let pointer_span = if let Some(mut_span) = mut_span {
                    pointer_tok.span.to(mut_span)
                } else {
                    pointer_tok.span
                };
                (
                    ParsedExpression::PointerTo(ParsedPointerTo {
                        pointer_span,
                        inner: Box::new(expr),
                        is_mut: mut_span.is_some(),
                    }),
                    errors,
                )
            }
            Token {
                kind: TokenKind::Star,
                span: star_span,
            } => {
                *idx += 1; // Consume `*` token
                let (expr, errors) = parse_term(tokens, idx, restriction)?;
                (
                    ParsedExpression::Deref(ParsedDeref {
                        star_span: *star_span,
                        inner: Box::new(expr),
                    }),
                    errors,
                )
            }
            tok @ Token {
                kind: TokenKind::Ident(name),
                ..
            } => match tokens.get(*idx + 1) {
                Some(Token {
                    kind: TokenKind::OParen,
                    ..
                }) => {
                    let (func_call, mut errs) = parse_function_call(tokens, idx)?;
                    errors.append(&mut errs);
                    (ParsedExpression::FunctionCall(func_call), errors)
                }
                Some(Token {
                    kind: TokenKind::OBrace,
                    ..
                }) => {
                    if restriction == Restriction::NoStructLiteral {
                        // TODO: Add help for how to use struct literals in restricted expressions
                        *idx += 1; // Consume ident token
                        (ParsedExpression::Variable(name.clone(), tok.span), errors)
                    } else {
                        let (struct_literal, mut errs) = parse_struct_literal(tokens, idx)?;
                        errors.append(&mut errs);
                        let span = struct_literal.span;
                        (
                            ParsedExpression::Literal(Literal::Struct(struct_literal, span)),
                            errors,
                        )
                    }
                }
                _ => {
                    *idx += 1; // Consume ident token
                    (ParsedExpression::Variable(name.clone(), tok.span), errors)
                }
            },
            tok @ Token {
                kind: TokenKind::StringLiteral(string),
                ..
            } => {
                *idx += 1;
                (
                    ParsedExpression::Literal(Literal::String(string.clone(), tok.span)),
                    errors,
                )
            }
            tok @ Token {
                kind: TokenKind::IntLiteral(int),
                ..
            } => {
                *idx += 1;
                (
                    ParsedExpression::Literal(Literal::Int(*int, tok.span)),
                    errors,
                )
            }
            tok @ Token {
                kind: TokenKind::True | TokenKind::False,
                ..
            } => {
                *idx += 1;
                let bool_value = matches!(tok.kind, TokenKind::True);
                (
                    ParsedExpression::Literal(Literal::Bool(bool_value, tok.span)),
                    errors,
                )
            }
            Token {
                kind: TokenKind::OBracket,
                ..
            } => {
                let (array, span, errors) = parse_array_literal(tokens, idx)?;
                (
                    ParsedExpression::Literal(Literal::Array(array, span)),
                    errors,
                )
            }
            tok => {
                errors.push(ParseError::UnexpectedToken(tok.span));
                *idx += 1;
                continue;
            }
        };
    };

    let expr = if let Some(Token {
        kind: TokenKind::Dot,
        ..
    }) = tokens.get(*idx)
    {
        *idx += 1; // Consume dot token.

        let (field_name, field_name_span, mut errs) = parse_name(tokens, idx)?;
        errors.append(&mut errs);

        let object_span = expr.span();
        let span = object_span.to(field_name_span);

        ParsedExpression::FieldAccess(ParsedFieldAccess {
            object: Box::new(expr),
            object_span,
            field_name,
            field_name_span,
            span,
        })
    } else {
        expr
    };

    let expr = if let Some(Token {
        kind: TokenKind::OBracket,
        ..
    }) = tokens.get(*idx)
    {
        *idx += 1; // Consume `[` token

        let (index, mut errs) = parse_expression(tokens, idx, restriction)?;
        errors.append(&mut errs);

        expect!(&mut errors, tokens, idx, TokenKind::CBracket);

        ParsedExpression::ArrayIndex(ParsedArrayIndex {
            index: Box::new(index),
            array: Box::new(expr),
        })
    } else {
        expr
    };

    Some((expr, errors))
}

fn parse_array_literal(
    tokens: &[Token],
    idx: &mut usize,
) -> Option<(ParsedArrayLiteral, Span, Vec<ParseError>)> {
    let mut errors = vec![];

    expect!(&mut errors, tokens, idx, TokenKind::OBracket);
    let o_brace_span = tokens[*idx - 1].span;

    let mut elements = vec![];
    while *idx < tokens.len()
        && !matches!(
            tokens.get(*idx)?,
            &Token {
                kind: TokenKind::CBracket,
                ..
            }
        )
    {
        let (arg, mut errs) = parse_expression(tokens, idx, Restriction::None)?;
        elements.push(arg);
        errors.append(&mut errs);

        if matches!(
            &tokens.get(*idx)?,
            &Token {
                kind: TokenKind::Comma,
                ..
            }
        ) {
            *idx += 1;
        } else {
            break;
        }
    }

    expect!(&mut errors, tokens, idx, TokenKind::CBracket);
    let c_brace_span = tokens[*idx - 1].span;

    Some((
        ParsedArrayLiteral { elements },
        o_brace_span.to(c_brace_span),
        errors,
    ))
}

fn parse_struct_literal(
    tokens: &[Token],
    idx: &mut usize,
) -> Option<(ParsedStructLiteral, Vec<ParseError>)> {
    let (name, name_span, mut errors) = parse_name(tokens, idx)?;

    expect!(&mut errors, tokens, idx, TokenKind::OBrace);

    let mut fields = vec![];
    while *idx < tokens.len()
        && !matches!(
            &tokens.get(*idx)?,
            &Token {
                kind: TokenKind::CBrace,
                ..
            }
        )
    {
        let (field_name, field_name_span, mut errs) = parse_name(tokens, idx)?;
        errors.append(&mut errs);

        expect!(&mut errors, tokens, idx, TokenKind::Colon);

        let (field_value, mut errs) = parse_expression(tokens, idx, Restriction::None)?;
        errors.append(&mut errs);

        fields.push((field_name, field_name_span, field_value));

        if matches!(
            &tokens.get(*idx)?,
            &Token {
                kind: TokenKind::Comma,
                ..
            }
        ) {
            *idx += 1;
        } else {
            break;
        }
    }

    expect!(&mut errors, tokens, idx, TokenKind::CBrace);
    let c_brace_span = tokens[*idx - 1].span;

    Some((
        ParsedStructLiteral {
            name,
            name_span,
            fields,
            span: name_span.to(c_brace_span),
        },
        errors,
    ))
}

fn parse_function_call(
    tokens: &[Token],
    idx: &mut usize,
) -> Option<(ParsedFunctionCall, Vec<ParseError>)> {
    let (name, name_span, mut errors) = parse_name(tokens, idx)?;

    expect!(&mut errors, tokens, idx, TokenKind::OParen);

    let mut args = vec![];
    while *idx < tokens.len()
        && !matches!(
            tokens.get(*idx)?,
            &Token {
                kind: TokenKind::CParen,
                ..
            }
        )
    {
        let (arg, mut errs) = parse_expression(tokens, idx, Restriction::None)?;
        args.push(arg);
        errors.append(&mut errs);

        if matches!(
            &tokens.get(*idx)?,
            &Token {
                kind: TokenKind::Comma,
                ..
            }
        ) {
            *idx += 1;
        } else {
            break;
        }
    }

    let cparen_span = tokens.get(*idx)?.span;
    expect!(&mut errors, tokens, idx, TokenKind::CParen);

    let func_call = ParsedFunctionCall {
        name,
        name_span,
        args,
        span: name_span.to(cparen_span),
    };

    Some((func_call, errors))
}

fn parse_name(tokens: &[Token], idx: &mut usize) -> Option<(String, Span, Vec<ParseError>)> {
    Some(
        if let tok @ &Token {
            kind: TokenKind::Ident(ref name),
            ..
        } = tokens.get(*idx)?
        {
            *idx += 1;
            (name.clone(), tok.span, vec![])
        } else {
            (
                String::new(),
                tokens.get(*idx)?.span,
                vec![ParseError::ExpectedIdentifier(tokens.get(*idx)?.span)],
            )
        },
    )
}
