use std::sync::Arc;

use crate::hir::{self, Type};

use super::{error::NotImplemented, FindDeclaration};

/// Trait to check if type implements trait
pub trait Implements
where
    Self: Sized,
{
    /// Does this class implement given trait?
    fn implements(&self, tr: Arc<hir::TraitDeclaration>) -> ImplementsCheck<Self> {
        ImplementsCheck { ty: self, tr }
    }
}

impl Implements for Arc<hir::TypeDeclaration> {}

/// Helper struct to do check within context
pub struct ImplementsCheck<'s, S> {
    ty: &'s S,
    tr: Arc<hir::TraitDeclaration>,
}

impl ImplementsCheck<'_, Arc<hir::TypeDeclaration>> {
    pub fn within(self, context: &impl FindDeclaration) -> Result<(), NotImplemented> {
        let unimplemented: Vec<_> = self
            .tr
            .functions
            .values()
            .filter(|f| {
                matches!(f, hir::Function::Declaration(_))
                    && context
                        .find_implementation(&f, &Type::from(self.ty.clone()))
                        .is_none()
            })
            .cloned()
            .collect();

        if !unimplemented.is_empty() {
            return Err(NotImplemented {
                ty: self.ty.clone().into(),
                tr: self.tr,
                unimplemented,
            });
        }

        Ok(())
    }
}
