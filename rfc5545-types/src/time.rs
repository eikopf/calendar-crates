//! Basic time types.

use calendar_types::{
    primitive::Sign,
    time::{Date, DateTime, Hour, Minute, NonLeapSecond},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateTimeOrDate<M> {
    DateTime(DateTime<M>),
    Date(Date),
}

/// An offset from UTC to some local time (RFC 5545 §3.3.14).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UtcOffset {
    pub sign: Sign,
    pub hour: Hour,
    pub minute: Minute,
    pub second: NonLeapSecond,
}

impl std::fmt::Display for UtcOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{:02}:{:02}",
            self.sign.as_char(),
            self.hour as u8,
            self.minute as u8
        )?;
        let sec = self.second as u8;
        if sec != 0 {
            write!(f, ":{sec:02}")?;
        }
        Ok(())
    }
}
