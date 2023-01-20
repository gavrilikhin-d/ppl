mod assignment;
pub use assignment::*;

mod ret;
pub use ret::*;

mod r#if;
pub use r#if::*;

mod r#loop;
pub use r#loop::*;

use derive_more::{From, TryInto};

use crate::hir::{Declaration, Expression};

/// Any PPL statement
#[derive(Debug, PartialEq, Eq, Clone, From, TryInto)]
pub enum Statement {
    Declaration(Declaration),
    Expression(Expression),
    Assignment(Assignment),
	Return(Return),
	If(If),
	Loop(Loop)
}
