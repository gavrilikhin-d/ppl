use std::fmt::Display;

use derive_visitor::DriveMut;

use crate::hir::{Generic, Type, Typed};
use crate::mutability::Mutable;
use crate::syntax::Ranged;

use super::Expression;

/// Kind of implicit conversion
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ImplicitConversionKind {
    /// Convert to reference
    Reference,
    /// Dereference a reference
    Dereference,
    /// Copy or clone a value
    Copy,
}

#[derive(Debug, PartialEq, Eq, Clone, DriveMut)]
pub struct ImplicitConversion {
    /// Kind of conversion
    #[drive(skip)]
    pub kind: ImplicitConversionKind,
    /// Type of converted expression
    #[drive(skip)]
    pub ty: Type,
    /// Expression to convert
    pub expression: Box<Expression>,
}

impl Display for ImplicitConversion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ImplicitConversionKind::*;
        let op = match self.kind {
            Reference => "&",
            Dereference => "*",
            Copy => "copy ",
        };
        write!(
            f,
            "({op}{expr:#}:{ty})",
            expr = self.expression,
            ty = self.ty
        )
    }
}

impl Ranged for ImplicitConversion {
    fn range(&self) -> std::ops::Range<usize> {
        self.expression.range()
    }
}

impl Generic for ImplicitConversion {
    fn is_generic(&self) -> bool {
        self.expression.is_generic()
    }
}

impl Mutable for ImplicitConversion {
    fn is_mutable(&self) -> bool {
        self.expression.is_mutable()
    }
}
impl Typed for ImplicitConversion {
    fn ty(&self) -> Type {
        self.ty.clone()
    }
}
