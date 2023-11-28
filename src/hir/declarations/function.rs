use std::borrow::Cow;
use std::fmt::Display;
use std::ops::Range;
use std::sync::Arc;

use derive_more::{Display, From, TryInto};

use crate::hir::{FunctionType, Generic, Statement, Type, TypeReference, Typed};
use crate::mutability::Mutable;
use crate::named::Named;
use crate::syntax::{Identifier, Ranged};

/// Declaration of a function parameter
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Parameter {
    /// Type's name
    pub name: Identifier,
    /// Type of parameter
    pub ty: TypeReference,
    /// Range of the whole parameter
    pub range: Range<usize>,
}

impl Ranged for Parameter {
    fn range(&self) -> std::ops::Range<usize> {
        self.range.clone()
    }
}

impl Generic for Parameter {
    /// Is this a generic parameter?
    fn is_generic(&self) -> bool {
        self.ty.referenced_type.is_generic()
    }
}

impl Named for Parameter {
    /// Get name of parameter
    fn name(&self) -> Cow<'_, str> {
        self.name.as_str().into()
    }
}

impl Typed for Parameter {
    /// Get type of parameter
    fn ty(&self) -> Type {
        self.ty.referenced_type.clone()
    }
}

impl Mutable for Parameter {
    fn is_mutable(&self) -> bool {
        self.ty.is_mutable()
    }
}

/// Part of a function name
#[derive(Debug, PartialEq, Eq, Clone, From)]
pub enum FunctionNamePart {
    Text(Identifier),
    Parameter(Arc<Parameter>),
}

impl Display for FunctionNamePart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FunctionNamePart::Text(text) => write!(f, "{}", text),
            FunctionNamePart::Parameter(parameter) => {
                write!(f, "<{}: {}>", parameter.name, parameter.ty.referenced_type)
            }
        }
    }
}

impl From<Parameter> for FunctionNamePart {
    fn from(parameter: Parameter) -> Self {
        FunctionNamePart::Parameter(parameter.into())
    }
}

impl Ranged for FunctionNamePart {
    fn range(&self) -> std::ops::Range<usize> {
        match self {
            FunctionNamePart::Text(text) => text.range(),
            FunctionNamePart::Parameter(parameter) => parameter.range(),
        }
    }
}

/// Declaration of a type
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FunctionDeclaration {
    /// Generic parameters of a function
    pub generic_types: Vec<Type>,
    /// Type's name
    pub name_parts: Vec<FunctionNamePart>,
    /// Type of returned value
    pub return_type: Type,

    /// Mangled name to use instead of default
    pub(crate) mangled_name: Option<String>,
    /// Cached format for name of function
    name_format: String,
    /// Cached name of function
    name: String,
}

impl FunctionDeclaration {
    /// Create a new builder for a function declaration
    pub fn build() -> FunctionDeclarationBuilder {
        FunctionDeclarationBuilder::new()
    }

    /// Get name parts of function
    pub fn name_parts(&self) -> &[FunctionNamePart] {
        &self.name_parts
    }

    /// Format for the function's name
    ///
    /// # Example
    ///
    /// The following functions:
    /// ```ppl
    /// fn print <x: None> -> None
    /// fn print <x: Integer> -> None
    /// ```
    /// both have `print <>` name format
    pub fn name_format(&self) -> &str {
        &self.name_format
    }

    /// Get iterator over function's parameters
    pub fn parameters(&self) -> impl Iterator<Item = Arc<Parameter>> + '_ {
        self.name_parts.iter().filter_map(|part| match part {
            FunctionNamePart::Parameter(p) => Some(p.clone()),
            _ => None,
        })
    }

    /// Get mangled name of function
    pub fn mangled_name(&self) -> Cow<'_, str> {
        self.mangled_name
            .as_deref()
            .map(|n| n.into())
            .unwrap_or(self.name())
    }
}

impl Ranged for FunctionDeclaration {
    fn start(&self) -> usize {
        self.name_parts().first().unwrap().start()
    }

    fn end(&self) -> usize {
        self.name_parts().last().unwrap().end()
    }
}

impl Generic for FunctionDeclaration {
    fn is_generic(&self) -> bool {
        self.parameters().any(|p| p.is_generic()) || self.return_type.is_generic()
    }
}

impl Named for FunctionDeclaration {
    /// Get name of function
    fn name(&self) -> Cow<'_, str> {
        self.name.as_str().into()
    }
}

impl Typed for FunctionDeclaration {
    fn ty(&self) -> Type {
        FunctionType::build()
            .with_parameters(
                self.name_parts
                    .iter()
                    .filter_map(|part| match part {
                        FunctionNamePart::Parameter(p) => Some(p.ty()),
                        _ => None,
                    })
                    .collect(),
            )
            .with_return_type(self.return_type.clone())
            .into()
    }
}

impl Display for FunctionDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let indent = "\t".repeat(f.width().unwrap_or(0));
        write!(f, "{indent}")?;

        let generics = if self.generic_types.is_empty() {
            String::new()
        } else {
            format!(
                "<{}>",
                self.generic_types
                    .iter()
                    .map(|ty| format!("{ty:+}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        let name_parts = self
            .name_parts
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        let return_type = self.return_type.name();
        write!(f, "fn{generics} {name_parts} -> {return_type}")
    }
}

/// Builder for a function declaration
pub struct FunctionDeclarationBuilder {
    /// Generic parameters of a function
    generic_types: Vec<Type>,
    /// Type's name
    name_parts: Vec<FunctionNamePart>,
    /// Mangled name of function
    mangled_name: Option<String>,
}

impl FunctionDeclarationBuilder {
    /// Create a new builder for a function declaration
    pub fn new() -> Self {
        FunctionDeclarationBuilder {
            generic_types: Vec::new(),
            name_parts: Vec::new(),
            mangled_name: None,
        }
    }

    /// Set generic parameters of a function
    pub fn with_generic_types(mut self, generic_types: Vec<Type>) -> Self {
        self.generic_types = generic_types;
        self
    }

    /// Set name parts of the function
    pub fn with_name(mut self, name_parts: Vec<FunctionNamePart>) -> Self {
        self.name_parts = name_parts;
        self
    }

    /// Set mangled name of function
    pub fn with_mangled_name(mut self, mangled_name: Option<String>) -> Self {
        self.mangled_name = mangled_name;
        self
    }

    /// Build function's name format
    fn build_name_format(&self) -> String {
        let mut name_format = String::new();
        for (i, part) in self.name_parts.iter().enumerate() {
            if i > 0 {
                name_format.push_str(" ");
            }
            name_format.push_str(match part {
                FunctionNamePart::Text(text) => &text,
                FunctionNamePart::Parameter(_) => "<>",
            })
        }
        name_format
    }

    /// Build function's name
    fn build_name(&self) -> String {
        Function::build_name(&self.name_parts)
    }

    /// Set the return type of the function and return the declaration
    pub fn with_return_type(self, return_type: Type) -> FunctionDeclaration {
        let name_format = self.build_name_format();
        let name = self.build_name();
        FunctionDeclaration {
            generic_types: self.generic_types,
            name_parts: self.name_parts,
            return_type,
            name_format,
            name,
            mangled_name: self.mangled_name,
        }
    }
}

/// Declaration of a type
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FunctionDefinition {
    /// Declaration of function
    pub declaration: Arc<FunctionDeclaration>,
    /// Body of function
    pub body: Vec<Statement>,
}

impl Display for FunctionDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let indent = f.width().unwrap_or(0);
        let new_indent = indent + 1;

        let indent = "\t".repeat(indent);
        write!(f, "{indent}")?;

        writeln!(f, "{}:", self.declaration)?;
        for statement in &self.body {
            writeln!(f, "{statement:#new_indent$}")?;
        }
        Ok(())
    }
}

impl FunctionDefinition {
    /// Get name parts of function
    pub fn name_parts(&self) -> &[FunctionNamePart] {
        &self.declaration.name_parts
    }

    /// Get name format of function
    pub fn name_format(&self) -> &str {
        self.declaration.name_format()
    }

    /// Get iterator over function's parameters
    pub fn parameters(&self) -> impl Iterator<Item = Arc<Parameter>> + '_ {
        self.declaration.parameters()
    }

    /// Get mangled name of function
    pub fn mangled_name(&self) -> Cow<'_, str> {
        self.declaration.mangled_name()
    }

    /// Get return type of function
    pub fn return_type(&self) -> Type {
        self.declaration.return_type.clone()
    }
}

impl Generic for FunctionDefinition {
    fn is_generic(&self) -> bool {
        self.declaration.is_generic()
    }
}

impl Typed for FunctionDefinition {
    fn ty(&self) -> Type {
        self.declaration.ty()
    }
}

impl Named for FunctionDefinition {
    fn name(&self) -> Cow<'_, str> {
        self.declaration.name()
    }
}

/// Function definition or declaration
#[derive(Debug, Display, PartialEq, Eq, Clone, From, TryInto)]
pub enum Function {
    Declaration(Arc<FunctionDeclaration>),
    Definition(Arc<FunctionDefinition>),
}

impl Function {
    /// Build function name from name parts
    pub fn build_name(name_parts: &[FunctionNamePart]) -> String {
        let mut name = String::new();
        for (i, part) in name_parts.iter().enumerate() {
            if i > 0 {
                name.push_str(" ");
            }

            match part {
                FunctionNamePart::Text(text) => name.push_str(&text),
                FunctionNamePart::Parameter(p) => {
                    name.push_str(format!("<:{}>", p.ty().name()).as_str())
                }
            }
        }
        name
    }

    /// Get name parts of function
    pub fn name_parts(&self) -> &[FunctionNamePart] {
        match self {
            Function::Declaration(declaration) => declaration.name_parts(),
            Function::Definition(definition) => definition.name_parts(),
        }
    }

    /// Get name format of function
    pub fn name_format(&self) -> &str {
        match self {
            Function::Declaration(declaration) => declaration.name_format(),
            Function::Definition(definition) => definition.name_format(),
        }
    }

    /// Get iterator over function's parameters
    pub fn parameters(&self) -> impl Iterator<Item = Arc<Parameter>> + '_ {
        match self {
            Function::Declaration(declaration) => declaration.parameters(),
            Function::Definition(definition) => definition.declaration.parameters(),
        }
    }

    /// Get mangled name of function
    pub fn mangled_name(&self) -> Cow<'_, str> {
        match self {
            Function::Declaration(declaration) => declaration.mangled_name(),
            Function::Definition(definition) => definition.mangled_name(),
        }
    }

    /// Get return type of function
    pub fn return_type(&self) -> Type {
        self.declaration().return_type.clone()
    }

    /// Get declaration of function
    pub fn declaration(&self) -> Arc<FunctionDeclaration> {
        match self {
            Function::Declaration(declaration) => declaration.clone(),
            Function::Definition(definition) => definition.declaration.clone(),
        }
    }
}

impl Generic for Function {
    fn is_generic(&self) -> bool {
        self.declaration().is_generic()
    }
}

impl Typed for Function {
    fn ty(&self) -> Type {
        match self {
            Function::Declaration(declaration) => declaration.ty(),
            Function::Definition(definition) => definition.ty(),
        }
    }
}

impl Named for Function {
    fn name(&self) -> Cow<'_, str> {
        match self {
            Function::Declaration(declaration) => declaration.name(),
            Function::Definition(definition) => definition.name(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{assert_matches::assert_matches, sync::Arc};

    use crate::{
        ast,
        hir::{
            Function, FunctionDeclaration, FunctionDefinition, GenericType, Parameter, Return,
            Statement, TypeReference, VariableReference,
        },
        semantics::ToHIR,
        syntax::Identifier,
    };

    use pretty_assertions::assert_eq;

    #[test]
    fn generic_parameters() {
        let ast = "fn<T> <x: T> -> T => x"
            .parse::<ast::FunctionDeclaration>()
            .unwrap();
        let hir = ast.to_hir_without_context().unwrap();
        assert_matches!(hir, Function::Definition(_));

        let hir: Arc<FunctionDefinition> = hir.try_into().unwrap();

        let ty = GenericType {
            name: Identifier::from("T").at(3),
            generated: false,
            constraint: None,
        };
        let param = Parameter {
            name: Identifier::from("x").at(7),
            ty: TypeReference {
                referenced_type: ty.clone().into(),
                span: 10..11,
                type_for_type: hir.parameters().next().unwrap().ty.type_for_type.clone(),
            },
            range: 6..12,
        };
        assert_eq!(
            *hir.declaration,
            FunctionDeclaration::build()
                .with_generic_types(vec![ty.clone().into()])
                .with_name(vec![param.clone().into()])
                .with_return_type(ty.into())
        );
        assert_eq!(
            hir.body[0],
            Statement::Return(Return {
                value: Some(
                    VariableReference {
                        span: 21..22,
                        variable: param.into()
                    }
                    .into()
                )
            })
        )
    }
}
