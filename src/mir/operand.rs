use inkwell::values::BasicValueEnum;

use crate::ir::{FunctionContext, ToIR};

use super::{constant::Constant, local::LocalID};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Operand {
    Copy(LocalID),
    Move(LocalID),
    Constant(Constant),
}

impl<T: Into<Constant>> From<T> for Operand {
    fn from(value: T) -> Self {
        Operand::Constant(value.into())
    }
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm>> for Operand {
    type IR = Option<BasicValueEnum<'llvm>>;

    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        use Operand::*;
        match self {
            Copy(_local) => todo!(),
            Move(local) => context.load(*local),
            Constant(constant) => constant.to_ir(context),
        }
    }
}
