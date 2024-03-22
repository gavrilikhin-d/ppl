use derive_more::From;
use salsa::DebugWithDb;

use crate::Db;

use self::{function::Function, ty::Type};

pub mod function;
pub mod ty;

#[derive(Debug, PartialEq, Eq, Clone, From)]
pub enum Declaration {
    Function(Function),
    Type(Type),
}

impl<DB: Sized + Db> DebugWithDb<DB> for Declaration {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        db: &DB,
        include_all_fields: bool,
    ) -> std::fmt::Result {
        match self {
            Declaration::Function(fun) => fun.fmt(f, db, include_all_fields),
            Declaration::Type(t) => t.fmt(f, db, include_all_fields),
        }
    }
}
