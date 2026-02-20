//! Property parameters.
//!
//! # Grammar
//!
//! The ABNF grammar in RFC 5545 §3.1 defines a parameter as follows:
//!
//! ```custom,{class=language-abnf}
//! param       = param-name "=" param-value *("," param-value)
//! param-name  = iana-token / x-name
//! param-value = paramtext / quoted-string
//! paramtext   = *SAFE-CHAR
//! ```
//!
//! This is a rough general rule, and each `param-name` is associated with a
//! more specific grammar for the corresponding value on the right-hand side of
//! the `=` character.

use mitsein::vec1::Vec1;
use winnow::{
    Parser,
    ascii::Caseless,
    combinator::{delimited, preceded, terminated},
    error::{FromExternalError, ParserError},
    stream::{AsBStr, AsChar, Compare, SliceLen, Stream, StreamIsPartial},
};

use crate::{
    model::{
        parameter::{KnownParam, Param, ParamName, StaticParam, UnknownParam, UnknownParamValue},
        string::{Name, Uri},
    },
    parser::{
        InputStream,
        primitive::{
            alarm_trigger_relationship, ascii_lower, bool_caseless, comma_seq1, feature_type,
            format_type, free_busy_type, inline_encoding, language, param_value,
            participation_role, participation_status, positive_integer, relationship_type, tz_id,
            uri, value_type,
        },
    },
};

use super::{
    error::CalendarParseError,
    primitive::{calendar_user_type, display_type, name},
};

/// Parses a [`Param`].
pub fn parameter<I, E>(input: &mut I) -> Result<Param, E>
where
    I: InputStream,
    I::Token: AsChar + Clone,
    I::Slice: AsBStr + Clone + SliceLen,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    /// Parses a single URI delimited by double quotes.
    fn quoted_uri<I, E>(input: &mut I) -> Result<Box<Uri>, E>
    where
        I: InputStream,
        I::Token: AsChar + Clone,
        E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
    {
        delimited('"', uri::<_, _, true>, '"').parse_next(input)
    }

    fn quoted_addresses<I, E>(input: &mut I) -> Result<Vec1<Box<Uri>>, E>
    where
        I: InputStream,
        I::Token: AsChar + Clone,
        E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
    {
        comma_seq1(quoted_uri).parse_next(input)
    }

    let name = terminated(param_name, '=').parse_next(input)?;

    match name {
        ParamName::Unknown(name) => {
            let kind = name.kind();
            let values = comma_seq1(param_value).parse_next(input)?;

            Ok(Param::Unknown(UnknownParam {
                name,
                value: UnknownParamValue { kind, values },
            }))
        }
        ParamName::Known(name) => match name {
            // RFC 5545 PARAMETERS
            StaticParam::AltRep => quoted_uri.map(KnownParam::AltRep).parse_next(input),
            StaticParam::CommonName => param_value.map(KnownParam::CommonName).parse_next(input),
            StaticParam::CalUserType => {
                calendar_user_type.map(KnownParam::CUType).parse_next(input)
            }
            StaticParam::DelFrom => quoted_addresses.map(KnownParam::DelFrom).parse_next(input),
            StaticParam::DelTo => quoted_addresses.map(KnownParam::DelTo).parse_next(input),
            StaticParam::Dir => quoted_uri.map(KnownParam::Dir).parse_next(input),
            StaticParam::Encoding => inline_encoding.map(KnownParam::Encoding).parse_next(input),
            StaticParam::FormatType => format_type.map(KnownParam::FormatType).parse_next(input),
            StaticParam::FreeBusyType => free_busy_type.map(KnownParam::FBType).parse_next(input),
            StaticParam::Language => language.map(KnownParam::Language).parse_next(input),
            StaticParam::Member => quoted_addresses.map(KnownParam::Member).parse_next(input),
            StaticParam::PartStat => participation_status
                .map(KnownParam::PartStatus)
                .parse_next(input),
            StaticParam::Range => Caseless("THISANDFUTURE")
                .value(KnownParam::RecurrenceIdentifierRange)
                .parse_next(input),
            StaticParam::Related => alarm_trigger_relationship
                .map(KnownParam::AlarmTrigger)
                .parse_next(input),
            StaticParam::RelType => relationship_type.map(KnownParam::RelType).parse_next(input),
            StaticParam::Role => participation_role.map(KnownParam::Role).parse_next(input),
            StaticParam::Rsvp => bool_caseless.map(KnownParam::Rsvp).parse_next(input),
            StaticParam::SentBy => quoted_uri.map(KnownParam::SentBy).parse_next(input),
            StaticParam::TzId => tz_id.map(KnownParam::TzId).parse_next(input),
            StaticParam::Value => value_type.map(KnownParam::Value).parse_next(input),

            // RFC 7986 PARAMETERS
            StaticParam::Display => display_type.map(KnownParam::Display).parse_next(input),
            StaticParam::Email => param_value.map(KnownParam::Email).parse_next(input),
            StaticParam::Feature => feature_type.map(KnownParam::Feature).parse_next(input),
            StaticParam::Label => param_value.map(KnownParam::Label).parse_next(input),

            // RFC 9073 PARAMETERS
            StaticParam::Order => positive_integer.map(KnownParam::Order).parse_next(input),
            StaticParam::Schema => quoted_uri.map(KnownParam::Schema).parse_next(input),
            StaticParam::Derived => bool_caseless.map(KnownParam::Derived).parse_next(input),
        }
        .map(Param::Known),
    }
}

/// Parses a [`ParamName`].
pub fn param_name<I, E>(input: &mut I) -> Result<ParamName<Box<Name>>, E>
where
    I: InputStream,
    I::Token: AsChar + Clone,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    fn static_name<I>(input: &mut I) -> Result<StaticParam, ()>
    where
        I: StreamIsPartial + Stream + Compare<Caseless<&'static str>> + Compare<char>,
        I::Token: AsChar + Clone,
    {
        macro_rules! tail {
            ($s:literal, $c:expr) => {
                Caseless($s).value($c).parse_next(input)
            };
        }

        match ascii_lower.parse_next(input)? {
            'a' => tail!("ltrep", StaticParam::AltRep),
            // CN | CUTYPE
            'c' => match ascii_lower.parse_next(input)? {
                'n' => tail!("", StaticParam::CommonName),
                'u' => tail!("type", StaticParam::CalUserType),
                _ => Err(()),
            },
            // DELEGATED-FROM | DELEGATED-TO | DIR | DISPLAY
            'd' => match ascii_lower.parse_next(input)? {
                // DELEGATED-FROM | DELEGATED-TO
                'e' => match preceded(Caseless("legated-"), ascii_lower).parse_next(input)? {
                    'f' => tail!("rom", StaticParam::DelFrom),
                    't' => tail!("o", StaticParam::DelTo),
                    _ => Err(()),
                },
                // DIR | DISPLAY
                'i' => match ascii_lower.parse_next(input)? {
                    'r' => tail!("", StaticParam::Dir),
                    's' => tail!("play", StaticParam::Display),
                    _ => Err(()),
                },
                _ => Err(()),
            },
            // ENCODING | EMAIL
            'e' => match ascii_lower.parse_next(input)? {
                'm' => tail!("ail", StaticParam::Email),
                'n' => tail!("coding", StaticParam::Encoding),
                _ => Err(()),
            },
            // FMTTYPE | FBTYPE | FEATURE
            'f' => match ascii_lower.parse_next(input)? {
                'b' => tail!("type", StaticParam::FreeBusyType),
                'e' => tail!("ature", StaticParam::Feature),
                'm' => tail!("ttype", StaticParam::FormatType),
                _ => Err(()),
            },
            // LABEL | LANGUAGE
            'l' => match preceded(Caseless("a"), ascii_lower).parse_next(input)? {
                'b' => tail!("el", StaticParam::Label),
                'n' => tail!("guage", StaticParam::Language),
                _ => Err(()),
            },
            'm' => tail!("ember", StaticParam::Member),
            'p' => tail!("artstat", StaticParam::PartStat),
            // RANGE | RELATED | RELTYPE | ROLE | RSVP
            'r' => match ascii_lower.parse_next(input)? {
                'a' => tail!("nge", StaticParam::Range),
                // RELATED | RELTYPE
                'e' => match preceded(Caseless("l"), ascii_lower).parse_next(input)? {
                    'a' => {
                        tail!("ted", StaticParam::Related)
                    }
                    't' => tail!("ype", StaticParam::RelType),
                    _ => Err(()),
                },
                'o' => tail!("le", StaticParam::Role),
                's' => tail!("vp", StaticParam::Rsvp),
                _ => Err(()),
            },
            's' => tail!("ent-by", StaticParam::SentBy),
            't' => tail!("zid", StaticParam::TzId),
            'v' => tail!("alue", StaticParam::Value),
            _ => Err(()),
        }
    }

    let checkpoint = input.checkpoint();

    match static_name.parse_next(input) {
        Ok(name) => Ok(ParamName::Known(name)),
        Err(()) => {
            input.reset(&checkpoint);
            name.map(ParamName::Unknown).parse_next(input)
        }
    }
}
#[cfg(test)]
mod tests {
    use mitsein::{iter1::IntoIterator1, vec1};

    use crate::{
        model::{
            parameter::KnownParam,
            primitive::{DisplayType, Encoding, FeatureType, Token, TriggerRelation, ValueType},
            string::{ParamValue, TzId},
        },
        text,
    };

    use super::*;

    #[test]
    fn simple_parameter_parsing() {
        assert_eq!(
            parameter::<_, ()>
                .parse_peek("VALUE=CAL-ADDRESS")
                .ok()
                .and_then(|(_, p)| p.try_into_known().ok()),
            Some(KnownParam::Value(Token::Known(ValueType::CalAddress))),
        );

        assert_eq!(
            parameter::<_, ()>
                .parse_peek("tzid=America/New_York")
                .ok()
                .and_then(|(_, p)| p.try_into_known().ok()),
            Some(KnownParam::TzId(TzId::from_boxed_text(text!(
                "America/New_York"
            )))),
        );

        assert_eq!(
            parameter::<_, ()>
                .parse_peek("Rsvp=FALSE")
                .ok()
                .and_then(|(_, p)| p.try_into_known().ok()),
            Some(KnownParam::Rsvp(false)),
        );

        assert_eq!(
            parameter::<_, ()>
                .parse_peek("RANGE=thisandfuture")
                .ok()
                .and_then(|(_, p)| p.try_into_known().ok()),
            Some(KnownParam::RecurrenceIdentifierRange),
        );
    }

    #[test]
    fn inline_encoding() {
        for input in ["ENCODING=8BIT", "ENCODING=8Bit", "ENCODING=8bit"] {
            assert_eq!(
                parameter::<_, ()>
                    .parse_peek(input)
                    .ok()
                    .and_then(|(_, p)| p.try_into_known().ok()),
                Some(KnownParam::Encoding(Encoding::Bit8)),
            );
        }

        for input in ["ENCODING=BASE64", "ENCODING=Base64", "ENCODING=base64"] {
            assert_eq!(
                parameter::<_, ()>
                    .parse_peek(input)
                    .ok()
                    .and_then(|(_, p)| p.try_into_known().ok()),
                Some(KnownParam::Encoding(Encoding::Base64)),
            );
        }

        for input in ["ENCODING=base", "ENCODING=bit", "ENCODING=64", "ENCODING=8"] {
            assert!(parameter::<_, ()>.parse_peek(input).is_err());
        }
    }

    #[test]
    fn format_type() {
        for input in [
            "FMTTYPE=audio/aac",
            "FMTTYPE=image/bmp",
            "FMTTYPE=text/css",
            "FMTTYPE=application/x-bzip",
        ] {
            assert!(matches!(
                parameter::<_, ()>
                    .parse_peek(input)
                    .ok()
                    .and_then(|(_, p)| p.try_into_known().ok()),
                Some(KnownParam::FormatType(_))
            ));
        }

        for input in ["FMTTYPE=", "FMTTYPE=missing slash", "FMTTYPE=back\\slash"] {
            assert!(parameter::<_, ()>.parse_peek(input).is_err());
        }
    }

    #[test]
    fn recurrence_identifier_range() {
        for input in [
            "RANGE=THISANDFUTURE",
            "RANGE=ThisAndFuture",
            "RANGE=thisandfuture",
        ] {
            assert!(matches!(
                parameter::<_, ()>
                    .parse_peek(input)
                    .ok()
                    .and_then(|(_, p)| p.try_into_known().ok()),
                Some(KnownParam::RecurrenceIdentifierRange)
            ));
        }

        for input in ["RANGE=", "RANGE=garbage", "RANGE=this-and-future"] {
            assert!(parameter::<_, ()>.parse_peek(input).is_err());
        }
    }

    #[test]
    fn alarm_trigger_relationship() {
        for input in ["RELATED=START", "RELATED=Start", "RELATED=start"] {
            assert!(matches!(
                parameter::<_, ()>
                    .parse_peek(input)
                    .ok()
                    .and_then(|(_, p)| p.try_into_known().ok()),
                Some(KnownParam::AlarmTrigger(TriggerRelation::Start)),
            ));
        }

        for input in ["RELATED=END", "RELATED=End", "RELATED=end"] {
            assert!(matches!(
                parameter::<_, ()>
                    .parse_peek(input)
                    .ok()
                    .and_then(|(_, p)| p.try_into_known().ok()),
                Some(KnownParam::AlarmTrigger(TriggerRelation::End)),
            ));
        }

        for input in ["RELATED=", "RELATED=,garbage", "RELATED=anything-else"] {
            assert!(parameter::<_, ()>.parse_peek(input).is_err());
        }
    }

    #[test]
    fn parameter_edge_cases() {
        assert!(parameter::<_, ()>.parse_peek("VALUE=").is_err()); // missing value
        assert!(parameter::<_, ()>.parse_peek("=RECUR").is_err()); // missing name
        assert!(parameter::<_, ()>.parse_peek("=").is_err()); // missing name & value

        // trailing semicolon should not be stripped
        assert_eq!(
            parameter::<_, ()>.parse_peek("LANGUAGE=en-GB;").unwrap().0,
            ";"
        );
    }

    #[test]
    fn rfc7986_parameter_parsing() {
        assert_eq!(
            parameter::<_, ()>
                .parse_peek("DISPLAY=THUMBNAIL")
                .ok()
                .and_then(|(_, p)| p.try_into_known().ok()),
            Some(KnownParam::Display(Token::Known(DisplayType::Thumbnail))),
        );

        assert_eq!(
            parameter::<_, ()>
                .parse_peek("display=Badge")
                .ok()
                .and_then(|(_, p)| p.try_into_known().ok()),
            Some(KnownParam::Display(Token::Known(DisplayType::Badge))),
        );

        assert_eq!(
            parameter::<_, ()>
                .parse_peek("DISPLAY=X-SOMETHING-ELSE")
                .ok()
                .and_then(|(_, p)| p.try_into_known().ok()),
            Some(KnownParam::Display(Token::Unknown(
                Name::new("X-SOMETHING-ELSE").unwrap().into()
            ))),
        );

        assert_eq!(
            parameter::<_, ()>
                .parse_peek("Email=literally anything")
                .ok()
                .and_then(|(_, p)| p.try_into_known().ok()),
            Some(KnownParam::Email(
                ParamValue::new("literally anything").unwrap().into()
            )),
        );

        assert_eq!(
            parameter::<_, ()>
                .parse_peek("Email=\"a quoted string\"")
                .ok()
                .and_then(|(_, p)| p.try_into_known().ok()),
            Some(KnownParam::Email(
                ParamValue::new("a quoted string").unwrap().into()
            )),
        );

        assert_eq!(
            parameter::<_, ()>
                .parse_peek("FEATURE=moderator")
                .ok()
                .and_then(|(_, p)| p.try_into_known().ok()),
            Some(KnownParam::Feature(Token::Known(FeatureType::Moderator))),
        );

        assert_eq!(
            parameter::<_, ()>
                .parse_peek("feature=Screen")
                .ok()
                .and_then(|(_, p)| p.try_into_known().ok()),
            Some(KnownParam::Feature(Token::Known(FeatureType::Screen))),
        );

        assert_eq!(
            parameter::<_, ()>
                .parse_peek("feature=random-iana-token")
                .ok()
                .and_then(|(_, p)| p.try_into_known().ok()),
            Some(KnownParam::Feature(Token::Unknown(
                Name::new("random-iana-token").unwrap().into()
            ))),
        );

        assert_eq!(
            parameter::<_, ()>
                .parse_peek("LABEL=some text")
                .ok()
                .and_then(|(_, p)| p.try_into_known().ok()),
            Some(KnownParam::Label(
                ParamValue::new("some text").unwrap().into()
            )),
        );

        assert_eq!(
            parameter::<_, ()>
                .parse_peek("label=\"some quoted text\"")
                .ok()
                .and_then(|(_, p)| p.try_into_known().ok()),
            Some(KnownParam::Label(
                ParamValue::new("some quoted text").unwrap().into()
            )),
        );
    }

    #[test]
    fn multiple_uris() {
        let param = concat!(
            "DELEGATED-FROM=",
            "\"mailto:alice@place.com\",",
            "\"mailto:brice@place.com\",",
            "\"mailto:carla@place.com\"",
        );

        let uris = vec1![
            Box::<str>::from("mailto:alice@place.com"),
            Box::<str>::from("mailto:brice@place.com"),
            Box::<str>::from("mailto:carla@place.com"),
        ]
        .into_iter1()
        .map(|s| {
            // SAFETY: these literals are definitely valid URIs
            unsafe { Uri::from_boxed_unchecked(s) }
        })
        .collect1();

        assert_eq!(
            parameter::<_, ()>.parse_peek(param).map(|(_, p)| p),
            Ok(Param::Known(KnownParam::DelFrom(uris))),
        );
    }
}
