//! Basic time types.

use calendar_types::time::{Date, DateTime};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateTimeOrDate<M> {
    DateTime(DateTime<M>),
    Date(Date),
}
