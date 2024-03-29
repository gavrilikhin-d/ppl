mod assignment;
use std::ops::Range;

pub use assignment::*;

mod ret;
pub use ret::*;

mod r#if;
pub use r#if::*;

mod r#loop;
pub use r#loop::*;

mod r#while;
pub use r#while::*;

mod r#use;
pub use r#use::*;

extern crate ast_derive;
use ast_derive::AST;

use crate::ast::{Declaration, Expression};
use crate::syntax::error::MissingStatement;
use crate::syntax::{error::ParseError, Lexer, Parse, Token};
use crate::syntax::{Context, Ranged, StartsHere};

use derive_more::From;

use super::Annotation;

/// Any PPL statement
#[derive(Debug, PartialEq, Eq, AST, Clone, From)]
pub enum Statement {
    Declaration(Declaration),
    Expression(Expression),
    Assignment(Assignment),
    Return(Return),
    If(If),
    Loop(Loop),
    While(While),
    Use(Use),
}

impl Ranged for Statement {
    fn range(&self) -> Range<usize> {
        use Statement::*;
        match self {
            Declaration(s) => s.range(),
            Expression(s) => s.range(),
            Assignment(s) => s.range(),
            Return(s) => s.range(),
            If(s) => s.range(),
            Loop(s) => s.range(),
            While(s) => s.range(),
            Use(s) => s.range(),
        }
    }
}

impl StartsHere for Statement {
    /// Check that statement may start at current lexer position
    fn starts_here(context: &mut Context<impl Lexer>) -> bool {
        Annotation::starts_here(context)
            || Declaration::starts_here(context)
            || Expression::starts_here(context)
            || Assignment::starts_here(context)
            || Return::starts_here(context)
            || If::starts_here(context)
            || Loop::starts_here(context)
            || While::starts_here(context)
            || Use::starts_here(context)
    }
}

impl Parse for Statement {
    type Err = ParseError;

    /// Parse statement using lexer
    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        if !Statement::starts_here(context) {
            return Err(MissingStatement {
                at: context.lexer.span().end.into(),
            }
            .into());
        }

        let mut annotations = Vec::new();
        while Annotation::starts_here(context) {
            annotations.push(Annotation::parse(context)?);
            context.lexer.skip_spaces();
        }

        let mut res: Statement = if Declaration::starts_here(context) {
            Declaration::parse(context)?.into()
        } else if Expression::starts_here(context) {
            let target = Expression::parse(context)?;

            if context.lexer.consume(Token::Assign).is_err() {
                target.into()
            } else {
                Assignment {
                    target,
                    value: Expression::parse(context)?,
                }
                .into()
            }
        } else {
            match context.lexer.peek() {
                Some(Token::Return) => Return::parse(context)?.into(),
                Some(Token::If) => If::parse(context)?.into(),
                Some(Token::Loop) => Loop::parse(context)?.into(),
                Some(Token::While) => While::parse(context)?.into(),
                Some(Token::Use) => Use::parse(context)?.into(),
                t => unreachable!("Unexpected token {:#?} at start of statement", t),
            }
        };

        if !annotations.is_empty() {
            match res {
                Statement::Declaration(Declaration::Function(ref mut decl)) => {
                    decl.annotations = annotations;
                }
                Statement::Declaration(Declaration::Type(ref mut decl)) => {
                    decl.annotations = annotations;
                }
                _ => unimplemented!("Annotations are not supported for this statement"),
            }
        }

        if matches!(
            res,
            Statement::Assignment(_)
                | Statement::Expression(_)
                | Statement::Return(_)
                | Statement::Use(_)
        ) {
            context.consume_eol()?;
        }

        Ok(res)
    }
}
