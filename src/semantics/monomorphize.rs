use log::{debug, trace};

use crate::{
    hir::{
        Assignment, Call, Class, Constructor, Declaration, Else, ElseIf, Expression, Function,
        FunctionData, FunctionNamePart, Generic, If, ImplicitConversion, ImplicitConversionKind,
        Initializer, Loop, Member, MemberReference, ModuleData, Parameter, ParameterOrVariable,
        Return, Statement, Type, TypeReference, Typed, Variable, VariableReference, While,
    },
    mutability::Mutable,
    semantics::GenericContext,
};

use crate::DataHolder;

use super::{Context, ReplaceWithTypeInfo};

/// Trait to get monomorphized version of statements
pub trait Monomorphize {
    /// Get monomorphized version of statement
    fn monomorphize(&mut self, context: &mut impl Context);
}

impl<T: Monomorphize> Monomorphize for Vec<T> {
    fn monomorphize(&mut self, context: &mut impl Context) {
        self.iter_mut().for_each(|val| val.monomorphize(context))
    }
}

impl Monomorphize for Statement {
    fn monomorphize(&mut self, context: &mut impl Context) {
        match self {
            Statement::Expression(expr) => expr.monomorphize(context),
            Statement::Assignment(a) => a.monomorphize(context),
            Statement::If(stmt) => stmt.monomorphize(context),
            Statement::Loop(l) => l.monomorphize(context),
            Statement::While(l) => l.monomorphize(context),
            Statement::Return(ret) => ret.monomorphize(context),
            Statement::Declaration(d) => d.monomorphize(context),
            Statement::Block(b) => b.statements.monomorphize(context),
            Statement::Use(_) => return,
        }
    }
}

impl Monomorphize for Declaration {
    fn monomorphize(&mut self, context: &mut impl Context) {
        match self {
            Declaration::Variable(v) => v.monomorphize(context),
            _ => return,
        }
    }
}

impl Monomorphize for Variable {
    fn monomorphize(&mut self, context: &mut impl Context) {
        if !self.is_generic()
            && !self
                .read()
                .unwrap()
                .initializer
                .as_ref()
                .map(Generic::is_generic)
                .unwrap_or(false)
        {
            trace!(target: "monomorphizing-skipped", "{self}");
            return;
        }

        let from = self.to_string();
        trace!(target: "monomorphizing", "{from}");

        let mut data = self.read().unwrap().clone();
        data.ty.monomorphize(context);
        data.initializer.as_mut().map(|i| i.monomorphize(context));
        *self = Variable::new(data);

        debug!(target: "monomorphized-from", "{from}");
        debug!(target: "monomorphized-to", "{self}");
    }
}

impl Monomorphize for Assignment {
    fn monomorphize(&mut self, context: &mut impl Context) {
        self.target.monomorphize(context);
        self.value.monomorphize(context);
    }
}

impl Monomorphize for If {
    fn monomorphize(&mut self, context: &mut impl Context) {
        self.condition.monomorphize(context);
        self.body.monomorphize(context);
        self.else_ifs.monomorphize(context);
        self.else_block
            .as_mut()
            .map(|else_block| else_block.monomorphize(context));
    }
}

impl Monomorphize for ElseIf {
    fn monomorphize(&mut self, context: &mut impl Context) {
        self.condition.monomorphize(context);
        self.body.monomorphize(context);
    }
}

impl Monomorphize for Else {
    fn monomorphize(&mut self, context: &mut impl Context) {
        self.body.monomorphize(context);
    }
}

impl Monomorphize for Return {
    fn monomorphize(&mut self, context: &mut impl Context) {
        self.value_mut().map(|value| value.monomorphize(context));
    }
}

impl Monomorphize for Loop {
    fn monomorphize(&mut self, context: &mut impl Context) {
        self.body.monomorphize(context);
    }
}

impl Monomorphize for While {
    fn monomorphize(&mut self, context: &mut impl Context) {
        self.condition.monomorphize(context);
        self.body.monomorphize(context);
    }
}

impl Monomorphize for ImplicitConversion {
    fn monomorphize(&mut self, context: &mut impl Context) {
        self.expression.monomorphize(context);
        use ImplicitConversionKind::*;
        let ty = self.expression.ty();
        self.ty = match self.kind {
            Reference => {
                if self.expression.is_mutable() {
                    context.builtin().types().reference_mut_to(ty)
                } else {
                    context.builtin().types().reference_to(ty)
                }
            }
            Dereference => ty.without_ref(),
            Copy => ty,
        };
    }
}

impl Monomorphize for Expression {
    fn monomorphize(&mut self, context: &mut impl Context) {
        match self {
            Expression::Call(c) => c.monomorphize(context),
            Expression::VariableReference(var) => var.monomorphize(context),
            Expression::TypeReference(ty) => {
                ty.monomorphize(context);
                *self = ty.replace_with_type_info(context).into();
            }
            Expression::Literal(_) => return,
            Expression::MemberReference(m) => m.monomorphize(context),
            Expression::Constructor(c) => c.monomorphize(context),
            Expression::ImplicitConversion(c) => c.monomorphize(context),
        }
    }
}

impl Monomorphize for Constructor {
    fn monomorphize(&mut self, context: &mut impl Context) {
        if !self.is_generic() {
            trace!(target: "monomorphizing-skipped", "{self}");
            return;
        }

        let from = self.to_string();
        trace!(target: "monomorphizing", "{from}");

        self.initializers.monomorphize(context);
        self.ty.monomorphize(context);

        debug!(target: "monomorphized-from", "{from}");
        debug!(target: "monomorphized-to", "{self}");
    }
}

impl Monomorphize for Initializer {
    fn monomorphize(&mut self, context: &mut impl Context) {
        self.member.monomorphize(context);
        self.value.monomorphize(context);
    }
}

impl Monomorphize for TypeReference {
    fn monomorphize(&mut self, context: &mut impl Context) {
        if !self.is_generic() {
            trace!(target: "monomorphizing-skipped", "{self}");
            return;
        }

        let from = self.to_string();
        trace!(target: "monomorphizing", "{self}");

        self.referenced_type.monomorphize(context);
        self.type_for_type = context
            .builtin()
            .types()
            .type_of(self.referenced_type.clone());

        debug!(target: "monomorphized-from", "{from}");
        debug!(target: "monomorphized-to", "{self}");
    }
}

impl Monomorphize for Type {
    fn monomorphize(&mut self, context: &mut impl Context) {
        match self {
            Type::Class(c) => c.monomorphize(context),
            Type::Function(_) => todo!(),
            Type::Generic(_) | Type::SelfType(_) | Type::Trait(_) => {
                if let Some(spec) = context.get_specialized(self.clone()) {
                    *self = spec
                }
            }
            Type::Unknown => unreachable!("Trying to monomorphize not-inferred type"),
        }
    }
}

impl Monomorphize for Class {
    fn monomorphize(&mut self, context: &mut impl Context) {
        if !self.is_generic() {
            trace!(target: "monomorphizing-skipped", "\n{self:#}");
            return;
        }

        let from = format!("{:#}", self);
        trace!(target: "monomorphizing", "\n{from}");

        let mut cl = self.read().unwrap().clone();
        cl.generic_parameters.monomorphize(context);
        cl.members.monomorphize(context);

        let res = Class::new(cl);

        debug!(target: "monomorphized-from", "\n{from}");
        debug!(target: "monomorphized-to", "\n{res:#}");

        *self = res
    }
}

impl Monomorphize for Member {
    fn monomorphize(&mut self, context: &mut impl Context) {
        if !self.is_generic() {
            return;
        }

        let mut m = self.read().unwrap().clone();
        m.ty.monomorphize(context);

        *self = Member::new(m);
    }
}

impl Monomorphize for Call {
    fn monomorphize(&mut self, context: &mut impl Context) {
        if !self.is_generic() {
            trace!(target: "monomorphizing-skipped", "{self}");
            return;
        }

        let from = self.to_string();
        trace!(target: "monomorphizing", "{from}");
        self.args.monomorphize(context);

        let mut context = GenericContext::for_fn_with_args(
            &self.function.read().unwrap(),
            self.args.iter().cloned(),
            context,
        );

        let mut f = self.function.read().unwrap().clone();
        f.monomorphize(&mut context);

        if *self.function.read().unwrap() != f {
            f.generic_version = Some(self.function.clone());
            self.function = Function::new(f);
            context
                .module_mut()
                .monomorphized_functions
                .push(self.function.clone());
        }

        debug!(target: "monomorphized-from", "{from}");
        debug!(target: "monomorphized-to", "{self}");
    }
}

impl Monomorphize for VariableReference {
    fn monomorphize(&mut self, context: &mut impl Context) {
        if !self.is_generic() {
            trace!(target: "monomorphizing-skipped", "{self}");
            return;
        }

        let from = self.to_string();
        trace!(target: "monomorphizing", "{self}");

        self.variable.monomorphize(context);

        debug!(target: "monomorphized-from", "{from}");
        debug!(target: "monomorphized-to", "{self}");
    }
}

impl Monomorphize for ParameterOrVariable {
    fn monomorphize(&mut self, context: &mut impl Context) {
        match self {
            ParameterOrVariable::Variable(v) => v.monomorphize(context),
            ParameterOrVariable::Parameter(p) => p.monomorphize(context),
        }
    }
}

impl Monomorphize for FunctionData {
    fn monomorphize(&mut self, context: &mut impl Context) {
        if !self.is_generic() {
            trace!(target: "monomorphizing-skipped", "{self}");
            return;
        }

        let from = self.to_string();
        trace!(target: "monomorphizing", "{from}");

        let f = self;

        f.generic_types.monomorphize(context);
        f.name_parts.monomorphize(context);
        f.name = FunctionData::build_name(&f.name_parts);
        f.return_type.monomorphize(context);

        if let Some(mangled_name) = context
            .function_with_name(&f.name)
            .map(|f| f.read().unwrap().mangled_name.clone())
            .flatten()
        {
            f.mangled_name = Some(mangled_name);
        }

        f.body.monomorphize(context);

        debug!(target: "monomorphized-from", "{from}");
        debug!(target: "monomorphized-to", "{f}");
    }
}

impl Monomorphize for FunctionNamePart {
    fn monomorphize(&mut self, context: &mut impl Context) {
        match self {
            FunctionNamePart::Text(_) => return,
            FunctionNamePart::Parameter(p) => p.monomorphize(context),
        }
    }
}

impl Monomorphize for Parameter {
    fn monomorphize(&mut self, context: &mut impl Context) {
        if !self.is_generic() {
            return;
        }

        let mut p = self.read().unwrap().clone();
        p.ty.monomorphize(context);
        *self = Parameter::new(p);
    }
}

impl Monomorphize for MemberReference {
    fn monomorphize(&mut self, context: &mut impl Context) {
        if !self.is_generic() {
            trace!(target: "monomorphizing-skipped", "{self}");
            return;
        }

        let from = self.to_string();
        trace!(target: "monomorphizing", "{self}");

        self.base.monomorphize(context);
        self.member = self.base.ty().members()[self.index].clone();

        debug!(target: "monomorphized-from", "{from}");
        debug!(target: "monomorphized-to", "{self}");
    }
}

impl Monomorphize for ModuleData {
    fn monomorphize(&mut self, context: &mut impl Context) {
        trace!(target: "monomorphizing", "{}", context.module().source_file().name());

        self.statements.monomorphize(context);
        self.variables
            .values_mut()
            .for_each(|v| v.monomorphize(context));
    }
}
