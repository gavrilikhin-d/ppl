extern crate ast_derive;
use ast_derive::AST;

use crate::syntax::{error::ParseError, Context, Identifier, Lexer, Parse, Ranged, Token};

use super::{Expression, TypeReference, VariableReference};

/// Field initializer inside constructor
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct Initializer {
    /// Name of member
    pub name: Option<Identifier>,
    /// Value to initialize with
    pub value: Expression,
}

impl Ranged for Initializer {
    fn start(&self) -> usize {
        self.name
            .as_ref()
            .map_or_else(|| self.value.start(), |n| n.start())
    }

    fn end(&self) -> usize {
        self.value.end()
    }
}

impl Parse for Initializer {
    type Err = ParseError;

    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        let id = context.consume_id()?;

        let mut name = None;
        let value = if context.lexer.consume(Token::Colon).is_ok() {
            name = Some(id);
            Expression::parse(context)?
        } else {
            VariableReference { name: id }.into()
        };

        Ok(Initializer { name, value })
    }
}

/// AST for object constructor
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct Constructor {
    /// Type of constructed object
    pub ty: TypeReference,
    /// Offset of '{'
    pub lbrace: usize,
    /// Member initializers
    pub initializers: Vec<Initializer>,
    /// Offset of '}'
    pub rbrace: usize,
}

impl Ranged for Constructor {
    fn start(&self) -> usize {
        self.ty.start()
    }

    fn end(&self) -> usize {
        self.rbrace + 1
    }
}

impl Parse for Constructor {
    type Err = ParseError;

    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        let ty = TypeReference::parse(context)?;
        Self::parse_with_ty(context, ty)
    }
}

impl Constructor {
    /// Parse the rest of constructor if you already parsed type reference
    pub(crate) fn parse_with_ty(
        context: &mut Context<impl Lexer>,
        ty: TypeReference,
    ) -> Result<Self, <Self as Parse>::Err> {
        let lbrace = context.lexer.consume(Token::LBrace)?.start();
        let mut initializers = Vec::new();
        while context.lexer.peek() != Some(Token::RBrace) {
            initializers.push(Initializer::parse(context)?);

            if context.lexer.peek() == Some(Token::RBrace) {
                break;
            }

            context.lexer.consume(Token::Comma)?;
        }
        let rbrace = context.lexer.consume(Token::RBrace)?.start();

        Ok(Constructor {
            ty,
            lbrace,
            initializers,
            rbrace,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::Literal;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_empty() {
        let res = "Empty {}".parse::<Constructor>().unwrap();
        assert_eq!(
            res,
            Constructor {
                ty: TypeReference {
                    name: Identifier::from("Empty").into(),
                    generic_parameters: Vec::new(),
                },
                lbrace: 6,
                initializers: Vec::new(),
                rbrace: 7,
            }
        );
    }

    #[test]
    fn test_using_names() {
        let res = "Point {x, y}".parse::<Constructor>().unwrap();
        assert_eq!(
            res,
            Constructor {
                ty: TypeReference {
                    name: Identifier::from("Point").into(),
                    generic_parameters: Vec::new(),
                },
                lbrace: 6,
                initializers: vec![
                    Initializer {
                        name: None,
                        value: VariableReference {
                            name: Identifier::from("x").at(7).into()
                        }
                        .into(),
                    },
                    Initializer {
                        name: None,
                        value: VariableReference {
                            name: Identifier::from("y").at(10).into()
                        }
                        .into(),
                    },
                ],
                rbrace: 11,
            }
        );
    }

    #[test]
    fn test_using_values() {
        let res = "Point {x: 0, y: 0}".parse::<Constructor>().unwrap();
        assert_eq!(
            res,
            Constructor {
                ty: TypeReference {
                    name: Identifier::from("Point").into(),
                    generic_parameters: Vec::new(),
                },
                lbrace: 6,
                initializers: vec![
                    Initializer {
                        name: Identifier::from("x").at(7).into(),
                        value: Literal::Integer {
                            offset: 10,
                            value: "0".to_string()
                        }
                        .into()
                    },
                    Initializer {
                        name: Identifier::from("y").at(13).into(),
                        value: Literal::Integer {
                            offset: 16,
                            value: "0".to_string()
                        }
                        .into()
                    },
                ],
                rbrace: 17,
            }
        );
    }
}
