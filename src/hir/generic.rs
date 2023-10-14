use std::borrow::Cow;

/// Trait for items that may be generic
pub trait Generic {
    /// Is this a generic item?
    fn is_generic(&self) -> bool;
}

/// Trait to get name with generic parameters
pub trait GenericName {
    /// Get generic name
    fn generic_name(&self) -> Cow<'_, str>;
}