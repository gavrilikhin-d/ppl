use std::{collections::HashMap, rc::Rc};

use crate::{
    patterns::{Group, Repeat},
    Match, Pattern, PatternMatch, Rule, RuleMatch,
};

/// Syntax parser
pub struct Parser {
    /// Rule to start from
    pub root: Rc<Rule>,
    /// Rules for the parser
    pub rules: HashMap<String, Rc<Rule>>,
}

impl Parser {
    /// Add a rule to the parser.
    ///
    /// Returns previously added rule with the same name, if any.
    pub fn add_rule(&mut self, rule: Rc<Rule>) -> Option<Rc<Rule>> {
        self.rules.insert(rule.name().into(), rule)
    }

    /// Get a rule by name, or return an error
    pub fn try_rule(&self, name: &str) -> Result<Rc<Rule>, UnknownRule> {
        self.rules
            .get(name)
            .ok_or_else(|| UnknownRule { name: name.into() })
            .cloned()
    }

    /// Parse a list of tokens, starting from the root rule.
    ///
    /// Tokens must be subslices of `source`.
    pub fn parse<'source>(
        &mut self,
        source: &'source str,
        mut token: impl Iterator<Item = &'source str> + Clone,
    ) -> RuleMatch<'source> {
        self.root.clone().apply(source, &mut token, self)
    }
}

/// Error for unknown rule
#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone)]
#[error("Unknown rule '{name}'")]
pub struct UnknownRule {
    /// Rule's name
    pub name: String,
}

/// Create a rule from a syntax definition
fn create_rule(parser: &mut Parser, rule: &RuleMatch) {
    parser.add_rule(Rc::new(Rule {
        name: rule["name"].as_token().to_string(),
        patterns: vec![],
        action: None,
    }));
}

impl Default for Parser {
    fn default() -> Self {
        // syntax Identifier = [a-zA-Z_][a-zA-Z0-9_]*
        let identifier = Rc::new(Rule {
            name: "Identifier".into(),
            patterns: vec![r"[a-zA-Z_][a-zA-Z0-9_]*".try_into().unwrap()],
            action: None,
        });

        // syntax Pattern = [a-zA-Z_][a-zA-Z0-9_]*
        let pattern = Rc::new(Rule {
            name: "Pattern".into(),
            patterns: vec![r"[a-zA-Z_][a-zA-Z0-9_]*".try_into().unwrap()],
            action: None,
        });

        // syntax Syntax = syntax <name: Identifier> = Pattern+
        let syntax = Rc::new(Rule {
            name: "Syntax".into(),
            patterns: vec![
                "syntax".try_into().unwrap(),
                Group {
                    name: "name".into(),
                    patterns: vec![identifier.clone().into()],
                }
                .into(),
                "=".try_into().unwrap(),
                Repeat::once_or_more(pattern.clone().into()).into(),
            ],
            action: Some(Box::new(create_rule)),
        });

        Parser {
            root: syntax.clone(),
            rules: vec![
                ("Identifier".into(), identifier),
                ("Pattern".into(), pattern),
                ("Syntax".into(), syntax),
            ]
            .into_iter()
            .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Match;

    use super::*;

    #[test]
    fn unknown_rule() {
        let parser = Parser::default();
        let rule = parser.try_rule("Unknown");
        assert_eq!(
            rule.err(),
            Some(UnknownRule {
                name: "Unknown".into()
            })
        );
    }

    #[test]
    fn rule() {
        let mut parser = Parser::default();

        let source = "syntax Test = test";
        let tokens = source.split_whitespace();
        let rule = parser.parse(source, tokens);
        assert!(rule.is_ok());

        let name = rule.get("name");
        assert_eq!(name.map(|m| m.tokens().next()).flatten(), Some("Test"));

        let rule = parser.try_rule("Test");
        assert!(rule.is_ok());
    }
}
