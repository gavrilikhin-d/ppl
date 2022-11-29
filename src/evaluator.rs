use rug;

use crate::syntax::ast::Literal;

/// Evaluator for PPL
pub struct Evaluator {

}

/// Value, that may be produced by the evaluator
#[derive(Debug, PartialEq)]
pub enum Value {
	None,
	Integer(rug::Integer),
}


impl Value {
	/// Is the value a none value?
	///
	/// # Example
	/// ```
	/// use ppl::evaluator::Value;
	///
	/// let value = Value::None;
	/// assert!(value.is_none());
	///
	/// let value = Value::Integer(rug::Integer::from(42));
	/// assert!(!value.is_none());
	/// ```
	pub fn is_none(&self) -> bool {
		match self {
			Value::None => true,
			_ => false,
		}
	}
}

impl From<rug::Integer> for Value {
	fn from(value: rug::Integer) -> Self {
		Value::Integer(value)
	}
}

impl Evaluator {
	/// Evaluate value for integer literal
	pub fn evaluate_integer(&self, value: &str) -> rug::Integer {
		rug::Integer::from_str_radix(value, 10).unwrap()
	}

	/// Evaluate value for literal
	///
	/// # Example
	/// ```
	/// use ppl::Evaluator;
	/// use ppl::syntax::ast::Literal;
	///
	/// let evaluator = Evaluator {};
	/// let literal = "none".parse::<Literal>().unwrap();
	/// let value = evaluator.evaluate_literal(&literal);
	///	assert!(value.is_none());
	/// ```
	pub fn evaluate_literal(&self, literal: &Literal) -> Value {
		match literal {
			Literal::None { offset: _ } => Value::None,
			Literal::Integer { offset: _, value } => self.evaluate_integer(&value).into(),
		}
	}

	/// Print value received from the evaluator
	///
	/// # Example
	/// ```
	/// use ppl::Evaluator;
	///
	/// let evaluator = Evaluator {};
	/// evaluator.print_value(evaluator.evaluate_integer("42"));
	/// ```
	pub fn print_value<V: Into<Value>>(&self, value: V) {
		match value.into() {
			Value::None => println!("none"),
			Value::Integer(value) => println!("{}", value),
		}
	}
}