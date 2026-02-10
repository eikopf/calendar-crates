//! Types for values which appear in sets.

/// A value which may appear in the `relation` field of a `Relation` object (RFC 8984 §1.4.10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum RelationValue<S> {
    First,
    Next,
    Child,
    Parent,
    Other(S),
}
