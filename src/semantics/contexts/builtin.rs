use std::fmt::Display;

use crate::{
    hir::{ModuleData, SpecializeParameters, Type},
    named::Named,
};

/// Helper struct to get builtin things
pub struct BuiltinContext<'m> {
    /// Builtin module
    pub module: &'m ModuleData,
}

impl Display for BuiltinContext<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "BuiltinContext")
    }
}

impl<'m> BuiltinContext<'m> {
    /// Get builtin types
    pub fn types(&self) -> BuiltinTypes<'m> {
        BuiltinTypes {
            module: self.module,
        }
    }
}

impl AsRef<ModuleData> for BuiltinContext<'_> {
    fn as_ref(&self) -> &ModuleData {
        self.module
    }
}

/// Helper struct to get builtin types
pub struct BuiltinTypes<'m> {
    /// Builtin module
    module: &'m ModuleData,
}

/// Helper macro to add builtin types
macro_rules! builtin_types {
    ($($name: ident),*) => {
        $(pub fn $name(&self) -> Type {
            let name = stringify!($name);
            self.get_type(&format!("{}{}", name[0..1].to_uppercase(), &name[1..]))
        })*
    };
}

impl BuiltinTypes<'_> {
    /// Get builtin type by name
    fn get_type(&self, name: &str) -> Type {
        debug_assert!(self.module.name() == "ppl", "Wrong module used as builtin");
        self.module
            .types
            .get(name)
            .expect(
                format!(
                    "Builtin type `{name}` should be present. Present types: {}",
                    self.module
                        .types
                        .iter()
                        .map(|t| t.0.clone())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
                .as_str(),
            )
            .clone()
            .into()
    }

    builtin_types!(none, bool, integer, rational, string, reference, i32, f64);

    /// Get builtin type for types
    pub fn type_(&self) -> Type {
        self.get_type("Type")
    }

    /// Get `Type<T>` of this type
    pub fn type_of(&self, ty: Type) -> Type {
        self.type_()
            .as_class()
            .specialize_parameters(std::iter::once(ty))
            .into()
    }

    /// Get `Reference<T>` for this type
    pub fn reference_to(&self, ty: Type) -> Type {
        self.reference()
            .as_class()
            .specialize_parameters(std::iter::once(ty))
            .into()
    }

    /// Get builtin type for `ReferenceMut<T>`
    pub fn reference_mut(&self) -> Type {
        self.get_type("ReferenceMut")
    }

    /// Get `ReferenceMut<T>` for this type
    pub fn reference_mut_to(&self, ty: Type) -> Type {
        self.reference_mut()
            .as_class()
            .specialize_parameters(std::iter::once(ty))
            .into()
    }
}

#[cfg(test)]
mod test {
    use crate::{compilation::Compiler, hir::ClassData, named::Named};

    use super::BuiltinTypes;

    use pretty_assertions::{assert_eq, assert_str_eq};

    #[test]
    fn type_of() {
        let compiler = Compiler::new();
        let builtin = BuiltinTypes {
            module: compiler.builtin_module().unwrap(),
        };
        let none = builtin.none();
        let ty = builtin.type_();
        let none_ty = builtin.type_of(none.clone());
        assert_str_eq!(none_ty.name(), "Type<None>");
        assert_eq!(
            *none_ty.clone().as_class().read().unwrap(),
            ClassData {
                specialization_of: Some(ty.clone().as_class()),
                generic_parameters: vec![none.clone().into()],
                ..ty.clone().as_class().read().unwrap().clone()
            }
        );

        let type_of_type = builtin.type_of(none_ty.clone());
        assert_str_eq!(type_of_type.name(), "Type<Type<None>>");
        assert_eq!(
            *type_of_type.clone().as_class().read().unwrap(),
            ClassData {
                specialization_of: Some(ty.clone().as_class()),
                generic_parameters: vec![none_ty.clone().into()],
                ..ty.clone().as_class().read().unwrap().clone()
            }
        );
    }
}
