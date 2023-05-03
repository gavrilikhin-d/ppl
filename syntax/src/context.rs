use std::sync::Arc;

use crate::{
    errors::{ExpectedTypename, TypenameNotCapitalized},
    parsers::ParseResult,
    Pattern, Rule,
};

/// Rule with action to be executed after parsing
pub struct RuleWithAction {
    pub rule: Arc<Rule>,
    pub on_parsed: Option<fn(at: usize, res: ParseResult) -> ParseResult>,
}

impl From<Rule> for RuleWithAction {
    fn from(rule: Rule) -> Self {
        RuleWithAction {
            rule: Arc::new(rule),
            on_parsed: None,
        }
    }
}

/// Parsing context
pub struct Context {
    /// Parsing rules
    pub rules: Vec<RuleWithAction>,
}

impl Context {
    /// Create a new context without any rules
    pub fn new() -> Context {
        Context { rules: vec![] }
    }

    // Add a rule to the context
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule.into())
    }

    /// Find rule by name in the parsing context
    pub fn find_rule(&self, name: &str) -> Option<Arc<Rule>> {
        self.rules
            .iter()
            .map(|r| &r.rule)
            .find(|r| r.name == name)
            .cloned()
    }

    /// Get the callback to be called after parsing a rule
    pub fn on_parsed(&self, name: &str) -> Option<fn(usize, ParseResult) -> ParseResult> {
        self.rules
            .iter()
            .find(|r| r.rule.name == name)
            .and_then(|r| r.on_parsed)
    }
}

impl Default for Context {
    fn default() -> Self {
        Context {
            rules: vec![
                Rule {
                    name: "Regex".to_string(),
                    pattern: r"[^\s]+".into(),
                }
                .into(),
                RuleWithAction {
                    rule: Arc::new(Rule {
                        name: "Typename".to_string(),
                        pattern: r"[a-zA-Z0-9_]+".into(),
                    }),
                    on_parsed: Some(|at, mut res| {
                        if res.has_errors() {
                            res.tree.children = vec![ExpectedTypename { at: at.into() }.into()];
                            return res;
                        }

                        let typename = res.tree.tokens().next().unwrap();
                        let first_char = typename.chars().next().unwrap();
                        if !first_char.is_ascii_uppercase() {
                            res.tree.children =
                                vec![TypenameNotCapitalized { at: at.into() }.into()]
                        }

                        res
                    }),
                },
                RuleWithAction {
                    rule: Arc::new(Rule {
                        name: "RuleReference".to_string(),
                        pattern: Pattern::RuleReference("Typename".to_string()),
                    }),
                    on_parsed: Some(|_at, mut res| {
                        if res.has_errors() {
                            res.delta = 0;
                            return res;
                        }
                        res
                    }),
                },
                Rule {
                    name: "Pattern".to_string(),
                    pattern: Pattern::Alternatives(vec![
                        Pattern::RuleReference("RuleReference".to_string()),
                        Pattern::RuleReference("Regex".to_string()),
                    ]),
                }
                .into(),
                Rule {
                    name: "Rule".to_string(),
                    pattern: Pattern::Group(vec![
                        Pattern::RuleReference("Typename".to_string()),
                        ":".into(),
                        Pattern::RuleReference("Pattern".to_string()),
                    ]),
                }
                .into(),
            ],
        }
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use crate::{
        errors::{ExpectedTypename, TypenameNotCapitalized},
        parsers::{ParseResult, Parser},
        Context, ParseTree,
    };

    #[test]
    fn typename() {
        let mut ctx = Context::default();
        let typename = ctx.find_rule("Typename").unwrap();
        assert_eq!(typename.name, "Typename");
        assert_eq!(
            typename.parse("Foo", &mut ctx),
            ParseResult {
                delta: 3,
                tree: ParseTree::named("Typename").with("Foo"),
                ast: json!({"Typename": "Foo"})
            }
        );
        assert_eq!(
            typename.parse("foo", &mut ctx),
            ParseResult {
                delta: 3,
                tree: ParseTree::named("Typename").with(TypenameNotCapitalized { at: 0 }),
                ast: json!({"Typename": "foo"})
            }
        );
        assert_eq!(
            typename.parse("", &mut ctx),
            ParseResult {
                delta: 0,
                tree: ParseTree::named("Typename").with(ExpectedTypename { at: 0 }),
                ast: json!({ "Typename": null })
            }
        );
    }

    #[test]
    fn rule_reference() {
        let mut context = Context::default();
        let r = context.find_rule("RuleReference").unwrap();
        assert_eq!(r.name, "RuleReference");
        assert_eq!(
            r.parse("Foo", &mut context),
            ParseResult {
                delta: 3,
                tree: ParseTree::named("RuleReference")
                    .with(ParseTree::named("Typename").with("Foo")),
                ast: json!({"RuleReference": {"Typename": "Foo"}})
            }
        );
        assert_eq!(
            r.parse("foo", &mut context),
            ParseResult {
                delta: 0,
                tree: ParseTree::named("RuleReference")
                    .with(ParseTree::named("Typename").with(TypenameNotCapitalized { at: 0 })),
                ast: json!({"RuleReference": {"Typename": "foo" }})
            }
        );
    }

    #[test]
    fn pattern() {
        let mut context = Context::default();
        let r = context.find_rule("Pattern").unwrap();
        assert_eq!(r.name, "Pattern");
        assert_eq!(
            r.parse("Foo", &mut context),
            ParseResult {
                delta: 3,
                tree: ParseTree::named("Pattern").with(
                    ParseTree::named("RuleReference")
                        .with(ParseTree::named("Typename").with("Foo"))
                ),
                ast: json!({"Pattern": { "RuleReference": {"Typename": "Foo"}}})
            }
        );
        assert_eq!(
            r.parse("foo", &mut context),
            ParseResult {
                delta: 3,
                tree: ParseTree::named("Pattern").with(ParseTree::named("Regex").with("foo")),
                ast: json!({"Pattern": {"Regex": "foo"}})
            }
        );
    }

    #[test]
    fn rule() {
        let mut context = Context::default();
        let r = context.find_rule("Rule").unwrap();
        assert_eq!(r.name, "Rule");

        let tree_text = json!({
            "Rule": [
                { "Typename": "Lol" },
                ":",
                {
                    "Pattern": {
                        "Regex": {
                            "value": "kek",
                            "trivia": " "
                        }
                    }
                }
            ]
        })
        .to_string();
        assert_eq!(
            r.parse("Lol: kek", &mut context),
            ParseResult {
                delta: 8,
                tree: serde_json::from_str(&tree_text).unwrap(),
                ast: json!({
                    "Rule": [
                        {"Typename": "Lol"},
                        ":",
                        {
                            "Pattern": { "Regex": "kek" }
                        }
                    ]
                })
            }
        )
    }
}
