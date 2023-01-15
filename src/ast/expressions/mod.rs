mod call;
pub use call::*;

mod literal;
pub use literal::*;

mod variable;
pub use variable::*;

mod unary;
pub use unary::*;

mod binary;
pub use binary::*;

mod tuple;
pub use tuple::*;

extern crate ast_derive;
use ast_derive::AST;

use crate::syntax::{
    error::{MissingExpression, ParseError},
    Lexer, Parse, Ranged, StartsHere, Context,
};

use derive_more::{From, TryInto};

/// Any PPL expression
#[derive(Debug, PartialEq, Eq, AST, Clone, From, TryInto)]
pub enum Expression {
    Literal(Literal),
    VariableReference(VariableReference),
    UnaryOperation(UnaryOperation),
	BinaryOperation(BinaryOperation),
    Call(Call),
	Tuple(Tuple),
}

impl StartsHere for Expression {
    /// Check that expression 100% starts at current lexer position
    fn starts_here(context: &mut Context<impl Lexer>) -> bool {
        Literal::starts_here(context) ||
		VariableReference::starts_here(context) ||
		UnaryOperation::starts_here(context) ||
		Tuple::starts_here(context)
    }
}

/// Parse atomic expression
pub(crate) fn parse_atomic_expression(context: &mut Context<impl Lexer>)
	-> Result<Expression, ParseError> {
	if Literal::starts_here(context) {
		return Ok(Literal::parse(context)?.into());
	}

	if Tuple::starts_here(context) {
		return Ok(Tuple::parse(context)?.into());
	}

	if VariableReference::starts_here(context) {
		return Ok(VariableReference::parse(context)?.into());
	}

	if UnaryOperation::starts_here(context) {
		return Ok(UnaryOperation::parse(context)?.into());
	}

	Err(MissingExpression {
		at: context.lexer.span().end.into(),
	}.into())
}

/// Parse right hand side of binary expression
fn parse_binary_rhs(
	context: &mut Context<impl Lexer>,
	prev_op: Option<&str>,
	mut left: Expression
)
	-> Result<Expression, ParseError>
{
	while context.lexer.peek().is_some_and(|t| t.is_operator()) {
		let op = context.lexer.consume_operator()?;

		if prev_op.is_some_and(|prev_op| context.precedence_groups.has_less_precedence(&op, prev_op)) {
			break;
		}

		let mut right = parse_atomic_expression(context)?;
		if  context.lexer.peek().is_some_and(|t| t.is_operator()) {
			let next_op = context.lexer.peek_slice();
		   	if context.precedence_groups.has_greater_precedence(next_op, &op) {
				right = parse_binary_rhs(context, Some(&op), right)?;
			}
		}

		left = BinaryOperation {
			left: Box::new(left),
			operator: op,
			right: Box::new(right)
		}.into();
	}

	Ok(left)
}

/// Parse binary expression
pub(crate) fn parse_binary_expression(context: &mut Context<impl Lexer>)
	-> Result<Expression, ParseError> {
	let left = parse_atomic_expression(context)?;
	parse_binary_rhs(context, None, left)
}

impl Parse for Expression {
    type Err = ParseError;

    /// Parse expression using lexer
    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        if !Expression::starts_here(context) {
            return Err(MissingExpression {
                at: context.lexer.span().end.into(),
            }
            .into());
        }

		let call = Call::parse(context)?;
		if call.name_parts.len() > 1 {
			return Ok(call.into())
		}

		Ok(match call.name_parts.first().unwrap() {
			CallNamePart::Argument(arg) => arg.clone(),
			CallNamePart::Text(t) => VariableReference {
				name: t.clone(),
			}.into(),
		})
    }
}

impl Ranged for Expression {
    /// Get range of expression
    fn range(&self) -> std::ops::Range<usize> {
        match self {
            Expression::Literal(l) => l.range(),
            Expression::VariableReference(var) => var.range(),
            Expression::UnaryOperation(op) => op.range(),
			Expression::BinaryOperation(op) => op.range(),
            Expression::Call(call) => call.range(),
			Expression::Tuple(tuple) => tuple.range(),
        }
    }
}
