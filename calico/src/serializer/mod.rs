//! iCalendar serialization (RFC 5545 text format).
//!
//! The [`WriteIcal`] trait provides iCalendar wire-format serialization. Unlike `Display`,
//! which uses human-readable separators (`2003-12-25T12:00:00Z`, `+08:00`), `WriteIcal`
//! produces compact representations required by RFC 5545 (`20031225T120000Z`, `+0800`).
//!
//! [`FoldingWriter`] handles RFC 5545 §3.1 line folding (75-octet limit).

mod component;
mod parameter;
mod primitive;
mod property;

pub use self::property::{write_content_line, write_prop, write_opt_prop, write_vec_prop};

use std::fmt;

/// Writes a value in iCalendar text format.
pub trait WriteIcal {
    /// Writes this value to the given writer in iCalendar wire format.
    fn write_ical(&self, w: &mut dyn fmt::Write) -> fmt::Result;

    /// Convenience method that serializes to a `String`.
    fn to_ical_string(&self) -> String {
        let mut s = String::new();
        self.write_ical(&mut s).expect("writing to String cannot fail");
        s
    }
}

/// A writer that folds content lines at 75 octets per RFC 5545 §3.1.
///
/// Each time the accumulated line length would exceed 75 bytes, a CRLF + space
/// fold sequence is inserted before continuing.
pub struct FoldingWriter<W> {
    inner: W,
    line_len: usize,
}

impl<W: fmt::Write> FoldingWriter<W> {
    /// Maximum octets per line before folding.
    const MAX_LINE_OCTETS: usize = 75;

    /// Creates a new `FoldingWriter` wrapping the given writer.
    pub fn new(inner: W) -> Self {
        Self { inner, line_len: 0 }
    }

    /// Consumes the `FoldingWriter` and returns the inner writer.
    pub fn into_inner(self) -> W {
        self.inner
    }

    /// Resets the line length counter (call after writing CRLF).
    pub fn reset_line(&mut self) {
        self.line_len = 0;
    }
}

impl<W: fmt::Write> fmt::Write for FoldingWriter<W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for ch in s.chars() {
            let ch_len = ch.len_utf8();
            if ch == '\r' || ch == '\n' {
                self.inner.write_char(ch)?;
                // We'll reset line_len on the newline char of a CRLF pair
                if ch == '\n' {
                    self.line_len = 0;
                }
                continue;
            }
            if self.line_len + ch_len > Self::MAX_LINE_OCTETS {
                self.inner.write_str("\r\n ")?;
                self.line_len = 1; // the space counts
            }
            self.inner.write_char(ch)?;
            self.line_len += ch_len;
        }
        Ok(())
    }
}

/// Escapes a TEXT value for iCalendar content lines.
///
/// Backslash-escapes semicolons, commas, backslashes, and newlines per RFC 5545 §3.3.11.
pub fn escape_text(s: &str, w: &mut dyn fmt::Write) -> fmt::Result {
    for ch in s.chars() {
        match ch {
            '\\' => w.write_str("\\\\")?,
            ';' => w.write_str("\\;")?,
            ',' => w.write_str("\\,")?,
            '\n' => w.write_str("\\n")?,
            _ => w.write_char(ch)?,
        }
    }
    Ok(())
}

/// Writes a CRLF line ending.
pub fn write_crlf(w: &mut dyn fmt::Write) -> fmt::Result {
    w.write_str("\r\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    #[test]
    fn folding_writer_short_line() {
        let mut fw = FoldingWriter::new(String::new());
        write!(fw, "SHORT").unwrap();
        assert_eq!(fw.into_inner(), "SHORT");
    }

    #[test]
    fn folding_writer_exact_75() {
        let mut fw = FoldingWriter::new(String::new());
        let line = "A".repeat(75);
        write!(fw, "{line}").unwrap();
        // Exactly 75 bytes, no fold needed
        assert_eq!(fw.into_inner(), line);
    }

    #[test]
    fn folding_writer_76_folds() {
        let mut fw = FoldingWriter::new(String::new());
        let line = "B".repeat(76);
        write!(fw, "{line}").unwrap();
        let result = fw.into_inner();
        // First 75 chars, then CRLF+space, then remaining 1 char
        assert_eq!(result, format!("{}\r\n {}", "B".repeat(75), "B"));
    }

    #[test]
    fn folding_writer_multi_fold() {
        let mut fw = FoldingWriter::new(String::new());
        let line = "C".repeat(200);
        write!(fw, "{line}").unwrap();
        let result = fw.into_inner();
        // First 75, then fold, then 74 (75 - 1 for space), then fold, then 74, ...
        // 75 + 74 + 74 = 223 > 200, so: 75 + 74 = 149, remaining 51
        // Actually: first segment 75, second segment 74 (since space takes 1), third segment rest
        assert!(result.contains("\r\n "));
        // Verify total content chars
        let content: String = result.replace("\r\n ", "");
        assert_eq!(content.len(), 200);
    }

    #[test]
    fn escape_text_basic() {
        let mut buf = String::new();
        escape_text("hello;world,foo\\bar\nbaz", &mut buf).unwrap();
        assert_eq!(buf, "hello\\;world\\,foo\\\\bar\\nbaz");
    }

    #[test]
    fn escape_text_no_escaping_needed() {
        let mut buf = String::new();
        escape_text("simple text", &mut buf).unwrap();
        assert_eq!(buf, "simple text");
    }
}
