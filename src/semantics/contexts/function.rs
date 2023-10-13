use std::sync::Arc;

use crate::{
    hir::{
        Function, FunctionDeclaration, ParameterOrVariable, TraitDeclaration, TypeDeclaration,
        VariableDeclaration,
    },
    named::Named,
    semantics::{AddDeclaration, FindDeclaration, FindDeclarationHere},
};

use super::Context;

/// Context for lowering body of function
pub struct FunctionContext<'p> {
    /// Function, which is being lowered
    pub function: Arc<FunctionDeclaration>,

    /// Parent context for this function
    pub parent: &'p mut dyn Context,
}

impl FindDeclarationHere for FunctionContext<'_> {
    fn find_variable_here(&self, name: &str) -> Option<ParameterOrVariable> {
        self.function
            .parameters()
            .find(|p| p.name() == name)
            .map(|p| p.into())
    }
}

impl FindDeclaration for FunctionContext<'_> {
    fn parent(&self) -> Option<&dyn FindDeclaration> {
        Some(self.parent as _)
    }
}

impl AddDeclaration for FunctionContext<'_> {
    fn parent_mut(&mut self) -> Option<&mut dyn AddDeclaration> {
        Some(self.parent)
    }

    fn add_type(&mut self, _ty: Arc<TypeDeclaration>) {
        todo!("local types")
    }

    fn add_trait(&mut self, _tr: Arc<TraitDeclaration>) {
        todo!("local traits")
    }

    fn add_function(&mut self, f: Function) {
        // TODO: local functions
        self.parent.add_function(f)
    }

    fn add_variable(&mut self, _v: Arc<VariableDeclaration>) {
        todo!("local variables")
    }
}

impl Context for FunctionContext<'_> {
    fn parent(&self) -> Option<&dyn Context> {
        Some(self.parent)
    }

    fn parent_mut(&mut self) -> Option<&mut dyn Context> {
        Some(self.parent)
    }

    fn function(&self) -> Option<Arc<FunctionDeclaration>> {
        Some(self.function.clone())
    }
}
