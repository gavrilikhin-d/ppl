use std::fmt::Display;

use logos::Logos;

/// The different kinds of tokens that can be lexed.
#[derive(Logos, Debug, PartialEq, Eq, Clone)]
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

    /// Token for operator
	#[regex(r"[-+*/=<>?!~|&^%$#\\]+")]
	Operator,

    /// Identifier
    #[regex("[_a-zA-Z][_a-zA-Z0-9]*")]
    Id,

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
    #[regex("\"[^\n\"]*\"")]
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

    /// Error token
    #[error]
    #[regex("[ ]+", logos::skip)]
    #[regex("//[^\n]*", logos::skip)]
    Error,
}

impl Token {
    /// Check if token is an operator
    pub fn is_operator(&self) -> bool {
        match self {
            Token::Assign | Token::Less | Token::Greater | Token::Operator => true,
            _ => false,
        }
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
			Token::Newline | Token::RParen | Token::Colon | Token::Comma
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
            _ => write!(f, "{:?}", self.to_string().to_lowercase()),
        }
    }
}
