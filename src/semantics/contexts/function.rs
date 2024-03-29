use std::{fmt::Display, sync::Arc};

use crate::{
    hir::{Class, Function, ParameterOrVariable, TraitDeclaration, Type, Variable},
    named::Named,
    semantics::{AddDeclaration, FindDeclaration, FindDeclarationHere},
};

use super::Context;

/// Context for lowering body of function
pub struct FunctionContext<'p> {
    /// Function, which is being lowered
    pub function: Function,

    /// Local variables declared so far
    pub variables: Vec<Variable>,

    /// Parent context for this function
    pub parent: &'p mut dyn Context,
}

impl Display for FunctionContext<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "FunctionContext")?;
        writeln!(f, "\tfor function: {}", self.function.name())
    }
}

impl FindDeclarationHere for FunctionContext<'_> {
    fn find_variable_here(&self, name: &str) -> Option<ParameterOrVariable> {
        self.variables
            .iter()
            .cloned()
            .find(|p| p.name() == name)
            .map(|p| p.into())
            .or_else(|| {
                self.function
                    .read()
                    .unwrap()
                    .parameters()
                    .find(|p| p.name() == name)
                    .map(|p| p.into())
            })
    }

    fn find_type_here(&self, name: &str) -> Option<Type> {
        self.function
            .read()
            .unwrap()
            .generic_types
            .iter()
            .find(|p| p.name() == name)
            .cloned()
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

    fn add_type(&mut self, _ty: Class) {
        todo!("local types")
    }

    fn add_trait(&mut self, _tr: Arc<TraitDeclaration>) {
        todo!("local traits")
    }

    fn add_function(&mut self, f: Function) {
        // TODO: local functions
        self.parent.add_function(f)
    }

    fn add_variable(&mut self, v: Variable) {
        self.variables.push(v)
    }
}

impl Context for FunctionContext<'_> {
    fn parent(&self) -> Option<&dyn Context> {
        Some(self.parent)
    }

    fn parent_mut(&mut self) -> Option<&mut dyn Context> {
        Some(self.parent)
    }

    fn function(&self) -> Option<Function> {
        Some(self.function.clone())
    }
}
