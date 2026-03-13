//! Content line writing helpers and property serialization.

use std::fmt;

use super::{WriteIcal, escape_text, write_crlf};
use crate::model::{
    parameter::Params,
    primitive::Value,
    property::{Prop, StructuredDataProp},
    string::CaselessStr,
};

/// Writes a content line: `NAME` + params + `:` + value + CRLF.
pub fn write_content_line<P: WriteIcal, V: WriteIcal, W: fmt::Write>(
    name: &str,
    params: &P,
    value: &V,
    w: &mut W,
) -> fmt::Result {
    w.write_str(name)?;
    params.write_ical(w)?;
    w.write_str(":")?;
    value.write_ical(w)?;
    write_crlf(w)
}

/// Writes a single `Prop<V, P>` as a content line.
pub fn write_prop<V: WriteIcal, P: WriteIcal, W: fmt::Write>(
    name: &str,
    prop: &Prop<V, P>,
    w: &mut W,
) -> fmt::Result {
    write_content_line(name, &prop.params, &prop.value, w)
}

/// Writes an optional prop if present. Accepts `Option<&Prop>` (structible getter style).
pub fn write_opt_prop<V: WriteIcal, P: WriteIcal, W: fmt::Write>(
    name: &str,
    prop: Option<&Prop<V, P>>,
    w: &mut W,
) -> fmt::Result {
    if let Some(p) = prop {
        write_prop(name, p, w)?;
    }
    Ok(())
}

/// Writes a vec of props if present. Accepts `Option<&Vec<Prop>>` (structible getter style).
pub fn write_vec_prop<V: WriteIcal, P: WriteIcal, W: fmt::Write>(
    name: &str,
    props: Option<&Vec<Prop<V, P>>>,
    w: &mut W,
) -> fmt::Result {
    if let Some(ps) = props {
        for p in ps {
            write_prop(name, p, w)?;
        }
    }
    Ok(())
}

/// Writes a `StructuredDataProp`.
pub fn write_structured_data_prop<W: fmt::Write>(
    prop: &StructuredDataProp,
    w: &mut W,
) -> fmt::Result {
    match prop {
        StructuredDataProp::Binary(p) => {
            w.write_str("STRUCTURED-DATA;VALUE=BINARY")?;
            p.params.write_ical(w)?;
            w.write_str(":")?;
            let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &p.value);
            w.write_str(&encoded)?;
            write_crlf(w)
        }
        StructuredDataProp::Text(p) => {
            w.write_str("STRUCTURED-DATA;VALUE=TEXT")?;
            p.params.write_ical(w)?;
            w.write_str(":")?;
            escape_text(&p.value, w)?;
            write_crlf(w)
        }
        StructuredDataProp::Uri(p) => {
            w.write_str("STRUCTURED-DATA;VALUE=URI")?;
            p.params.write_ical(w)?;
            w.write_str(":")?;
            w.write_str(p.value.as_str())?;
            write_crlf(w)
        }
    }
}

/// Writes a vec of `StructuredDataProp`s.
pub fn write_structured_data_props<W: fmt::Write>(
    props: Option<&Vec<StructuredDataProp>>,
    w: &mut W,
) -> fmt::Result {
    if let Some(ps) = props {
        for p in ps {
            write_structured_data_prop(p, w)?;
        }
    }
    Ok(())
}

/// Writes x-properties by iterating the structible catch-all.
pub fn write_x_property_iter<'a, W: fmt::Write>(
    iter: impl Iterator<Item = (&'a Box<CaselessStr>, &'a Vec<Prop<Value<String>, Params>>)>,
    w: &mut W,
) -> fmt::Result {
    for (name, props) in iter {
        for prop in props {
            w.write_str(name.as_str())?;
            prop.params.write_ical(w)?;
            w.write_str(":")?;
            prop.value.write_ical(w)?;
            write_crlf(w)?;
        }
    }
    Ok(())
}

/// Writes a DateTimeOrDate property with correct VALUE parameter.
pub fn write_dtod_prop<W: fmt::Write>(
    name: &str,
    prop: &Prop<rfc5545_types::time::DateTimeOrDate, Params>,
    w: &mut W,
) -> fmt::Result {
    w.write_str(name)?;
    if prop.value.is_date() {
        w.write_str(";VALUE=DATE")?;
    }
    prop.params.write_ical(w)?;
    w.write_str(":")?;
    prop.value.write_ical(w)?;
    write_crlf(w)
}

/// Writes an optional DateTimeOrDate property.
pub fn write_opt_dtod_prop<W: fmt::Write>(
    name: &str,
    prop: Option<&Prop<rfc5545_types::time::DateTimeOrDate, Params>>,
    w: &mut W,
) -> fmt::Result {
    if let Some(p) = prop {
        write_dtod_prop(name, p, w)?;
    }
    Ok(())
}

/// Writes RDATE properties with correct VALUE parameter for date-only sequences.
pub fn write_rdate_seq_prop<W: fmt::Write>(
    name: &str,
    prop: &Prop<rfc5545_types::time::RDateSeq, Params>,
    w: &mut W,
) -> fmt::Result {
    w.write_str(name)?;
    match &prop.value {
        rfc5545_types::time::RDateSeq::Date(_) => w.write_str(";VALUE=DATE")?,
        rfc5545_types::time::RDateSeq::Period(_) => w.write_str(";VALUE=PERIOD")?,
        rfc5545_types::time::RDateSeq::DateTime(_) => {}
    }
    prop.params.write_ical(w)?;
    w.write_str(":")?;
    prop.value.write_ical(w)?;
    write_crlf(w)
}

/// Writes EXDATE property with correct VALUE parameter.
pub fn write_exdate_prop<W: fmt::Write>(
    prop: &Prop<rfc5545_types::time::DateTimeOrDate, Params>,
    w: &mut W,
) -> fmt::Result {
    w.write_str("EXDATE")?;
    if prop.value.is_date() {
        w.write_str(";VALUE=DATE")?;
    }
    prop.params.write_ical(w)?;
    w.write_str(":")?;
    prop.value.write_ical(w)?;
    write_crlf(w)
}

/// Writes an Attachment property with correct VALUE/ENCODING params.
pub fn write_attach_prop<W: fmt::Write>(
    name: &str,
    prop: &Prop<rfc5545_types::value::Attachment, Params>,
    w: &mut W,
) -> fmt::Result {
    w.write_str(name)?;
    match &prop.value {
        rfc5545_types::value::Attachment::Binary(_) => {
            w.write_str(";VALUE=BINARY")?;
        }
        rfc5545_types::value::Attachment::Uri(_) => {}
    }
    prop.params.write_ical(w)?;
    w.write_str(":")?;
    prop.value.write_ical(w)?;
    write_crlf(w)
}

/// Writes a StyledDescriptionValue property with correct VALUE param.
pub fn write_styled_description_prop<W: fmt::Write>(
    prop: &Prop<rfc5545_types::value::StyledDescriptionValue, Params>,
    w: &mut W,
) -> fmt::Result {
    w.write_str("STYLED-DESCRIPTION")?;
    match &prop.value {
        rfc5545_types::value::StyledDescriptionValue::Text(_) => {
            w.write_str(";VALUE=TEXT")?;
        }
        rfc5545_types::value::StyledDescriptionValue::Uri(_) => {
            w.write_str(";VALUE=URI")?;
        }
        rfc5545_types::value::StyledDescriptionValue::Iana { value_type, .. } => {
            w.write_str(";VALUE=")?;
            w.write_str(value_type)?;
        }
    }
    prop.params.write_ical(w)?;
    w.write_str(":")?;
    prop.value.write_ical(w)?;
    write_crlf(w)
}

/// Writes a TriggerValue property with correct VALUE param.
pub fn write_trigger_prop<W: fmt::Write>(
    prop: &Prop<rfc5545_types::time::TriggerValue, Params>,
    w: &mut W,
) -> fmt::Result {
    w.write_str("TRIGGER")?;
    match &prop.value {
        rfc5545_types::time::TriggerValue::DateTime(_) => {
            w.write_str(";VALUE=DATE-TIME")?;
        }
        rfc5545_types::time::TriggerValue::Duration(_) => {}
    }
    prop.params.write_ical(w)?;
    w.write_str(":")?;
    prop.value.write_ical(w)?;
    write_crlf(w)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::parameter::Params;

    #[test]
    fn simple_content_line() {
        let mut buf = String::new();
        let params = Params::default();
        let value = "Hello World";
        write_content_line("SUMMARY", &params, &value.to_string(), &mut buf).unwrap();
        assert_eq!(buf, "SUMMARY:Hello World\r\n");
    }

    #[test]
    fn content_line_with_escaping() {
        let mut buf = String::new();
        let params = Params::default();
        let value = "Meeting; with, team\nSecond line".to_string();
        write_content_line("DESCRIPTION", &params, &value, &mut buf).unwrap();
        assert_eq!(
            buf,
            "DESCRIPTION:Meeting\\; with\\, team\\nSecond line\r\n"
        );
    }
}
