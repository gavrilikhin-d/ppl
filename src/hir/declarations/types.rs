use std::{
    borrow::Cow,
    fmt::Display,
    hash::Hash,
    ops::Range,
    str::FromStr,
    sync::{Arc, RwLock},
};

use derive_visitor::DriveMut;

use crate::{
    hir::{Basename, Generic, Type, Typed},
    mutability::Mutable,
    named::Named,
    syntax::{Identifier, Keyword, Ranged},
    AddSourceLocation,
};

use crate::DataHolder;

/// Member data holder
#[derive(Debug, Clone)]
pub struct Member {
    inner: Arc<RwLock<MemberData>>,
}

impl DataHolder for Member {
    type Data = MemberData;

    fn new(data: Self::Data) -> Self {
        Self {
            inner: Arc::new(RwLock::new(data)),
        }
    }

    fn inner(&self) -> &Arc<RwLock<Self::Data>> {
        &self.inner
    }
}

impl PartialEq for Member {
    fn eq(&self, other: &Self) -> bool {
        *self.read().unwrap() == *other.read().unwrap()
    }
}

impl Eq for Member {}

impl Named for Member {
    fn name(&self) -> Cow<'_, str> {
        self.read().unwrap().name().to_string().into()
    }
}

impl Generic for Member {
    fn is_generic(&self) -> bool {
        self.read().unwrap().is_generic()
    }
}

impl Typed for Member {
    fn ty(&self) -> Type {
        self.read().unwrap().ty()
    }
}

impl Ranged for Member {
    fn range(&self) -> std::ops::Range<usize> {
        self.read().unwrap().range()
    }
}

impl DriveMut for Member {
    fn drive_mut<V: derive_visitor::VisitorMut>(&mut self, visitor: &mut V) {
        derive_visitor::VisitorMut::visit(visitor, self, ::derive_visitor::Event::Enter);
        self.write().unwrap().drive_mut(visitor);
        derive_visitor::VisitorMut::visit(visitor, self, ::derive_visitor::Event::Exit);
    }
}

impl Hash for Member {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.read().unwrap().hash(state)
    }
}

/// Member of type
#[derive(Debug, PartialEq, Eq, Hash, Clone, DriveMut)]
pub struct MemberData {
    /// Member's name
    #[drive(skip)]
    pub name: Identifier,
    /// Member's type
    pub ty: Type,
}

impl Generic for MemberData {
    /// Is this a generic member?
    fn is_generic(&self) -> bool {
        self.ty.is_generic()
    }
}

impl Named for MemberData {
    /// Get name of member
    fn name(&self) -> Cow<'_, str> {
        self.name.as_str().into()
    }
}

impl Typed for MemberData {
    /// Get type of member
    fn ty(&self) -> Type {
        self.ty.clone()
    }
}

impl Ranged for MemberData {
    fn start(&self) -> usize {
        self.name.start()
    }

    fn end(&self) -> usize {
        // FIXME: Replace type with type reference and use `self.ty.end()` here
        self.name.end()
    }
}

/// Size of pointer in bytes
const POINTER_SIZE: usize = 8;

macro_rules! builtin_class {
    ($($name:ident),+) => {
        /// Enum of all builtin classes
        #[derive(Debug, PartialEq, Eq, Hash, Clone)]
        pub enum BuiltinClass {
            $($name),+
        }

        impl FromStr for BuiltinClass {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                use BuiltinClass::*;
                Ok(match s {
                    $(stringify!($name) => $name,)+
                    _ => return Err(format!("Invalid builtin type `{s}`")),
                })
            }
        }
    };
}

builtin_class! {
    None,
    Bool,
    I32,
    F64,
    Integer,
    Rational,
    String,
    Reference,
    ReferenceMut
}

impl BuiltinClass {
    /// Get size in bytes for this type
    pub fn size_in_bytes(&self) -> usize {
        use BuiltinClass::*;
        match self {
            None => 0,
            Bool => 1,
            I32 => 4,
            F64 => 8,
            Integer | Rational | String | Reference | ReferenceMut => POINTER_SIZE,
        }
    }
}

/// Class data holder
#[derive(Debug, Clone)]
pub struct Class {
    inner: Arc<RwLock<ClassData>>,
}

impl DataHolder for Class {
    type Data = ClassData;

    fn new(data: Self::Data) -> Self {
        Class {
            inner: Arc::new(RwLock::new(data)),
        }
    }

    fn inner(&self) -> &Arc<RwLock<Self::Data>> {
        &self.inner
    }
}

impl Class {
    /// Is this a builtin type?
    pub fn is_builtin(&self) -> bool {
        self.read().unwrap().is_builtin()
    }

    /// Is this a builtin "None" type?
    pub fn is_none(&self) -> bool {
        self.read().unwrap().is_none()
    }

    /// Is this a builtin "Bool" type?
    pub fn is_bool(&self) -> bool {
        self.read().unwrap().is_bool()
    }

    /// Is this a builtin `I32` type?
    pub fn is_i32(&self) -> bool {
        self.read().unwrap().is_i32()
    }

    /// Is this a builtin "Integer" type?
    pub fn is_integer(&self) -> bool {
        self.read().unwrap().is_integer()
    }

    /// Is this a builtin "Rational" type?
    pub fn is_rational(&self) -> bool {
        self.read().unwrap().is_rational()
    }

    /// Is this a builtin "String" type?
    pub fn is_string(&self) -> bool {
        self.read().unwrap().is_string()
    }

    /// Is this a builtin `Reference` or `ReferenceMut` type?
    pub fn is_any_reference(&self) -> bool {
        self.read().unwrap().is_any_reference()
    }

    /// Is this an opaque type?
    pub fn is_opaque(&self) -> bool {
        self.read().unwrap().is_opaque()
    }

    /// Get size in bytes for this type
    pub fn size_in_bytes(&self) -> usize {
        self.read().unwrap().size_in_bytes()
    }
}

impl PartialEq for Class {
    fn eq(&self, other: &Self) -> bool {
        *self.read().unwrap() == *other.read().unwrap()
    }
}

impl Eq for Class {}

impl Hash for Class {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.read().unwrap().hash(state)
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read().unwrap().fmt(f)
    }
}

impl Generic for Class {
    /// Is this a generic type?
    fn is_generic(&self) -> bool {
        self.read().unwrap().is_generic()
    }
}

impl Basename for Class {
    fn basename(&self) -> Cow<'_, str> {
        self.read().unwrap().basename().to_string().into()
    }
}

impl AddSourceLocation for Class {}

impl Named for Class {
    /// Get name of type
    fn name(&self) -> Cow<'_, str> {
        self.read().unwrap().name().to_string().into()
    }
}

impl Mutable for Class {
    fn is_mutable(&self) -> bool {
        self.read().unwrap().is_mutable()
    }
}

impl Ranged for Class {
    fn range(&self) -> Range<usize> {
        self.read().unwrap().range()
    }
}

impl DriveMut for Class {
    fn drive_mut<V: derive_visitor::VisitorMut>(&mut self, visitor: &mut V) {
        derive_visitor::VisitorMut::visit(visitor, self, ::derive_visitor::Event::Enter);
        self.write().unwrap().drive_mut(visitor);
        derive_visitor::VisitorMut::visit(visitor, self, ::derive_visitor::Event::Exit);
    }
}

/// Declaration of a type
#[derive(Debug, PartialEq, Eq, Hash, Clone, DriveMut)]
pub struct ClassData {
    /// Keyword `type`
    #[drive(skip)]
    pub keyword: Keyword<"type">,
    /// Type's name
    #[drive(skip)]
    pub basename: Identifier,
    /// Base generic type, if this is a specialization
    #[drive(skip)]
    pub specialization_of: Option<Class>,
    /// Generic parameters of type
    pub generic_parameters: Vec<Type>,
    /// Kind of a builtin type, if it is a builtin class
    #[drive(skip)]
    pub builtin: Option<BuiltinClass>,
    /// Members of type
    pub members: Vec<Member>,
}

impl ClassData {
    /// Get member by name
    pub fn members(&self) -> &[Member] {
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

    /// Is this a builtin `I32` type?
    pub fn is_i32(&self) -> bool {
        self.builtin == Some(BuiltinClass::I32)
    }

    /// Is this a builtin `I32` type?
    pub fn is_f64(&self) -> bool {
        self.builtin == Some(BuiltinClass::F64)
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
            .map(|m| m.ty().size_in_bytes())
            .sum::<usize>()
    }
}

impl Generic for ClassData {
    /// Is this a generic type?
    fn is_generic(&self) -> bool {
        self.generic_parameters.iter().any(|p| p.is_generic())
            || self.members.iter().any(|m| m.is_generic())
    }
}

impl Basename for ClassData {
    fn basename(&self) -> Cow<'_, str> {
        self.basename.as_str().into()
    }
}

impl Display for ClassData {
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
                writeln!(f, "{}: {}", member.name(), member.ty())?;
            }
        } else {
            write!(f, "{}", self.name())?;
        }
        Ok(())
    }
}

impl Named for ClassData {
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

impl Mutable for ClassData {
    fn is_mutable(&self) -> bool {
        match self.builtin {
            Some(BuiltinClass::ReferenceMut) => true,
            _ => false,
        }
    }
}

impl Ranged for ClassData {
    fn start(&self) -> usize {
        self.keyword.start()
    }

    fn end(&self) -> usize {
        self.members
            .last()
            .map_or_else(|| self.basename.end(), |m| m.end())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast;
    use crate::compilation::Compiler;
    use crate::hir::{GenericType, ModuleData};
    use crate::semantics::{Context, ModuleContext, ToHIR};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_type_without_body() {
        let type_decl = "type x"
            .parse::<ast::TypeDeclaration>()
            .unwrap()
            .to_hir_without_context()
            .unwrap();

        assert_eq!(
            *type_decl.read().unwrap(),
            ClassData {
                keyword: Keyword::<"type">::at(0),
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
            *type_decl.read().unwrap(),
            ClassData {
                keyword: Keyword::<"type">::at(0),
                basename: Identifier::from("Point").at(5),
                specialization_of: None,
                generic_parameters: vec![GenericType {
                    name: Identifier::from("U").at(11),
                    generated: false,
                    constraint: None
                }
                .into()],
                builtin: None,
                members: vec![Member::new(MemberData {
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
        let mut context = ModuleContext::new(ModuleData::default(), &mut compiler);
        let type_decl = include_str!("../../../examples/point.ppl")
            .parse::<ast::TypeDeclaration>()
            .unwrap()
            .to_hir(&mut context)
            .unwrap();

        let integer: Type = context.builtin().types().integer();

        assert_eq!(
            *type_decl.read().unwrap(),
            ClassData {
                keyword: Keyword::<"type">::at(0),
                basename: Identifier::from("Point").at(5),
                specialization_of: None,
                generic_parameters: vec![],
                builtin: None,
                members: vec![
                    Member::new(MemberData {
                        name: Identifier::from("x").at(13),
                        ty: integer.clone(),
                    }),
                    Member::new(MemberData {
                        name: Identifier::from("y").at(16),
                        ty: integer,
                    }),
                ],
            }
        );
    }
}
