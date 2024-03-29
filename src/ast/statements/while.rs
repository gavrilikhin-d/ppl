extern crate ast_derive;

use ast_derive::AST;

use crate::ast::{Expression, Statement};
use crate::syntax::error::EmptyBlock;
use crate::syntax::{error::ParseError, Lexer, Parse, Token};
use crate::syntax::{Context, Keyword, Ranged, StartsHere};

/// AST for while loop
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct While {
    /// Keyword `while`
    pub keyword: Keyword<"while">,
    /// Condition of loop
    pub condition: Expression,
    /// Body of loop
    pub body: Vec<Statement>,
}

impl Ranged for While {
    fn start(&self) -> usize {
        self.keyword.start()
    }

    fn end(&self) -> usize {
        self.body
            .last()
            .map_or_else(|| self.condition.end(), |s| s.end())
    }
}

impl StartsHere for While {
    /// Check that loop starts at current lexer position
    fn starts_here(context: &mut Context<impl Lexer>) -> bool {
        context.lexer.peek() == Some(Token::While)
    }
}

impl Parse for While {
    type Err = ParseError;

    /// Parse loop using lexer
    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        let keyword = context.consume_keyword::<"while">()?;

        let condition = Expression::parse(context)?;

        let colon = context.lexer.consume(Token::Colon)?;

        let error_range = keyword.start()..colon.start();
        let body = context.parse_block(Statement::parse, error_range)?;

        if body.is_empty() {
            return Err(EmptyBlock {
                at: (keyword.start()..colon.start()).into(),
            }
            .into());
        }

        Ok(While {
            keyword,
            condition,
            body,
        })
    }
}
