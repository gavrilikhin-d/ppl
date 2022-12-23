use thiserror::Error;
use miette::{SourceSpan, Diagnostic};

use crate::{hir::Type, syntax::error};

/// Diagnostic for undefined variables
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("variable '{name}' is not defined")]
#[diagnostic(code(semantics::undefined_variable))]
pub struct UndefinedVariable {
	/// Name of undefined variable
	pub name: String,

	/// Span of name
	#[label("reference to undefined variable")]
	pub at: SourceSpan
}

/// Diagnostic for unknown type
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("unknown type '{name}'")]
#[diagnostic(code(semantics::unknown_type))]
pub struct UnknownType {
	/// Name of unknown type
	pub name: String,

	/// Span of name
	#[label("reference to unknown type")]
	pub at: SourceSpan
}

/// Diagnostic for assignment to immutable
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("assignment to immutable")]
#[diagnostic(code(semantics::assignment_to_immutable))]
pub struct AssignmentToImmutable {
	/// Span of immutable thing
	#[label("this value is immutable")]
	pub at: SourceSpan,
}

/// Diagnostic for not convertible types
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("can't convert {from:?} to {to:?}")]
#[diagnostic(code(semantics::no_conversion))]
pub struct NoConversion {
	/// Type, that must be converted
	pub from: Type,
	/// Target type
	pub to: Type,

	/// Span of `from` type
	#[label("this has {from:?} type")]
	pub from_span: SourceSpan,

	/// Span of `to` type
	#[label("this has {to:?} type")]
	pub to_span: SourceSpan
}

/// Diagnostic for unresolved unary operator
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("no unary operator '{name}'")]
#[diagnostic(code(semantics::no_unary_operator))]
pub struct NoUnaryOperator {
	/// Expected name of unary operator
	pub name: String,

	/// Type of operand
	pub operand_type: Type,

	/// Span of operator
	#[label("can't resolve this unary operator")]
	pub operator_span: SourceSpan,

	/// Span of operator
	#[label("<:{operand_type}>")]
	pub operand_span: SourceSpan,
}

/// Diagnostic for unresolved function call
#[derive(Error, Debug, Clone, PartialEq)]
#[error("no function '{name}'")]
pub struct NoFunction {
	/// Expected name of function
	pub name: String,

	/// Span of whole function call
	pub at: SourceSpan,

	/// Types of arguments
	pub arguments: Vec<(Type, SourceSpan)>,
}

impl Diagnostic for NoFunction {
	fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
		Some(Box::new("semantics::no_function"))
	}

	fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
		if self.arguments.is_empty() {
			Some(Box::new(std::iter::once(
				miette::LabeledSpan::new_with_span(
					Some("can't resolve this function call".to_string()), self.at
				)
			)))
		}
		else {
			Some(Box::new(self.arguments
				.iter()
				.map(
					|(t, s)| miette::LabeledSpan::new_with_span(
						Some(format!("<:{}>", t)), *s
					)
				)))
		}
	}
}

/// Possible semantics errors
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
pub enum Error {
	#[error(transparent)]
	#[diagnostic(transparent)]
	UndefinedVariable(#[from] UndefinedVariable),
	#[error(transparent)]
	#[diagnostic(transparent)]
	AssignmentToImmutable(#[from] AssignmentToImmutable),
	#[error(transparent)]
	#[diagnostic(transparent)]
	NoConversion(#[from] NoConversion),
	#[error(transparent)]
	#[diagnostic(transparent)]
	UnknownType(#[from] UnknownType),
	#[error(transparent)]
	#[diagnostic(transparent)]
	NoUnaryOperator(#[from] NoUnaryOperator),
	#[error(transparent)]
	#[diagnostic(transparent)]
	NoFunction(#[from] NoFunction),
}