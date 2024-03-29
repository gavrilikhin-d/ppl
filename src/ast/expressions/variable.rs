extern crate ast_derive;
use ast_derive::AST;

use crate::syntax::{
    error::ParseError, Context, Identifier, Lexer, Parse, Ranged, StartsHere, Token,
};

/// AST for variable reference
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct VariableReference {
    /// Referenced variable name
    pub name: Identifier,
}

impl StartsHere for VariableReference {
    /// Check that variable reference may start at current lexer position
    fn starts_here(context: &mut Context<impl Lexer>) -> bool {
        context
            .lexer
            .try_match_one_of(&[Token::Id, Token::EscapedId])
            .is_ok_and(|_| {
                Identifier::from(context.lexer.peek_string_with_offset())
                    .as_str()
                    .chars()
                    .nth(0)
                    .is_some_and(|c| c.is_lowercase())
            })
    }
}

impl Parse for VariableReference {
    type Err = ParseError;

    /// Parse variable reference using lexer
    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        Ok(VariableReference {
            name: context.consume_id()?,
        })
    }
}

impl Ranged for VariableReference {
    /// Get range of variable reference
    fn range(&self) -> std::ops::Range<usize> {
        self.name.range()
    }
}
