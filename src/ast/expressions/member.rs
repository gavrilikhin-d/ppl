extern crate ast_derive;
use ast_derive::AST;

use crate::syntax::{error::ParseError, Lexer, Parse, Ranged, StringWithOffset, Context};

use super::{Expression, parse_atomic_expression};

/// AST for member reference
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct MemberReference {
	/// Base expression
	pub base: Box<Expression>,
    /// Referenced member name
    pub name: StringWithOffset,
}

impl Parse for MemberReference {
    type Err = ParseError;

    /// Parse member reference using lexer
    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        let expr = parse_atomic_expression(context)?;

		if !matches!(expr, Expression::MemberReference(_)) {
			todo!("expected member reference error")
		}

		Ok(expr.try_into().unwrap())
    }
}

impl Ranged for MemberReference {
    fn start(&self) -> usize {
		self.base.start()
	}

	fn end(&self) -> usize {
		self.name.end()
	}
}

#[cfg(test)]
mod tests {
	use crate::ast::VariableReference;

	use super::*;

	#[test]
	fn test_one_level_referencing() {
		let m =
			"point.x"
				.parse::<MemberReference>()
				.unwrap();
		assert_eq!(
			m,
			MemberReference {
				name: StringWithOffset::from("x").at(6),
				base: Box::new(
					VariableReference {
						name: StringWithOffset::from("point"),
					}.into()
				),
			}
		);
	}

	#[test]
	fn test_multiple_level_referencing() {
		let m =
			"var.ty.name"
				.parse::<MemberReference>()
				.unwrap();
		assert_eq!(
			m,
			MemberReference {
				name: StringWithOffset::from("name").at(7),
				base: Box::new(
					MemberReference {
						name: StringWithOffset::from("ty").at(4),
						base: Box::new(
							VariableReference {
								name: StringWithOffset::from("var"),
							}.into()
						),
					}.into()
				),
			}
		);
	}
}
