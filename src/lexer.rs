use ariadne::{Color, Label, Report, ReportKind};

use crate::{
    error::ReportError,
    span::{Span, Spanned},
};

#[derive(Debug)]
pub enum TokenKind {
    StringLiteral(String),
    IntLiteral(i32),
    Ident(String),
    Extern,
    Fn,
    OParen,
    CParen,
    OBrace,
    CBrace,
    SemiColon,
    Comma,
    Colon,
    Unknown,
}

impl TokenKind {
    pub fn human_name(&self) -> &'static str {
        use TokenKind::*;
        match *self {
            StringLiteral(_) => "string literal",
            IntLiteral(_) => "integer literal",
            Ident(_) => "identifier",
            Fn => "`fn` keyword",
            Extern => "`extern` keyword",
            OParen => "`(`",
            CParen => "`)`",
            OBrace => "`{`",
            CBrace => "`}`",
            SemiColon => "`;`",
            Comma => "`,`",
            Colon => "`:`",
            Unknown => "unknown token",
        }
    }
}

#[derive(Debug)]
pub enum LexError {
    UnknownToken(char, Span),
    UnterminatedString(Span),
}

impl ReportError for LexError {
    fn report(&self) -> Report<Span> {
        use LexError::*;
        match *self {
            UnknownToken(c, span) => Report::build(ReportKind::Error, (), span.start)
                .with_message(format!("unknown token `{}`", c))
                .with_label(Label::new(span).with_color(Color::Red)),
            UnterminatedString(span) => Report::build(ReportKind::Error, (), span.start)
                .with_message("unterminated string")
                .with_label(
                    Label::new(span)
                        .with_color(Color::Red)
                        .with_message("Each string needs to be terminated with a matching `\"`."),
                ),
        }
        .finish()
    }
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub start: usize,
    pub len: usize,
}

impl Token {
    fn new(kind: TokenKind, start: usize, len: usize) -> Self {
        Self { kind, start, len }
    }
}

impl Spanned for Token {
    fn span(&self) -> Span {
        Span {
            start: self.start,
            len: self.len,
        }
    }
}

pub fn lex(source: &str) -> (Vec<Token>, Vec<LexError>) {
    let source = source.as_bytes();
    let mut idx = 0;

    let mut tokens = vec![];
    let mut errors = vec![];

    loop {
        while idx < source.len() && source[idx].is_ascii_whitespace() {
            idx += 1;
        }

        if idx == source.len() {
            break;
        }

        // Identifiers & keywords
        if source[idx].is_ascii_alphabetic() {
            let start = idx;

            while idx < source.len() && source[idx].is_ascii_alphabetic() {
                idx += 1;
            }

            let name = std::str::from_utf8(&source[start..idx]).unwrap();

            let len = idx - start;
            let token = match name {
                "fn" => Token::new(TokenKind::Fn, start, len),
                "extern" => Token::new(TokenKind::Extern, start, len),
                _ => {
                    let name = name.to_owned();
                    Token::new(TokenKind::Ident(name), start, len)
                }
            };

            tokens.push(token);

            continue;
        }

        // String literals
        if source[idx] == b'"' {
            let start = idx;
            idx += 1; // Consume opening quote

            while idx < source.len() && source[idx] != b'"' {
                idx += 1;
            }

            if idx == source.len() {
                errors.push(LexError::UnterminatedString(Span::new(start, idx - start)));
            } else {
                idx += 1; // Consume closing quote
            }

            // +1 and -1 on the bounds to exclude quotation marks
            let string = std::str::from_utf8(&source[(start + 1)..(idx - 1)])
                .unwrap()
                .to_owned();

            tokens.push(Token::new(
                TokenKind::StringLiteral(string),
                start,
                idx - start,
            ));

            continue;
        }

        // Integer literals
        if source[idx].is_ascii_digit() {
            let start = idx;
            while idx < source.len() && source[idx].is_ascii_digit() {
                idx += 1;
            }

            let int_value = std::str::from_utf8(&source[start..idx])
                .unwrap()
                .parse()
                .unwrap();
            tokens.push(Token::new(
                TokenKind::IntLiteral(int_value),
                start,
                idx - start,
            ));

            continue;
        }

        match source[idx] {
            b'(' => tokens.push(Token::new(TokenKind::OParen, idx, 1)),
            b')' => tokens.push(Token::new(TokenKind::CParen, idx, 1)),
            b'{' => tokens.push(Token::new(TokenKind::OBrace, idx, 1)),
            b'}' => tokens.push(Token::new(TokenKind::CBrace, idx, 1)),
            b';' => tokens.push(Token::new(TokenKind::SemiColon, idx, 1)),
            b',' => tokens.push(Token::new(TokenKind::Comma, idx, 1)),
            b':' => tokens.push(Token::new(TokenKind::Colon, idx, 1)),
            e => {
                tokens.push(Token::new(TokenKind::Unknown, idx, 1));
                errors.push(LexError::UnknownToken(e as char, Span::new(idx, 1)));
            }
        }

        idx += 1;
    }

    (tokens, errors)
}
