use std::{borrow::Cow, collections::BTreeMap, sync::Arc};

use crate::{
    hir::{
        Assignment, Call, Constructor, ElseIf, Expression, Function, FunctionDeclaration,
        FunctionDefinition, FunctionNamePart, Generic, If, Loop, MemberReference, Return,
        Specialize, Statement, Type, Typed, VariableReference, While,
    },
    named::Named,
    semantics::FunctionContext,
};

use super::{AddDeclaration, Context, FindDeclaration};

/// Trait to get monomorphized version of statements
pub trait Monomorphized {
    /// Get monomorphized version of statement
    fn monomorphized(&self, context: &mut impl Context) -> Self;
}

impl<T: Monomorphized> Monomorphized for Vec<T> {
    fn monomorphized(&self, context: &mut impl Context) -> Self {
        self.iter().map(|val| val.monomorphized(context)).collect()
    }
}

impl Monomorphized for Statement {
    fn monomorphized(&self, context: &mut impl Context) -> Self {
        match self {
            Statement::Expression(expr) => expr.monomorphized(context).into(),
            Statement::Assignment(a) => a.monomorphized(context).into(),
            Statement::If(stmt) => stmt.monomorphized(context).into(),
            Statement::Loop(l) => l.monomorphized(context).into(),
            Statement::While(l) => l.monomorphized(context).into(),
            Statement::Return(ret) => ret.monomorphized(context).into(),

            // Declarations only monomorphized when referenced
            Statement::Declaration(_) | Statement::Use(_) => self.clone(),
        }
    }
}

impl Monomorphized for Assignment {
    fn monomorphized(&self, context: &mut impl Context) -> Self {
        Assignment {
            target: self.target.monomorphized(context),
            value: self.value.monomorphized(context),
        }
    }
}

impl Monomorphized for If {
    fn monomorphized(&self, context: &mut impl Context) -> Self {
        If {
            condition: self.condition.monomorphized(context),
            body: self.body.monomorphized(context),
            else_ifs: self.else_ifs.monomorphized(context),
            else_block: self.else_block.monomorphized(context),
        }
    }
}

impl Monomorphized for ElseIf {
    fn monomorphized(&self, context: &mut impl Context) -> Self {
        ElseIf {
            condition: self.condition.monomorphized(context),
            body: self.body.monomorphized(context),
        }
    }
}

impl Monomorphized for Return {
    fn monomorphized(&self, context: &mut impl Context) -> Self {
        Return {
            value: self.value.clone().map(|value| value.monomorphized(context)),
        }
    }
}

impl Monomorphized for Loop {
    fn monomorphized(&self, context: &mut impl Context) -> Self {
        Loop {
            body: self.body.monomorphized(context),
        }
    }
}

impl Monomorphized for While {
    fn monomorphized(&self, context: &mut impl Context) -> Self {
        While {
            condition: self.condition.monomorphized(context),
            body: self.body.monomorphized(context),
        }
    }
}

impl Monomorphized for Expression {
    fn monomorphized(&self, context: &mut impl Context) -> Self {
        match self {
            Expression::Call(c) => c.monomorphized(context).into(),
            Expression::VariableReference(var) => var.monomorphized(context).into(),
            Expression::TypeReference(_) => todo!(),
            Expression::Literal(_) => self.clone(),
            Expression::MemberReference(m) => m.monomorphized(context).into(),
            Expression::Constructor(c) => c.monomorphized(context).into(),
        }
    }
}

impl Monomorphized for Constructor {
    fn monomorphized(&self, _context: &mut impl Context) -> Self {
        // TODO: real monomorphization
        self.clone()
    }
}

impl Monomorphized for Call {
    fn monomorphized(&self, context: &mut impl Context) -> Self {
        let args = self.args.monomorphized(context);
        Call {
            function: self
                .function
                .monomorphized(context, args.iter().map(|arg| arg.ty())),
            args,
            ..self.clone()
        }
    }
}

impl Monomorphized for VariableReference {
    fn monomorphized(&self, context: &mut impl Context) -> Self {
        VariableReference {
            span: self.span.clone(),
            variable: context.find_variable(&self.variable.name()).unwrap(),
        }
    }
}

/// Trait to get monomorphized version of function
pub trait MonomorphizedWithArgs {
    /// Get monomorphized version of function
    fn monomorphized(
        &self,
        context: &mut impl Context,
        args: impl IntoIterator<Item = Type>,
    ) -> Self;
}

impl MonomorphizedWithArgs for Arc<FunctionDeclaration> {
    fn monomorphized(
        &self,
        context: &mut impl Context,
        args: impl IntoIterator<Item = Type>,
    ) -> Self {
        if !self.is_generic() {
            return self.clone();
        }

        let mut generics_map: BTreeMap<Cow<'_, str>, Type> = BTreeMap::new();

        let mut arg = args.into_iter();
        let name_parts = self
            .name_parts()
            .iter()
            .map(|part| match part {
                FunctionNamePart::Text(text) => text.clone().into(),
                FunctionNamePart::Parameter(param) => {
                    let arg_ty = arg.next().unwrap().clone();
                    if !param.is_generic() {
                        return param.clone().into();
                    }

                    let param_ty = param.ty();
                    let param = param.as_ref().clone().specialize_with(arg_ty.clone());

                    let diff = param_ty.diff(arg_ty);
                    for ty in diff.into_iter() {
                        let name = (&ty).generic.name().to_string();
                        generics_map.insert(name.into(), ty.into());
                    }

                    param.into()
                }
            })
            .collect::<Vec<_>>();

        let name = Function::build_name(&name_parts);

        let generic_types: Vec<Type> = self
            .generic_types
            .iter()
            .map(|g| generics_map.get(&g.name()).cloned().unwrap_or(g.clone()))
            .collect();
        Arc::new(
            FunctionDeclaration::build()
                .with_generic_types(generic_types)
                .with_name(name_parts)
                .with_mangled_name(
                    context
                        .function_with_name(&name)
                        .map(|f| f.declaration().mangled_name.clone())
                        .flatten(),
                )
                .with_return_type(
                    generics_map
                        .get(&self.return_type.name())
                        .cloned()
                        .unwrap_or(self.return_type.clone()),
                ),
        )
    }
}

impl MonomorphizedWithArgs for Arc<FunctionDefinition> {
    fn monomorphized(
        &self,
        context: &mut impl Context,
        args: impl IntoIterator<Item = Type>,
    ) -> Self {
        if !self.is_generic() {
            return self.clone();
        }

        let declaration = self.declaration.monomorphized(context, args);

        let mut context = FunctionContext {
            function: declaration.clone(),
            variables: vec![],
            parent: context,
        };

        let body = self
            .body
            .iter()
            .map(|stmt| stmt.monomorphized(&mut context))
            .collect();

        let f = Arc::new(FunctionDefinition { declaration, body });

        if context.function_with_name(&f.name()).is_none() {
            context.add_function(f.clone().into());
        }

        f
    }
}

impl MonomorphizedWithArgs for Function {
    fn monomorphized(
        &self,
        context: &mut impl Context,
        args: impl IntoIterator<Item = Type>,
    ) -> Self {
        match self {
            Function::Declaration(d) => d.monomorphized(context, args).into(),
            Function::Definition(d) => d.monomorphized(context, args).into(),
        }
    }
}

impl Monomorphized for MemberReference {
    fn monomorphized(&self, context: &mut impl Context) -> Self {
        if !self.is_generic() {
            return self.clone();
        }

        let base = Box::new(self.base.monomorphized(context));
        let member = base.ty().members()[self.index].clone();

        MemberReference {
            base,
            member,
            ..(self.clone())
        }
    }
}
