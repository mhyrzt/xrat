/// Result of resolving a user-supplied ref prefix to an internal id.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefMatch {
    /// No row matched the prefix.
    None,
    /// Exactly one row matched; carries its internal id.
    Unique(i64),
    /// More than one row matched the prefix.
    Ambiguous,
}
