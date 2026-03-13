//! `WriteIcal` implementations for property parameters.

use std::fmt;

use mitsein::vec1::Vec1;

use super::WriteIcal;
use crate::model::{
    parameter::{Params, StructuredDataParams},
    string::{ParamValue, Uri},
};

/// Writes a parameter value that must be quoted (URI values per RFC 5545 §3.2).
fn write_quoted_uri<W: fmt::Write>(uri: &Uri, w: &mut W) -> fmt::Result {
    w.write_char('"')?;
    w.write_str(uri.as_str())?;
    w.write_char('"')
}

/// Writes a list of quoted URIs, comma-separated.
fn write_quoted_uri_list<W: fmt::Write>(uris: &Vec1<Box<Uri>>, w: &mut W) -> fmt::Result {
    for (i, uri) in uris.iter().enumerate() {
        if i > 0 {
            w.write_char(',')?;
        }
        write_quoted_uri(uri, w)?;
    }
    Ok(())
}

/// Writes a `ParamValue`, quoting it if it contains characters that require quoting
/// (colons, semicolons, commas, or spaces).
fn write_param_value<W: fmt::Write>(pv: &ParamValue, w: &mut W) -> fmt::Result {
    let s = pv.as_str();
    write_maybe_quoted(s, w)
}

/// Writes a string, quoting it if it contains `:`, `;`, `,`, or space.
fn write_maybe_quoted<W: fmt::Write>(s: &str, w: &mut W) -> fmt::Result {
    let needs_quoting = s.contains(':') || s.contains(';') || s.contains(',') || s.contains(' ');
    if needs_quoting {
        w.write_char('"')?;
        w.write_str(s)?;
        w.write_char('"')
    } else {
        w.write_str(s)
    }
}

/// Writes common (shared) parameters that appear in both `Params` and `StructuredDataParams`.
///
/// This is a macro because both types have the same field names but are different types.
macro_rules! write_shared_params {
    ($self:expr, $w:expr) => {{
        let w = $w;
        if let Some(uri) = $self.alternate_representation() {
            w.write_str(";ALTREP=")?;
            write_quoted_uri(uri, w)?;
        }
        if let Some(cn) = $self.common_name() {
            w.write_str(";CN=")?;
            write_param_value(cn, w)?;
        }
        if let Some(cutype) = $self.calendar_user_type() {
            w.write_str(";CUTYPE=")?;
            cutype.write_ical(w)?;
        }
        if let Some(del_from) = $self.delegated_from() {
            w.write_str(";DELEGATED-FROM=")?;
            write_quoted_uri_list(del_from, w)?;
        }
        if let Some(del_to) = $self.delegated_to() {
            w.write_str(";DELEGATED-TO=")?;
            write_quoted_uri_list(del_to, w)?;
        }
        if let Some(dir) = $self.directory_reference() {
            w.write_str(";DIR=")?;
            write_quoted_uri(dir, w)?;
        }
        if let Some(enc) = $self.inline_encoding() {
            w.write_str(";ENCODING=")?;
            enc.write_ical(w)?;
        }
        if let Some(ft) = $self.free_busy_type() {
            w.write_str(";FBTYPE=")?;
            ft.write_ical(w)?;
        }
        if let Some(lang) = $self.language() {
            w.write_str(";LANGUAGE=")?;
            w.write_str(lang.as_str())?;
        }
        if let Some(member) = $self.membership() {
            w.write_str(";MEMBER=")?;
            write_quoted_uri_list(member, w)?;
        }
        if let Some(ps) = $self.participation_status() {
            w.write_str(";PARTSTAT=")?;
            ps.write_ical(w)?;
        }
        if $self.recurrence_range().is_some() {
            w.write_str(";RANGE=THISANDFUTURE")?;
        }
        if let Some(rel) = $self.trigger_relationship() {
            w.write_str(";RELATED=")?;
            rel.write_ical(w)?;
        }
        if let Some(rt) = $self.relationship_type() {
            w.write_str(";RELTYPE=")?;
            rt.write_ical(w)?;
        }
        if let Some(role) = $self.participation_role() {
            w.write_str(";ROLE=")?;
            role.write_ical(w)?;
        }
        if let Some(rsvp) = $self.rsvp_expectation() {
            w.write_str(";RSVP=")?;
            w.write_str(if *rsvp { "TRUE" } else { "FALSE" })?;
        }
        if let Some(sb) = $self.sent_by() {
            w.write_str(";SENT-BY=")?;
            write_quoted_uri(sb, w)?;
        }
        if let Some(tz) = $self.tz_id() {
            w.write_str(";TZID=")?;
            write_maybe_quoted(tz.as_str(), w)?;
        }
        // RFC 7986
        if let Some(dt) = $self.display_type() {
            w.write_str(";DISPLAY=")?;
            dt.write_ical(w)?;
        }
        if let Some(email) = $self.email() {
            w.write_str(";EMAIL=")?;
            write_param_value(email, w)?;
        }
        if let Some(ft) = $self.feature_type() {
            w.write_str(";FEATURE=")?;
            ft.write_ical(w)?;
        }
        if let Some(label) = $self.label() {
            w.write_str(";LABEL=")?;
            write_param_value(label, w)?;
        }
        // RFC 9073
        if let Some(order) = $self.order() {
            write!(w, ";ORDER={}", order.get())?;
        }
        if let Some(derived) = $self.derived() {
            w.write_str(";DERIVED=")?;
            w.write_str(if *derived { "TRUE" } else { "FALSE" })?;
        }
        // Unknown params
        for (name, values) in $self.unknown_param_iter() {
            w.write_char(';')?;
            // CaselessStr stores the original casing
            w.write_str(name.as_str())?;
            w.write_char('=')?;
            for (i, val) in values.iter().enumerate() {
                if i > 0 {
                    w.write_char(',')?;
                }
                write_param_value(val, w)?;
            }
        }
        fmt::Result::Ok(())
    }};
}

impl WriteIcal for Params {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        if let Some(ft) = self.format_type() {
            w.write_str(";FMTTYPE=")?;
            w.write_str(ft.as_str())?;
        }
        if let Some(schema) = self.schema() {
            w.write_str(";SCHEMA=")?;
            write_quoted_uri(schema, w)?;
        }
        write_shared_params!(self, w)
    }
}

impl WriteIcal for StructuredDataParams {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        // Required params
        w.write_str(";FMTTYPE=")?;
        w.write_str(self.format_type().as_str())?;
        w.write_str(";SCHEMA=")?;
        write_quoted_uri(self.schema(), w)?;
        write_shared_params!(self, w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::parameter::Params;

    #[test]
    fn empty_params() {
        let params = Params::default();
        assert_eq!(params.to_ical_string(), "");
    }

    #[test]
    fn params_with_rsvp() {
        let mut params = Params::default();
        params.set_rsvp_expectation(true);
        assert_eq!(params.to_ical_string(), ";RSVP=TRUE");
    }

    #[test]
    fn params_with_language() {
        let mut params = Params::default();
        let lang = calendar_types::string::LanguageTag::parse("en-US").unwrap().into();
        params.set_language(lang);
        assert_eq!(params.to_ical_string(), ";LANGUAGE=en-US");
    }
}
