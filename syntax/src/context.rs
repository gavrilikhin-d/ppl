use std::sync::{Arc, Mutex};

use once_cell::sync::Lazy;

use crate::{errors::TypenameNotCapitalized, Rule};

/// Current parsing context
static CONTEXT: Lazy<Mutex<Context>> = Lazy::new(|| Mutex::new(Context::default()));

/// Parsing context
pub struct Context {
    /// Parsing rules
    pub rules: Vec<Arc<Rule>>,
}

impl Default for Context {
    fn default() -> Self {
        Context {
            rules: vec![
                Arc::new(Rule {
                    name: "Regex".to_string(),
                    patterns: vec![r"[^\s]+".into()],
                    on_parsed: None,
                }),
                Arc::new(Rule {
                    name: "Typename".to_string(),
                    patterns: vec![r"[a-zA-Z0-9_]+".into()],
                    on_parsed: Some(Box::new(|at, mut res| {
                        if res.has_errors() {
                            return res;
                        }

                        let typename = res.tree.tokens().next().unwrap();
                        let first_char = typename.chars().next().unwrap();
                        if !first_char.is_ascii_uppercase() {
                            res.tree.children = vec![TypenameNotCapitalized {
                                span: (at, res.delta).into(),
                                at: at.into(),
                            }
                            .into()]
                        }

                        res
                    })),
                }),
            ],
        }
    }
}

/// Get the current parsing context
pub fn with_context<T>(f: impl FnOnce(&mut Context) -> T) -> T {
    let mut context = CONTEXT.lock().unwrap();
    f(&mut context)
}

/// Add a rule to the current parsing context
pub fn add_rule(rule: Rule) {
    with_context(|c| c.rules.push(Arc::new(rule)))
}

/// Find rule by name in the current parsing context
pub fn find_rule(name: &str) -> Option<Arc<Rule>> {
    with_context(|c| c.rules.iter().find(|r| r.name == name).map(|r| r.clone()))
}

#[cfg(test)]
mod test {
    use crate::{
        context,
        errors::TypenameNotCapitalized,
        parsers::{ParseResult, Parser},
        ParseTree,
    };

    #[test]
    fn default_rules() {
        let typename = context::find_rule("Typename").unwrap();
        assert_eq!(typename.name, "Typename");
        assert_eq!(
            typename.parse("Foo"),
            ParseResult {
                delta: 3,
                tree: ParseTree::named("Typename").with("Foo"),
            }
        );
        assert_eq!(
            typename.parse("foo"),
            ParseResult {
                delta: 3,
                tree: ParseTree::named("Typename").with(TypenameNotCapitalized {
                    span: (0, 3).into(),
                    at: 0.into()
                })
            }
        );
    }
}
