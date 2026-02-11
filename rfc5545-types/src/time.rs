//! Basic time types.

pub use calendar_types::{duration::*, time::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateTimeOrDate<M> {
    DateTime(DateTime<M>),
    Date(Date),
}
