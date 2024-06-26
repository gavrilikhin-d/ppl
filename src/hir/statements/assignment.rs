use std::fmt::Display;

use derive_visitor::DriveMut;

use crate::{hir::Expression, syntax::Ranged};

/// Assignment of a value to a reference
#[derive(Debug, PartialEq, Eq, Clone, DriveMut)]
pub struct Assignment {
    /// Reference to assign to
    pub target: Expression,
    /// Value to assign
    pub value: Expression,
}

impl Display for Assignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let indent = "\t".repeat(f.width().unwrap_or(0));
        write!(f, "{indent}")?;

        write!(f, "{} = {}", self.target, self.value)
    }
}

impl Ranged for Assignment {
    fn start(&self) -> usize {
        self.target.start()
    }

    fn end(&self) -> usize {
        self.value.end()
    }
}
