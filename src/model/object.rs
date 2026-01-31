//! Distinguished object types.

use std::collections::{HashMap, HashSet};

use super::{primitive::UnsignedInt, string::ImplicitJsonPointer};

/// A set of patches to be applied to a JSON object (RFC 8984 §1.4.9).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PatchObject<V>(HashMap<Box<ImplicitJsonPointer>, V>);

// TODO: define an appropriate value type for Relation::relations

/// A set of relationship types (RFC 8984 §1.4.10).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Relation {
    relations: HashSet<String>,
}

/// A link to an external resource (RFC 8984 §1.4.11).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Link {
    uri: String,
    content_id: Option<String>,
    media_type: Option<String>,
    size: Option<UnsignedInt>,
    relation: Option<String>,
    display: Option<String>,
    title: Option<String>,
}
