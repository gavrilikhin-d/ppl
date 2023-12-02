use std::{borrow::Cow, fmt::Display, str::FromStr, sync::Arc};

use crate::{
    hir::{Basename, Generic, Type, Typed},
    mutability::Mutable,
    named::Named,
    syntax::Identifier,
    AddSourceLocation,
};

/// Member of type
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Member {
    /// Member's name
    pub name: Identifier,
    /// Member's type
    pub ty: Type,
}

impl Generic for Member {
    /// Is this a generic member?
    fn is_generic(&self) -> bool {
        self.ty.is_generic()
    }
}

impl Named for Member {
    /// Get name of member
    fn name(&self) -> Cow<'_, str> {
        self.name.as_str().into()
    }
}

impl Typed for Member {
    /// Get type of member
    fn ty(&self) -> Type {
        self.ty.clone()
    }
}

/// Size of pointer in bytes
const POINTER_SIZE: usize = 8;

/// Enum of all builtin classes
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum BuiltinClass {
    None,
    Bool,
    Integer,
    Rational,
    String,
    Reference,
    ReferenceMut,
    Array,
}

impl BuiltinClass {
    /// Get size in bytes for this type
    pub fn size_in_bytes(&self) -> usize {
        use BuiltinClass::*;
        match self {
            None => 0,
            Bool => 1,
            Integer | Rational | String | Reference | ReferenceMut | Array => POINTER_SIZE,
        }
    }
}

impl FromStr for BuiltinClass {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use BuiltinClass::*;
        Ok(match s {
            "None" => None,
            "Bool" => Bool,
            "Integer" => Integer,
            "Rational" => Rational,
            "String" => String,
            "Reference" => Reference,
            "ReferenceMut" => ReferenceMut,
            "Array" => Array,
            _ => return Err(format!("Invalid builtin type `{s}`")),
        })
    }
}

/// Declaration of a type
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct TypeDeclaration {
    /// Type's name
    pub basename: Identifier,
    /// Base generic type, if this is a specialization
    pub specialization_of: Option<Arc<TypeDeclaration>>,
    /// Generic parameters of type
    pub generic_parameters: Vec<Type>,
    /// Kind of a builtin type, if it is a builtin class
    pub builtin: Option<BuiltinClass>,
    /// Members of type
    pub members: Vec<Arc<Member>>,
}

impl TypeDeclaration {
    /// Get member by name
    pub fn members(&self) -> &[Arc<Member>] {
        self.members.as_slice()
    }

    /// Get generic parameters of a type
    pub fn generics(&self) -> &[Type] {
        self.generic_parameters.as_slice()
    }

    /// Is this a builtin type?
    pub fn is_builtin(&self) -> bool {
        self.builtin.is_some()
    }

    /// Is this a builtin "None" type?
    pub fn is_none(&self) -> bool {
        self.builtin == Some(BuiltinClass::None)
    }

    /// Is this a builtin "Bool" type?
    pub fn is_bool(&self) -> bool {
        self.builtin == Some(BuiltinClass::Bool)
    }

    /// Is this a builtin "Integer" type?
    pub fn is_integer(&self) -> bool {
        self.builtin == Some(BuiltinClass::Integer)
    }

    /// Is this a builtin "Rational" type?
    pub fn is_rational(&self) -> bool {
        self.builtin == Some(BuiltinClass::Rational)
    }

    /// Is this a builtin "String" type?
    pub fn is_string(&self) -> bool {
        self.builtin == Some(BuiltinClass::String)
    }

    /// Is this a builtin `Reference` or `ReferenceMut` type?
    pub fn is_any_reference(&self) -> bool {
        matches!(
            self.builtin,
            Some(BuiltinClass::Reference | BuiltinClass::ReferenceMut)
        )
    }

    /// Is this an opaque type?
    pub fn is_opaque(&self) -> bool {
        self.members.is_empty()
    }

    /// Get size in bytes for this type
    pub fn size_in_bytes(&self) -> usize {
        if let Some(builtin) = &self.builtin {
            return builtin.size_in_bytes();
        }

        if self.is_opaque() {
            return POINTER_SIZE;
        }

        self.members
            .iter()
            .map(|m| m.ty.size_in_bytes())
            .sum::<usize>()
    }
}

impl Generic for TypeDeclaration {
    /// Is this a generic type?
    fn is_generic(&self) -> bool {
        self.generic_parameters.iter().any(|p| p.is_generic())
            || self.members.iter().any(|m| m.is_generic())
    }
}

impl Basename for TypeDeclaration {
    fn basename(&self) -> Cow<'_, str> {
        self.basename.as_str().into()
    }
}

impl Display for TypeDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            let indent = f.width().unwrap_or(0);
            write!(f, "{}", "\t".repeat(indent))?;

            write!(f, "type {}", self.name())?;
            if self.members.is_empty() {
                return Ok(());
            }

            writeln!(f, ":")?;
            for member in &self.members {
                write!(f, "{}", "\t".repeat(indent + 1))?;
                writeln!(f, "{}: {}", member.name, member.ty)?;
            }
        } else {
            write!(f, "{}", self.name())?;
        }
        Ok(())
    }
}

impl AddSourceLocation for Arc<TypeDeclaration> {}

impl Named for TypeDeclaration {
    /// Get name of type
    fn name(&self) -> Cow<'_, str> {
        if self.generic_parameters.is_empty() {
            return self.basename();
        }

        format!(
            "{}<{}>",
            self.basename.as_str(),
            self.generic_parameters
                .iter()
                .map(|p| p.name())
                .collect::<Vec<_>>()
                .join(", ")
        )
        .into()
    }
}

impl Mutable for TypeDeclaration {
    fn is_mutable(&self) -> bool {
        match self.builtin {
            Some(BuiltinClass::ReferenceMut) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast;
    use crate::compilation::Compiler;
    use crate::hir::{GenericType, Member, Type};
    use crate::semantics::{ModuleContext, ToHIR};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_type_without_body() {
        let type_decl = "type x"
            .parse::<ast::TypeDeclaration>()
            .unwrap()
            .to_hir_without_context()
            .unwrap();

        assert_eq!(
            *type_decl,
            TypeDeclaration {
                basename: Identifier::from("x").at(5),
                specialization_of: None,
                generic_parameters: vec![],
                builtin: None,
                members: vec![],
            }
        );
    }

    #[test]
    fn type_with_generics() {
        let type_decl = "type Point<U>:\n\tx: U"
            .parse::<ast::TypeDeclaration>()
            .unwrap()
            .to_hir_without_context()
            .unwrap();

        assert_eq!(
            *type_decl,
            TypeDeclaration {
                basename: Identifier::from("Point").at(5),
                specialization_of: None,
                generic_parameters: vec![GenericType {
                    name: Identifier::from("U").at(11),
                    generated: false,
                    constraint: None
                }
                .into()],
                builtin: None,
                members: vec![Arc::new(Member {
                    name: Identifier::from("x").at(16),
                    ty: GenericType {
                        name: Identifier::from("U").at(11),
                        generated: false,
                        constraint: None,
                    }
                    .into(),
                }),],
            }
        );
    }

    #[test]
    fn test_type_with_body() {
        let mut compiler = Compiler::new();
        let mut context = ModuleContext::new(&mut compiler);
        let type_decl = include_str!("../../../examples/point.ppl")
            .parse::<ast::TypeDeclaration>()
            .unwrap()
            .to_hir(&mut context)
            .unwrap();

        let integer: Type = compiler
            .builtin_module()
            .unwrap()
            .types
            .get("Integer")
            .cloned()
            .unwrap()
            .into();

        assert_eq!(
            *type_decl,
            TypeDeclaration {
                basename: Identifier::from("Point").at(5),
                specialization_of: None,
                generic_parameters: vec![],
                builtin: None,
                members: vec![
                    Arc::new(Member {
                        name: Identifier::from("x").at(13),
                        ty: integer.clone(),
                    }),
                    Arc::new(Member {
                        name: Identifier::from("y").at(16),
                        ty: integer,
                    }),
                ],
            }
        );
    }
}
