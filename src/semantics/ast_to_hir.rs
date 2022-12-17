use crate::mutability::Mutable;
use crate::syntax::{Ranged, WithOffset};
pub use crate::hir::{self, Type, Module, Typed};

use super::error::*;
use crate::ast;

/// AST to HIR lowering context
pub struct ASTLoweringContext {
	/// Module, which is being lowered
	pub module: Module,
}

impl ASTLoweringContext {
	/// Create new lowering context
	pub fn new() -> Self {
		Self { module: Module::new() }
	}

	/// Recursively find variable starting from current scope
	pub fn find_variable(&self, name: &str)
	-> Option<&hir::VariableDeclaration> {
		self.module.variables.get(name)
	}

	/// Recursively find type starting from current scope
	pub fn find_type(&self, name: &str) -> Option<Type> {
		self.module.types.get(name).map(|t| t.clone().into())
	}

	/// Recursively find type starting from current scope, or return error
	pub fn get_type(&self, name: &WithOffset<String>) -> Result<Type, UnknownType> {
		let t = self.find_type(&name.value);
		if t.is_none() {
			return Err(UnknownType {
				name: name.value.clone(),
				at: name.range().into()
			}.into());
		}

		Ok(t.unwrap())
	}
}

pub trait ASTLoweringWithinContext {
	type HIR;

	/// Lower AST to HIR within some context
	fn lower_to_hir_within_context(
		&self,
		context: &mut ASTLoweringContext
	) -> Result<Self::HIR, Error>;
}

impl ASTLoweringWithinContext for ast::Statement {
	type HIR = hir::Statement;

	/// Lower [`ast::Statement`] to [`hir::Statement`] within lowering context
	fn lower_to_hir_within_context(
		&self,
		context: &mut ASTLoweringContext
	) -> Result<Self::HIR, Error> {
		let stmt: hir::Statement =
		match self {
			ast::Statement::Declaration(decl) =>
				decl.lower_to_hir_within_context(context)?.into(),
			ast::Statement::Assignment(assign) =>
				assign.lower_to_hir_within_context(context)?.into(),
			ast::Statement::Expression(expr) =>
				expr.lower_to_hir_within_context(context)?.into(),
		};

		context.module.statements.push(stmt.clone());

		Ok(stmt)
	}
}

impl ASTLoweringWithinContext for ast::Literal {
	type HIR = hir::Literal;

	/// Lower [`ast::Literal`] to [`hir::Literal`] within lowering context
	fn lower_to_hir_within_context(
			&self,
			_context: &mut ASTLoweringContext
		) -> Result<Self::HIR, Error> {
		Ok(match self {
			ast::Literal::None { offset } =>
				hir::Literal::None { offset: *offset },
			ast::Literal::Integer { offset, value } =>
				hir::Literal::Integer {
					span: *offset..offset + value.len(),
					value: value.parse::<rug::Integer>().unwrap(),
				},
		})
	}
}

impl ASTLoweringWithinContext for ast::VariableReference {
	type HIR = hir::VariableReference;

	/// Lower [`ast::VariableReference`] to [`hir::VariableReference`] within lowering context
	fn lower_to_hir_within_context(
			&self,
			context: &mut ASTLoweringContext
		) -> Result<Self::HIR, Error> {
		let var = context.find_variable(&self.name.value);
		if var.is_none() {
			return Err(UndefinedVariable {
				name: self.name.value.clone(),
				at: self.name.range().into()
			}.into());
		}

		Ok(hir::VariableReference {
			span: self.name.range().into(),
			variable: Box::new(var.unwrap().clone()),
		})
	}
}

impl ASTLoweringWithinContext for ast::Expression {
	type HIR = hir::Expression;

	/// Lower [`ast::Expression`] to [`hir::Expression`] within lowering context
	fn lower_to_hir_within_context(
			&self,
			context: &mut ASTLoweringContext
		) -> Result<Self::HIR, Error> {
		Ok(
			match self {
				ast::Expression::Literal(l) =>
					l.lower_to_hir_within_context(context)?.into(),
				ast::Expression::VariableReference(var) =>
					var.lower_to_hir_within_context(context)?.into(),
				ast::Expression::UnaryOperation(op) => unimplemented!()
			}
		)
	}
}

impl ASTLoweringWithinContext for ast::VariableDeclaration {
	type HIR = hir::VariableDeclaration;

	/// Lower [`ast::VariableDeclaration`] to [`hir::VariableDeclaration`] within lowering context
	fn lower_to_hir_within_context(
			&self,
			context: &mut ASTLoweringContext
		) -> Result<Self::HIR, Error> {
		let var = hir::VariableDeclaration {
			name: self.name.clone(),
			initializer:
				self.initializer.lower_to_hir_within_context(context)?,
			mutability: self.mutability.clone(),
		};

		let name = &self.name.value;

		context.module.variables.insert(name.to_owned(), var.clone());

		Ok(var)
	}
}

impl ASTLoweringWithinContext for ast::TypeDeclaration {
	type HIR = hir::TypeDeclaration;

	/// Lower [`ast::TypeDeclaration`] to [`hir::TypeDeclaration`] within lowering context
	fn lower_to_hir_within_context(
		&self,
		context: &mut ASTLoweringContext
	) -> Result<Self::HIR, Error> {
		let ty = hir::TypeDeclaration {
			name: self.name.clone(),
		};

		let name = &self.name.value;

		context.module.types.insert(name.to_owned(), ty.clone());

		Ok(ty)
	}
}

impl ASTLoweringWithinContext for ast::Parameter {
	type HIR = hir::Parameter;

	/// Lower [`ast::Parameter`] to [`hir::Parameter`] within lowering context
	fn lower_to_hir_within_context(
		&self,
		context: &mut ASTLoweringContext
	) -> Result<Self::HIR, Error> {
		let ty = context.get_type(&self.ty)?;

		Ok(hir::Parameter {
			name: self.name.clone(),
			ty,
		})
	}
}

impl ASTLoweringWithinContext for ast::FunctionDeclaration {
	type HIR = hir::FunctionDeclaration;

	/// Lower [`ast::FunctionDeclaration`] to [`hir::FunctionDeclaration`] within lowering context
	fn lower_to_hir_within_context(
		&self,
		context: &mut ASTLoweringContext
	) -> Result<Self::HIR, Error> {
		let mut name_parts: Vec<hir::FunctionNamePart> = Vec::new();
		for part in &self.name_parts {
			match part {
				ast::FunctionNamePart::Text(t) =>
					name_parts.push(t.clone().into()),
				ast::FunctionNamePart::Parameter(p) =>
					name_parts.push(
						p.lower_to_hir_within_context(context)?.into()
					),
			}
		}

		let return_type = match &self.return_type {
			Some(ty) => context.get_type(ty)?,
			None => Type::None,
		};

		Ok(hir::FunctionDeclaration {
			name_parts,
			return_type
		})
	}
}

impl ASTLoweringWithinContext for ast::Declaration {
	type HIR = hir::Declaration;

	/// Lower [`ast::Declaration`] to [`hir::Declaration`] within lowering context
	fn lower_to_hir_within_context(
			&self,
			context: &mut ASTLoweringContext
		) -> Result<Self::HIR, Error> {
		Ok(match self {
			ast::Declaration::Variable(decl) =>
				decl.lower_to_hir_within_context(context)?.into(),
			ast::Declaration::Type(decl) =>
				decl.lower_to_hir_within_context(context)?.into(),
			ast::Declaration::Function(decl) =>
				decl.lower_to_hir_within_context(context)?.into(),
		})
	}
}

impl ASTLoweringWithinContext for ast::Assignment {
	type HIR = hir::Assignment;

	/// Lower [`ast::Assignment`] to [`hir::Assignment`] within lowering context
	fn lower_to_hir_within_context(
			&self,
			context: &mut ASTLoweringContext
		) -> Result<Self::HIR, Error> {
		let target = self.target.lower_to_hir_within_context(context)?;
		if target.is_immutable() {
			return Err(AssignmentToImmutable {
				at: self.target.range().into()
			}.into());
		}

		let value = self.value.lower_to_hir_within_context(context)?;
		if target.get_type() != value.get_type() {
			return Err (
				NoConversion {
					from: value.get_type(),
					from_span: self.value.range().into(),

					to: target.get_type(),
					to_span: self.target.range().into(),
				}.into()
			);
		}

		Ok(hir::Assignment { target, value, })
	}
}


/// Trait for lowering and adding statements to module
pub trait ASTLowering  {
	type HIR;

	/// Lower AST to HIR
	fn lower_to_hir(&self) -> Result<Self::HIR, Error>;
}

impl<T: ASTLoweringWithinContext> ASTLowering for T {
	type HIR = T::HIR;

	/// Lower AST to HIR
	fn lower_to_hir(&self) -> Result<Self::HIR, Error> {
		let mut context = ASTLoweringContext::new();
		self.lower_to_hir_within_context(&mut context)
	}
}

impl ASTLowering for ast::Module {
	type HIR = Module;

	/// Lower [`ast::Module`] to [`semantics::Module`](Module)
	fn lower_to_hir(&self) -> Result<Self::HIR, Error> {
		let mut context = ASTLoweringContext::new();

		for stmt in &self.statements {
			stmt.lower_to_hir_within_context(&mut context)?;
		}

		Ok(context.module)
	}
}
