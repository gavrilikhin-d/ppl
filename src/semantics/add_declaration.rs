use std::sync::Arc;

use crate::hir::{Function, TraitDeclaration, Type, TypeDeclaration, VariableDeclaration};

pub trait AddDeclaration {
    /// Get parent context
    fn parent_mut(&mut self) -> Option<&mut dyn AddDeclaration> {
        None
    }

    /// Add type to context
    fn add_type(&mut self, ty: Arc<TypeDeclaration>) {
        self.parent_mut().unwrap().add_type(ty)
    }

    /// Add trait to context
    fn add_trait(&mut self, tr: Arc<TraitDeclaration>) {
        self.parent_mut().unwrap().add_trait(tr)
    }

    /// Add function to context
    fn add_function(&mut self, f: Function) {
        self.parent_mut().unwrap().add_function(f)
    }

    /// Add variable to context
    fn add_variable(&mut self, v: Arc<VariableDeclaration>) {
        self.parent_mut().unwrap().add_variable(v)
    }

    /// Maps generic type to a concrete type.
    /// Returns previous concrete type, if any.
    fn map_generic(&mut self, generic: Type, concrete: Type) -> Option<Type> {
        self.parent_mut()
            .map(|p| p.map_generic(generic, concrete))
            .flatten()
    }
}
