mod literal;
use enum_dispatch::enum_dispatch;
pub use literal::*;

mod variable;
pub use variable::*;

mod call;
pub use call::*;

mod r#type;
pub use r#type::*;

mod member;
pub use member::*;

mod constructor;
pub use constructor::*;

use crate::{mutability::Mutable, syntax::Ranged};

/// Any PPL expression
#[enum_dispatch(Ranged, Mutable, Typed)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expression {
    Literal(Literal),
    VariableReference(VariableReference),
    Call(Call),
    TypeReference(TypeReference),
    MemberReference(MemberReference),
    Constructor(Constructor),
}

impl Expression {
    /// Check if expression is a reference to something
    pub fn is_reference(&self) -> bool {
        matches!(
            self,
            Expression::VariableReference(_)
                | Expression::MemberReference(_)
                | Expression::TypeReference(_)
        )
    }
}
