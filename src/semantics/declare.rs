use std::collections::HashMap;

use indexmap::IndexMap;

use crate::{
    ast,
    hir::{self, Function, Trait, Type, Typed},
    syntax::Ranged,
    AddSourceLocation,
};

use super::{
    error::{CantDeduceReturnType, Error, ReturnTypeMismatch},
    Context, Convert, FunctionContext, GenericContext, Monomorphize, ToHIR, TraitContext,
};

use crate::DataHolder;

/// Trait to pre-declare something
pub trait Declare {
    type Declaration;
    type Definition;

    /// Declare entity in context
    fn declare(&self, context: &mut impl Context) -> Result<Self::Declaration, Error>;

    /// Define entity in context
    fn define(
        &self,
        declaration: Self::Declaration,
        context: &mut impl Context,
    ) -> Result<Self::Definition, Error>;
}

impl Declare for ast::FunctionDeclaration {
    type Declaration = Function;
    type Definition = Function;

    fn declare(&self, context: &mut impl Context) -> Result<Self::Declaration, Error> {
        // TODO: check for collision
        let generic_parameters: Vec<Type> = self.generic_parameters.to_hir(context)?;

        let (name_parts, return_type, generic_parameters) =
            GenericContext::for_generics(generic_parameters, context).run(|context| {
                let mut name_parts: Vec<hir::FunctionNamePart> = Vec::new();
                for part in &self.name_parts {
                    match part {
                        ast::FunctionNamePart::Text(t) => name_parts.push(t.clone().into()),
                        ast::FunctionNamePart::Parameter(p) => {
                            name_parts.push(p.to_hir(context)?.into())
                        }
                    }
                }

                let return_type = match &self.return_type {
                    Some(ty) => ty.to_hir(context)?.referenced_type,
                    None if self.implicit_return => Type::Unknown,
                    None => context.builtin().types().none(),
                };

                // Copy generic parameters from generic context, as we may have added new parameters
                Ok::<_, Error>((name_parts, return_type, context.generic_parameters.clone()))
            })?;

        // TODO: error if invalid annotation
        let annotations = self
            .annotations
            .iter()
            .map(|a| a.to_hir(context))
            .collect::<Result<Vec<_>, _>>()?;
        let mangled_name = annotations.iter().find_map(|a| match a {
            hir::Annotation::MangleAs(name) => Some(name.clone()),
            _ => None,
        });

        let f = Function::new(
            hir::FunctionData::build(context.compiler().current_module(), self.keyword)
                .with_generic_types(generic_parameters)
                .with_name(name_parts)
                .with_mangled_name(mangled_name)
                .with_return_type(return_type),
        );

        context.add_function(f.clone());

        Ok(f)
    }

    fn define(
        &self,
        declaration: Self::Declaration,
        context: &mut impl Context,
    ) -> Result<Self::Definition, Error> {
        if self.body.is_empty() {
            return Ok(declaration.into());
        }

        let mut f_context = FunctionContext {
            function: declaration.clone(),
            variables: vec![],
            parent: context,
        };

        let mut body = Vec::new();
        for stmt in &self.body {
            body.push(stmt.to_hir(&mut f_context)?);
        }

        if self.implicit_return {
            let mut return_type = f_context.function.read().unwrap().return_type.clone();
            let expr: hir::Expression = body.pop().unwrap().try_into().unwrap();
            if return_type == Type::Unknown {
                if expr.ty() == Type::Unknown {
                    return Err(CantDeduceReturnType {
                        at: self.name_parts.range().into(),
                    }
                    .into());
                }

                return_type = expr.ty();
                declaration.write().unwrap().return_type = return_type.clone();
            }

            let conversion = expr
                .convert_to(return_type.clone().at(expr.range())) // Fake source location doesn't matter as long as we don't reuse this error
                .within(context);
            if conversion.is_err() {
                return Err(ReturnTypeMismatch {
                    expected: return_type.clone(),
                    got: expr.ty(),
                    got_span: expr.range().into(),
                }
                .into());
            }

            let value = conversion.unwrap();
            body = vec![hir::Return::Implicit { value }.into()];
        }

        declaration.write().unwrap().body = body.clone();

        let instances: Vec<_> = context
            .module()
            .monomorphized_functions
            .iter()
            .filter(|f| {
                let data = f.read().unwrap();
                !data.is_definition() && data.generic_version == Some(declaration.clone())
            })
            .cloned()
            .collect();

        for f in instances {
            f.write().unwrap().body = body.clone();

            let mut context = GenericContext::for_fn_with_args(
                &declaration.read().unwrap(),
                f.read().unwrap().parameters(),
                context,
            );

            f.write().unwrap().monomorphize(&mut context);
        }

        Ok(declaration)
    }
}

impl Declare for ast::TraitDeclaration {
    type Declaration = Trait;
    type Definition = Trait;

    fn declare(&self, context: &mut impl Context) -> Result<Self::Declaration, Error> {
        let supertraits = self
            .supertraits
            .iter()
            .map(|t| t.to_hir(context).map(|t| t.referenced_type.as_trait()))
            .try_collect()?;

        let tr = Trait::new(hir::TraitData {
            keyword: self.keyword.clone(),
            name: self.name.clone(),
            supertraits,
            functions: IndexMap::new(),
            module: context.compiler().current_module(),
        });

        TraitContext::new(tr.clone(), context).run(|context| {
            self.functions
                .iter()
                .try_for_each(|f| f.declare(context).map(|_| ()))
        })?;

        context.add_trait(tr.clone());

        Ok(tr)
    }

    fn define(
        &self,
        declaration: Self::Declaration,
        context: &mut impl Context,
    ) -> Result<Self::Definition, Error> {
        TraitContext::new(declaration.clone(), context).run(|context| {
            let funcs = context.tr.read().unwrap().functions.clone();
            self.functions.iter().enumerate().try_for_each(|(i, f)| {
                f.define(funcs.get_index(i).unwrap().1.clone(), context)
                    .map(|_| ())
            })
        })?;

        Ok(declaration)
    }
}

impl Declare for ast::TypeDeclaration {
    type Declaration = hir::Class;
    type Definition = hir::Class;

    fn declare(&self, context: &mut impl Context) -> Result<Self::Declaration, Error> {
        let annotations = self
            .annotations
            .iter()
            .map(|a| a.to_hir(context))
            .collect::<Result<Vec<_>, _>>()?;
        let is_builtin = annotations
            .iter()
            .any(|a| matches!(a, hir::Annotation::Builtin));
        // TODO: error for incorrect builtin type name
        let builtin = if is_builtin {
            Some(self.name.parse().unwrap())
        } else {
            None
        };

        // TODO: check for collisions, etc
        let generic_parameters: Vec<Type> = self.generic_parameters.to_hir(context)?;

        // TODO: recursive types
        let ty = hir::Class::new(hir::ClassData {
            keyword: self.keyword.clone(),
            basename: self.name.clone(),
            specialization_of: None,
            generic_parameters,
            builtin,
            members: vec![],
        });

        context.add_type(ty.clone());

        Ok(ty)
    }

    fn define(
        &self,
        declaration: Self::Declaration,
        context: &mut impl Context,
    ) -> Result<Self::Definition, Error> {
        let mut generic_context = GenericContext {
            parent: context,
            generic_parameters: declaration
                .read()
                .unwrap()
                .generics()
                .into_iter()
                .cloned()
                .collect(),
            generics_mapping: HashMap::new(),
        };

        let members = self
            .members
            .iter()
            .map(|m| m.to_hir(&mut generic_context))
            .try_collect()?;

        declaration.write().unwrap().members = members;

        Ok(declaration)
    }
}

impl Declare for ast::VariableDeclaration {
    type Declaration = hir::Variable;
    type Definition = hir::Variable;

    fn declare(&self, context: &mut impl Context) -> Result<Self::Declaration, Error> {
        let type_reference = self.ty.as_ref().map(|t| t.to_hir(context)).transpose()?;
        let var = hir::Variable::new(hir::VariableData {
            keyword: self.keyword.clone(),
            name: self.name.clone(),
            ty: type_reference
                .as_ref()
                .map(|t| t.referenced_type.clone())
                .unwrap_or(Type::Unknown),
            type_reference,
            initializer: None,
            mutability: self.mutability.clone(),
        });

        context.add_variable(var.clone());

        Ok(var)
    }

    fn define(
        &self,
        declaration: Self::Declaration,
        context: &mut impl Context,
    ) -> Result<Self::Definition, Error> {
        let mut initializer = self.initializer.to_hir(context)?;
        initializer.monomorphize(context);

        let range = declaration.read().unwrap().name.range();
        let mut ty = declaration.read().unwrap().ty();
        if ty == Type::Unknown {
            ty = initializer.ty();
            declaration.write().unwrap().ty = ty.clone();
        }
        let initializer = initializer.convert_to(ty.at(range)).within(context)?;
        declaration.write().unwrap().initializer = Some(initializer);

        Ok(declaration)
    }
}

impl Declare for ast::Declaration {
    type Declaration = hir::Declaration;
    type Definition = hir::Declaration;

    fn declare(&self, context: &mut impl Context) -> Result<Self::Declaration, Error> {
        match self {
            ast::Declaration::Function(f) => f.declare(context).map(Into::into),
            ast::Declaration::Trait(t) => t.declare(context).map(Into::into),
            ast::Declaration::Type(t) => t.declare(context).map(Into::into),
            ast::Declaration::Variable(v) => v.declare(context).map(Into::into),
        }
    }

    fn define(
        &self,
        declaration: Self::Declaration,
        context: &mut impl Context,
    ) -> Result<Self::Definition, Error> {
        match self {
            ast::Declaration::Function(f) => f
                .define(declaration.try_into().unwrap(), context)
                .map(Into::into),
            ast::Declaration::Trait(t) => t
                .define(declaration.try_into().unwrap(), context)
                .map(Into::into),
            ast::Declaration::Type(t) => t
                .define(declaration.try_into().unwrap(), context)
                .map(Into::into),
            ast::Declaration::Variable(v) => v
                .define(declaration.try_into().unwrap(), context)
                .map(Into::into),
        }
    }
}

impl<D: Declare> ToHIR for D {
    type HIR = D::Definition;

    fn to_hir(&self, context: &mut impl Context) -> Result<Self::HIR, Error> {
        self.define(self.declare(context)?, context)
    }
}
