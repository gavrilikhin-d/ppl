use std::{fmt::Display, ops::Range};

use logos::{Lexer, Logos};

use super::error::{InvalidIndentation, InvalidToken, LexerError};

/// Kind of operator
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum OperatorKind {
    Prefix,
    Infix,
    Postfix,
}

/// Decide which kind of operator it is
fn operator(lexer: &mut Lexer<Token>) -> OperatorKind {
    if lexer.span().start == 0 {
        return OperatorKind::Prefix;
    }
    if lexer.span().end == lexer.source().len() {
        return OperatorKind::Postfix;
    }

    let before = lexer.source().chars().nth(lexer.span().start - 1).unwrap();
    // If we reach eof, we assume that it is a whitespace
    let after = lexer.source().chars().nth(lexer.span().end).unwrap_or(' ');
    if before.is_whitespace() == after.is_whitespace() {
        OperatorKind::Infix
    } else if after.is_whitespace() {
        OperatorKind::Postfix
    } else {
        OperatorKind::Prefix
    }
}

/// [`Token::Error`] kind
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum ErrorKind {
    /// Invalid (unknown or non-utf) token
    #[default]
    InvalidToken,
    /// Indentation with spaces
    InvalidIndentation,
}

impl ErrorKind {
    /// Construct lexer error from error kind
    pub fn at(&self, at: Range<usize>) -> LexerError {
        match self {
            ErrorKind::InvalidToken => InvalidToken { at: at.into() }.into(),
            ErrorKind::InvalidIndentation => InvalidIndentation {
                // Skip newline
                at: (at.start + 1..at.end).into(),
            }
            .into(),
        }
    }
}

/// The different kinds of tokens that can be lexed.
#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(error = ErrorKind)]
#[logos(skip "[ ]+")]
#[logos(skip "//[^\n]*")]
pub enum Token {
    /// None literal
    #[token("none")]
    None,

    /// Integer literal
    #[regex("[0-9]+")]
    Integer,

    /// Assign token
    #[token("=")]
    Assign,

    /// '*' token
    #[token("*")]
    Star,

    /// Token for operator
    #[regex(r"[-+*/=<>?!~|&^%$#\\]+", operator, priority = 0)]
    Operator(OperatorKind),

    /// '&' token
    #[token("&")]
    Ampersand,

    /// Identifier
    #[regex("[_a-zA-Z][_a-zA-Z0-9]*")]
    Id,

    /// Escaped identifier
    #[regex("[`][^`]*[`]")]
    EscapedId,

    /// "let" token
    #[token("let")]
    Let,

    /// "mut" token
    #[token("mut")]
    Mut,

    /// "type" token
    #[token("type")]
    Type,

    /// '\n' token
    #[token("\n")]
    Newline,

    /// ':' token
    #[token(":")]
    Colon,

    /// '<' token
    #[token("<")]
    Less,

    /// '>' token
    #[token(">")]
    Greater,

    /// '[' token
    #[token("[")]
    LBracket,

    /// ']' token
    #[token("]")]
    RBracket,

    /// "fn" token
    #[token("fn")]
    Fn,

    /// "->" token
    #[token("->")]
    Arrow,

    /// "=>" token
    #[token("=>")]
    FatArrow,

    /// String literal
    #[regex(r#""(?:[^"\\]|\\.)*""#)]
    String,

    /// '@' token
    #[token("@")]
    At,

    /// '(' token
    #[token("(")]
    LParen,

    /// ')' token
    #[token(")")]
    RParen,

    /// ',' comma
    #[token(",")]
    Comma,

    /// '\t' token
    #[token("\t")]
    Tab,

    /// "return" token
    #[token("return")]
    Return,

    /// "if" token
    #[token("if")]
    If,

    /// "else" token
    #[token("else")]
    Else,

    /// "true" token
    #[token("true")]
    True,

    /// "false" token
    #[token("false")]
    False,

    /// "loop" token
    #[token("loop")]
    Loop,

    /// "while" token
    #[token("while")]
    While,

    /// "trait" token
    #[token("trait")]
    Trait,

    /// '.' token
    #[token(".")]
    Dot,

    /// '{' token
    #[token("{")]
    LBrace,

    /// '}' token
    #[token("}")]
    RBrace,

    /// Rational literal
    #[regex("[0-9]*[.][0-9]+")]
    Rational,

    /// "use" token
    #[token("use")]
    Use,

    /// Error token
    #[regex("\n[ ]+", |_| ErrorKind::InvalidIndentation)]
    Error(ErrorKind),
}

impl Token {
    /// Check if token is an operator
    pub fn is_operator(&self) -> bool {
        self.is_infix_operator() || matches!(self, Token::Operator(_))
    }

    /// Check if token is an infix operator
    pub fn is_infix_operator(&self) -> bool {
        matches!(
            self,
            Token::Operator(OperatorKind::Infix)
                | Token::Greater
                | Token::Star
                | Token::Ampersand
                | Token::Less
        )
    }

    /// Check if token is a whitespace token
    pub fn is_whitespace(&self) -> bool {
        match self {
            Token::Newline => true,
            _ => false,
        }
    }

    /// Check if token ends expression
    pub fn ends_expression(&self) -> bool {
        matches!(
            self,
            Token::Newline
                | Token::RParen
                | Token::Colon
                | Token::Comma
                | Token::Assign
                | Token::RBrace
                | Token::RBracket
        )
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Assign => write!(f, "="),
            Token::Colon => write!(f, ":"),
            Token::Less => write!(f, "<"),
            Token::Greater => write!(f, ">"),
            Token::Arrow => write!(f, "->"),
            Token::FatArrow => write!(f, "=>"),
            Token::At => write!(f, "@"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::Dot => write!(f, "."),
            Token::Comma => write!(f, ","),
            Token::Star => write!(f, "*"),
            Token::Ampersand => write!(f, "&"),
            _ => write!(f, "{}", format!("{:?}", self).to_lowercase()),
        }
    }
}
