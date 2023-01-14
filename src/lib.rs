#![feature(anonymous_lifetime_in_impl_trait)]
#![feature(once_cell)]
#![feature(iterator_try_collect)]
#![feature(is_some_and)]
#![feature(let_chains)]

pub mod mutability;
pub mod named;

pub mod ast;
pub mod syntax;

pub mod hir;
pub mod semantics;

pub mod ir;
