//! Types in the JSCalendar data model.

pub mod primitive;
pub mod string;
pub mod time;

// TODO: define a module for object primitives, including:
// - PatchObject (RFC 8984 §1.4.9)
// - Relation (RFC 8984 §1.4.10)
// - Link (RFC 8984 §1.4.11)
// -------------------------------------------------------
// if i'm going to be defining all these custom object types, perhaps i should also define my own
// JSON value type and be parser agnostic?
