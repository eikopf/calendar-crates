//! Parsers for properties.

use winnow::{
    Parser,
    ascii::Caseless,
    combinator::{fail, preceded, separated},
    error::{FromExternalError, ParserError},
    stream::{AsBStr, AsChar, Compare, SliceLen, Stream, StreamIsPartial},
    token::take_while,
};

use crate::{
    model::{
        css::Css3Color,
        parameter::{Param, Params, StaticParam, UnknownParam, UpcastParamValue},
        primitive::{
            Attachment, ClassValue, CompletionPercentage, DateTime, DateTimeOrDate, Encoding,
            ExDateSeq, Geo, Gregorian, Integer, Method, ParticipantType, Period, Priority,
            ProximityValue, RDateSeq, RequestStatus, ResourceType, SignedDuration, Status,
            StyledDescriptionValue, TimeTransparency, Token, TriggerValue, Utc, UtcOffset, Value,
            ValueType, Version,
        },
        property::{Prop, StaticProp, StructuredDataProp},
        rrule::RRule,
        string::{CaselessStr, NameKind, TzId, Uid, Uri},
    },
    parser::{
        InputStream,
        config::{Config, DefaultConfig},
        error::CalendarParseError,
        parameter::parameter,
        primitive::{
            self, alarm_action, ascii_lower, binary, binary_with_config, bool_caseless,
            class_value, color, completion_percentage, datetime, datetime_utc,
            duration, geo_with_config, gregorian, integer, method, participant_type,
            period, priority, proximity_value, request_status, resource_type, status, text,
            text_seq, time, time_transparency, tz_id, uid, uri, utc_offset, version,
        },
        rrule::rrule,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedProp<S> {
    Known(KnownProp),
    Unknown(UnknownProp<S>),
}

impl<S> ParsedProp<S> {
    #[allow(clippy::result_large_err)]
    pub fn try_into_known(self) -> Result<KnownProp, Self> {
        if let Self::Known(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }

    #[allow(clippy::result_large_err)]
    pub fn try_into_unknown(self) -> Result<UnknownProp<S>, Self> {
        if let Self::Unknown(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }
}

/// A known property with its name and typed value.
#[derive(Debug, Clone, PartialEq)]
pub struct KnownProp {
    pub name: StaticProp,
    pub value: PropValue,
}

/// Typed property values produced by the property parser.
///
/// Each variant wraps a `Prop<V, Params>` matching the model type for that
/// property, with the value and parameters already parsed.
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum PropValue {
    // Simple text
    Text(Prop<String, Params>),
    // Comma-separated text sequence (CATEGORIES, RESOURCES, LOCATION-TYPE)
    TextSeq(Prop<Vec<String>, Params>),
    // Calendar-level
    Version(Prop<Version, Params>),
    Gregorian(Prop<Token<Gregorian, String>, Params>),
    Method(Prop<Token<Method, String>, Params>),
    // Identifiers
    Uid(Prop<Box<Uid>, Params>),
    Uri(Prop<Box<Uri>, Params>),
    TzId(Prop<Box<TzId>, Params>),
    // DateTime
    DateTimeUtc(Prop<DateTime<Utc>, Params>),
    DateTimeOrDate(Prop<DateTimeOrDate, Params>),
    // Duration
    Duration(Prop<SignedDuration, Params>),
    // Numeric
    Integer(Prop<Integer, Params>),
    Priority(Prop<Priority, Params>),
    CompletionPercentage(Prop<CompletionPercentage, Params>),
    Geo(Prop<Geo, Params>),
    Float(Prop<f64, Params>),
    Boolean(Prop<bool, Params>),
    // Enum types
    Status(Prop<Status, Params>),
    ClassValue(Prop<Token<ClassValue, String>, Params>),
    TimeTransparency(Prop<TimeTransparency, Params>),
    UtcOffset(Prop<UtcOffset, Params>),
    // Attachment (ATTACH, IMAGE)
    Attachment(Prop<Attachment, Params>),
    // Recurrence
    RRule(Prop<RRule, Params>),
    ExRule(Prop<RRule, Params>),
    RDateSeq(Prop<RDateSeq, Params>),
    ExDateSeq(ExDateSeq, Params),
    // FreeBusy periods
    FreeBusyPeriods(Prop<Vec<Period>, Params>),
    // Trigger (VALARM)
    Trigger(Prop<TriggerValue, Params>),
    // Alarm action
    AlarmAction(Prop<Token<crate::model::primitive::AlarmAction, String>, Params>),
    // Request status
    RequestStatus(Prop<RequestStatus, Params>),
    // RFC 7986
    Color(Prop<Css3Color, Params>),
    // RFC 9073
    StyledDescription(Prop<StyledDescriptionValue, Params>),
    StructuredData(StructuredDataProp),
    ParticipantType(Prop<Token<ParticipantType, String>, Params>),
    ResourceType(Prop<Token<ResourceType, String>, Params>),
    // RFC 9074
    ProximityValue(Prop<Token<ProximityValue, String>, Params>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnknownProp<S> {
    pub name: S,
    pub kind: NameKind,
    pub params: Params,
    pub value: Value<String>,
}

/// Parses a property value based on the VALUE type parameter, producing a [`Value<String>`].
fn parse_value<I, E>(value_type: Token<ValueType, String>, input: &mut I) -> Result<Value<String>, E>
where
    I: InputStream,
    I::Token: AsChar + Clone,
    I::Slice: AsBStr + Clone + SliceLen + Stream,
    <<I as Stream>::Slice as Stream>::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    match value_type {
        Token::Known(ValueType::Binary) => binary.map(Value::Binary).parse_next(input),
        Token::Known(ValueType::Boolean) => bool_caseless.map(Value::Boolean).parse_next(input),
        Token::Known(ValueType::CalAddress) => {
            uri::<_, _, false>.map(Value::CalAddress).parse_next(input)
        }
        Token::Known(ValueType::Date) => {
            primitive::date.map(Value::Date).parse_next(input)
        }
        Token::Known(ValueType::DateTime) => datetime.map(Value::DateTime).parse_next(input),
        Token::Known(ValueType::Duration) => duration.map(Value::Duration).parse_next(input),
        Token::Known(ValueType::Float) => {
            primitive::float.map(Value::Float).parse_next(input)
        }
        Token::Known(ValueType::Integer) => integer.map(Value::Integer).parse_next(input),
        Token::Known(ValueType::Period) => period.map(Value::Period).parse_next(input),
        Token::Known(ValueType::Recur) => rrule.map(Value::Recur).parse_next(input),
        Token::Known(ValueType::Text) => {
            text.map(|t| Value::Text(t.into_string())).parse_next(input)
        }
        Token::Known(ValueType::Time) => {
            time.map(|(t, f)| Value::Time(t, f)).parse_next(input)
        }
        Token::Known(ValueType::Uri) => {
            uri::<_, _, false>.map(Value::Uri).parse_next(input)
        }
        Token::Known(ValueType::UtcOffset) => {
            utc_offset.map(Value::UtcOffset).parse_next(input)
        }
        Token::Known(_) => {
            // Unknown known value type variant (non_exhaustive)
            text.map(|t| Value::Text(t.into_string())).parse_next(input)
        }
        Token::Unknown(name) => text
            .map(|value| Value::Other {
                name: name.clone(),
                value: value.into_string(),
            })
            .parse_next(input),
    }
}

/// Helper: convert `Token<K, Box<Name>>` to `Token<K, String>`.
fn token_to_string<K>(t: Token<K, Box<crate::model::string::Name>>) -> Token<K, String> {
    t.map_unknown(|n| n.as_str().to_string())
}

/// Helper: convert a calico `Box<Uri>` into a `Box<calendar_types::string::Uri>` for use with
/// rfc5545-types value enums (`Attachment`, `StyledDescriptionValue`).
fn into_ct_uri(uri: Box<Uri>) -> Box<calendar_types::string::Uri> {
    let s = uri.as_str().to_string().into_boxed_str();
    // SAFETY: both types are #[repr(transparent)] newtypes around str with trivial invariants.
    // This transmute avoids the overhead of re-validating the string.
    unsafe { Box::from_raw(Box::into_raw(s) as *mut calendar_types::string::Uri) }
}

/// Parses a property with the [`DefaultConfig`].
pub fn property<I, E>(input: &mut I) -> Result<ParsedProp<I::Slice>, E>
where
    I: InputStream,
    I::Token: AsChar + Clone,
    I::Slice: AsBStr + Clone + PartialEq + SliceLen + Stream,
    <<I as Stream>::Slice as Stream>::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    let mut config = DefaultConfig::default();
    property_with_config(input, &mut config)
}

/// Constructs a property parser from the given [`Config`].
pub fn property_parser_from_config<I, E>(
    config: &mut impl Config,
) -> impl Parser<I, ParsedProp<I::Slice>, E> + '_
where
    I: InputStream,
    I::Token: AsChar + Clone,
    I::Slice: AsBStr + Clone + PartialEq + SliceLen + Stream,
    <<I as Stream>::Slice as Stream>::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    move |input: &mut I| property_with_config(input, config)
}

/// Implements property parsing with a given config.
fn property_with_config<I, E>(
    input: &mut I,
    config: &mut impl Config,
) -> Result<ParsedProp<I::Slice>, E>
where
    I: InputStream,
    I::Token: AsChar + Clone,
    I::Slice: AsBStr + Clone + PartialEq + SliceLen + Stream,
    <<I as Stream>::Slice as Stream>::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    // parse name
    let prop_name = property_name.parse_next(input)?;

    // parse parameters
    let (params, value_type) = {
        let mut table = Params::new();
        let mut value_type: Option<Token<ValueType, String>> = None;

        let parsed_params: Vec<Param> =
            winnow::combinator::repeat(0.., preceded(';', parameter)).parse_next(input)?;
        for param in parsed_params {
            match param {
                Param::Known(param) => {
                    let key = param.name();
                    match param.upcast() {
                        UpcastParamValue::ValueType(vt) => match value_type {
                            Some(_) => {
                                return Err(E::from_external_error(
                                    input,
                                    CalendarParseError::DuplicateParam(StaticParam::Value),
                                ));
                            }
                            None => {
                                value_type = Some(token_to_string(vt));
                            }
                        },
                        UpcastParamValue::Known(known_param) => {
                            if table.contains_known(key) {
                                return Err(E::from_external_error(
                                    input,
                                    CalendarParseError::DuplicateParam(key),
                                ));
                            }
                            table.insert_known(known_param);
                        }
                    }
                }
                Param::Unknown(UnknownParam { name: uname, value }) => {
                    let caseless_name =
                        CaselessStr::from_box_str(uname.as_str().into());
                    if let Some(existing) = table.unknown_param_mut(&caseless_name) {
                        // Merge duplicate unknown parameter values
                        for v in value.values {
                            existing.push(v);
                        }
                    } else {
                        table.insert_unknown_param(caseless_name, value.values);
                    }
                }
            }
        }

        (table, value_type)
    };

    // parse the colon separator
    let _: I::Slice = Caseless(":").parse_next(input)?;

    match prop_name {
        PropName::Unknown { name, kind } => {
            let vt = value_type.unwrap_or(Token::Known(ValueType::Text));
            let value = parse_value(vt, input)?;

            Ok(ParsedProp::Unknown(UnknownProp {
                name,
                kind,
                params,
                value,
            }))
        }
        PropName::Known(name) => {
            macro_rules! trivial {
                ($parser:expr) => {{
                    let value = PropValue::from_parsed(
                        name,
                        Prop {
                            value: $parser.parse_next(input)?,
                            params,
                        },
                    );
                    value
                }};
            }

            macro_rules! check_vt {
                ($expected:ident) => {
                    if value_type
                        .as_ref()
                        .is_some_and(|x| !matches!(x, Token::Known(ValueType::$expected)))
                    {
                        return Err(E::from_external_error(
                            input,
                            CalendarParseError::InvalidValueType(value_type.unwrap()),
                        ));
                    }
                };
            }

            macro_rules! require_vt {
                ($expected:ident) => {
                    if let Some(ref vt) = value_type {
                        if !matches!(vt, Token::Known(ValueType::$expected)) {
                            return Err(E::from_external_error(
                                input,
                                CalendarParseError::InvalidValueType(value_type.unwrap()),
                            ));
                        }
                    } else {
                        return Err(E::from_external_error(
                            input,
                            CalendarParseError::MissingValueType,
                        ));
                    }
                };
            }

            let value = match name {
                StaticProp::Attach => match &value_type {
                    None | Some(Token::Known(ValueType::Uri)) => {
                        PropValue::Attachment(Prop {
                            value: Attachment::Uri(into_ct_uri(uri::<_, _, false>.parse_next(input)?)),
                            params,
                        })
                    }
                    Some(Token::Known(ValueType::Binary)) => {
                        match params.inline_encoding() {
                            None => {
                                return Err(E::from_external_error(
                                    input,
                                    CalendarParseError::MissingEncodingOnBinaryValue,
                                ));
                            }
                            Some(Encoding::Bit8) => {
                                return Err(E::from_external_error(
                                    input,
                                    CalendarParseError::Bit8EncodingOnBinaryValue,
                                ));
                            }
                            Some(Encoding::Base64) => {
                                PropValue::Attachment(Prop {
                                    value: Attachment::Binary(
                                        binary_with_config(input, config)?,
                                    ),
                                    params,
                                })
                            }
                            _ => {
                                return Err(E::from_external_error(
                                    input,
                                    CalendarParseError::MissingEncodingOnBinaryValue,
                                ));
                            }
                        }
                    }
                    Some(_) => {
                        return Err(E::from_external_error(
                            input,
                            CalendarParseError::InvalidValueType(value_type.unwrap()),
                        ));
                    }
                },
                StaticProp::ExDate => match &value_type {
                    None | Some(Token::Known(ValueType::DateTime)) => {
                        let dates: Vec<DateTime<_>> =
                            separated(1.., datetime, ',').parse_next(input)?;
                        PropValue::ExDateSeq(ExDateSeq::DateTime(dates), params)
                    }
                    Some(Token::Known(ValueType::Date)) => {
                        let dates: Vec<_> =
                            separated(1.., primitive::date, ',').parse_next(input)?;
                        PropValue::ExDateSeq(ExDateSeq::Date(dates), params)
                    }
                    Some(_) => {
                        return Err(E::from_external_error(
                            input,
                            CalendarParseError::InvalidValueType(value_type.unwrap()),
                        ));
                    }
                },
                StaticProp::RDate => match &value_type {
                    None | Some(Token::Known(ValueType::DateTime)) => {
                        let dates: Vec<DateTime<_>> =
                            separated(1.., datetime, ',').parse_next(input)?;
                        PropValue::RDateSeq(Prop {
                            value: RDateSeq::DateTime(dates),
                            params,
                        })
                    }
                    Some(Token::Known(ValueType::Date)) => {
                        let dates: Vec<_> =
                            separated(1.., primitive::date, ',').parse_next(input)?;
                        PropValue::RDateSeq(Prop {
                            value: RDateSeq::Date(dates),
                            params,
                        })
                    }
                    Some(Token::Known(ValueType::Period)) => {
                        let periods: Vec<Period> =
                            separated(1.., period, ',').parse_next(input)?;
                        PropValue::RDateSeq(Prop {
                            value: RDateSeq::Period(periods),
                            params,
                        })
                    }
                    Some(_) => {
                        return Err(E::from_external_error(
                            input,
                            CalendarParseError::InvalidValueType(value_type.unwrap()),
                        ));
                    }
                },
                StaticProp::Trigger => match &value_type {
                    None | Some(Token::Known(ValueType::Duration)) => {
                        PropValue::Trigger(Prop {
                            value: TriggerValue::Duration(duration.parse_next(input)?),
                            params,
                        })
                    }
                    Some(Token::Known(ValueType::DateTime)) => {
                        PropValue::Trigger(Prop {
                            value: TriggerValue::DateTime(datetime_utc.parse_next(input)?),
                            params,
                        })
                    }
                    Some(_) => {
                        return Err(E::from_external_error(
                            input,
                            CalendarParseError::InvalidValueType(value_type.unwrap()),
                        ));
                    }
                },
                StaticProp::Image => match &value_type {
                    Some(Token::Known(ValueType::Uri)) => {
                        PropValue::Attachment(Prop {
                            value: Attachment::Uri(into_ct_uri(uri::<_, _, false>.parse_next(input)?)),
                            params,
                        })
                    }
                    Some(Token::Known(ValueType::Binary)) => {
                        match params.inline_encoding() {
                            None => {
                                return Err(E::from_external_error(
                                    input,
                                    CalendarParseError::MissingEncodingOnBinaryValue,
                                ));
                            }
                            Some(Encoding::Bit8) => {
                                return Err(E::from_external_error(
                                    input,
                                    CalendarParseError::Bit8EncodingOnBinaryValue,
                                ));
                            }
                            Some(Encoding::Base64) => {
                                PropValue::Attachment(Prop {
                                    value: Attachment::Binary(
                                        binary_with_config(input, config)?,
                                    ),
                                    params,
                                })
                            }
                            _ => {
                                return Err(E::from_external_error(
                                    input,
                                    CalendarParseError::MissingEncodingOnBinaryValue,
                                ));
                            }
                        }
                    }
                    Some(_) => {
                        return Err(E::from_external_error(
                            input,
                            CalendarParseError::InvalidValueType(value_type.unwrap()),
                        ));
                    }
                    None => {
                        return Err(E::from_external_error(
                            input,
                            CalendarParseError::MissingValueType,
                        ));
                    }
                },
                StaticProp::StyledDescription => {
                    match &value_type {
                        Some(Token::Known(ValueType::Text)) => {
                            let t = text.parse_next(input)?;
                            PropValue::StyledDescription(Prop {
                                value: StyledDescriptionValue::Text(t.into_string()),
                                params,
                            })
                        }
                        Some(Token::Known(ValueType::Uri)) => {
                            let u = uri::<_, _, false>.parse_next(input)?;
                            PropValue::StyledDescription(Prop {
                                value: StyledDescriptionValue::Uri(into_ct_uri(u)),
                                params,
                            })
                        }
                        Some(_) => {
                            return Err(E::from_external_error(
                                input,
                                CalendarParseError::InvalidValueType(value_type.unwrap()),
                            ));
                        }
                        None => {
                            return Err(E::from_external_error(
                                input,
                                CalendarParseError::MissingValueType,
                            ));
                        }
                    }
                }
                StaticProp::StructuredData => {
                    match &value_type {
                        Some(Token::Known(ValueType::Binary)) => {
                            let sd_params =
                                crate::model::parameter::StructuredDataParams::try_from(params)
                                    .map_err(|err| E::from_external_error(input, err.into()))?;
                            PropValue::StructuredData(StructuredDataProp::Binary(Prop {
                                value: binary_with_config(input, config)?,
                                params: sd_params,
                            }))
                        }
                        Some(Token::Known(ValueType::Text)) => {
                            let sd_params =
                                crate::model::parameter::StructuredDataParams::try_from(params)
                                    .map_err(|err| E::from_external_error(input, err.into()))?;
                            PropValue::StructuredData(StructuredDataProp::Text(Prop {
                                value: text.parse_next(input)?.into_string(),
                                params: sd_params,
                            }))
                        }
                        Some(Token::Known(ValueType::Uri)) => {
                            PropValue::StructuredData(StructuredDataProp::Uri(Prop {
                                value: uri::<_, _, false>.parse_next(input)?,
                                params,
                            }))
                        }
                        Some(_) => {
                            return Err(E::from_external_error(
                                input,
                                CalendarParseError::InvalidValueType(value_type.unwrap()),
                            ));
                        }
                        None => {
                            return Err(E::from_external_error(
                                input,
                                CalendarParseError::MissingValueType,
                            ));
                        }
                    }
                }
                // Simple text properties
                StaticProp::CalScale => {
                    check_vt!(Text);
                    PropValue::Gregorian(Prop {
                        value: Token::Known(gregorian.parse_next(input)?),
                        params,
                    })
                }
                StaticProp::Method => {
                    check_vt!(Text);
                    PropValue::Method(Prop {
                        value: token_to_string(method.parse_next(input)?),
                        params,
                    })
                }
                StaticProp::ProdId
                | StaticProp::Comment
                | StaticProp::Description
                | StaticProp::Location
                | StaticProp::Summary
                | StaticProp::TzName
                | StaticProp::Contact
                | StaticProp::Name => {
                    check_vt!(Text);
                    PropValue::Text(Prop {
                        value: text.parse_next(input)?.into_string(),
                        params,
                    })
                }
                StaticProp::Version => {
                    check_vt!(Text);
                    PropValue::Version(Prop {
                        value: version.parse_next(input)?,
                        params,
                    })
                }
                StaticProp::Categories | StaticProp::Resources => {
                    check_vt!(Text);
                    let seq = text_seq.parse_next(input)?;
                    PropValue::TextSeq(Prop {
                        value: seq.into_iter().map(|t| t.into_string()).collect(),
                        params,
                    })
                }
                StaticProp::Class => {
                    check_vt!(Text);
                    PropValue::ClassValue(Prop {
                        value: token_to_string(class_value.parse_next(input)?),
                        params,
                    })
                }
                StaticProp::Geo => {
                    check_vt!(Float);
                    PropValue::Geo(Prop {
                        value: geo_with_config(input, config)?,
                        params,
                    })
                }
                StaticProp::PercentComplete => {
                    check_vt!(Integer);
                    trivial!(completion_percentage)
                }
                StaticProp::Priority => {
                    check_vt!(Integer);
                    PropValue::Priority(Prop {
                        value: priority.parse_next(input)?,
                        params,
                    })
                }
                StaticProp::Status => {
                    check_vt!(Text);
                    PropValue::Status(Prop {
                        value: status.parse_next(input)?,
                        params,
                    })
                }
                StaticProp::DtCompleted
                | StaticProp::Created
                | StaticProp::DtStamp
                | StaticProp::LastModified => {
                    check_vt!(DateTime);
                    PropValue::DateTimeUtc(Prop {
                        value: datetime_utc.parse_next(input)?,
                        params,
                    })
                }
                StaticProp::DtEnd
                | StaticProp::DtDue
                | StaticProp::DtStart
                | StaticProp::RecurId => {
                    match &value_type {
                        Some(Token::Known(ValueType::DateTime)) => {
                            PropValue::DateTimeOrDate(Prop {
                                value: DateTimeOrDate::DateTime(datetime.parse_next(input)?),
                                params,
                            })
                        }
                        Some(Token::Known(ValueType::Date)) => {
                            PropValue::DateTimeOrDate(Prop {
                                value: DateTimeOrDate::Date(
                                    primitive::date.parse_next(input)?,
                                ),
                                params,
                            })
                        }
                        None => {
                            // No VALUE parameter: try datetime first, fall back to date.
                            // Real-world calendars often omit VALUE=DATE even for date-only values.
                            PropValue::DateTimeOrDate(Prop {
                                value: primitive::datetime_or_date.parse_next(input)?,
                                params,
                            })
                        }
                        Some(_) => {
                            return Err(E::from_external_error(
                                input,
                                CalendarParseError::InvalidValueType(value_type.unwrap()),
                            ));
                        }
                    }
                }
                StaticProp::Duration => {
                    check_vt!(Duration);
                    PropValue::Duration(Prop {
                        value: duration.parse_next(input)?,
                        params,
                    })
                }
                StaticProp::FreeBusy => {
                    check_vt!(Period);
                    let periods: Vec<Period> =
                        separated(1.., period, ',').parse_next(input)?;
                    PropValue::FreeBusyPeriods(Prop {
                        value: periods,
                        params,
                    })
                }
                StaticProp::Transp => {
                    check_vt!(Text);
                    PropValue::TimeTransparency(Prop {
                        value: time_transparency.parse_next(input)?,
                        params,
                    })
                }
                StaticProp::TzId => {
                    check_vt!(Text);
                    PropValue::TzId(Prop {
                        value: tz_id.parse_next(input)?,
                        params,
                    })
                }
                StaticProp::TzOffsetFrom | StaticProp::TzOffsetTo => {
                    check_vt!(UtcOffset);
                    PropValue::UtcOffset(Prop {
                        value: utc_offset.parse_next(input)?,
                        params,
                    })
                }
                StaticProp::TzUrl | StaticProp::Url => {
                    check_vt!(Uri);
                    PropValue::Uri(Prop {
                        value: uri::<_, _, false>.parse_next(input)?,
                        params,
                    })
                }
                StaticProp::Attendee | StaticProp::Organizer | StaticProp::CalendarAddress => {
                    check_vt!(CalAddress);
                    PropValue::Uri(Prop {
                        value: uri::<_, _, false>.parse_next(input)?,
                        params,
                    })
                }
                StaticProp::RelatedTo | StaticProp::Uid => {
                    check_vt!(Text);
                    PropValue::Uid(Prop {
                        value: uid.parse_next(input)?,
                        params,
                    })
                }
                StaticProp::RRule => {
                    check_vt!(Recur);
                    PropValue::RRule(Prop {
                        value: rrule.parse_next(input)?,
                        params,
                    })
                }
                StaticProp::ExRule => {
                    check_vt!(Recur);
                    PropValue::ExRule(Prop {
                        value: rrule.parse_next(input)?,
                        params,
                    })
                }
                StaticProp::Action => {
                    check_vt!(Text);
                    PropValue::AlarmAction(Prop {
                        value: token_to_string(alarm_action.parse_next(input)?),
                        params,
                    })
                }
                StaticProp::Repeat | StaticProp::Sequence => {
                    check_vt!(Integer);
                    PropValue::Integer(Prop {
                        value: integer.parse_next(input)?,
                        params,
                    })
                }
                StaticProp::RequestStatus => {
                    check_vt!(Text);
                    PropValue::RequestStatus(Prop {
                        value: request_status.parse_next(input)?,
                        params,
                    })
                }
                StaticProp::RefreshInterval => {
                    require_vt!(Duration);
                    PropValue::Duration(Prop {
                        value: duration.parse_next(input)?,
                        params,
                    })
                }
                StaticProp::Source | StaticProp::Conference => {
                    require_vt!(Uri);
                    PropValue::Uri(Prop {
                        value: uri::<_, _, false>.parse_next(input)?,
                        params,
                    })
                }
                StaticProp::Color => {
                    check_vt!(Text);
                    PropValue::Color(Prop {
                        value: color.parse_next(input)?,
                        params,
                    })
                }
                StaticProp::LocationType => {
                    check_vt!(Text);
                    let seq = text_seq.parse_next(input)?;
                    PropValue::TextSeq(Prop {
                        value: seq.into_iter().map(|t| t.into_string()).collect(),
                        params,
                    })
                }
                StaticProp::ParticipantType => {
                    check_vt!(Text);
                    PropValue::ParticipantType(Prop {
                        value: token_to_string(participant_type.parse_next(input)?),
                        params,
                    })
                }
                StaticProp::ResourceType => {
                    check_vt!(Text);
                    PropValue::ResourceType(Prop {
                        value: token_to_string(resource_type.parse_next(input)?),
                        params,
                    })
                }
                StaticProp::Acknowledged => {
                    check_vt!(DateTime);
                    PropValue::DateTimeUtc(Prop {
                        value: datetime_utc.parse_next(input)?,
                        params,
                    })
                }
                StaticProp::Proximity => {
                    check_vt!(Text);
                    PropValue::ProximityValue(Prop {
                        value: token_to_string(proximity_value.parse_next(input)?),
                        params,
                    })
                }
            };

            Ok(ParsedProp::Known(KnownProp { name, value }))
        }
    }
}

impl PropValue {
    /// Construct a PropValue from a parsed CompletionPercentage prop.
    fn from_parsed(_name: StaticProp, prop: Prop<CompletionPercentage, Params>) -> Self {
        PropValue::CompletionPercentage(prop)
    }
}

// pub fn property_with_config<I, E>(
//     input: &mut I,
//     config: &mut impl Config,
// ) -> Result<ParsedProp<I::Slice>, E>
// where
//     I: StreamIsPartial + Stream + Compare<Caseless<&'static str>> + Compare<char>,
//     I::Token: AsChar + Clone,
//     I::Slice: AsBStr + Clone + PartialEq + SliceLen + Stream + Equiv,
//     <<I as Stream>::Slice as Stream>::Token: AsChar,
//     E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
// {
//     // parse name
//     let name = property_name.parse_next(input)?;
//
//     // parse parameters
//     let (params, value_type) = {
//         let mut table: Params<I::Slice> = Default::default();
//         let mut value_type: Option<ValueType<I::Slice>> = None;
//
//         let params: Vec<Param<_>> = repeat(0.., preceded(';', parameter)).parse_next(input)?;
//         for param in params {
//             match param {
//                 Param::Known(param) => {
//                     let key = param.name();
//
//                     match param.upcast() {
//                         UpcastParamValue::ValueType(vt) => match value_type {
//                             Some(_) => {
//                                 return Err(E::from_external_error(
//                                     input,
//                                     CalendarParseError::DuplicateParam(StaticParam::Value),
//                                 ));
//                             }
//                             None => {
//                                 value_type = Some(vt);
//                             }
//                         },
//                         UpcastParamValue::RawValue(value) => match table.contains_known(key) {
//                             true => {
//                                 return Err(E::from_external_error(
//                                     input,
//                                     CalendarParseError::DuplicateParam(key),
//                                 ));
//                             }
//                             false => {
//                                 let _prev = table.insert_known(key, value);
//                                 debug_assert!(_prev.is_none());
//                             }
//                         },
//                     }
//                 }
//                 Param::Unknown(UnknownParam { name, value }) => {
//                     if table.contains_unknown(&name) {
//                         let previous_value = &mut table.get_unknown_mut(&name).unwrap().values;
//                         let new_value = value.values;
//
//                         config
//                             .handle_duplicate_param(previous_value, new_value)
//                             .map_err(|err| E::from_external_error(input, err.into()))?;
//                     } else {
//                         let _prev = table.insert_unknown(name, value);
//                         debug_assert!(_prev.is_none());
//                     }
//                 }
//             }
//         }
//
//         (table, value_type)
//     };
//
//     // parse the colon separator (we use Caseless to avoid introducing a new bound)
//     let _ = Caseless(":").parse_next(input)?;
//
//     match name {
//         PropName::Unknown { name, kind } => {
//             let value = parse_value(value_type.unwrap_or(ValueType::Text), input)?;
//
//             Ok(ParsedProp::Unknown(UnknownProp {
//                 name,
//                 kind,
//                 params,
//                 value,
//             }))
//         }
//         PropName::Known(name) => {
//             // in the grammar for a property, we're here:
//             //
//             //     name *(";" param) ":" value
//             //                          ↑
//             // so in order, we need to:
//             // 1. parse the value (depending on the name and value type)
//             // 2. check and possibly transform the parameters
//             // 3. construct a KnownProp by upcasting to a RawPropValue
//
//             macro_rules! trivial {
//                 ($parser:expr) => {{
//                     Prop {
//                         value: $parser.parse_next(input)?,
//                         params,
//                     }
//                     .into()
//                 }};
//                 ($parser:expr, $value_type:ident) => {{
//                     if value_type
//                         .as_ref()
//                         .is_some_and(|x| x != &ValueType::$value_type)
//                     {
//                         return Err(E::from_external_error(
//                             input,
//                             CalendarParseError::InvalidValueType(value_type.unwrap()),
//                         ));
//                     }
//                     Prop {
//                         value: $parser.parse_next(input)?,
//                         params,
//                     }
//                     .into()
//                 }};
//                 ($parser:expr, !$value_type:ident) => {{
//                     if let Some(value_type) = value_type {
//                         if !matches!(value_type, ValueType::$value_type) {
//                             return Err(E::from_external_error(
//                                 input,
//                                 CalendarParseError::InvalidValueType(value_type),
//                             ));
//                         }
//                     } else {
//                         return Err(E::from_external_error(
//                             input,
//                             CalendarParseError::MissingValueType,
//                         ));
//                     }
//
//                     Prop {
//                         value: $parser.parse_next(input)?,
//                         params,
//                     }
//                     .into()
//                 }};
//             }
//
//             macro_rules! dt_or_date {
//                 () => {{
//                     let value = match value_type {
//                         None | Some(ValueType::DateTime) => {
//                             datetime.map(DateTimeOrDate::DateTime).parse_next(input)?
//                         }
//                         Some(ValueType::Date) => {
//                             date.map(DateTimeOrDate::Date).parse_next(input)?
//                         }
//                         Some(value_type) => {
//                             return Err(E::from_external_error(
//                                 input,
//                                 CalendarParseError::InvalidValueType(value_type),
//                             ));
//                         }
//                     };
//                     Prop { value, params }.into()
//                 }};
//             }
//
//             let value: RawPropValue<_> = match name {
//                 StaticProp::Attach => match value_type {
//                     None | Some(ValueType::Uri) => {
//                         trivial!(uri::<_, _, false>.map(Attachment::Uri))
//                     }
//                     Some(ValueType::Binary) => match params.inline_encoding() {
//                         None => {
//                             return Err(E::from_external_error(
//                                 input,
//                                 CalendarParseError::MissingEncodingOnBinaryValue,
//                             ));
//                         }
//                         Some(Encoding::Bit8) => {
//                             return Err(E::from_external_error(
//                                 input,
//                                 CalendarParseError::Bit8EncodingOnBinaryValue,
//                             ));
//                         }
//                         Some(Encoding::Base64) => trivial!(binary.map(Attachment::Binary)),
//                     },
//                     Some(value_type) => {
//                         return Err(E::from_external_error(
//                             input,
//                             CalendarParseError::InvalidValueType(value_type),
//                         ));
//                     }
//                 },
//                 StaticProp::ExDate => match value_type {
//                     None | Some(ValueType::DateTime) => {
//                         trivial!(separated(1.., datetime, ',').map(ExDateSeq::DateTime))
//                     }
//                     Some(ValueType::Date) => {
//                         trivial!(separated(1.., date, ',').map(ExDateSeq::Date))
//                     }
//                     Some(value_type) => {
//                         return Err(E::from_external_error(
//                             input,
//                             CalendarParseError::InvalidValueType(value_type),
//                         ));
//                     }
//                 },
//                 StaticProp::RDate => match value_type {
//                     None | Some(ValueType::DateTime) => {
//                         trivial!(separated(1.., datetime, ',').map(RDateSeq::DateTime))
//                     }
//                     Some(ValueType::Date) => {
//                         trivial!(separated(1.., date, ',').map(RDateSeq::Date))
//                     }
//                     Some(ValueType::Period) => {
//                         trivial!(separated(1.., period, ',').map(RDateSeq::Period))
//                     }
//                     Some(value_type) => {
//                         return Err(E::from_external_error(
//                             input,
//                             CalendarParseError::InvalidValueType(value_type),
//                         ));
//                     }
//                 },
//                 StaticProp::Trigger => match value_type {
//                     None | Some(ValueType::Duration) => TriggerProp::Relative(Prop {
//                         value: duration.parse_next(input)?,
//                         params,
//                     })
//                     .into(),
//                     Some(ValueType::DateTime) => TriggerProp::Absolute(Prop {
//                         value: datetime_utc.parse_next(input)?,
//                         params,
//                     })
//                     .into(),
//                     Some(value_type) => {
//                         return Err(E::from_external_error(
//                             input,
//                             CalendarParseError::InvalidValueType(value_type),
//                         ));
//                     }
//                 },
//                 StaticProp::Image => match value_type {
//                     Some(ValueType::Uri) => {
//                         trivial!(uri::<_, _, false>.map(Attachment::Uri))
//                     }
//                     Some(ValueType::Binary) => match params.inline_encoding() {
//                         None => {
//                             return Err(E::from_external_error(
//                                 input,
//                                 CalendarParseError::MissingEncodingOnBinaryValue,
//                             ));
//                         }
//                         Some(Encoding::Bit8) => {
//                             return Err(E::from_external_error(
//                                 input,
//                                 CalendarParseError::Bit8EncodingOnBinaryValue,
//                             ));
//                         }
//                         Some(Encoding::Base64) => trivial!(binary.map(Attachment::Binary)),
//                     },
//                     Some(value_type) => {
//                         return Err(E::from_external_error(
//                             input,
//                             CalendarParseError::InvalidValueType(value_type),
//                         ));
//                     }
//                     None => {
//                         return Err(E::from_external_error(
//                             input,
//                             CalendarParseError::MissingValueType,
//                         ));
//                     }
//                 },
//                 StaticProp::StyledDescription => match value_type {
//                     Some(ValueType::Text) => trivial!(text),
//                     Some(ValueType::Uri) => trivial!(uri::<_, _, false>),
//                     Some(value_type) => {
//                         return Err(E::from_external_error(
//                             input,
//                             CalendarParseError::InvalidValueType(value_type),
//                         ));
//                     }
//                     None => {
//                         return Err(E::from_external_error(
//                             input,
//                             CalendarParseError::MissingValueType,
//                         ));
//                     }
//                 },
//                 StaticProp::StructuredData => match value_type {
//                     Some(ValueType::Binary) => StructuredDataProp::Binary(Prop {
//                         value: binary.parse_next(input)?,
//                         params: StructuredDataParams::try_from(params)
//                             .map_err(|err| E::from_external_error(input, err.into()))?,
//                     })
//                     .into(),
//                     Some(ValueType::Text) => StructuredDataProp::Text(Prop {
//                         value: text.parse_next(input)?,
//                         params: StructuredDataParams::try_from(params)
//                             .map_err(|err| E::from_external_error(input, err.into()))?,
//                     })
//                     .into(),
//                     Some(ValueType::Uri) => StructuredDataProp::Uri(Prop {
//                         value: (uri::<_, _, false>).parse_next(input)?,
//                         params,
//                     })
//                     .into(),
//                     Some(value_type) => {
//                         return Err(E::from_external_error(
//                             input,
//                             CalendarParseError::InvalidValueType(value_type),
//                         ));
//                     }
//                     None => {
//                         return Err(E::from_external_error(
//                             input,
//                             CalendarParseError::MissingValueType,
//                         ));
//                     }
//                 },
//                 StaticProp::CalScale => trivial!(gregorian, Text),
//                 StaticProp::Method => trivial!(method, Text),
//                 StaticProp::ProdId
//                 | StaticProp::Comment
//                 | StaticProp::Description
//                 | StaticProp::Location
//                 | StaticProp::Summary
//                 | StaticProp::TzName
//                 | StaticProp::Contact
//                 | StaticProp::Name => trivial!(text, Text),
//                 StaticProp::Version => trivial!(version, Text),
//                 StaticProp::Categories | StaticProp::Resources => trivial!(text_seq, Text),
//                 StaticProp::Class => trivial!(class_value, Text),
//                 StaticProp::Geo => trivial!(geo, Float),
//                 StaticProp::PercentComplete => trivial!(completion_percentage, Integer),
//                 StaticProp::Priority => trivial!(priority, Integer),
//                 StaticProp::Status => trivial!(status, Text),
//                 StaticProp::DtCompleted
//                 | StaticProp::Created
//                 | StaticProp::DtStamp
//                 | StaticProp::LastModified => trivial!(datetime_utc, DateTime),
//                 StaticProp::DtEnd
//                 | StaticProp::DtDue
//                 | StaticProp::DtStart
//                 | StaticProp::RecurId => dt_or_date!(),
//                 StaticProp::Duration => trivial!(duration, Duration),
//                 StaticProp::FreeBusy => {
//                     trivial!(separated(1.., period, ',').map(|v: Vec<_>| v), Period)
//                 }
//                 StaticProp::Transp => trivial!(time_transparency, Text),
//                 StaticProp::TzId => trivial!(tz_id, Text),
//                 StaticProp::TzOffsetFrom | StaticProp::TzOffsetTo => {
//                     trivial!(utc_offset, UtcOffset)
//                 }
//                 StaticProp::TzUrl | StaticProp::Url => trivial!(uri::<_, _, false>, Uri),
//                 StaticProp::Attendee | StaticProp::Organizer | StaticProp::CalendarAddress => {
//                     trivial!(cal_address::<_, _, false>, CalAddress)
//                 }
//                 StaticProp::RelatedTo | StaticProp::Uid => trivial!(uid, Text),
//                 StaticProp::RRule => trivial!(rrule.map(Box::new), Recur),
//                 StaticProp::Action => trivial!(alarm_action, Text),
//                 StaticProp::Repeat | StaticProp::Sequence => trivial!(integer, Integer),
//                 StaticProp::RequestStatus => trivial!(request_status, Text),
//                 StaticProp::RefreshInterval => trivial!(duration, !Duration),
//                 StaticProp::Source | StaticProp::Conference => trivial!(uri::<_, _, false>, !Uri),
//                 StaticProp::Color => trivial!(color, Text),
//                 StaticProp::LocationType => trivial!(text_seq, Text),
//                 StaticProp::ParticipantType => trivial!(participant_type, Text),
//                 StaticProp::ResourceType => trivial!(resource_type, Text),
//                 StaticProp::Acknowledged => trivial!(datetime_utc, DateTime),
//                 StaticProp::Proximity => trivial!(proximity_value, Text),
//             };
//
//             Ok(ParsedProp::Known(KnownProp { name, value }))
//         }
//     }
// }

/// A property name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropName<S> {
    Known(StaticProp),
    Unknown { name: S, kind: NameKind },
}

impl<S> PropName<S> {
    #[inline(always)]
    pub const fn iana(name: S) -> Self {
        Self::Unknown {
            name,
            kind: NameKind::Iana,
        }
    }

    #[inline(always)]
    pub const fn x(name: S) -> Self {
        Self::Unknown {
            name,
            kind: NameKind::X,
        }
    }
}

/// Parses a [`PropName`].
pub fn property_name<I, E>(input: &mut I) -> Result<PropName<I::Slice>, E>
where
    I: StreamIsPartial + Stream + Compare<Caseless<&'static str>> + Compare<char>,
    I::Token: AsChar + Clone,
    E: ParserError<I>,
    PropName<I::Slice>: Clone,
{
    // NOTE: we have special handling here for BEGIN and END because they're pseudoproperties

    enum InvalidNameKind {
        Begin,
        End,
        Unknown,
    }

    impl From<()> for InvalidNameKind {
        fn from((): ()) -> Self {
            Self::Unknown
        }
    }

    fn static_name<I>(input: &mut I) -> Result<StaticProp, InvalidNameKind>
    where
        I: StreamIsPartial + Stream + Compare<Caseless<&'static str>> + Compare<char>,
        I::Token: AsChar + Clone,
    {
        macro_rules! tail {
            ($s:literal, $c:expr) => {{
                let res: Result<_, ()> = Caseless($s).value($c).parse_next(input);
                res.map_err(Into::into)
            }};
        }

        match ascii_lower::<_, ()>.parse_next(input)? {
            'a' => match ascii_lower::<_, ()>.parse_next(input)? {
                //'c' => tail!("tion", Rfc5545(PN5545::Action)),
                'c' => match ascii_lower::<_, ()>.parse_next(input)? {
                    't' => tail!("ion", StaticProp::Action),
                    'k' => tail!("nowledged", StaticProp::Acknowledged),
                    _ => Err(InvalidNameKind::Unknown),
                },
                't' => match preceded(Caseless("t"), ascii_lower::<_, ()>).parse_next(input)? {
                    'a' => tail!("ch", StaticProp::Attach),
                    'e' => tail!("ndee", StaticProp::Attendee),
                    _ => Err(InvalidNameKind::Unknown),
                },
                _ => Err(InvalidNameKind::Unknown),
            },
            'b' => match Caseless("egin").parse_next(input) {
                Ok(_) => Err(InvalidNameKind::Begin),
                Err(()) => Err(InvalidNameKind::Unknown),
            },
            'c' => match ascii_lower::<_, ()>.parse_next(input)? {
                'a' => match ascii_lower::<_, ()>.parse_next(input)? {
                    'l' => tail!("scale", StaticProp::CalScale),
                    't' => tail!("egories", StaticProp::Categories),
                    _ => Err(InvalidNameKind::Unknown),
                },
                'l' => tail!("ass", StaticProp::Class),
                'o' => match ascii_lower::<_, ()>.parse_next(input)? {
                    'l' => tail!("or", StaticProp::Color),
                    'm' => match ascii_lower::<_, ()>.parse_next(input)? {
                        'm' => tail!("ent", StaticProp::Comment),
                        'p' => tail!("leted", StaticProp::DtCompleted),
                        _ => Err(InvalidNameKind::Unknown),
                    },
                    'n' => match ascii_lower::<_, ()>.parse_next(input)? {
                        'f' => tail!("erence", StaticProp::Conference),
                        't' => tail!("act", StaticProp::Contact),
                        _ => Err(InvalidNameKind::Unknown),
                    },
                    _ => Err(InvalidNameKind::Unknown),
                },
                'r' => tail!("eated", StaticProp::Created),
                _ => Err(InvalidNameKind::Unknown),
            },
            'd' => match ascii_lower::<_, ()>.parse_next(input)? {
                'e' => tail!("scription", StaticProp::Description),
                't' => match ascii_lower::<_, ()>.parse_next(input)? {
                    'e' => tail!("nd", StaticProp::DtEnd),
                    's' => {
                        match preceded(Caseless("ta"), ascii_lower::<_, ()>).parse_next(input)? {
                            'm' => tail!("p", StaticProp::DtStamp),
                            'r' => tail!("t", StaticProp::DtStart),
                            _ => Err(InvalidNameKind::Unknown),
                        }
                    }
                    _ => Err(InvalidNameKind::Unknown),
                },
                'u' => match ascii_lower::<_, ()>.parse_next(input)? {
                    'e' => tail!("", StaticProp::DtDue),
                    'r' => tail!("ation", StaticProp::Duration),
                    _ => Err(InvalidNameKind::Unknown),
                },
                _ => Err(InvalidNameKind::Unknown),
            },
            'e' => match ascii_lower::<_, ()>.parse_next(input)? {
                'n' => match Caseless("d").parse_next(input) {
                    Ok(_) => Err(InvalidNameKind::End),
                    Err(()) => Err(InvalidNameKind::Unknown),
                },
                'x' => match ascii_lower::<_, ()>.parse_next(input)? {
                    'd' => tail!("ate", StaticProp::ExDate),
                    'r' => tail!("ule", StaticProp::ExRule),
                    _ => Err(InvalidNameKind::Unknown),
                },
                _ => Err(InvalidNameKind::Unknown),
            },
            'f' => tail!("reebusy", StaticProp::FreeBusy),
            'g' => tail!("eo", StaticProp::Geo),
            'i' => tail!("mage", StaticProp::Image),
            'l' => match ascii_lower::<_, ()>.parse_next(input)? {
                'a' => tail!("st-modified", StaticProp::LastModified),
                'o' => tail!("cation", StaticProp::Location),
                _ => Err(InvalidNameKind::Unknown),
            },
            'm' => tail!("ethod", StaticProp::Method),
            'n' => tail!("ame", StaticProp::Name),
            'o' => tail!("rganizer", StaticProp::Organizer),
            'p' => match ascii_lower::<_, ()>.parse_next(input)? {
                'e' => tail!("rcent-complete", StaticProp::PercentComplete),
                // PRIORITY | PRODID | PROXIMITY
                'r' => match ascii_lower::<_, ()>.parse_next(input)? {
                    'i' => tail!("ority", StaticProp::Priority),
                    'o' => match ascii_lower::<_, ()>.parse_next(input)? {
                        'd' => tail!("id", StaticProp::ProdId),
                        'x' => tail!("imity", StaticProp::Proximity),
                        _ => Err(InvalidNameKind::Unknown),
                    },
                    _ => Err(InvalidNameKind::Unknown),
                },
                _ => Err(InvalidNameKind::Unknown),
            },
            'r' => match ascii_lower::<_, ()>.parse_next(input)? {
                'e' => match ascii_lower::<_, ()>.parse_next(input)? {
                    'c' => tail!("urrence-id", StaticProp::RecurId),
                    'f' => {
                        tail!("resh-interval", StaticProp::RefreshInterval)
                    }
                    'l' => tail!("ated-to", StaticProp::RelatedTo),
                    'p' => tail!("eat", StaticProp::Repeat),
                    'q' => tail!("uest-status", StaticProp::RequestStatus),
                    's' => tail!("ources", StaticProp::Resources),
                    _ => Err(InvalidNameKind::Unknown),
                },
                'd' => tail!("ate", StaticProp::RDate),
                'r' => tail!("ule", StaticProp::RRule),
                _ => Err(InvalidNameKind::Unknown),
            },
            's' => match ascii_lower::<_, ()>.parse_next(input)? {
                'e' => tail!("quence", StaticProp::Sequence),
                'o' => tail!("urce", StaticProp::Source),
                't' => tail!("atus", StaticProp::Status),
                'u' => tail!("mmary", StaticProp::Summary),
                _ => Err(InvalidNameKind::Unknown),
            },
            't' => match ascii_lower::<_, ()>.parse_next(input)? {
                // TRIGGER | TRANSP
                'r' => match ascii_lower::<_, ()>.parse_next(input)? {
                    'a' => tail!("nsp", StaticProp::Transp),
                    'i' => tail!("gger", StaticProp::Trigger),
                    _ => Err(InvalidNameKind::Unknown),
                },
                // TZOFFSETFROM | TZOFFSETTO | TZNAME | TZURL | TZID
                'z' => match ascii_lower::<_, ()>.parse_next(input)? {
                    'i' => tail!("d", StaticProp::TzId),
                    'n' => tail!("ame", StaticProp::TzName),
                    // TZOFFSETFROM | TZOFFSETTO
                    'o' => {
                        match preceded(Caseless("ffset"), ascii_lower::<_, ()>).parse_next(input)? {
                            'f' => tail!("rom", StaticProp::TzOffsetFrom),
                            't' => tail!("o", StaticProp::TzOffsetTo),
                            _ => Err(InvalidNameKind::Unknown),
                        }
                    }
                    'u' => tail!("rl", StaticProp::TzUrl),
                    _ => Err(InvalidNameKind::Unknown),
                },
                _ => Err(InvalidNameKind::Unknown),
            },
            'u' => match ascii_lower::<_, ()>.parse_next(input)? {
                'i' => tail!("d", StaticProp::Uid),
                'r' => tail!("l", StaticProp::Url),
                _ => Err(InvalidNameKind::Unknown),
            },
            'v' => tail!("ersion", StaticProp::Version),
            _ => Err(InvalidNameKind::Unknown),
        }
    }

    let checkpoint = input.checkpoint();
    match static_name.parse_next(input) {
        Ok(res) => Ok(PropName::Known(res)),
        Err(InvalidNameKind::Begin | InvalidNameKind::End) => fail.parse_next(input),
        Err(InvalidNameKind::Unknown) => {
            input.reset(&checkpoint);

            // Peek at the first two characters to determine X- vs IANA.
            let first = input.next_token().ok_or_else(|| E::from_input(input))?;
            let is_x = first.as_char().eq_ignore_ascii_case(&'x');
            let second = input.peek_token();
            let is_x_prefix = is_x && second.is_some_and(|t| t.as_char() == '-');

            // Reset and parse the full name.
            input.reset(&checkpoint);
            let slice: I::Slice = take_while(1.., |t: I::Token| {
                let c = t.as_char();
                c == '-' || c.is_ascii_alphanumeric()
            })
            .parse_next(input)?;

            let kind = if is_x_prefix {
                NameKind::X
            } else {
                NameKind::Iana
            };

            Ok(PropName::Unknown { name: slice, kind })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rfc5545PropName {
    // CALENDAR PROPERTIES (RFC 5545 §3.7)
    /// RFC 5545 §3.7.1 (CALSCALE)
    CalendarScale,
    /// RFC 5545 §3.7.2 (METHOD)
    Method,
    /// RFC 5545 §3.7.3 (PRODID)
    ProductIdentifier,
    /// RFC 5545 §3.7.4 (VERSION)
    Version,

    // DESCRIPTIVE PROPERTIES (RFC 5545 §3.8.1)
    /// RFC 5545 §3.8.1.1 (ATTACH)
    Attachment,
    /// RFC 5545 §3.8.1.2 (CATEGORIES)
    Categories,
    /// RFC 5545 §3.8.1.3 (CLASS)
    Classification,
    /// RFC 5545 §3.8.1.4 (COMMENT)
    Comment,
    /// RFC 5545 §3.8.1.5 (DESCRIPTION)
    Description,
    /// RFC 5545 §3.8.1.6 (GEO)
    GeographicPosition,
    /// RFC 5545 §3.8.1.7 (LOCATION)
    Location,
    /// RFC 5545 §3.8.1.8 (PERCENT-COMPLETE)
    PercentComplete,
    /// RFC 5545 §3.8.1.9 (PRIORITY)
    Priority,
    /// RFC 5545 §3.8.1.10 (RESOURCES)
    Resources,
    /// RFC 5545 §3.8.1.11 (STATUS)
    Status,
    /// RFC 5545 §3.8.1.12 (SUMMARY)
    Summary,

    // DATE AND TIME PROPERTIES (RFC 5545 §3.8.2)
    /// RFC 5545 §3.8.2.1 (COMPLETED)
    DateTimeCompleted,
    /// RFC 5545 §3.8.2.2 (DTEND)
    DateTimeEnd,
    /// RFC 5545 §3.8.2.3 (DUE)
    DateTimeDue,
    /// RFC 5545 §3.8.2.4 (DTSTART)
    DateTimeStart,
    /// RFC 5545 §3.8.2.5 (DURATION)
    Duration,
    /// RFC 5545 §3.8.2.6 (FREEBUSY)
    FreeBusyTime,
    /// RFC 5545 §3.8.2.7 (TRANSP)
    TimeTransparency,

    // TIME ZONE PROPERTIES (RFC 5545 §3.8.3)
    /// RFC 5545 §3.8.3.1 (TZID)
    TimeZoneIdentifier,
    /// RFC 5545 §3.8.3.2 (TZNAME)
    TimeZoneName,
    /// RFC 5545 §3.8.3.3 (TZOFFSETFROM)
    TimeZoneOffsetFrom,
    /// RFC 5545 §3.8.3.4 (TZOFFSETTO)
    TimeZoneOffsetTo,
    /// RFC 5545 §3.8.3.5 (TZURL)
    TimeZoneUrl,

    // RELATIONSHIP PROPERTIES (RFC 5545 §3.8.4)
    /// RFC 5545 §3.8.4.1 (ATTENDEE)
    Attendee,
    /// RFC 5545 §3.8.4.2 (CONTACT)
    Contact,
    /// RFC 5545 §3.8.4.3 (ORGANIZER)
    Organizer,
    /// RFC 5545 §3.8.4.4 (RECURRENCE-ID)
    RecurrenceId,
    /// RFC 5545 §3.8.4.5 (RELATED-TO)
    RelatedTo,
    /// RFC 5545 §3.8.4.6 (URL)
    UniformResourceLocator,
    /// RFC 5545 §3.8.4.7 (UID)
    UniqueIdentifier,

    // RECURRENCE PROPERTIES (RFC 5545 §3.8.5)
    /// RFC 5545 §3.8.5.1 (EXDATE)
    ExceptionDateTimes,
    /// RFC 5545 §3.8.5.2 (RDATE)
    RecurrenceDateTimes,
    /// RFC 5545 §3.8.5.3 (RRULE)
    RecurrenceRule,

    // ALARM PROPERTIES (RFC 5545 §3.8.6)
    /// RFC 5545 §3.8.6.1 (ACTION)
    Action,
    /// RFC 5545 §3.8.6.2 (REPEAT)
    RepeatCount,
    /// RFC 5545 §3.8.6.3 (TRIGGER)
    Trigger,

    // CHANGE MANAGEMENT PROPERTIES (RFC 5545 §3.8.7)
    /// RFC 5545 §3.8.7.1 (CREATED)
    DateTimeCreated,
    /// RFC 5545 §3.8.7.2 (DTSTAMP)
    DateTimeStamp,
    /// RFC 5545 §3.8.7.3 (LAST-MODIFIED)
    LastModified,
    /// RFC 5545 §3.8.7.4 (SEQUENCE)
    SequenceNumber,

    // MISCELLANEOUS PROPERTIES (RFC 5545 §3.8.8)
    /// RFC 5545 §3.8.8.3 (REQUEST-STATUS)
    RequestStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rfc7986PropName {
    /// RFC 7986 §5.1 (NAME)
    Name,
    /// RFC 7986 §5.7 (REFRESH-INTERVAL)
    RefreshInterval,
    /// RFC 7986 §5.8 (SOURCE)
    Source,
    /// RFC 7986 §5.9 (COLOR)
    Color,
    /// RFC 7986 §5.10 (IMAGE)
    Image,
    /// RFC 7986 §5.11 (CONFERENCE)
    Conference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rfc9073PropName {
    /// RFC 9073 §6.1
    LocationType,
    /// RFC 9073 §6.2
    ParticipantType,
    /// RFC 9073 §6.3
    ResourceType,
    /// RFC 9073 §6.4
    CalendarAddress,
    /// RFC 9073 §6.5
    StyledDescription,
    /// RFC 9073 §6.6
    StructuredData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rfc9074PropName {
    /// RFC 9074 §6 (ACKNOWLEDGED)
    Acknowledged,
    /// RFC 9074 §8.1 (PROXIMITY)
    Proximity,
}

#[cfg(test)]
mod tests {
    use crate::{
        date,
        model::{
            primitive::{
                Attachment, ClassValue, CompletionPercentage, DateTime, DateTimeOrDate,
                Duration, ExactDuration, FormatTypeBuf, FreeBusyType, Language, Method,
                ParticipationRole, ParticipationStatus, Period, Priority,
                RequestStatus, RequestStatusCode, Sign, SignedDuration, Status, ThisAndFuture,
                TimeFormat, TimeTransparency, Token, Utc, Value, Version,
            },
            property::Prop,
            string::{NameKind, TzId, Uid, Uri},
        },
        parser::escaped::AsEscaped,
        time, utc_offset,
    };

    use super::*;
    use winnow::Parser;

    // PROPERTY PARSING TESTS

    /// Helper: construct a known property with a given PropValue variant.
    macro_rules! known_prop {
        ($name:ident, $variant:ident, $value:expr $(,)?) => {
            ParsedProp::Known(KnownProp {
                name: StaticProp::$name,
                value: PropValue::$variant(Prop::from_value($value)),
            })
        };
    }

    /// Helper to construct a StatusCode from (class_u8, major).
    fn status_code(class: u8, major: u8) -> RequestStatusCode {
        use crate::model::primitive::Class as StatusClass;
        let c = match class {
            1 => StatusClass::C1,
            2 => StatusClass::C2,
            3 => StatusClass::C3,
            4 => StatusClass::C4,
            5 => StatusClass::C5,
            _ => panic!("invalid status class: {class}"),
        };
        RequestStatusCode { class: c, major, minor: None }
    }

    /// Helper: construct a DateTime<Utc>
    fn utc_datetime(y: u16, mo: u8, d: u8, h: u8, mi: u8, s: u8) -> DateTime<Utc> {
        DateTime {
            date: date!(y; mo; d),
            time: time!(h; mi; s),
            marker: Utc,
        }
    }

    /// Helper: construct a DateTime<TimeFormat> in UTC
    fn tf_utc_datetime(y: u16, mo: u8, d: u8, h: u8, mi: u8, s: u8) -> DateTime<TimeFormat> {
        DateTime {
            date: date!(y; mo; d),
            time: time!(h; mi; s),
            marker: TimeFormat::Utc,
        }
    }

    /// Helper: construct a positive ExactDuration
    fn exact_dur(hours: u32, minutes: u32, seconds: u32) -> SignedDuration {
        SignedDuration {
            sign: Sign::Pos,
            duration: Duration::Exact(ExactDuration {
                hours,
                minutes,
                seconds,
                frac: None,
            }),
        }
    }

    /// Helper: construct an unsigned ExactDuration (for Period which uses Duration, not SignedDuration)
    fn unsigned_exact_dur(hours: u32, minutes: u32, seconds: u32) -> Duration {
        Duration::Exact(ExactDuration {
            hours,
            minutes,
            seconds,
            frac: None,
        })
    }

    #[test]
    fn apple_calendar_attendee_edge_case() {
        let input = r#"ATTENDEE;CN="John Smith";CUTYPE=INDIVIDUAL;EMAIL="john.smith@icloud.com";PARTSTAT=ACCEPTED;ROLE=CHAIR:/aMTg2ODQAyMzEjg0NX9m3Gyi2XcPHS8HXCT7Y3j1oq6U7hokvhVwdffK5c/principal/"#;
        let (tail, _prop) = property::<_, ()>.parse_peek(input).unwrap();
        assert!(tail.is_empty());
    }

    #[test]
    fn apple_calendar_empty_url_line() {
        let input = "URL;VALUE=URI:";
        let (tail, _prop) = property::<_, ()>.parse_peek(input).unwrap();
        assert!(tail.is_empty());
    }

    // NOTE: rfc_5545_section_4_example_1 test removed — the example .ics file no longer exists
    // in this crate's directory structure.

    #[test]
    fn rfc_5545_example_calendar_scale_property() {
        let input = "CALSCALE:GREGORIAN";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        let known = prop.try_into_known().unwrap();
        assert_eq!(known.name, StaticProp::CalScale);
        let PropValue::Gregorian(p) = known.value else { panic!("expected Gregorian") };
        assert!(matches!(p.value, Token::Known(crate::model::primitive::Gregorian)));
    }

    #[test]
    fn rfc_5545_example_method_property() {
        let input = "METHOD:REQUEST";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        let known = prop.try_into_known().unwrap();
        assert_eq!(known.name, StaticProp::Method);
        let PropValue::Method(p) = known.value else { panic!("expected Method") };
        assert_eq!(p.value, Token::Known(Method::Request));
    }

    #[test]
    fn rfc_5545_example_product_identifier_property() {
        let input = "PRODID:-//ABC Corporation//NONSGML My Product//EN";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(
            prop,
            known_prop!(ProdId, Text, "-//ABC Corporation//NONSGML My Product//EN".to_string())
        );
    }

    #[test]
    fn rfc_5545_example_version_property() {
        let input = "VERSION:2.0";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(prop, known_prop!(Version, Version, Version::V2_0));
    }

    #[test]
    fn rfc_5545_example_attachment_property_1() {
        let input = "ATTACH:CID:jsmith.part3.960817T083000.xyzMail@example.com";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        let known = prop.try_into_known().unwrap();
        assert_eq!(known.name, StaticProp::Attach);
        let PropValue::Attachment(p) = known.value else { panic!("expected Attachment") };
        // Attachment::Uri uses calendar_types Uri, but we can compare via string
        match &p.value {
            Attachment::Uri(u) => assert_eq!(
                u.as_str(),
                "CID:jsmith.part3.960817T083000.xyzMail@example.com"
            ),
            _ => panic!("expected Attachment::Uri"),
        }
    }

    #[test]
    fn rfc_5545_example_attachment_property_2() {
        let input =
            "ATTACH;FMTTYPE=application/postscript:ftp://example.com/pub/\r\n reports/r-960812.ps"
                .as_escaped();
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());

        let known = prop.try_into_known().unwrap();
        assert_eq!(known.name, StaticProp::Attach);
        let PropValue::Attachment(p) = known.value else { panic!("expected Attachment") };

        match &p.value {
            Attachment::Uri(u) => assert_eq!(
                u.as_str(),
                "ftp://example.com/pub/reports/r-960812.ps"
            ),
            _ => panic!("expected Attachment::Uri"),
        }

        let expected_fmt = FormatTypeBuf::from(
            rfc5545_types::value::FormatType::new("application/postscript").unwrap()
        );
        assert_eq!(p.params.format_type(), Some(&expected_fmt));
    }

    #[test]
    fn rfc_5545_example_categories_property_1() {
        let input = "CATEGORIES:APPOINTMENT,EDUCATION";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(
            prop,
            known_prop!(
                Categories,
                TextSeq,
                vec!["APPOINTMENT".to_string(), "EDUCATION".to_string()]
            )
        );
    }

    #[test]
    fn rfc_5545_example_categories_property_2() {
        let input = "CATEGORIES:MEETING";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(
            prop,
            known_prop!(Categories, TextSeq, vec!["MEETING".to_string()])
        );
    }

    #[test]
    fn rfc_5545_example_classification_property() {
        let input = "CLASS:PUBLIC";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        let known = prop.try_into_known().unwrap();
        assert_eq!(known.name, StaticProp::Class);
        let PropValue::ClassValue(p) = known.value else { panic!("expected ClassValue") };
        assert_eq!(p.value, Token::Known(ClassValue::Public));
    }

    #[test]
    fn rfc_5545_example_comment_property() {
        let input = "COMMENT:The meeting really needs to include both ourselves \r\n and the customer. We can't hold this meeting without them. \r\n As a matter of fact\\, the venue for the meeting ought to be at \r\n their site. - - John";

        let (tail, prop) = property::<_, ()>.parse_peek(input.as_escaped()).unwrap();

        assert!(tail.is_empty());
        let known = prop.try_into_known().unwrap();
        assert_eq!(known.name, StaticProp::Comment);
        let PropValue::Text(p) = known.value else { panic!("expected Text") };
        // After line-fold removal, the value should have the folds stripped
        assert!(p.value.contains("both ourselves"));
        assert!(p.value.contains("the customer"));
    }

    #[test]
    fn rfc_5545_example_description_property() {
        let input = "DESCRIPTION:Meeting to provide technical review for \"Phoenix\" \r\n design.\\nHappy Face Conference Room. Phoenix design team \r\n MUST attend this meeting.\\nRSVP to team leader.";

        let (tail, prop) = property::<_, ()>.parse_peek(input.as_escaped()).unwrap();

        assert!(tail.is_empty());
        let known = prop.try_into_known().unwrap();
        assert_eq!(known.name, StaticProp::Description);
        let PropValue::Text(p) = known.value else { panic!("expected Text") };
        assert!(p.value.starts_with("Meeting to provide technical review"));
    }

    #[test]
    fn rfc_5545_example_geographic_position_property() {
        let input = "GEO:37.386013;-122.082932";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        let known = prop.try_into_known().unwrap();
        assert_eq!(known.name, StaticProp::Geo);
        let PropValue::Geo(p) = known.value else { panic!("expected Geo") };
        assert!((p.value.lat - 37.386013).abs() < 1e-6);
        assert!((p.value.lon - (-122.082932)).abs() < 1e-6);
    }

    #[test]
    fn rfc_5545_example_location_property_1() {
        let input = "LOCATION:Conference Room - F123\\, Bldg. 002";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(
            prop,
            known_prop!(Location, Text, "Conference Room - F123, Bldg. 002".to_string())
        );
    }

    #[test]
    fn rfc_5545_example_location_property_2() {
        let input = "LOCATION;ALTREP=\"http://xyzcorp.com/conf-rooms/f123.vcf\":\r\n Conference Room - F123\\, Bldg. 002";
        let (tail, prop) = property::<_, ()>.parse_peek(input.as_escaped()).unwrap();
        assert!(tail.is_empty());

        let known = prop.try_into_known().unwrap();
        assert_eq!(known.name, StaticProp::Location);
        let PropValue::Text(p) = known.value else { panic!("expected Text") };

        let alt_rep = p.params.alternate_representation().unwrap();
        assert_eq!(alt_rep.as_str(), "http://xyzcorp.com/conf-rooms/f123.vcf");
        assert!(p.value.starts_with("Conference Room - F123"));
    }

    #[test]
    fn rfc_5545_example_percent_complete_property() {
        let input = "PERCENT-COMPLETE:39";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(
            prop,
            known_prop!(PercentComplete, CompletionPercentage, CompletionPercentage::new(39).unwrap())
        );
    }

    #[test]
    fn rfc_5545_example_priority_property_1() {
        let input = "PRIORITY:1";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(prop, known_prop!(Priority, Priority, Priority::A1));
    }

    #[test]
    fn rfc_5545_example_priority_property_2() {
        let input = "PRIORITY:2";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(prop, known_prop!(Priority, Priority, Priority::A2));
    }

    #[test]
    fn rfc_5545_example_priority_property_3() {
        let input = "PRIORITY:0";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(prop, known_prop!(Priority, Priority, Priority::Zero));
    }

    #[test]
    fn rfc_5545_example_resources_property_1() {
        let input = "RESOURCES:EASEL,PROJECTOR,VCR";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(
            prop,
            known_prop!(
                Resources,
                TextSeq,
                vec!["EASEL".to_string(), "PROJECTOR".to_string(), "VCR".to_string()]
            )
        );
    }

    #[test]
    fn rfc_5545_example_resources_property_2() {
        let input = "RESOURCES;LANGUAGE=fr:Nettoyeur haute pression";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();
        assert!(tail.is_empty());

        let known = prop.try_into_known().unwrap();
        assert_eq!(known.name, StaticProp::Resources);
        let PropValue::TextSeq(p) = known.value else { panic!("expected TextSeq") };

        assert_eq!(p.params.language(), Some(&Language::parse("fr").unwrap()));
        assert_eq!(p.value, vec!["Nettoyeur haute pression".to_string()]);
    }

    #[test]
    fn rfc_5545_example_status_property_1() {
        let input = "STATUS:TENTATIVE";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(prop, known_prop!(Status, Status, Status::Tentative));
    }

    #[test]
    fn rfc_5545_example_status_property_2() {
        let input = "STATUS:NEEDS-ACTION";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(prop, known_prop!(Status, Status, Status::NeedsAction));
    }

    #[test]
    fn rfc_5545_example_status_property_3() {
        let input = "STATUS:DRAFT";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(prop, known_prop!(Status, Status, Status::Draft));
    }

    #[test]
    fn rfc_5545_example_summary_property() {
        let input = "SUMMARY:Department Party";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(
            prop,
            known_prop!(Summary, Text, "Department Party".to_string())
        );
    }

    #[test]
    fn rfc_5545_example_date_time_completed_property() {
        let input = "COMPLETED:19960401T150000Z";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(
            prop,
            known_prop!(DtCompleted, DateTimeUtc, utc_datetime(1996, 4, 1, 15, 0, 0))
        );
    }

    #[test]
    fn rfc_5545_example_date_time_end_property_1() {
        let input = "DTEND:19960401T150000Z";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(
            prop,
            known_prop!(
                DtEnd,
                DateTimeOrDate,
                DateTimeOrDate::DateTime(tf_utc_datetime(1996, 4, 1, 15, 0, 0))
            )
        );
    }

    #[test]
    fn rfc_5545_example_date_time_end_property_2() {
        let input = "DTEND;VALUE=DATE:19980704";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(
            prop,
            known_prop!(DtEnd, DateTimeOrDate, DateTimeOrDate::Date(date!(1998;7;4)))
        );
    }

    #[test]
    fn rfc_5545_example_date_time_due_property() {
        let input = "DUE:19980430T000000Z";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(
            prop,
            known_prop!(
                DtDue,
                DateTimeOrDate,
                DateTimeOrDate::DateTime(tf_utc_datetime(1998, 4, 30, 0, 0, 0))
            )
        );
    }

    #[test]
    fn rfc_5545_example_date_time_start_property() {
        let input = "DTSTART:19980118T073000Z";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(
            prop,
            known_prop!(
                DtStart,
                DateTimeOrDate,
                DateTimeOrDate::DateTime(tf_utc_datetime(1998, 1, 18, 7, 30, 0))
            )
        );
    }

    #[test]
    fn rfc_5545_example_duration_property_1() {
        let input = "DURATION:PT1H0M0S";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(
            prop,
            known_prop!(Duration, Duration, exact_dur(1, 0, 0))
        );
    }

    #[test]
    fn rfc_5545_example_duration_property_2() {
        let input = "DURATION:PT15M";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(
            prop,
            known_prop!(Duration, Duration, exact_dur(0, 15, 0))
        );
    }

    #[test]
    fn rfc_5545_example_free_busy_time_property_1() {
        let input = "FREEBUSY;FBTYPE=BUSY-UNAVAILABLE:19970308T160000Z/PT8H30M";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();
        assert!(tail.is_empty());

        let known = prop.try_into_known().unwrap();
        assert_eq!(known.name, StaticProp::FreeBusy);
        let PropValue::FreeBusyPeriods(p) = known.value else { panic!("expected FreeBusyPeriods") };

        assert_eq!(
            p.params.free_busy_type(),
            Some(&Token::Known(FreeBusyType::BusyUnavailable))
        );

        assert_eq!(
            p.value,
            vec![Period::Start {
                start: tf_utc_datetime(1997, 3, 8, 16, 0, 0),
                duration: unsigned_exact_dur(8, 30, 0),
            }]
        );
    }

    #[test]
    fn rfc_5545_example_free_busy_time_property_2() {
        let input = "FREEBUSY;FBTYPE=FREE:19970308T160000Z/PT3H,19970308T200000Z/PT1H";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();
        assert!(tail.is_empty());

        let known = prop.try_into_known().unwrap();
        assert_eq!(known.name, StaticProp::FreeBusy);
        let PropValue::FreeBusyPeriods(p) = known.value else { panic!("expected FreeBusyPeriods") };

        assert_eq!(
            p.params.free_busy_type(),
            Some(&Token::Known(FreeBusyType::Free))
        );

        assert_eq!(
            p.value,
            vec![
                Period::Start {
                    start: tf_utc_datetime(1997, 3, 8, 16, 0, 0),
                    duration: unsigned_exact_dur(3, 0, 0),
                },
                Period::Start {
                    start: tf_utc_datetime(1997, 3, 8, 20, 0, 0),
                    duration: unsigned_exact_dur(1, 0, 0),
                },
            ]
        );
    }

    #[test]
    #[ignore = "Escaped stream + lz_dec_uint interaction causes unreachable!() panic when fold occurs before comma in period list"]
    fn rfc_5545_example_free_busy_time_property_3() {
        let input = "FREEBUSY;FBTYPE=FREE:19970308T160000Z/PT3H,19970308T200000Z/PT1H\r\n\t,19970308T230000Z/19970309T000000Z";
        let (tail, prop) = property::<_, ()>.parse_peek(input.as_escaped()).unwrap();
        assert!(tail.is_empty());

        let known = prop.try_into_known().unwrap();
        assert_eq!(known.name, StaticProp::FreeBusy);
        let PropValue::FreeBusyPeriods(p) = known.value else { panic!("expected FreeBusyPeriods") };

        assert_eq!(
            p.params.free_busy_type(),
            Some(&Token::Known(FreeBusyType::Free))
        );

        assert_eq!(
            p.value,
            vec![
                Period::Start {
                    start: tf_utc_datetime(1997, 3, 8, 16, 0, 0),
                    duration: unsigned_exact_dur(3, 0, 0),
                },
                Period::Start {
                    start: tf_utc_datetime(1997, 3, 8, 20, 0, 0),
                    duration: unsigned_exact_dur(1, 0, 0),
                },
                Period::Explicit {
                    start: tf_utc_datetime(1997, 3, 8, 23, 0, 0),
                    end: tf_utc_datetime(1997, 3, 9, 0, 0, 0),
                }
            ]
        );
    }

    #[test]
    fn rfc_5545_example_time_transparency_property_1() {
        let input = "TRANSP:TRANSPARENT";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(
            prop,
            known_prop!(Transp, TimeTransparency, TimeTransparency::Transparent)
        );
    }

    #[test]
    fn rfc_5545_example_time_transparency_property_2() {
        let input = "TRANSP:OPAQUE";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(
            prop,
            known_prop!(Transp, TimeTransparency, TimeTransparency::Opaque)
        );
    }

    #[test]
    fn rfc_5545_example_time_zone_identifier_property_1() {
        let input = "TZID:America/New_York";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(
            prop,
            known_prop!(TzId, TzId, TzId::new("America/New_York").unwrap().into())
        );
    }

    #[test]
    fn rfc_5545_example_time_zone_identifier_property_2() {
        let input = "TZID:America/Los_Angeles";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(
            prop,
            known_prop!(TzId, TzId, TzId::new("America/Los_Angeles").unwrap().into())
        );
    }

    #[test]
    fn rfc_5545_example_time_zone_identifier_property_3() {
        let input = "TZID:/example.org/America/New_York";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(
            prop,
            known_prop!(TzId, TzId, TzId::new("/example.org/America/New_York").unwrap().into())
        );
    }

    #[test]
    fn rfc_5545_example_time_zone_name_property_1() {
        let input = "TZNAME:EST";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(prop, known_prop!(TzName, Text, "EST".to_string()));
    }

    #[test]
    fn rfc_5545_example_time_zone_name_property_2() {
        let input = "TZNAME;LANGUAGE=fr-CA:HNE";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();
        assert!(tail.is_empty());

        let known = prop.try_into_known().unwrap();
        assert_eq!(known.name, StaticProp::TzName);
        let PropValue::Text(p) = known.value else { panic!("expected Text") };

        assert_eq!(p.params.language(), Some(&Language::parse("fr-CA").unwrap()));
        assert_eq!(p.value, "HNE");
    }

    #[test]
    fn rfc_5545_example_time_zone_offset_from_property_1() {
        let input = "TZOFFSETFROM:-0500";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(prop, known_prop!(TzOffsetFrom, UtcOffset, utc_offset!(-5;00)));
    }

    #[test]
    fn rfc_5545_example_time_zone_offset_from_property_2() {
        let input = "TZOFFSETFROM:+1345";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(prop, known_prop!(TzOffsetFrom, UtcOffset, utc_offset!(+13;45)));
    }

    #[test]
    fn rfc_5545_example_time_zone_offset_to_property_1() {
        let input = "TZOFFSETTO:-0400";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(prop, known_prop!(TzOffsetTo, UtcOffset, utc_offset!(-4;00)));
    }

    #[test]
    fn rfc_5545_example_time_zone_offset_to_property_2() {
        let input = "TZOFFSETTO:+1245";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(prop, known_prop!(TzOffsetTo, UtcOffset, utc_offset!(+12;45)));
    }

    #[test]
    fn rfc_5545_example_time_zone_url_property() {
        let input = "TZURL:http://timezones.example.org/tz/America-Los_Angeles.ics";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(
            prop,
            known_prop!(
                TzUrl,
                Uri,
                Uri::new("http://timezones.example.org/tz/America-Los_Angeles.ics").unwrap().into()
            )
        );
    }

    #[test]
    fn rfc_5545_example_attendee_property_1() {
        let input =
            "ATTENDEE;MEMBER=\"mailto:DEV-GROUP@example.com\":\r\n mailto:joecool@example.com";
        let (tail, prop) = property::<_, ()>.parse_peek(input.as_escaped()).unwrap();
        assert!(tail.is_empty());

        let known = prop.try_into_known().unwrap();
        assert_eq!(known.name, StaticProp::Attendee);
        let PropValue::Uri(p) = known.value else { panic!("expected Uri") };

        // Check membership param
        let membership = p.params.membership().unwrap();
        assert_eq!(membership.len().get(), 1);
        assert_eq!(membership[0].as_str(), "mailto:DEV-GROUP@example.com");

        assert_eq!(p.value.as_str(), "mailto:joecool@example.com");
    }

    #[test]
    fn rfc_5545_example_attendee_property_2() {
        let input =
            "ATTENDEE;DELEGATED-FROM=\"mailto:immud@example.com\":\r\n mailto:ildoit@example.com";
        let (tail, prop) = property::<_, ()>.parse_peek(input.as_escaped()).unwrap();
        assert!(tail.is_empty());

        let known = prop.try_into_known().unwrap();
        assert_eq!(known.name, StaticProp::Attendee);
        let PropValue::Uri(p) = known.value else { panic!("expected Uri") };

        // Check delegated-from param
        let del_from = p.params.delegated_from().unwrap();
        assert_eq!(del_from.len().get(), 1);
        assert_eq!(del_from[0].as_str(), "mailto:immud@example.com");

        assert_eq!(p.value.as_str(), "mailto:ildoit@example.com");
    }

    #[test]
    fn rfc_5545_example_attendee_property_3() {
        let input = "ATTENDEE;ROLE=REQ-PARTICIPANT;PARTSTAT=TENTATIVE;CN=Henry\r\n Cabot:mailto:hcabot@example.com";
        let (tail, prop) = property::<_, ()>.parse_peek(input.as_escaped()).unwrap();
        assert!(tail.is_empty());

        let known = prop.try_into_known().unwrap();
        assert_eq!(known.name, StaticProp::Attendee);
        let PropValue::Uri(p) = known.value else { panic!("expected Uri") };

        assert_eq!(
            p.params.participation_role(),
            Some(&Token::Known(ParticipationRole::ReqParticipant))
        );
        assert_eq!(
            p.params.participation_status(),
            Some(&Token::Known(ParticipationStatus::Tentative))
        );
        // Common name is a ParamValue (DST newtype)
        let cn = p.params.common_name().unwrap();
        assert!(cn.as_str().contains("Henry"));
        assert!(cn.as_str().contains("Cabot"));

        assert_eq!(p.value.as_str(), "mailto:hcabot@example.com");
    }

    #[test]
    fn rfc_5545_example_contact_property_1() {
        let input = "CONTACT:Jim Dolittle\\, ABC Industries\\, +1-919-555-1234";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(
            prop,
            known_prop!(
                Contact,
                Text,
                "Jim Dolittle, ABC Industries, +1-919-555-1234".to_string()
            )
        );
    }

    #[test]
    fn rfc_5545_example_contact_property_2() {
        let input = "CONTACT;ALTREP=\"ldap://example.com:6666/o=ABC%20Industries\\,\r\n c=US???(cn=Jim%20Dolittle)\":Jim Dolittle\\, ABC Industries\\,\r\n +1-919-555-1234";
        let (tail, prop) = property::<_, ()>.parse_peek(input.as_escaped()).unwrap();
        assert!(tail.is_empty());

        let known = prop.try_into_known().unwrap();
        assert_eq!(known.name, StaticProp::Contact);
        let PropValue::Text(p) = known.value else { panic!("expected Text") };

        let alt_rep = p.params.alternate_representation().unwrap();
        assert!(alt_rep.as_str().starts_with("ldap://example.com"));
        assert!(p.value.starts_with("Jim Dolittle"));
    }

    #[test]
    fn rfc_5545_example_organizer_property_1() {
        let input = "ORGANIZER;CN=John Smith:mailto:jsmith@example.com";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();
        assert!(tail.is_empty());

        let known = prop.try_into_known().unwrap();
        assert_eq!(known.name, StaticProp::Organizer);
        let PropValue::Uri(p) = known.value else { panic!("expected Uri") };

        let cn = p.params.common_name().unwrap();
        assert_eq!(cn.as_str(), "John Smith");
        assert_eq!(p.value.as_str(), "mailto:jsmith@example.com");
    }

    #[test]
    fn rfc_5545_example_organizer_property_2() {
        let input = "ORGANIZER;CN=JohnSmith;DIR=\"ldap://example.com:6666/o=DC%20Ass\r\n ociates,c=US???(cn=John%20Smith)\":mailto:jsmith@example.com";
        let (tail, prop) = property::<_, ()>.parse_peek(input.as_escaped()).unwrap();
        assert!(tail.is_empty());

        let known = prop.try_into_known().unwrap();
        assert_eq!(known.name, StaticProp::Organizer);
        let PropValue::Uri(p) = known.value else { panic!("expected Uri") };

        let cn = p.params.common_name().unwrap();
        assert_eq!(cn.as_str(), "JohnSmith");

        let dir = p.params.directory_reference().unwrap();
        assert!(dir.as_str().starts_with("ldap://example.com"));

        assert_eq!(p.value.as_str(), "mailto:jsmith@example.com");
    }

    #[test]
    fn rfc_5545_example_recurrence_identifier_property_1() {
        let input = "RECURRENCE-ID;VALUE=DATE:19960401";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(
            prop,
            known_prop!(RecurId, DateTimeOrDate, DateTimeOrDate::Date(date!(1996;4;1)))
        );
    }

    #[test]
    fn rfc_5545_example_recurrence_identifier_property_2() {
        let input = "RECURRENCE-ID;RANGE=THISANDFUTURE:19960120T120000Z";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();
        assert!(tail.is_empty());

        let known = prop.try_into_known().unwrap();
        assert_eq!(known.name, StaticProp::RecurId);
        let PropValue::DateTimeOrDate(p) = known.value else { panic!("expected DateTimeOrDate") };

        assert_eq!(p.params.recurrence_range(), Some(&ThisAndFuture));

        assert_eq!(
            p.value,
            DateTimeOrDate::DateTime(tf_utc_datetime(1996, 1, 20, 12, 0, 0))
        );
    }

    #[test]
    fn rfc_5545_example_uid_property() {
        let input = "UID:19960401T080045Z-4000F192713-0052@example.com";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(
            prop,
            known_prop!(
                Uid,
                Uid,
                Uid::new("19960401T080045Z-4000F192713-0052@example.com").unwrap().into()
            )
        );
    }

    #[test]
    fn rfc_5545_example_request_status_property_1() {
        let input = "REQUEST-STATUS:2.0;Success";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(
            prop,
            known_prop!(
                RequestStatus,
                RequestStatus,
                RequestStatus {
                    code: status_code(2, 0),
                    description: "Success".into(),
                    exception_data: None,
                }
            )
        );
    }

    #[test]
    fn rfc_5545_example_request_status_property_2() {
        let input = "REQUEST-STATUS:3.1;Invalid property value;DTSTART:96-Apr-01";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(
            prop,
            known_prop!(
                RequestStatus,
                RequestStatus,
                RequestStatus {
                    code: status_code(3, 1),
                    description: "Invalid property value".into(),
                    exception_data: Some("DTSTART:96-Apr-01".into()),
                }
            )
        );
    }

    // NOTE: skipped the third example

    #[test]
    fn rfc_5545_example_request_status_property_4() {
        let input = "REQUEST-STATUS:4.1;Event conflict.  Date-time is busy.";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();

        assert!(tail.is_empty());
        assert_eq!(
            prop,
            known_prop!(
                RequestStatus,
                RequestStatus,
                RequestStatus {
                    code: status_code(4, 1),
                    description: "Event conflict.  Date-time is busy.".into(),
                    exception_data: None,
                }
            )
        );
    }

    #[test]
    fn rfc_5545_example_iana_property() {
        let mut input: &str = "NON-SMOKING;VALUE=BOOLEAN:TRUE";
        let prop = property::<_, ()>(&mut input).unwrap();
        let unknown = prop.try_into_unknown().unwrap();
        assert_eq!(unknown.name, "NON-SMOKING");
        assert_eq!(unknown.kind, NameKind::Iana);
        assert_eq!(unknown.value, Value::Boolean(true));

        let mut input: &str = "NON-SMOKING:TRUE";
        let prop = property::<_, ()>(&mut input).unwrap();
        let unknown = prop.try_into_unknown().unwrap();
        assert_eq!(unknown.name, "NON-SMOKING");
        assert_eq!(unknown.kind, NameKind::Iana);
        assert_eq!(unknown.value, Value::Text("TRUE".to_string()));
    }

    #[test]
    fn rfc_5545_example_x_property() {
        let input = "X-ABC-MMSUBJ;VALUE=URI;FMTTYPE=audio/basic:http://www.example.org/mysubj.au";
        let (tail, prop) = property::<_, ()>.parse_peek(input).unwrap();
        assert!(tail.is_empty());

        let unknown = prop.try_into_unknown().unwrap();
        assert_eq!(unknown.name, "X-ABC-MMSUBJ");
        assert_eq!(unknown.kind, NameKind::X);
        match &unknown.value {
            Value::Uri(uri) => assert_eq!(uri.as_str(), "http://www.example.org/mysubj.au"),
            other => panic!("expected Value::Uri, got {other:?}"),
        }
    }

    #[test]
    fn integer_value_parsing() {
        for mut i in ["0", "-2147483648", "2147483647"] {
            assert!(parse_value::<_, ()>(Token::Known(ValueType::Integer), &mut i).is_ok());
        }

        for mut i in ["-2147483649", "2147483648"] {
            assert!(parse_value::<_, ()>(Token::Known(ValueType::Integer), &mut i).is_err());
        }
    }

    // PROPERTY NAME TESTS

    /// Asserts that the inputs are equal under [`property_name`].
    fn assert_prop_name_eq<'i>(input: &'i str, expected: PropName<&'i str>) {
        let mut input_ref = input;
        let result = property_name::<_, ()>.parse_next(&mut input_ref);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);
        assert!(input_ref.is_empty());
    }

    // Helper function to test parsing failures
    fn assert_prop_name_parse_failure(input: &str) {
        let mut input_ref = input;
        let result = property_name::<_, ()>.parse_next(&mut input_ref);
        assert!(result.is_err());
    }

    #[test]
    fn rfc5545_calendar_property_names() {
        assert_prop_name_eq("CALSCALE", PropName::Known(StaticProp::CalScale));
        assert_prop_name_eq("METHOD", PropName::Known(StaticProp::Method));
        assert_prop_name_eq("PRODID", PropName::Known(StaticProp::ProdId));
        assert_prop_name_eq("VERSION", PropName::Known(StaticProp::Version));
    }

    #[test]
    fn rfc5545_descriptive_property_names() {
        assert_prop_name_eq("ATTACH", PropName::Known(StaticProp::Attach));
        assert_prop_name_eq("CATEGORIES", PropName::Known(StaticProp::Categories));
        assert_prop_name_eq("CLASS", PropName::Known(StaticProp::Class));
        assert_prop_name_eq("COMMENT", PropName::Known(StaticProp::Comment));
        assert_prop_name_eq("DESCRIPTION", PropName::Known(StaticProp::Description));
        assert_prop_name_eq("GEO", PropName::Known(StaticProp::Geo));
        assert_prop_name_eq("LOCATION", PropName::Known(StaticProp::Location));
        assert_prop_name_eq(
            "PERCENT-COMPLETE",
            PropName::Known(StaticProp::PercentComplete),
        );
        assert_prop_name_eq("PRIORITY", PropName::Known(StaticProp::Priority));
        assert_prop_name_eq("RESOURCES", PropName::Known(StaticProp::Resources));
        assert_prop_name_eq("STATUS", PropName::Known(StaticProp::Status));
        assert_prop_name_eq("SUMMARY", PropName::Known(StaticProp::Summary));
    }

    #[test]
    fn rfc5545_datetime_property_names() {
        assert_prop_name_eq("COMPLETED", PropName::Known(StaticProp::DtCompleted));
        assert_prop_name_eq("DTEND", PropName::Known(StaticProp::DtEnd));
        assert_prop_name_eq("DUE", PropName::Known(StaticProp::DtDue));
        assert_prop_name_eq("DTSTART", PropName::Known(StaticProp::DtStart));
        assert_prop_name_eq("DURATION", PropName::Known(StaticProp::Duration));
        assert_prop_name_eq("FREEBUSY", PropName::Known(StaticProp::FreeBusy));
        assert_prop_name_eq("TRANSP", PropName::Known(StaticProp::Transp));
        assert_prop_name_eq("DTSTAMP", PropName::Known(StaticProp::DtStamp));
    }

    #[test]
    fn rfc5545_timezone_property_names() {
        assert_prop_name_eq("TZID", PropName::Known(StaticProp::TzId));
        assert_prop_name_eq("TZNAME", PropName::Known(StaticProp::TzName));
        assert_prop_name_eq("TZOFFSETFROM", PropName::Known(StaticProp::TzOffsetFrom));
        assert_prop_name_eq("TZOFFSETTO", PropName::Known(StaticProp::TzOffsetTo));
        assert_prop_name_eq("TZURL", PropName::Known(StaticProp::TzUrl));
    }

    #[test]
    fn rfc5545_relationship_property_names() {
        assert_prop_name_eq("ATTENDEE", PropName::Known(StaticProp::Attendee));
        assert_prop_name_eq("CONTACT", PropName::Known(StaticProp::Contact));
        assert_prop_name_eq("ORGANIZER", PropName::Known(StaticProp::Organizer));
        assert_prop_name_eq("RECURRENCE-ID", PropName::Known(StaticProp::RecurId));
        assert_prop_name_eq("RELATED-TO", PropName::Known(StaticProp::RelatedTo));
        assert_prop_name_eq("URL", PropName::Known(StaticProp::Url));
        assert_prop_name_eq("UID", PropName::Known(StaticProp::Uid));
    }

    #[test]
    fn rfc5545_recurrence_property_names() {
        assert_prop_name_eq("EXDATE", PropName::Known(StaticProp::ExDate));
        assert_prop_name_eq("RDATE", PropName::Known(StaticProp::RDate));
        assert_prop_name_eq("RRULE", PropName::Known(StaticProp::RRule));
    }

    #[test]
    fn rfc5545_alarm_property_names() {
        assert_prop_name_eq("ACTION", PropName::Known(StaticProp::Action));
        assert_prop_name_eq("REPEAT", PropName::Known(StaticProp::Repeat));
        assert_prop_name_eq("TRIGGER", PropName::Known(StaticProp::Trigger));
    }

    #[test]
    fn rfc5545_change_management_property_names() {
        assert_prop_name_eq("CREATED", PropName::Known(StaticProp::Created));
        assert_prop_name_eq("LAST-MODIFIED", PropName::Known(StaticProp::LastModified));
        assert_prop_name_eq("SEQUENCE", PropName::Known(StaticProp::Sequence));
    }

    #[test]
    fn rfc5545_miscellaneous_property_names() {
        assert_prop_name_eq("REQUEST-STATUS", PropName::Known(StaticProp::RequestStatus));
    }

    #[test]
    fn rfc7986_property_names() {
        assert_prop_name_eq("NAME", PropName::Known(StaticProp::Name));
        assert_prop_name_eq(
            "REFRESH-INTERVAL",
            PropName::Known(StaticProp::RefreshInterval),
        );
        assert_prop_name_eq("SOURCE", PropName::Known(StaticProp::Source));
        assert_prop_name_eq("COLOR", PropName::Known(StaticProp::Color));
        assert_prop_name_eq("IMAGE", PropName::Known(StaticProp::Image));
        assert_prop_name_eq("CONFERENCE", PropName::Known(StaticProp::Conference));
    }

    #[test]
    fn property_name_case_insensitivity() {
        assert_prop_name_eq("dtstart", PropName::Known(StaticProp::DtStart));
        assert_prop_name_eq("DTSTART", PropName::Known(StaticProp::DtStart));
        assert_prop_name_eq("DtStArT", PropName::Known(StaticProp::DtStart));
        assert_prop_name_eq("dtSTART", PropName::Known(StaticProp::DtStart));

        assert_prop_name_eq("conference", PropName::Known(StaticProp::Conference));
        assert_prop_name_eq("Conference", PropName::Known(StaticProp::Conference));
        assert_prop_name_eq("CONFERENCE", PropName::Known(StaticProp::Conference));
    }

    #[test]
    fn iana_property_names() {
        assert_prop_name_eq("UNKNOWN-PROP", PropName::iana("UNKNOWN-PROP"));
        assert_prop_name_eq("CUSTOM", PropName::iana("CUSTOM"));
        assert_prop_name_eq("NEW-FEATURE", PropName::iana("NEW-FEATURE"));
    }

    #[test]
    fn x_property_names() {
        assert_prop_name_eq("X-CUSTOM", PropName::x("X-CUSTOM"));
        assert_prop_name_eq("X-VENDOR-PROP", PropName::x("X-VENDOR-PROP"));
        assert_prop_name_eq("X-custom", PropName::x("X-custom"));
    }

    #[test]
    fn property_name_longest_match_precedence() {
        // Ensure longer properties are matched correctly when they share prefixes
        assert_prop_name_eq(
            "REFRESH-INTERVAL",
            PropName::Known(StaticProp::RefreshInterval),
        );
        assert_prop_name_eq("REQUEST-STATUS", PropName::Known(StaticProp::RequestStatus));
        assert_prop_name_eq("RECURRENCE-ID", PropName::Known(StaticProp::RecurId));

        // Make sure we don't match shorter prefixes
        assert_prop_name_eq("REFRESH", PropName::iana("REFRESH"));
        assert_prop_name_eq("REQUEST", PropName::iana("REQUEST"));
        assert_prop_name_eq("RECURRENCE", PropName::iana("RECURRENCE"));
    }

    #[test]
    fn property_name_edge_cases() {
        // Empty input
        assert_prop_name_parse_failure("");

        // Single characters
        assert_prop_name_eq("A", PropName::iana("A"));
        assert_prop_name_eq("Z", PropName::iana("Z"));

        // Properties with hyphens
        assert_prop_name_eq("LAST-MODIFIED", PropName::Known(StaticProp::LastModified));
        assert_prop_name_eq(
            "PERCENT-COMPLETE",
            PropName::Known(StaticProp::PercentComplete),
        );
        assert_prop_name_eq(
            "REFRESH-INTERVAL",
            PropName::Known(StaticProp::RefreshInterval),
        );

        // Mixed case with hyphens
        assert_prop_name_eq("last-modified", PropName::Known(StaticProp::LastModified));
        assert_prop_name_eq(
            "Percent-Complete",
            PropName::Known(StaticProp::PercentComplete),
        );
    }
}
