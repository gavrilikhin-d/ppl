mod function;
pub use function::*;

mod types;
pub use types::*;

mod variable;
pub use variable::*;

mod r#trait;
pub use r#trait::*;

extern crate ast_derive;
use ast_derive::AST;

use crate::syntax::{
    error::{MissingDeclaration, ParseError},
    Lexer, Parse, StartsHere, Token, Context,
};

use derive_more::From;

/// Any PPL declaration
#[derive(Debug, PartialEq, Eq, AST, Clone, From)]
pub enum Declaration {
    Variable(VariableDeclaration),
    Type(TypeDeclaration),
    Function(FunctionDeclaration),
	Trait(TraitDeclaration),
}

impl StartsHere for Declaration {
    /// Check literal may start at current lexer position
    fn starts_here(context: &mut Context<impl Lexer>) -> bool {
        VariableDeclaration::starts_here(context)
            || TypeDeclaration::starts_here(context)
            || FunctionDeclaration::starts_here(context)
			|| TraitDeclaration::starts_here(context)
    }
}

impl Parse for Declaration {
    type Err = ParseError;

    /// Parse declaration using lexer
    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        if !Declaration::starts_here(context) {
            return Err(MissingDeclaration {
                at: context.lexer.span().end.into(),
            }
            .into());
        }

        Ok(match context.lexer.peek().unwrap() {
            Token::Type => TypeDeclaration::parse(context)?.into(),
            Token::Let => VariableDeclaration::parse(context)?.into(),
            Token::Fn => FunctionDeclaration::parse(context)?.into(),
			Token::Trait => TraitDeclaration::parse(context)?.into(),
            _ => unreachable!("unexpected token in start of declaration"),
        })
    }
}
