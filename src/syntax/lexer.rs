use std::cell::RefCell;

use log::debug;
use logos::{Logos, Span};

use crate::syntax::error::{LexerError, MissingToken, UnexpectedToken};

use super::{OperatorKind, StringWithOffset, Token};

/// Convert Logos' `Result` to `Option` with `Token::Error` on error
trait LogosLexErrorToken {
    fn lex(&mut self) -> Option<Token>;
}

impl LogosLexErrorToken for logos::Lexer<'_, Token> {
    fn lex(&mut self) -> Option<Token> {
        Some(self.next()?.unwrap_or_else(|kind| Token::Error(kind)))
    }
}

/// Trait for PPL's lexers
pub trait Lexer: Iterator<Item = Token> {
    /// Get source code of lexer
    fn source(&self) -> &str;

    /// Get current token
    fn token(&self) -> Option<Token>;

    /// Peek next token
    fn peek(&self) -> Option<Token>;

    /// Get span of peeked token
    fn peek_span(&self) -> Span;

    /// Get slice of peeked token
    fn peek_slice(&self) -> &str;

    /// Get span of current token
    fn span(&self) -> Span;

    /// Get slice of current token
    fn slice(&self) -> &str;

    /// Get string with offset of current token
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, Lexer, FullSourceLexer, StringWithOffset};
    ///
    /// let mut lexer = FullSourceLexer::new("42");
    /// assert_eq!(lexer.next(), Some(Token::Integer));
    /// assert_eq!(
    /// 	lexer.string_with_offset(),
    /// 	StringWithOffset::from("42").at(0)
    /// );
    /// ```
    fn string_with_offset(&self) -> StringWithOffset {
        StringWithOffset::from(self.slice()).at(self.span().start)
    }

    /// Get string with offset of the next token
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, Lexer, FullSourceLexer, StringWithOffset};
    ///
    /// let mut lexer = FullSourceLexer::new("42");
    /// assert_eq!(lexer.peek(), Some(Token::Integer));
    /// assert_eq!(
    /// 	lexer.peek_string_with_offset(),
    /// 	StringWithOffset::from("42").at(0)
    /// );
    /// lexer.next();
    /// assert_eq!(
    ///     lexer.peek_string_with_offset(),
    ///     StringWithOffset::from("").at(2)
    /// );
    /// ```
    fn peek_string_with_offset(&self) -> StringWithOffset {
        StringWithOffset::from(self.peek_slice()).at(self.peek_span().start)
    }

    /// Try match next token with given type
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, Lexer, FullSourceLexer, error::*};
    ///
    /// let mut lexer = FullSourceLexer::new("42");
    /// assert_eq!(lexer.try_match(Token::Integer), Ok("42".into()));
    ///
    /// let mut lexer = FullSourceLexer::new("42");
    /// assert_eq!(
    /// 	lexer.try_match(Token::Id),
    /// 	Err(
    /// 		UnexpectedToken {
    /// 			expected: vec![Token::Id],
    /// 			got: Token::Integer,
    /// 			at: lexer.peek_span().into()
    /// 		}.into()
    /// 	)
    /// );
    /// ```
    fn try_match(&self, token: Token) -> Result<StringWithOffset, LexerError> {
        self.try_match_one_of(&[token])
            .map(|_| self.peek_string_with_offset())
    }

    /// Try match next token with one of specified types
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, Lexer, FullSourceLexer, error::*};
    ///
    /// let mut lexer = FullSourceLexer::new("42");
    /// assert_eq!(lexer.span(), 0..0);
    /// assert_eq!(lexer.try_match_one_of(&[Token::Integer, Token::Id]), Ok(Token::Integer));
    /// assert_eq!(lexer.span(), 0..0);
    ///
    /// let mut lexer = FullSourceLexer::new("42");
    /// assert_eq!(lexer.span(), 0..0);
    /// assert_eq!(
    /// 	lexer.try_match_one_of(&[Token::None, Token::Id]),
    /// 	Err(
    /// 		UnexpectedToken {
    /// 			expected: vec![Token::None, Token::Id],
    /// 			got: Token::Integer,
    /// 			at: lexer.peek_span().into()
    /// 		}.into()
    /// 	)
    /// );
    /// assert_eq!(lexer.span(), 0..0);
    /// ```
    fn try_match_one_of(&self, tokens: &[Token]) -> Result<Token, LexerError> {
        debug_assert!(tokens.len() > 0);

        let token = self.peek();
        if token.is_none() {
            return Err(MissingToken {
                expected: tokens.to_vec(),
                at: self.span().end.into(),
            }
            .into());
        }

        let token = token.unwrap();

        if !tokens.contains(&token) {
            if let Token::Error(kind) = token {
                return Err(kind.at(self.peek_span()));
            }

            return Err(UnexpectedToken {
                expected: tokens.to_owned(),
                got: token,
                at: self.peek_span().into(),
            }
            .into());
        }

        Ok(token)
    }

    /// Lex next token if it has specified type
    ///
    /// **Note:** doesn't lex, on failure
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, Lexer, FullSourceLexer, error::*};
    ///
    /// let mut lexer = FullSourceLexer::new("42");
    /// assert_eq!(lexer.consume(Token::Integer), Ok("42".into()));
    ///
    /// let mut lexer = FullSourceLexer::new("42");
    /// assert_eq!(
    /// 	lexer.consume(Token::Id),
    /// 	Err(
    /// 		UnexpectedToken {
    /// 			expected: vec![Token::Id],
    /// 			got: Token::Integer,
    /// 			at: lexer.peek_span().into()
    /// 		}.into()
    /// 	)
    /// );
    /// ```
    fn consume(&mut self, token: Token) -> Result<StringWithOffset, LexerError> {
        self.consume_one_of(&[token])
            .map(|_| self.string_with_offset())
    }

    /// Lex next token if it has one of specified types
    ///
    /// **Note:** doesn't lex, on failure
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, Lexer, FullSourceLexer, error::*};
    ///
    /// let mut lexer = FullSourceLexer::new("42");
    /// assert_eq!(lexer.consume_one_of(&[Token::Integer, Token::Id]), Ok(Token::Integer));
    ///
    /// let mut lexer = FullSourceLexer::new("42");
    /// assert_eq!(
    /// 	lexer.consume_one_of(&[Token::None, Token::Id]),
    /// 	Err(
    /// 		UnexpectedToken {
    /// 			expected: vec![Token::None, Token::Id],
    /// 			got: Token::Integer,
    /// 			at: lexer.peek_span().into()
    /// 		}.into()
    /// 	)
    /// );
    /// ```
    fn consume_one_of(&mut self, tokens: &[Token]) -> Result<Token, LexerError> {
        let token = self.try_match_one_of(tokens)?;
        self.next();
        Ok(token)
    }

    /// Lex next token if it's an operator
    ///
    /// **Note:** doesn't lex, on failure
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, OperatorKind, Lexer, FullSourceLexer, error::*};
    ///
    /// let mut lexer = FullSourceLexer::new("+");
    /// assert_eq!(lexer.consume_operator(), Ok("+".into()));
    ///
    /// let mut lexer = FullSourceLexer::new("42");
    /// assert_eq!(
    /// 	lexer.consume_operator(),
    /// 	Err(
    /// 		UnexpectedToken {
    /// 			expected: vec![
    /// 				Token::Operator(OperatorKind::Prefix),
    /// 				Token::Operator(OperatorKind::Infix),
    /// 				Token::Operator(OperatorKind::Postfix),
    /// 				Token::Less,
    /// 				Token::Greater,
    ///                 Token::Star,
    ///                 Token::Ampersand
    /// 			],
    /// 			got: Token::Integer,
    /// 			at: lexer.peek_span().into()
    /// 		}.into()
    /// 	)
    /// );
    /// ```
    fn consume_operator(&mut self) -> Result<StringWithOffset, LexerError> {
        self.consume_one_of(&[
            Token::Operator(OperatorKind::Prefix),
            Token::Operator(OperatorKind::Infix),
            Token::Operator(OperatorKind::Postfix),
            Token::Less,
            Token::Greater,
            Token::Star,
            Token::Ampersand,
        ])?;
        Ok(self.string_with_offset())
    }

    /// Consume [`Token::Greater`], even if it's part of another token
    fn consume_greater(&mut self) -> Result<StringWithOffset, LexerError> {
        let res = self.consume(Token::Greater);
        if res.is_ok() {
            return res;
        }

        if self.peek_slice().starts_with(">") {
            let at = self.peek_span().start;
            let token = StringWithOffset::from(">").at(at);
            self.set_start_position(at + 1);
            return Ok(token);
        }
        return res;
    }

    /// Skip space tokens
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, Lexer, FullSourceLexer};
    ///
    /// let mut lexer = FullSourceLexer::new("\n\n42");
    /// assert_eq!(lexer.peek(), Some(Token::Newline));
    /// lexer.skip_spaces();
    /// assert_eq!(lexer.peek(), Some(Token::Integer));
    /// ```
    fn skip_spaces(&mut self) -> &mut Self {
        while self.peek().map_or(false, |token| token.is_whitespace()) {
            self.next();
        }
        self
    }

    /// Get current indentation level
    fn indentation(&self) -> usize;

    /// Skip indentation.
    /// Changes current indentation level to the amount of tabs skipped
    fn skip_indentation(&mut self) -> &mut Self;

    /// Set lexer's start byte position
    fn set_start_position(&mut self, position: usize);

    /// Skip tokens until next line
    fn skip_till_next_line(&mut self) -> &mut Self {
        while self.peek().is_some_and(|t| t != Token::Newline) {
            self.next();
        }
        self.next();
        self
    }
}

/// Lexer for full source code of PPL
pub struct FullSourceLexer<'source> {
    /// Logos lexer for tokens
    lexer: RefCell<logos::Lexer<'source, Token>>,
    /// Span of current token
    span: Span,
    /// Current token
    token: Option<Token>,
    /// Peeked token
    peeked: RefCell<Option<Token>>,
    /// Current indentation level
    indentation: usize,
}

impl<'source> FullSourceLexer<'source> {
    /// Create new lexer
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, Lexer, FullSourceLexer};
    ///
    /// let mut lexer = FullSourceLexer::new("42");
    /// assert_eq!(lexer.span(), 0..0);
    /// ```
    pub fn new(source: &'source str) -> Self {
        Self {
            lexer: Token::lexer(source).into(),
            span: 0..0,
            token: None,
            peeked: None.into(),
            indentation: 0,
        }
    }

    /// Internal function to lex next token
    fn lex(&self) -> Option<Token> {
        self.lexer.borrow_mut().lex()
    }
}

impl<'source> Iterator for FullSourceLexer<'source> {
    type Item = Token;

    /// Lex next token
    fn next(&mut self) -> Option<Token> {
        if matches!(self.peek(), None | Some(Token::Newline)) {
            self.indentation = 0;
        }
        self.span = self.lexer.get_mut().span();
        self.token = self.peeked.take();
        debug!(target: "tokens", "{:?} {:?} @{:?}", self.slice(), self.token, self.span);
        self.token()
    }
}

impl Lexer for FullSourceLexer<'_> {
    /// Get current token
    fn token(&self) -> Option<Token> {
        self.token.clone()
    }

    /// Get source code of lexer
    fn source(&self) -> &str {
        self.lexer.borrow().source()
    }

    /// Peek next token
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, Lexer, FullSourceLexer};
    ///
    /// let mut lexer = FullSourceLexer::new("42");
    /// assert_eq!(lexer.span(), 0..0);
    /// assert_eq!(lexer.peek(), Some(Token::Integer));
    /// assert_eq!(lexer.span(), 0..0);
    ///
    /// assert_eq!(lexer.next(), Some(Token::Integer));
    /// assert_eq!(lexer.span(), 0..2);
    /// ```
    fn peek(&self) -> Option<Token> {
        if self.peeked.borrow().is_none() {
            *self.peeked.borrow_mut() = self.lex();
            if self.token == Some(Token::Newline) {
                while *self.peeked.borrow() == Some(Token::Newline) {
                    *self.peeked.borrow_mut() = self.lex();
                }
            }
        }
        self.peeked.borrow().clone()
    }

    /// Get span of next token
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, Lexer, FullSourceLexer};
    ///
    /// let mut lexer = FullSourceLexer::new("42");
    /// assert_eq!(lexer.span(), 0..0);
    /// assert_eq!(lexer.peek_span(), 0..2);
    /// assert_eq!(lexer.span(), 0..0);
    /// ```
    fn peek_span(&self) -> Span {
        self.peek();
        self.lexer.borrow_mut().span()
    }

    /// Get slice of source code for next token
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, Lexer, FullSourceLexer};
    ///
    /// let mut lexer = FullSourceLexer::new("42");
    /// assert_eq!(lexer.span(), 0..0);
    /// assert_eq!(lexer.peek_slice(), "42");
    /// assert_eq!(lexer.span(), 0..0);
    /// ```
    fn peek_slice(&self) -> &str {
        self.peek();
        self.lexer.borrow_mut().slice()
    }

    /// Get span of current token
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, Lexer, FullSourceLexer};
    ///
    /// let mut lexer = FullSourceLexer::new("42");
    /// assert_eq!(lexer.span(), 0..0);
    /// assert_eq!(lexer.next(), Some(Token::Integer));
    /// assert_eq!(lexer.span(), 0..2);
    /// ```
    fn span(&self) -> Span {
        self.span.clone()
    }

    /// Get slice of current token
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, Lexer, FullSourceLexer};
    ///
    /// let mut lexer = FullSourceLexer::new("42");
    /// assert_eq!(lexer.slice(), "");
    /// assert_eq!(lexer.next(), Some(Token::Integer));
    /// assert_eq!(lexer.slice(), "42");
    /// ```
    fn slice(&self) -> &str {
        &self.lexer.borrow_mut().source()[self.span.clone()]
    }

    /// Get current indentation level
    fn indentation(&self) -> usize {
        self.indentation
    }

    /// Skip indentation.
    /// Changes current indentation level to the amount of tabs skipped
    fn skip_indentation(&mut self) -> &mut Self {
        while self.peek() == Some(Token::Tab) {
            self.next();
            self.indentation += 1;
        }
        self
    }

    fn set_start_position(&mut self, position: usize) {
        let new_lexer = Token::lexer(self.lexer.borrow().source());
        *self.lexer.borrow_mut() = new_lexer;
        self.lexer.borrow_mut().bump(position);
        *self.peeked.borrow_mut() = None;
    }
}

/// Lexer for reading from interactive stream (stdin)
pub struct InteractiveLexer<F: Fn() -> String> {
    /// Function to request next line
    pub get_line: F,
    /// Current source code of lexer
    source: RefCell<String>,
    /// Span of current token
    span: Span,
    /// Current token
    token: Option<Token>,
    /// Current indentation level
    indentation: usize,
}

impl<F: Fn() -> String> InteractiveLexer<F> {
    /// Create new lexer with custom function to request next line
    pub fn new(get_line: F) -> Self {
        Self {
            get_line,
            source: String::new().into(),
            span: 0..0,
            token: None,
            indentation: 0,
        }
    }

    /// Create lexer and set it state at the end of current token
    fn lexer<'s>(&'s self) -> logos::Lexer<'_, Token> {
        let mut lexer = Token::lexer(self.source());
        lexer.bump(self.span.end);
        lexer
    }

    /// Request next line
    fn request_line(&self) {
        self.source.borrow_mut().push_str(&(self.get_line)());
    }

    /// Request next line if lexer is at the end of source code
    fn maybe_request_line(&self) {
        if self.span.end == self.source.borrow().len() {
            self.request_line()
        }
    }

    /// Implementation of peek without requesting new line
    fn peek_impl(&self, lexer: &mut logos::Lexer<'_, Token>) -> Option<Token> {
        let mut peeked = lexer.lex();
        if matches!(self.token, None | Some(Token::Newline)) {
            while peeked == Some(Token::Newline) {
                peeked = lexer.lex();
            }
        }
        peeked
    }

    /// Force lexer to go to the end of input
    pub fn go_to_end(&mut self) {
        let end = self.source().len();
        self.span = end..end;
        self.token = None;
    }
}

impl<F: Fn() -> String> Iterator for InteractiveLexer<F> {
    type Item = Token;

    /// Lex next token
    fn next(&mut self) -> Option<Token> {
        self.maybe_request_line();
        let mut lexer = self.lexer();
        let peeked = self.peek_impl(&mut lexer);

        self.span = lexer.span();
        self.token = peeked;
        if matches!(self.token, None | Some(Token::Newline)) {
            self.indentation = 0;
        }
        debug!(target: "tokens", "{:?} {:?} @{:?}", self.slice(), self.token, self.span);
        self.token()
    }
}

impl<F: Fn() -> String> Lexer for InteractiveLexer<F> {
    /// Get source code of lexer
    fn source(&self) -> &str {
        unsafe { &*self.source.as_ptr() }
    }

    /// Get current token
    fn token(&self) -> Option<Token> {
        self.token.clone()
    }

    /// Peek next token
    fn peek(&self) -> Option<Token> {
        let mut lexer = self.lexer();
        let mut peeked = self.peek_impl(&mut lexer);
        if peeked.is_none() {
            self.request_line();

            let mut lexer = self.lexer();
            peeked = self.peek_impl(&mut lexer);
        }
        peeked
    }

    /// Get span of next token
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, Lexer, InteractiveLexer};
    ///
    /// let mut lexer = InteractiveLexer::new(|| "42".into());
    /// assert_eq!(lexer.span(), 0..0);
    /// assert_eq!(lexer.peek_span(), 0..2);
    /// assert_eq!(lexer.span(), 0..0);
    /// ```
    fn peek_span(&self) -> Span {
        self.maybe_request_line();
        let mut lexer = self.lexer();
        self.peek_impl(&mut lexer);
        lexer.span()
    }

    /// Get slice of source code for next token
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, Lexer, InteractiveLexer};
    ///
    /// let mut lexer = InteractiveLexer::new(|| "42".into());
    /// assert_eq!(lexer.span(), 0..0);
    /// assert_eq!(lexer.peek_slice(), "42");
    /// assert_eq!(lexer.span(), 0..0);
    /// ```
    fn peek_slice(&self) -> &str {
        self.maybe_request_line();
        let mut lexer = self.lexer();
        self.peek_impl(&mut lexer);
        lexer.slice()
    }

    /// Get span of current token
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, Lexer, InteractiveLexer};
    ///
    /// let mut lexer = InteractiveLexer::new(|| "42".into());
    /// assert_eq!(lexer.span(), 0..0);
    /// assert_eq!(lexer.next(), Some(Token::Integer));
    /// assert_eq!(lexer.span(), 0..2);
    /// ```
    fn span(&self) -> Span {
        self.span.clone()
    }

    /// Get slice of current token
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, Lexer, InteractiveLexer};
    ///
    /// let mut lexer = InteractiveLexer::new(|| "42".into());
    /// assert_eq!(lexer.slice(), "");
    /// assert_eq!(lexer.next(), Some(Token::Integer));
    /// assert_eq!(lexer.slice(), "42");
    /// ```
    fn slice(&self) -> &str {
        self.source()[self.span()].into()
    }

    /// Get current indentation level
    fn indentation(&self) -> usize {
        self.indentation
    }

    /// Skip indentation.
    /// Changes current indentation level to the amount of tabs skipped
    fn skip_indentation(&mut self) -> &mut Self {
        while self.peek() == Some(Token::Tab) {
            self.next();
            self.indentation += 1;
        }
        self
    }

    fn set_start_position(&mut self, position: usize) {
        self.span.end = position;
    }
}

#[cfg(test)]
mod tests {
    use crate::syntax::Lexer;

    use super::InteractiveLexer;

    #[test]
    fn correct_peek_after_skipping_newlines() {
        let mut lexer = InteractiveLexer::new(|| "a\n\nx".into());

        assert_eq!(lexer.next(), Some(super::Token::Id));
        assert_eq!(lexer.slice(), "a");
        assert_eq!(lexer.span(), 0..1);

        assert_eq!(lexer.next(), Some(super::Token::Newline));
        assert_eq!(lexer.slice(), "\n");
        assert_eq!(lexer.span(), 1..2);

        assert_eq!(lexer.peek(), Some(super::Token::Id));
        assert_eq!(lexer.peek_slice(), "x");
        assert_eq!(lexer.peek_span(), 3..4);
    }

    #[test]
    fn skip_first_newlines() {
        let lexer = InteractiveLexer::new(|| "\n\nx".into());

        assert_eq!(lexer.peek(), Some(super::Token::Id));
        assert_eq!(lexer.peek_slice(), "x");
        assert_eq!(lexer.peek_span(), 2..3);
    }
}
