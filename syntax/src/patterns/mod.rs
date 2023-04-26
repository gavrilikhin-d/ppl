mod repeat;
use std::{any::Any, error::Error};

use derive_more::From;
use nom::{IResult, Parser};
use regex::Regex;
pub use repeat::*;

use crate::{
    context::with_context,
    err,
    errors::{RegexMismatch, UnknownRuleReference},
    parsers, ParseTree,
};

/// Possible patterns
#[derive(Debug, PartialEq, Clone, From)]
pub enum Pattern {
    /// Reference to another rule
    RuleReference(String),
    /// Group of patterns
    Group(Vec<Pattern>),
    /// Regex
    Regex(String),
    /// Pattern alternatives
    Alternatives(Vec<Pattern>),
    /// Repeat pattern
    #[from]
    Repeat(Repeat),
}

impl<'i> Parser<&'i str, (ParseTree<'i>, Box<dyn Any>), Box<dyn Error>> for Pattern {
    fn parse(
        &mut self,
        input: &'i str,
    ) -> IResult<&'i str, (ParseTree<'i>, Box<dyn Any>), Box<dyn Error>> {
        match self {
            Self::Regex(r) => {
                let re = Regex::new(&format!("^{}", r)).unwrap();
                let m = re.find(input);
                if let Some(m) = m {
                    Ok((
                        &input[m.end()..],
                        (ParseTree::from(m.as_str()), Box::new(m.as_str().to_owned())),
                    ))
                } else {
                    err!(RegexMismatch {})
                }
            }
            Self::Alternatives(alts) => {
                let mut last_result = None;
                for alt in alts {
                    let res = alt.parse(input);
                    if res.is_ok()
                        || res
                            .as_ref()
                            .is_err_and(|e| matches!(e, nom::Err::Failure(_)))
                    {
                        return res;
                    }
                    last_result = Some(res);
                }
                last_result.unwrap()
            }
            Self::Repeat(r) => Ok({
                let (r, (t, ast)) = r.parse(input)?;
                (r, (t, Box::new(ast)))
            }),
            Self::Group(patterns) => {
                let (r, (t, ast)) = parsers::grouped_patterns(patterns, input)?;
                Ok((r, (t, Box::new(ast))))
            }
            Self::RuleReference(r) => with_context(|ctx| {
                if let Some(rule) = ctx.rules.iter_mut().find(|rule| rule.name == *r) {
                    return rule.parse(input);
                } else {
                    return err!(UnknownRuleReference {});
                }
            }),
        }
    }
}