use logos::Logos;

/// The different kinds of tokens that can be lexed.
#[derive(Logos, Debug, PartialEq)]
pub enum Token
{
	/// None literal
	#[token("none")]
	None,

	/// Integer literal
	#[regex("[0-9]+")]
	Integer,

	/// Assign token
	#[token("=")]
	Assign,

	/// Identifier
	#[regex("[_a-zA-Z][_a-zA-Z0-9]*")]
	Id,

	/// Error token
	#[error]
	Error
}

#[test]
fn test_none() {
	let mut lexer = Token::lexer("none");
	assert_eq!(lexer.next(), Some(Token::None));
	assert_eq!(lexer.span(), 0..4);
	assert_eq!(lexer.next(), None);
}

#[test]
fn test_integer() {
	let mut lexer = Token::lexer("123");
	assert_eq!(lexer.next(), Some(Token::Integer));
	assert_eq!(lexer.span(), 0..3);
	assert_eq!(lexer.slice(), "123");

	assert_eq!(lexer.next(), None);
}

#[test]
fn test_identifier() {
	let mut lexer = Token::lexer("abc123");
	assert_eq!(lexer.next(), Some(Token::Id));
	assert_eq!(lexer.span(), 0..6);
	assert_eq!(lexer.slice(), "abc123");

	assert_eq!(lexer.next(), None);
}

#[test]
fn test_error() {
	let mut lexer = Token::lexer("123a");
	assert_eq!(lexer.next(), Some(Token::Integer));
	assert_eq!(lexer.span(), 0..3);
	assert_eq!(lexer.slice(), "123");

	assert_eq!(lexer.next(), Some(Token::Error));
	assert_eq!(lexer.span(), 3..4);
	assert_eq!(lexer.slice(), "a");

	assert_eq!(lexer.next(), None);
}