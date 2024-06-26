use std::ops::Range;

use super::{
    error::{EmptyBlock, LexerError, ParseError},
    Identifier, Keyword, PrecedenceGroups, Ranged, StringWithOffset, Token,
};

/// Context for parsing
pub struct Context<Lexer: super::Lexer> {
    /// Lexer to use for parsing
    pub lexer: Lexer,
    /// Currently active precedence groups for operators
    pub precedence_groups: PrecedenceGroups,
}

impl<Lexer: super::Lexer> Context<Lexer> {
    /// Consume end of line (newline or eof)
    pub fn consume_eol(&mut self) -> Result<Option<StringWithOffset>, LexerError> {
        if self.lexer.peek().is_some() {
            return Ok(Some(self.lexer.consume(Token::Newline)?));
        }
        Ok(None)
    }

    /// Consume keyword
    pub fn consume_keyword<const KEYWORD: &'static str>(
        &mut self,
    ) -> Result<Keyword<KEYWORD>, LexerError> {
        let token = Keyword::<KEYWORD>::as_token();
        let offset = self.lexer.consume(token)?.start();
        Ok(Keyword::<KEYWORD>::at(offset))
    }

    /// Consume id or escaped id
    pub fn consume_id(&mut self) -> Result<Identifier, LexerError> {
        self.lexer.consume_one_of(&[Token::Id, Token::EscapedId])?;
        Ok(self.lexer.string_with_offset().into())
    }

    /// Parse block of items
    pub fn parse_maybe_empty_block<T>(
        &mut self,
        parse: impl Fn(&mut Self) -> Result<T, ParseError>,
    ) -> Result<Vec<T>, ParseError> {
        let indentation = self.lexer.indentation() + 1;

        self.lexer.consume(Token::Newline)?;
        self.lexer.skip_indentation();

        let mut stmts = Vec::new();
        while self.lexer.indentation() == indentation && self.lexer.peek().is_some() {
            stmts.push(parse(self)?);
            self.lexer.skip_indentation();
        }

        Ok(stmts)
    }

    /// Parse block of items
    pub fn parse_block<T>(
        &mut self,
        parse: impl Fn(&mut Self) -> Result<T, ParseError>,
        error_range: Range<usize>,
    ) -> Result<Vec<T>, ParseError> {
        let stmts = self.parse_maybe_empty_block(parse)?;

        if stmts.is_empty() {
            return Err(EmptyBlock {
                at: error_range.into(),
            }
            .into());
        }

        Ok(stmts)
    }

    /// Parse items separated by separator
    pub fn parse_separated<T, E>(
        &mut self,
        parse: impl Fn(&mut Self) -> Result<T, E>,
        separator: Token,
    ) -> Vec<T> {
        let mut items = Vec::new();
        while let Ok(item) = parse(self) {
            items.push(item);

            if self.lexer.consume(separator.clone()).is_err() {
                break;
            }
        }

        items
    }

    /// Parse items separated by comma
    pub fn parse_comma_separated<T, E>(
        &mut self,
        parse: impl Fn(&mut Self) -> Result<T, E>,
    ) -> Vec<T> {
        self.parse_separated(parse, Token::Comma)
    }

    /// Has space between current token and the next one?
    pub fn has_space_before_next_token(&mut self) -> bool {
        self.lexer.peek_span().start != self.lexer.span().end
    }

    /// No space between current token and the next one?
    pub fn no_space_before_next_token(&mut self) -> bool {
        !self.has_space_before_next_token()
    }
}

impl<'l, Lexer: super::Lexer> Context<Lexer> {
    /// Create new context with default precedence groups
    pub fn new(lexer: Lexer) -> Self {
        Self {
            lexer,
            precedence_groups: PrecedenceGroups::default(),
        }
    }
}

/// Trait for parsing using context.lexer
pub trait Parse
where
    Self: Sized,
{
    type Err;

    /// Parse starting from current context.lexer state
    fn parse(context: &mut Context<impl super::Lexer>) -> Result<Self, Self::Err>;
}

/// Trait for checking that current context.lexer state is 100% start of a node
pub trait StartsHere {
    /// Check if current context.lexer state is 100% start of this node
    fn starts_here(context: &mut Context<impl super::Lexer>) -> bool;
}
