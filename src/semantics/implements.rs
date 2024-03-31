use std::sync::Arc;

use crate::{
    hir::{self, Type},
    syntax::Ranged,
};

use super::{error::NotImplemented, Context};

/// Trait to check if type implements trait
pub trait Implements
where
    Self: Sized,
{
    /// Does this class implement given trait?
    fn implements(&self, tr: Arc<hir::TraitData>) -> ImplementsCheck<Self> {
        ImplementsCheck { ty: self, tr }
    }
}

impl Implements for hir::Class {}

/// Helper struct to do check within context
pub struct ImplementsCheck<'s, S> {
    ty: &'s S,
    tr: Arc<hir::TraitData>,
}

impl ImplementsCheck<'_, hir::Class> {
    pub fn within(self, context: &mut impl Context) -> Result<(), NotImplemented> {
        for supertrait in &self.tr.supertraits {
            self.ty.implements(supertrait.clone()).within(context)?;
        }

        let unimplemented: Vec<_> = self
            .tr
            .functions
            .values()
            .filter(|f| {
                let f = f.read().unwrap();
                !f.is_definition()
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
                unimplemented: unimplemented
                    .into_iter()
                    .map(|f| f.range().into())
                    .collect(),
            });
        }

        Ok(())
    }
}
