extern crate ast_derive;
use ast_derive::AST;

use crate::syntax::{Token, Lexer, Parse, Ranged, error::ParseError, StartsHere};

/// AST for compile time known values
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub enum Literal {
	/// None literal
	None { offset: usize },
	/// Any precision decimal integer literal
	Integer { offset: usize, value: String },
	/// String literal
	String { offset: usize, value: String },
}

impl StartsHere for Literal {
	/// Check that literal may start at current lexer position
	fn starts_here(lexer: &mut Lexer) -> bool {
		lexer.try_match_one_of(
			&[Token::None, Token::Integer, Token::String]
		).is_ok()
	}
}

impl Parse for Literal {
	type Err = ParseError;

	/// Parse literal using lexer
	fn parse(lexer: &mut Lexer) -> Result<Self, Self::Err> {
		let token = lexer.consume_one_of(&[Token::None, Token::Integer, Token::String])?;

		Ok(
			match token {
				Token::None => Literal::None { offset: lexer.span().start },
				Token::Integer =>
					Literal::Integer {
						offset: lexer.span().start,
						value: lexer.slice().to_string()
					},
				Token::String =>
					Literal::String {
						offset: lexer.span().start,
						value:
							lexer.slice()[1..lexer.span().len() - 1].to_string()
					},

				_ => unreachable!("consume_one_of returned unexpected token"),
			}
		)
	}
}

impl Ranged for Literal {
	/// Get range of literal
	fn range(&self) -> std::ops::Range<usize> {
		match self {
			Literal::None { offset } =>
				*offset..*offset + 4,
			Literal::Integer { offset, value } =>
				*offset..*offset + value.len(),
			Literal::String { offset, value } =>
				*offset..*offset + value.len() + 2,
		}
	}
}

#[test]
fn test_none() {
	let literal = "none".parse::<Literal>().unwrap();
	assert_eq!(literal, Literal::None { offset: 0 });
}

#[test]
fn test_integer() {
	let literal = "123".parse::<Literal>().unwrap();
	assert_eq!(literal, Literal::Integer { offset: 0, value: "123".to_string() });
}

#[test]
fn test_string() {
	let literal = "\"123\"".parse::<Literal>().unwrap();
	assert_eq!(literal, Literal::String { offset: 0, value: "123".to_string() });
}

#[test]
fn test_error() {
	let literal = "123a".parse::<Literal>();
	assert!(literal.is_err());
}