use std::str::FromStr;
use crate::syntax::{Token, Lexer};

extern crate ast_derive;
use ast_derive::AST;

use crate::syntax::error::*;

use super::{WithOffset, Ranged, Mutability, Mutable};

use derive_more::From;


/// Trait for parsing using lexer
pub trait Parse where Self: Sized {
	type Err;

	/// Parse starting from current lexer state
	fn parse(lexer: &mut Lexer) -> Result<Self, Self::Err>;
}

/// AST for compile time known values
#[derive(Debug, PartialEq, AST, Clone)]
pub enum Literal {
	/// None literal
	None { offset: usize },
	/// Any precision decimal integer literal
	Integer { offset: usize, value: String },
}

impl Parse for Literal {
	type Err = ParseError;

	/// Parse literal using lexer
	fn parse(lexer: &mut Lexer) -> Result<Self, Self::Err> {
		let token = lexer.consume_one_of(&[Token::None, Token::Integer])?;

		Ok(
			match token {
				Token::None => Literal::None { offset: lexer.span().start },
				Token::Integer =>
					Literal::Integer {
						offset: lexer.span().start,
						value: lexer.slice().to_string()
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
		}
	}
}

/// AST for variable reference
#[derive(Debug, PartialEq, AST, Clone)]
pub struct VariableReference {
	/// Referenced variable name
	pub name: WithOffset<String>
}

impl Parse for VariableReference {
	type Err = ParseError;

	/// Parse variable reference using lexer
	fn parse(lexer: &mut Lexer) -> Result<Self, Self::Err> {
		lexer.consume(Token::Id)?;

		let offset = lexer.span().start;
		let name = lexer.slice().to_string();

		Ok(VariableReference { name: WithOffset { offset, value: name }})
	}
}

impl Ranged for VariableReference {
	/// Get range of variable reference
	fn range(&self) -> std::ops::Range<usize> {
		self.name.range()
	}
}

/// Unary operators
#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOperator {
	/// '+'
	Plus,
	/// '-'
	Minus
}

impl TryFrom<Token> for UnaryOperator {
	type Error = ();

	fn try_from(value: Token) -> Result<Self, Self::Error> {
		Ok(
			match value {
				Token::Plus => UnaryOperator::Plus,
				Token::Minus => UnaryOperator::Minus,
				_ => return Err(())
			}
		)
	}
}

/// Kind of unary operator
#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOperatorKind
{
	Prefix,
	Postfix
}

/// AST for unary expression
#[derive(Debug, PartialEq, AST, Clone)]
pub struct UnaryOperation {
	/// Operator of unary expression
	pub operator: WithOffset<UnaryOperator>,
	/// Operand of unary expression
	pub operand: Box<Expression>,

	/// Kind of unary operator
	pub kind: UnaryOperatorKind
}

impl Parse for UnaryOperation {
	type Err = ParseError;

	fn parse(lexer: &mut Lexer) -> Result<Self, Self::Err> {
		let prefix = lexer.consume_one_of(&[Token::Plus, Token::Minus])?;

		let offset = lexer.span().start;

		let operand = Expression::parse(lexer)?;

		Ok(UnaryOperation {
			operand: Box::new(operand),
			operator: WithOffset {
				offset,
				value: prefix.try_into().unwrap()
			},
			kind: UnaryOperatorKind::Prefix
		})
	}
}

impl Ranged for UnaryOperation {
	fn start(&self) -> usize {
		use UnaryOperatorKind::*;
		match self.kind {
			Prefix => self.operator.offset,
			Postfix => self.operand.start()
		}
	}

	fn end(&self) -> usize {
		use UnaryOperatorKind::*;
		match self.kind {
			Prefix => self.operand.end(),
			Postfix => self.operator.offset
		}
	}
}

/// Any PPL expression
#[derive(Debug, PartialEq, AST, Clone, From)]
pub enum Expression {
	Literal(Literal),
	VariableReference(VariableReference),
	UnaryOperation(UnaryOperation)
}

impl Parse for Expression {
	type Err = ParseError;

	/// Parse expression using lexer
	fn parse(lexer: &mut Lexer) -> Result<Self, Self::Err> {
		let token = lexer.try_match_one_of(
			&[Token::None, Token::Integer, Token::Id, Token::Plus, Token::Minus]
		);
		if token.is_err() {
			return Err(
				MissingExpression {
					at: lexer.span().end.into()
				}.into()
			)
		}

		Ok(
			match token.unwrap() {
				Token::None | Token::Integer =>
					Expression::Literal(Literal::parse(lexer)?),
				Token::Id =>
					Expression::VariableReference(VariableReference::parse(lexer)?),
				Token::Plus | Token::Minus =>
					UnaryOperation::parse(lexer)?.into(),
				_ => unreachable!("consume_one_of returned unexpected token"),
			}
		)
	}
}

impl Ranged for Expression {
	/// Get range of expression
	fn range(&self) -> std::ops::Range<usize> {
		match self {
			Expression::Literal(l) => l.range(),
			Expression::VariableReference(var) => var.range(),
			Expression::UnaryOperation(op) => op.range(),
		}
	}
}



/// Declaration of the variable
#[derive(Debug, PartialEq, AST, Clone)]
pub struct VariableDeclaration {
	/// Name of variable
	pub name: WithOffset<String>,
	/// Initializer for variable
	pub initializer: Expression,

	/// Is this variable mutable
	pub mutability: Mutability,
}

impl Mutable for VariableDeclaration {
	fn is_mutable(&self) -> bool {
		self.mutability.is_mutable()
	}
}

impl Parse for VariableDeclaration {
	type Err = ParseError;

	/// Parse variable declaration using lexer
	fn parse(lexer: &mut Lexer) -> Result<Self, Self::Err> {
		lexer.consume(Token::Let)?;

		let mutable = lexer.consume(Token::Mut).is_ok();

		lexer.consume(Token::Id)?;

		let name = WithOffset {
			value: lexer.slice().to_string(),
			offset: lexer.span().start
		};

		lexer.consume(Token::Assign)?;

		Ok(
			VariableDeclaration {
				name: name,
				initializer: Expression::parse(lexer)?,
				mutability: match mutable {
					true => Mutability::Mutable,
					false => Mutability::Immutable
				}
			}
		)
	}
}

/// Declaration of type
#[derive(Debug, PartialEq, AST, Clone)]
pub struct TypeDeclaration {
	/// Name of type
	pub name: WithOffset<String>,
}

impl Parse for TypeDeclaration {
	type Err = ParseError;

	/// Parse type declaration using lexer
	fn parse(lexer: &mut Lexer) -> Result<Self, Self::Err> {
		lexer.consume(Token::Type)?;

		lexer.consume(Token::Id)?;

		let name = WithOffset {
			value: lexer.slice().to_string(),
			offset: lexer.span().start
		};

		Ok(TypeDeclaration {name})
	}
}

/// Any PPL declaration
#[derive(Debug, PartialEq, AST, Clone, From)]
pub enum Declaration {
	Variable(VariableDeclaration),
	Type(TypeDeclaration),
}

impl Parse for Declaration {
	type Err = ParseError;

	/// Parse declaration using lexer
	fn parse(lexer: &mut Lexer) -> Result<Self, Self::Err> {
		let token = lexer.try_match_one_of(&[Token::Type, Token::Let])?;
		match token {
			Token::Type =>
				TypeDeclaration::parse(lexer).map(Declaration::Type),
			Token::Let =>
				VariableDeclaration::parse(lexer).map(Declaration::Variable),
			_ => unreachable!("try_ match_one_of returned unexpected token"),
		}
	}
}

/// AST for assignment
#[derive(Debug, PartialEq, AST, Clone)]
pub struct Assignment {
	/// Target to assign to
	pub target: Expression,
	/// Expression to assign
	pub value: Expression,
}

impl Parse for Assignment {
	type Err = ParseError;

	/// Parse assignment using lexer
	fn parse(lexer: &mut Lexer) -> Result<Self, Self::Err> {
		let target = Expression::parse(lexer)?;

		lexer.consume(Token::Assign)?;

		let value = Expression::parse(lexer)?;

		Ok(Assignment { target, value })
	}
}

/// Any PPL statement
#[derive(Debug, PartialEq, AST, Clone, From)]
pub enum Statement {
	Declaration(Declaration),
	Expression(Expression),
	Assignment(Assignment),
}

impl Parse for Statement {
	type Err = ParseError;

	/// Parse statement using lexer
	fn parse(lexer: &mut Lexer) -> Result<Self, Self::Err> {
		let token = lexer.try_match_one_of(
			&[Token::None, Token::Integer, Token::Id, Token::Let, Token::Plus, Token::Minus]
		);
		if token.is_err() {
			return Err(token.unwrap_err().into())
		}

		match token.unwrap() {
			Token::Let => Declaration::parse(lexer).map(|decl| Statement::Declaration(decl)),
			Token::None | Token::Integer | Token::Id | Token::Plus | Token::Minus => {
				let target = Expression::parse(lexer)?;

				if lexer.consume(Token::Assign).is_err() {
					return Ok(target.into())
				}

				Ok(Assignment {
					target,
					value: Expression::parse(lexer)?
				}.into())
			},
			_ => unreachable!("consume_one_of returned unexpected token"),
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
fn test_variable_declaration() {
	let var = "let x = 1".parse::<VariableDeclaration>().unwrap();
	assert_eq!(
		var,
		VariableDeclaration {
			name: WithOffset { offset: 4, value: "x".to_string(), },
			initializer: Literal::Integer { offset: 8, value: "1".to_string() }.into(),
			mutability: Mutability::Immutable,
		}
	);

	let var = "let mut x = 1".parse::<VariableDeclaration>().unwrap();
	assert_eq!(
		var,
		VariableDeclaration {
			name: WithOffset { offset: 8, value: "x".to_string(), },
			initializer: Literal::Integer { offset: 12, value: "1".to_string() }.into(),
			mutability: Mutability::Mutable,
		}
	)
}

#[test]
fn test_error() {
	let literal = "123a".parse::<Literal>();
	assert!(literal.is_err());
}