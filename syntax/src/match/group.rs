use crate::{Match, PatternMatch};

/// Matched group
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GroupMatch<'source> {
    /// Matched patterns
    pub matched: Vec<PatternMatch<'source>>,
}

impl<'source> Match<'source> for GroupMatch<'source> {
    fn is_ok(&self) -> bool {
        self.matched.iter().all(|m| m.is_ok())
    }

    fn tokens(&self) -> Box<dyn Iterator<Item = &'source str> + '_> {
        Box::new(self.matched.iter().map(|m| m.tokens()).flatten())
    }
}
