use std::{
    borrow::Cow,
    collections::HashMap,
    fmt::{Debug, Display},
};

use crate::{mutability::Mutable, named::Named, syntax::Identifier, AddSourceLocation};

use super::{Basename, BuiltinClass, Class, Generic, Member, Trait, TypeReference};
use derive_more::{Display, From, TryInto};
use derive_visitor::DriveMut;
use enum_dispatch::enum_dispatch;

use crate::DataHolder;

use super::Expression;

/// PPL's Function type
#[derive(Debug, PartialEq, Eq, Hash, Clone, DriveMut)]
pub struct FunctionType {
    /// Parameters
    pub parameters: Vec<Type>,
    /// Return type
    pub return_type: Box<Type>,
    /// Cached name of function type
    #[drive(skip)]
    name: String,
}

impl FunctionType {
    /// Build new function type
    pub fn build() -> FunctionTypeBuilder {
        FunctionTypeBuilder::new()
    }
}

impl Named for FunctionType {
    fn name(&self) -> Cow<'_, str> {
        self.name.as_str().into()
    }
}

impl Display for FunctionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Generic for FunctionType {
    fn is_generic(&self) -> bool {
        self.parameters.iter().any(|p| p.is_generic()) || self.return_type.is_generic()
    }
}

/// Builder for FunctionType
pub struct FunctionTypeBuilder {
    /// Parameters
    pub parameters: Vec<Type>,
}

impl FunctionTypeBuilder {
    /// Create new builder for function type
    pub fn new() -> Self {
        Self {
            parameters: Vec::new(),
        }
    }

    /// Set parameter to function type
    pub fn with_parameters(mut self, parameters: Vec<Type>) -> Self {
        self.parameters = parameters;
        self
    }

    /// Set return type to function type and build function
    pub fn with_return_type(self, return_type: Type) -> FunctionType {
        let name = self.build_name(&return_type);
        FunctionType {
            parameters: self.parameters,
            return_type: Box::new(return_type),
            name,
        }
    }

    /// Build name of function type
    fn build_name(&self, return_type: &Type) -> String {
        let mut name = String::new();
        name.push_str("(");
        for (i, parameter) in self.parameters.iter().enumerate() {
            if i != 0 {
                name.push_str(", ");
            }
            name.push_str(&parameter.name());
        }
        name.push_str(&format!(") -> {}", return_type.name()));
        name
    }
}

/// Self type is used to represent type of self in trait methods
#[derive(Debug, Clone, DriveMut)]
pub struct SelfType {
    /// Trait associated with self type
    #[drive(skip)]
    pub associated_trait: Trait,
}

impl SelfType {
    /// Create new self type for trait
    pub fn for_trait(associated_trait: Trait) -> Self {
        Self { associated_trait }
    }
}

impl PartialEq for SelfType {
    fn eq(&self, other: &Self) -> bool {
        self.associated_trait == other.associated_trait
    }
}
impl Eq for SelfType {}

impl std::hash::Hash for SelfType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.associated_trait.hash(state);
    }
}

impl Named for SelfType {
    fn name(&self) -> Cow<'_, str> {
        "Self".into()
    }
}

impl Display for SelfType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Type of a generic parameter
#[derive(Debug, Clone, PartialEq, Eq, Hash, DriveMut)]
pub struct GenericType {
    /// Name of the generic type
    #[drive(skip)]
    pub name: Identifier,
    /// Is this generic type generated by compiler?
    #[drive(skip)]
    pub generated: bool,
    /// Constraint for this type
    #[drive(skip)]
    pub constraint: Option<TypeReference>,
}

impl Named for GenericType {
    fn name(&self) -> Cow<'_, str> {
        self.name.as_str().into()
    }
}

impl Display for GenericType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            self.name(),
            self.constraint
                .as_ref()
                .map(|c| if f.sign_plus() {
                    format!(": {}", c.referenced_type)
                } else {
                    "".to_string()
                })
                .unwrap_or_default()
        )
    }
}

/// Type of values
#[derive(Debug, Display, PartialEq, Eq, Hash, Clone, From, TryInto, DriveMut)]
pub enum Type {
    /// User defined type
    Class(Class),
    /// User defined trait
    Trait(Trait),
    /// Self type and trait it represents
    SelfType(SelfType),
    /// Type for generic parameters
    Generic(Box<GenericType>),
    /// Function type
    Function(FunctionType),
    /// Type that compiler hasn't inferred yet
    Unknown,
}

impl From<GenericType> for Type {
    fn from(generic: GenericType) -> Self {
        Box::new(generic).into()
    }
}

impl Type {
    /// Get diff in specializations
    /// that needed to make `self` type to match `target` type.
    ///
    /// # Note
    /// This function ignores the fact that types may be non-generic
    pub fn diff(&self, target: Type) -> HashMap<Type, Type> {
        let from = self;
        let to = target;
        if from == &to {
            return HashMap::new();
        }

        match (&from, &to) {
            (Type::Class(from), Type::Class(to)) if from.basename() == to.basename() => from
                .read()
                .unwrap()
                .generics()
                .iter()
                .zip(to.read().unwrap().generics().iter())
                .flat_map(|(t1, t2)| t1.diff(t2.clone()))
                .collect(),
            _ => HashMap::from_iter(std::iter::once((from.clone(), to))),
        }
    }

    /// Get this type without reference
    pub fn without_ref(&self) -> Type {
        if !self.is_any_reference() {
            return self.clone();
        }

        self.generics()[0].clone()
    }

    /// Get generic parameters of type
    pub fn generics(&self) -> Vec<Type> {
        match self {
            Type::Class(c) => c.read().unwrap().generics().into(),
            _ => vec![],
        }
    }

    /// Get members of type
    pub fn members(&self) -> Vec<Member> {
        match self {
            Type::Class(c) => c.read().unwrap().members().into(),
            _ => vec![],
        }
    }

    /// Is this a builtin type?
    pub fn is_builtin(&self) -> bool {
        match self {
            Type::Class(c) => c.is_builtin(),
            _ => false,
        }
    }

    /// Get builtin class tag for this type
    pub fn builtin(&self) -> Option<BuiltinClass> {
        match self {
            Type::Class(c) => c.read().unwrap().builtin.clone(),
            _ => None,
        }
    }

    /// Is this a builtin "None" type?
    pub fn is_none(&self) -> bool {
        match self.without_ref() {
            Type::Class(c) => c.is_none(),
            _ => false,
        }
    }

    /// Is this a builtin "Bool" type?
    pub fn is_bool(&self) -> bool {
        match self.without_ref() {
            Type::Class(c) => c.is_bool(),
            _ => false,
        }
    }

    /// Is this a builtin `I32` type?
    pub fn is_i32(&self) -> bool {
        match self.without_ref() {
            Type::Class(c) => c.read().unwrap().is_i32(),
            _ => false,
        }
    }

    /// Is this a builtin "Integer" type?
    pub fn is_integer(&self) -> bool {
        match self.without_ref() {
            Type::Class(c) => c.read().unwrap().is_integer(),
            _ => false,
        }
    }

    /// Is this a builtin "String" type?
    pub fn is_string(&self) -> bool {
        match self.without_ref() {
            Type::Class(c) => c.read().unwrap().is_string(),
            _ => false,
        }
    }

    /// Is this a builtin `Reference` or `ReferenceMut` type?
    pub fn is_any_reference(&self) -> bool {
        match self {
            Type::Class(c) => c.read().unwrap().is_any_reference(),
            _ => false,
        }
    }

    /// Convert this to class type
    /// # Panics
    /// Panics if this is not a class type
    pub fn as_class(self) -> Class {
        self.try_into().unwrap()
    }

    /// Convert this to trait tpy
    /// # Panics
    /// Panics if this is not a trait type
    pub fn as_trait(self) -> Trait {
        self.try_into().unwrap()
    }

    /// Size of type in bytes
    pub fn size_in_bytes(&self) -> usize {
        match self {
            Type::Class(c) => c.read().unwrap().size_in_bytes(),
            // TODO: implement size for other types
            _ => 0,
        }
    }
}

impl Generic for Type {
    fn is_generic(&self) -> bool {
        match self {
            Type::SelfType(_) | Type::Trait(_) | Type::Generic(_) => true,
            Type::Class(c) => c.read().unwrap().is_generic(),
            Type::Function(f) => f.is_generic(),
            Type::Unknown => unreachable!("Trying to check if not inferred type is generic"),
        }
    }
}

impl Basename for Type {
    fn basename(&self) -> Cow<'_, str> {
        match self {
            Type::Class(c) => c.read().unwrap().basename().to_string().into(),
            _ => self.name(),
        }
    }
}

impl Named for Type {
    fn name(&self) -> Cow<'_, str> {
        match self {
            Type::Class(class) => class.read().unwrap().name().to_string().into(),
            Type::Trait(tr) => tr.name(),
            Type::SelfType(s) => s.name(),
            Type::Function(f) => f.name(),
            Type::Generic(g) => g.name(),
            Type::Unknown => "Unknown".into(),
        }
    }
}

impl Mutable for Type {
    fn is_mutable(&self) -> bool {
        match self {
            Type::Class(c) => c.read().unwrap().is_mutable(),
            _ => false,
        }
    }
}

impl AddSourceLocation for Type {}

/// Trait for values with a type
#[enum_dispatch]
pub trait Typed {
    /// Get type of value
    fn ty(&self) -> Type;
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use pretty_assertions::{assert_eq, assert_str_eq};

    use crate::{
        ast,
        hir::{Class, GenericType, SpecializeParameters, Type},
        named::Named,
        semantics::ToHIR,
    };

    /// Get type declaration from source
    fn type_decl(source: &str) -> Class {
        source
            .parse::<ast::TypeDeclaration>()
            .unwrap()
            .to_hir_without_context()
            .unwrap()
    }

    #[test]
    fn generic_name() {
        let a = type_decl("type A<T, U>");
        assert_str_eq!(a.name(), "A<T, U>");

        let b = type_decl("type B<T>");
        assert_str_eq!(b.name(), "B<T>");

        let x: Type = type_decl("type X").into();
        assert_str_eq!(x.name(), "X");
        let y: Type = type_decl("type Y").into();
        assert_str_eq!(y.name(), "Y");

        // B<Y>
        let by: Type = b.specialize_parameters(std::iter::once(y.clone())).into();
        assert_str_eq!(by.name(), "B<Y>");

        // A<X, B<Y>>
        let t1 = a.specialize_parameters(vec![x, by]);
        assert_str_eq!(t1.name(), "A<X, B<Y>>");
    }

    #[test]
    fn diff() {
        let a = type_decl("type A<T, U>");
        let b = type_decl("type B<T>");
        let c: Type = type_decl("type C").into();

        let x: Type = GenericType {
            name: "X".into(),
            generated: false,
            constraint: None,
        }
        .into();
        let y: Type = GenericType {
            name: "Y".into(),
            generated: false,
            constraint: None,
        }
        .into();

        let by: Type = b.clone().specialize_parameters(vec![y.clone()]).into();
        println!("{}", by.name());

        // A<X, B<Y>>
        let t1: Type = a
            .clone()
            .specialize_parameters(vec![x.clone(), by.clone()])
            .into();
        println!("{}", t1.name());

        // B<C>
        let bc: Type = b.specialize_parameters(vec![c.clone()]).into();
        // A<B<C>, B<C>>
        let t2: Type = a.specialize_parameters(vec![bc.clone(), bc.clone()]).into();
        let diff = t1.diff(t2);
        assert_eq!(diff, HashMap::from_iter(vec![(x, bc), (y, c)]));
    }
}
