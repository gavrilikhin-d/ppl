use std::sync::Arc;

use inkwell::types::BasicMetadataTypeEnum;

use inkwell::values::BasicMetadataValueEnum;

use super::inkwell::*;
use crate::hir::*;
use crate::mutability::Mutable;
use crate::named::Named;

use super::Context;
use super::FunctionContext;
use super::ModuleContext;

/// Trait for lowering HIR for global declarations to IR within module context
pub trait GlobalHIRLowering<'llvm> {
    type IR;

    /// Lower HIR for global declaration to IR within module context
    fn lower_global_to_ir(&self, context: &mut ModuleContext<'llvm>) -> Self::IR;
}

/// Trait for lowering HIR for local declarations to IR within function context
pub trait LocalHIRLowering<'llvm, 'm> {
    type IR;

    /// Lower HIR for local declaration to IR within function context
    fn lower_local_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR;
}

/// Trait for lowering HIR to IR within function context
pub trait HIRLoweringWithinFunctionContext<'llvm, 'm> {
    type IR;

    /// Lower HIR to IR within function context
    fn lower_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR;
}

/// Trait for convenient lowering of PPL's [`Type`](Type) to LLVM IR
pub trait HIRTypesLowering<'llvm> {
    type IR;

    /// Lower PPL's [`Type`](Type) to LLVM IR
    fn lower_to_ir(&self, context: &impl Context<'llvm>) -> Self::IR;
}

impl<'llvm> HIRTypesLowering<'llvm> for Type {
    type IR = inkwell::types::AnyTypeEnum<'llvm>;

    fn lower_to_ir(&self, context: &impl Context<'llvm>) -> Self::IR {
        match self {
            Type::Class(ty) => ty.lower_to_ir(context).into(),
            Type::Function { .. } => unimplemented!("Function type lowering"),
        }
    }
}

impl<'llvm> GlobalHIRLowering<'llvm> for Declaration {
    type IR = ();

    /// Lower global [`Declaration`] to LLVM IR
    fn lower_global_to_ir(&self, context: &mut ModuleContext<'llvm>) -> Self::IR {
        match self {
            Declaration::Variable(var) => {
                var.lower_global_to_ir(context);
            }
            Declaration::Type(ty) => {
                ty.lower_to_ir(context);
            }
            Declaration::Function(f) => {
                f.lower_global_to_ir(context);
            }
        }
    }
}

impl<'llvm, 'm> LocalHIRLowering<'llvm, 'm> for Declaration {
    type IR = ();

    /// Lower local [`Declaration`] to LLVM IR
    fn lower_local_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        match self {
            Declaration::Variable(var) => {
                var.lower_local_to_ir(context);
            }
            Declaration::Type(ty) => {
                ty.lower_to_ir(context);
            }
            Declaration::Function(f) => {
                f.lower_local_to_ir(context);
            }
        }
    }
}

/// Trait for declaring global entries in LLVM IR
trait DeclareGlobal<'llvm> {
    type IR;

    /// Declare global value without defining it
    fn declare_global(&self, context: &mut ModuleContext<'llvm>) -> Self::IR;
}

impl<'llvm> DeclareGlobal<'llvm> for VariableDeclaration {
    type IR = inkwell::values::GlobalValue<'llvm>;

    /// Declare global variable without defining it
    fn declare_global(&self, context: &mut ModuleContext<'llvm>) -> Self::IR {
        let ty = self.ty().lower_to_ir(context);
		if ty.is_void_type() {
			todo!("handle void type globals")
		}
        let global = context.module.add_global(ty.try_into_basic_type().expect("non-basic type global"), None, &self.name);

        if self.is_immutable() {
            global.set_constant(true);
        }

        global
    }
}

impl<'llvm> GlobalHIRLowering<'llvm> for Arc<VariableDeclaration> {
    type IR = inkwell::values::GlobalValue<'llvm>;

    /// Lower global [`VariableDeclaration`] to LLVM IR
    fn lower_global_to_ir(&self, context: &mut ModuleContext<'llvm>) -> Self::IR {
        let global = self.declare_global(context);

        context
            .variables
            .insert(self.clone().into(), global.clone().as_pointer_value());

        global.set_initializer(&self.ty().lower_to_ir(context).try_into_basic_type().expect("non-basic type global initializer").const_zero());

        let initialize = context.module.add_function(
            "initialize",
            context.llvm().void_type().fn_type(&[], false),
            None,
        );
        let mut f_context = FunctionContext::new(context, initialize);
        let value = self.initializer.lower_to_ir(&mut f_context);
        f_context
            .builder
            .build_store(global.as_pointer_value(), value.expect("initializer return None or Void"));

        global
    }
}

impl<'llvm, 'm> LocalHIRLowering<'llvm, 'm> for Arc<VariableDeclaration> {
    type IR = inkwell::values::PointerValue<'llvm>;

    /// Lower local [`VariableDeclaration`] to LLVM IR
    fn lower_local_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        unimplemented!("Local variable declaration")
    }
}

impl<'llvm> HIRTypesLowering<'llvm> for TypeDeclaration {
    type IR = inkwell::types::AnyTypeEnum<'llvm>;

    /// Lower [`TypeDeclaration`] to LLVM IR
    fn lower_to_ir(&self, context: &impl Context<'llvm>) -> Self::IR {
		if self.is_none() {
			return context.types().none().into();
		}
        context.types().class(&self.name).into()
    }
}

impl<'llvm> DeclareGlobal<'llvm> for FunctionDeclaration {
    type IR = inkwell::values::FunctionValue<'llvm>;

    /// Declare global function without defining it
    fn declare_global(&self, context: &mut ModuleContext<'llvm>) -> Self::IR {
        let ty = match self.ty() {
            Type::Function {
                parameters,
                return_type,
            } => {
                let parameters = parameters
                    .iter()
                    .filter_map(|p| p.lower_to_ir(context).try_into().ok())
                    .collect::<Vec<BasicMetadataTypeEnum>>();
                let return_type = return_type.lower_to_ir(context);
                return_type.fn_type(&parameters, false)
            }
            _ => unreachable!("FunctionDeclaration::ty() returned non-function type"),
        };
        context.module.add_function(self.mangled_name(), ty, None)
    }
}

impl<'llvm> GlobalHIRLowering<'llvm> for FunctionDeclaration {
    type IR = inkwell::values::FunctionValue<'llvm>;

    /// Lower global [`FunctionDeclaration`] to LLVM IR
    fn lower_global_to_ir(&self, context: &mut ModuleContext<'llvm>) -> Self::IR {
        let f = self.declare_global(context);

		self.emit_body(context);

        f
    }
}

impl<'llvm, 'm> LocalHIRLowering<'llvm, 'm> for FunctionDeclaration {
    type IR = inkwell::values::FunctionValue<'llvm>;

    /// Lower local [`FunctionDeclaration`] to LLVM IR
    fn lower_local_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        // TODO: limit function visibility, capture variables, etc.
        let f = self.declare_global(context.module_context);

		self.emit_body(context.module_context);

        f
    }
}

/// Trait for emitting body of function
trait EmitBody<'llvm> {
	/// Emit body of function
	fn emit_body(&self, context: &mut ModuleContext<'llvm>);
}

impl<'llvm> EmitBody<'llvm> for FunctionDeclaration {
	fn emit_body(&self, context: &mut ModuleContext<'llvm>) {
		let f = context.functions().get(self.mangled_name()).expect("Function was not declared before emitting body");
		if !self.body.is_empty() {
            let mut f_context = FunctionContext::new(context, f);
			for (i, p) in self.parameters().filter(
				|p| !p.name().is_empty() && !p.ty().is_none()
			).enumerate() {
				let alloca = f_context.builder
					.build_alloca(
						p.ty().lower_to_ir(&f_context).try_into_basic_type().unwrap(),
						&p.name()
					);
				f_context.parameters.insert(
					p.name().to_string(),
					alloca.clone()
				);
				f_context.builder.build_store(
					alloca,
					f.get_nth_param(i as u32).unwrap()
				);
			}
            for stmt in &self.body {
                stmt.lower_local_to_ir(&mut f_context);
            }
        }
	}
}

impl<'llvm, 'm> HIRLoweringWithinFunctionContext<'llvm, 'm> for Literal {
    type IR = Option<inkwell::values::BasicValueEnum<'llvm>>;

    /// Lower [`Literal`] to LLVM IR
    fn lower_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        Some(match self {
            Literal::None { .. } => return None,
            Literal::Integer { value, .. } => {
                if let Some(value) = value.to_i64() {
                    return Some(context
                        .builder
                        .build_call(
                            context.functions().integer_from_i64(),
                            &[context.types().i(64).const_int(value as u64, false).into()],
                            "",
                        )
                        .try_as_basic_value()
                        .left()
                        .unwrap());
                }

                let str = context
                    .builder
                    .build_global_string_ptr(&format!("{}", value), "");
                context
                    .builder
                    .build_call(
                        context.functions().integer_from_c_string(),
                        &[str.as_pointer_value().into()],
                        "",
                    )
                    .try_as_basic_value()
                    .left()
                    .unwrap()
            }
            Literal::String { value, .. } => {
                let str = context.builder.build_global_string_ptr(&value, "");
                context
                    .builder
                    .build_call(
                        context.functions().string_from_c_string_and_length(),
                        &[
                            str.as_pointer_value().into(),
                            context
                                .types()
                                .u(64)
                                .const_int(value.len() as u64, false)
                                .into(),
                        ],
                        "",
                    )
                    .try_as_basic_value()
                    .left()
                    .unwrap()
            }
        })
    }
}

impl<'llvm, 'm> HIRLoweringWithinFunctionContext<'llvm, 'm> for VariableReference {
    type IR = inkwell::values::PointerValue<'llvm>;

    /// Lower [`VariableReference`] to LLVM IR
    fn lower_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        if let Some(var) = context.get_variable(&self.variable) {
            return var;
        }

		match &self.variable {
			ParameterOrVariable::Parameter(p) => panic!(
				"Parameter {:?} not found", p.name()
			),
			ParameterOrVariable::Variable(var) =>
				var
           	 		.declare_global(context.module_context)
            		.as_pointer_value()
    	}
	}
}

impl<'llvm, 'm> HIRLoweringWithinFunctionContext<'llvm, 'm> for Call {
    type IR = inkwell::values::CallSiteValue<'llvm>;

    /// Lower [`Call`] to LLVM IR
    fn lower_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        let function = context
            .functions()
            .get(self.function.mangled_name())
            .or_else(|| Some(self.function.declare_global(context.module_context)))
            .unwrap();

        let arguments = self
            .args
            .iter()
            .filter_map(|arg| arg.lower_to_ir(context).map(|x| x.into()))
            .collect::<Vec<BasicMetadataValueEnum>>();

        context.builder.build_call(function, &arguments, "")
    }
}

/// Trait for [`Expression`] to lower HIR to LLVM IR without loading references
trait HIRExpressionLoweringWithoutLoad<'llvm, 'm> {
    /// Lower [`Expression`] to LLVM IR without loading variables
    fn lower_to_ir_without_load(
        &self,
        context: &mut FunctionContext<'llvm, 'm>,
    ) -> Option<inkwell::values::BasicValueEnum<'llvm>>;
}

impl<'llvm, 'm> HIRExpressionLoweringWithoutLoad<'llvm, 'm> for Expression {
    /// Lower [`Expression`] to LLVM IR without loading variables
    fn lower_to_ir_without_load(
        &self,
        context: &mut FunctionContext<'llvm, 'm>,
    ) -> Option<inkwell::values::BasicValueEnum<'llvm>> {
        match self {
            Expression::VariableReference(var) => Some(var.lower_to_ir(context).into()),

            Expression::Literal(l) => l.lower_to_ir(context),
            Expression::Call(call) => call
                .lower_to_ir(context)
                .try_as_basic_value()
				.left()
        }
    }
}

impl<'llvm, 'm> HIRLoweringWithinFunctionContext<'llvm, 'm> for Expression {
    type IR = Option<inkwell::values::BasicValueEnum<'llvm>>;

    /// Lower [`Expression`] to LLVM IR with loading references
    fn lower_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        match self {
            Expression::VariableReference(var) => {
                let var = var.lower_to_ir(context);
                context.builder.build_load(var, "").into()
            }
            _ => self.lower_to_ir_without_load(context),
        }
    }
}

impl<'llvm, 'm> HIRLoweringWithinFunctionContext<'llvm, 'm> for Assignment {
    type IR = inkwell::values::InstructionValue<'llvm>;

    /// Lower [`Assignment`] to LLVM IR
    fn lower_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        let target = self.target.lower_to_ir_without_load(context);
        let value = self.value.lower_to_ir(context);
        context
            .builder
            .build_store(target.expect("Assignment to none").into_pointer_value(), value.expect("Assigning none"))
    }
}

impl<'llvm> GlobalHIRLowering<'llvm> for Statement {
    type IR = ();

    /// Lower global [`Statement`] to LLVM IR
    fn lower_global_to_ir(&self, context: &mut ModuleContext<'llvm>) -> Self::IR {
        match self {
            Statement::Declaration(d) => d.lower_global_to_ir(context),
            Statement::Assignment(_) | Statement::If(_) => {
                let function = context.module.add_function(
                    "execute",
                    context.llvm().void_type().fn_type(&[], false),
                    None,
                );

                let mut context = FunctionContext::new(context, function);
                self.lower_local_to_ir(&mut context);
            }
            Statement::Expression(expr) => {
                let function = context.module.add_function(
                    "evaluate",
                    expr.ty().lower_to_ir(context).fn_type(&[], false),
                    None,
                );

                let mut context = FunctionContext::new(context, function);

                let value = expr.lower_to_ir(&mut context);
				if let Some(value) = value {
                	context.builder.build_return(Some(&value));
				} else {
					context.builder.build_return(None);
				}

                function.verify(true);
            }
			Statement::Return(_) =>
				unreachable!("Return statement is not allowed in global scope")
        };
    }
}

impl<'llvm, 'm> LocalHIRLowering<'llvm, 'm> for Statement {
    type IR = ();

    /// Lower local [`Statement`] to LLVM IR
    fn lower_local_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        match self {
            Statement::Declaration(d) => d.lower_local_to_ir(context),
            Statement::Assignment(a) => {
                a.lower_to_ir(context);
            }
            Statement::Expression(expr) => {
                expr.lower_to_ir(context);
            }
			Statement::Return(ret) => {
				ret.lower_to_ir(context);
			}
			Statement::If(if_stmt) => {
				if_stmt.lower_to_ir(context);
			}
        };
    }
}

impl HIRLoweringWithinFunctionContext<'_, '_> for Return {
	type IR = ();

	/// Lower [`Return`] to LLVM IR
	fn lower_to_ir(&self, context: &mut FunctionContext) -> Self::IR {
		let value = self.value.as_ref().map(
			|expr| expr.lower_to_ir(context)
		);
		if let Some(Some(value)) = value {
			context.builder.build_return(Some(&value));
		} else {
			context.builder.build_return(None);
		}
	}
}

impl HIRLoweringWithinFunctionContext<'_, '_> for If {
	type IR = ();

	/// Lower [`If`] to LLVM IR
	fn lower_to_ir(&self, context: &mut FunctionContext) -> Self::IR {
		todo!()
	}
}

/// Trait for lowering HIR Module to LLVM IR
pub trait HIRModuleLowering<'llvm> {
    /// Lower [`Module`] to LLVM IR
    fn lower_to_ir(&self, llvm: &'llvm inkwell::context::Context)
        -> inkwell::module::Module<'llvm>;
}

impl<'llvm> HIRModuleLowering<'llvm> for Module {
    /// Lower [`Module`] to LLVM IR
    fn lower_to_ir(
        &self,
        llvm: &'llvm inkwell::context::Context,
    ) -> inkwell::module::Module<'llvm> {
		let module = llvm.create_module(self.name());
		module.set_source_file_name(&self.filename);

        let mut context = ModuleContext::new(module);

        for statement in &self.statements {
            statement.lower_global_to_ir(&mut context);
        }

        context.module
    }
}
