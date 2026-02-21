//! Tools for interacting with types representing JSON values.

use std::{
    borrow::{Borrow, Cow},
    collections::{HashMap, HashSet, VecDeque},
    convert::Infallible,
    fmt,
    hash::Hash,
    str::FromStr,
};

use calendar_types::{
    duration::{Duration, InvalidDurationError, SignedDuration},
    set::Token,
    time::{DateTime, InvalidDateTimeError, Local, Utc},
};
use crate::model::set::{Percent, Priority};
use thiserror::Error;

use crate::parser::{
    DateTimeParseError, DurationParseError, OwnedParseError, SignedDurationParseError,
    UtcDateTimeParseError, duration, local_date_time, parse_full, signed_duration, utc_date_time,
};

/// Fallible conversion from a JSON value into a Rust type.
pub trait TryFromJson<V>
where
    Self: Sized,
    V: DestructibleJsonValue,
{
    /// The error type returned on failure.
    type Error;

    /// Attempts to convert a JSON value into this type.
    fn try_from_json(value: V) -> Result<Self, Self::Error>;
}

impl<V: DestructibleJsonValue> TryFromJson<V> for bool {
    type Error = TypeError;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        value.try_as_bool()
    }
}

impl<V: DestructibleJsonValue> TryFromJson<V> for String {
    type Error = TypeError;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        value.try_into_string().map(Into::into)
    }
}

impl<V: DestructibleJsonValue> TryFromJson<V> for DateTime<Local> {
    type Error = TypeErrorOr<OwnedParseError<DateTimeParseError, InvalidDateTimeError>>;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let input = value.try_into_string()?;
        let date_time = parse_full(local_date_time)(input.as_ref()).map_err(TypeErrorOr::Other)?;
        Ok(date_time)
    }
}

impl<V: DestructibleJsonValue> TryFromJson<V> for DateTime<Utc> {
    type Error = TypeErrorOr<OwnedParseError<UtcDateTimeParseError, InvalidDateTimeError>>;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let input = value.try_into_string()?;
        let date_time = parse_full(utc_date_time)(input.as_ref()).map_err(TypeErrorOr::Other)?;
        Ok(date_time)
    }
}

impl<V: DestructibleJsonValue> TryFromJson<V> for Duration {
    type Error = TypeErrorOr<OwnedParseError<DurationParseError, InvalidDurationError>>;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let input = value.try_into_string()?;
        let duration = parse_full(duration)(input.as_ref()).map_err(TypeErrorOr::Other)?;
        Ok(duration)
    }
}

impl<V: DestructibleJsonValue> TryFromJson<V> for SignedDuration {
    type Error = TypeErrorOr<OwnedParseError<SignedDurationParseError, InvalidDurationError>>;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let input = value.try_into_string()?;
        let duration = parse_full(signed_duration)(input.as_ref()).map_err(TypeErrorOr::Other)?;
        Ok(duration)
    }
}

impl<T, V> TryFromJson<V> for Token<T, Box<str>>
where
    T: FromStr,
    V: DestructibleJsonValue,
{
    type Error = TypeError;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let s = value.try_into_string()?;
        // Token::from_str is infallible when S = Box<str> (since &str: Into<Box<str>>)
        Ok(Token::from_str(s.as_ref()).unwrap())
    }
}

impl<T, V> TryFromJson<V> for Vec<T>
where
    T: TryFromJson<V>,
    T::Error: IntoDocumentError,
    <T::Error as IntoDocumentError>::Residual: LiftTypeError,
    V: DestructibleJsonValue,
{
    type Error = DocumentError<
        TypeErrorOr<<<T::Error as IntoDocumentError>::Residual as LiftTypeError>::Residual>,
    >;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let array = value
            .try_into_array()
            .map_err(TypeErrorOr::from)
            .map_err(|error| DocumentError {
                path: VecDeque::new(),
                error,
            })?;

        array
            .into_iter()
            .enumerate()
            .map(|(i, elem)| {
                T::try_from_json(elem).map_err(|error| {
                    let DocumentError { mut path, error } = error.into_document_error();
                    let error = error.lift_type_error();
                    path.push_front(PathSegment::Index(i));
                    DocumentError { error, path }
                })
            })
            .collect::<Result<Vec<_>, _>>()
    }
}

impl<K, T, V, S> TryFromJson<V> for HashMap<K, T, S>
where
    K: Hash + Eq + From<String>,
    T: TryFromJson<V>,
    T::Error: IntoDocumentError,
    <T::Error as IntoDocumentError>::Residual: LiftTypeError,
    V: DestructibleJsonValue,
    S: Default + std::hash::BuildHasher,
{
    type Error = DocumentError<
        TypeErrorOr<<<T::Error as IntoDocumentError>::Residual as LiftTypeError>::Residual>,
    >;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let object = value
            .try_into_object()
            .map_err(TypeErrorOr::from)
            .map_err(|error| DocumentError {
                path: VecDeque::new(),
                error,
            })?;

        object
            .into_iter()
            .map(|(key, value)| match T::try_from_json(value) {
                Ok(value) => Ok((
                    <V::Object as JsonObject>::key_into_string(key).into(),
                    value,
                )),
                Err(error) => {
                    let DocumentError { mut path, error } = error.into_document_error();
                    let error = error.lift_type_error();
                    path.push_front(PathSegment::String(
                        <V::Object as JsonObject>::key_into_string(key).into_boxed_str(),
                    ));
                    Err(DocumentError { error, path })
                }
            })
            .collect::<Result<HashMap<_, _, _>, _>>()
    }
}

/// Error returned when parsing a `HashSet` from a JSON object.
#[derive(Debug, Clone, Copy, PartialEq, Error)]
pub enum HashSetTryFromJsonError<E> {
    /// A set entry had `false` as its value (only `true` is valid).
    #[error("encountered `false` as a value in a set")]
    UnexpectedFalseValue,
    /// The set key could not be parsed via `FromStr`.
    #[error(transparent)]
    FromStr(E),
}

impl<T, V, S> TryFromJson<V> for HashSet<T, S>
where
    T: FromStr + Eq + Hash,
    V: DestructibleJsonValue,
    S: Default + std::hash::BuildHasher,
{
    type Error = DocumentError<TypeErrorOr<HashSetTryFromJsonError<T::Err>>>;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        value
            .try_into_object()
            .map_err(TypeErrorOr::from)
            .map_err(DocumentError::root)?
            .into_iter()
            .map(|(key, value)| {
                let s = V::Object::key_into_string(key);

                match value.try_as_bool() {
                    Ok(true) => T::from_str(&s).map_err(|error| DocumentError {
                        path: vec![PathSegment::String(s.into_boxed_str())].into(),
                        error: TypeErrorOr::Other(HashSetTryFromJsonError::FromStr(error)),
                    }),
                    Ok(false) => Err(DocumentError {
                        path: vec![PathSegment::String(s.into_boxed_str())].into(),
                        error: TypeErrorOr::Other(HashSetTryFromJsonError::UnexpectedFalseValue),
                    }),
                    Err(error) => Err(DocumentError {
                        path: vec![PathSegment::String(s.into_boxed_str())].into(),
                        error: TypeErrorOr::from(error),
                    }),
                }
            })
            .collect::<Result<HashSet<T, S>, _>>()
    }
}

/// Fallible conversion from a Rust type into a JSON value.
pub trait TryIntoJson<V>
where
    V: ConstructibleJsonValue,
{
    /// The error type returned on failure.
    type Error;

    /// Attempts to convert this value into a JSON value.
    fn try_into_json(self) -> Result<V, Self::Error>;
}

/// Infallible conversion from a Rust type into a JSON value.
pub trait IntoJson<V>
where
    V: ConstructibleJsonValue,
{
    /// Converts this value into a JSON value.
    fn into_json(self) -> V;
}

impl<T: IntoJson<V>, V: ConstructibleJsonValue> TryIntoJson<V> for T {
    type Error = std::convert::Infallible;

    fn try_into_json(self) -> Result<V, Self::Error> {
        Ok(self.into_json())
    }
}

/// Conversion of a field-level error into a [`DocumentError`] with a JSON path.
pub trait IntoDocumentError: Sized {
    /// The error type after extraction of any [`DocumentError`] path information.
    type Residual;

    /// Wraps this error in a [`DocumentError`].
    fn into_document_error(self) -> DocumentError<Self::Residual>;
}

macro_rules! trivial_into_document_error {
    ($name:ident) => {
        impl IntoDocumentError for $name {
            type Residual = $name;

            #[inline(always)]
            fn into_document_error(self) -> DocumentError<Self::Residual> {
                DocumentError {
                    error: self,
                    path: VecDeque::new(),
                }
            }
        }
    };
}

trivial_into_document_error!(IntoIntError);
trivial_into_document_error!(IntoUnsignedIntError);
trivial_into_document_error!(TypeError);

impl IntoDocumentError for Infallible {
    type Residual = Infallible;

    #[inline(always)]
    fn into_document_error(self) -> DocumentError<Self::Residual> {
        match self {}
    }
}

impl<E: IntoDocumentError> IntoDocumentError for TypeErrorOr<E> {
    type Residual = TypeErrorOr<E::Residual>;

    fn into_document_error(self) -> DocumentError<Self::Residual> {
        let (error, path) = match self {
            TypeErrorOr::TypeError(type_error) => (type_error.into(), VecDeque::new()),
            TypeErrorOr::Other(error) => {
                let DocumentError { error, path } = error.into_document_error();
                (TypeErrorOr::Other(error), path)
            }
        };

        DocumentError { error, path }
    }
}

impl<E: IntoDocumentError> IntoDocumentError for DocumentError<E> {
    type Residual = E;

    #[inline(always)]
    fn into_document_error(self) -> DocumentError<Self::Residual> {
        self
    }
}

/// Lifts a [`TypeError`] into the [`TypeErrorOr`] wrapper so that nested errors compose uniformly.
pub trait LiftTypeError {
    /// The remaining error type after extracting any [`TypeError`].
    type Residual;

    /// Lifts this error into a [`TypeErrorOr`].
    fn lift_type_error(self) -> TypeErrorOr<Self::Residual>;
}

macro_rules! trivial_lift_type_error {
    ($name:path) => {
        impl LiftTypeError for $name {
            type Residual = $name;

            #[inline(always)]
            fn lift_type_error(self) -> TypeErrorOr<Self::Residual> {
                TypeErrorOr::Other(self)
            }
        }
    };
}

trivial_lift_type_error!(IntoIntError);
trivial_lift_type_error!(IntoUnsignedIntError);
trivial_lift_type_error!(crate::model::object::InvalidPatchObjectError);

impl LiftTypeError for TypeError {
    type Residual = Infallible;

    #[inline(always)]
    fn lift_type_error(self) -> TypeErrorOr<Self::Residual> {
        self.into()
    }
}

impl<E> LiftTypeError for TypeErrorOr<E> {
    type Residual = E;

    #[inline(always)]
    fn lift_type_error(self) -> TypeErrorOr<Self::Residual> {
        self
    }
}

/// An error annotated with the JSON path at which it occurred.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentError<E> {
    pub(crate) path: VecDeque<PathSegment<Box<str>>>,
    pub(crate) error: E,
}

impl<E> DocumentError<E> {
    /// Creates a `DocumentError` with no path (at the document root).
    pub const fn root(error: E) -> Self {
        DocumentError {
            path: VecDeque::new(),
            error,
        }
    }

    /// Returns the JSON path at which the error occurred.
    pub fn path(&self) -> &VecDeque<PathSegment<Box<str>>> {
        &self.path
    }

    /// Returns a reference to the underlying error.
    pub fn error(&self) -> &E {
        &self.error
    }

    /// Decomposes this into its path and error components.
    pub fn into_parts(self) -> (VecDeque<PathSegment<Box<str>>>, E) {
        (self.path, self.error)
    }
}

impl<E: std::fmt::Display> std::fmt::Display for DocumentError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, segment) in self.path.iter().enumerate() {
            if i > 0 {
                write!(f, "/")?;
            }
            match segment {
                PathSegment::Index(idx) => write!(f, "[{idx}]")?,
                PathSegment::Static(s) => write!(f, "{s}")?,
                PathSegment::String(s) => write!(f, "{s}")?,
            }
        }
        if !self.path.is_empty() {
            write!(f, ": ")?;
        }
        write!(f, "{}", self.error)
    }
}

impl<E: std::fmt::Display + std::fmt::Debug> std::error::Error for DocumentError<E> {}

/// A single segment in a JSON path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathSegment<S> {
    /// An array index.
    Index(usize),
    /// A statically-known object key.
    Static(&'static str),
    /// A dynamically-owned object key.
    String(S),
}

impl<S> PathSegment<S> {
    #[inline(always)]
    fn map<T>(
        self,
        f: impl FnOnce(usize) -> usize,
        g: impl FnOnce(&'static str) -> &'static str,
        h: impl FnOnce(S) -> T,
    ) -> PathSegment<T> {
        match self {
            PathSegment::Index(i) => PathSegment::Index(f(i)),
            PathSegment::Static(s) => PathSegment::Static(g(s)),
            PathSegment::String(s) => PathSegment::String(h(s)),
        }
    }

    /// Borrows the string content of this segment.
    pub fn as_str(&self) -> PathSegment<&str>
    where
        S: AsRef<str>,
    {
        match self {
            PathSegment::Index(i) => PathSegment::Index(*i),
            PathSegment::Static(s) => PathSegment::Static(s),
            PathSegment::String(s) => PathSegment::String(s.as_ref()),
        }
    }
}

impl PathSegment<&str> {
    /// Converts the string content into an owned `Box<str>`.
    pub fn to_box_str(self) -> PathSegment<Box<str>> {
        self.map(|x| x, |x| x, Into::into)
    }
}

/// A signed integer in the inclusive range `[-2^53 + 1, 2^53 - 1]` (RFC 8984 §1.4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Int(i64);

impl Int {
    /// The minimum representable value (`-2^53 + 1`).
    pub const MIN: Self = Int(-(1 << 53) + 1);
    /// The maximum representable value (`2^53 - 1`).
    pub const MAX: Self = Int((1 << 53) - 1);

    /// Creates an `Int` from a raw `i64`, returning `None` if out of range.
    #[inline(always)]
    pub const fn new(value: i64) -> Option<Self> {
        match Self::MIN.get() <= value && value <= Self::MAX.get() {
            true => Some(Self(value)),
            false => None,
        }
    }

    /// Returns the numeric value.
    #[inline(always)]
    pub const fn get(self) -> i64 {
        self.0
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for Int {
    fn into_json(self) -> V {
        V::int(self)
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for bool {
    fn into_json(self) -> V {
        V::bool(self)
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for String {
    fn into_json(self) -> V {
        V::string(self)
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for DateTime<Local> {
    fn into_json(self) -> V {
        V::string(self.to_string())
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for DateTime<Utc> {
    fn into_json(self) -> V {
        V::string(self.to_string())
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for Duration {
    fn into_json(self) -> V {
        V::string(self.to_string())
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for SignedDuration {
    fn into_json(self) -> V {
        V::string(self.to_string())
    }
}

impl<T: fmt::Display, S: fmt::Display, V: ConstructibleJsonValue> IntoJson<V> for Token<T, S> {
    fn into_json(self) -> V {
        V::string(self.to_string())
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for Priority {
    fn into_json(self) -> V {
        V::unsigned_int(UnsignedInt::new(self as u64).unwrap())
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for Percent {
    fn into_json(self) -> V {
        V::unsigned_int(UnsignedInt::new(self.get() as u64).unwrap())
    }
}

impl<T: IntoJson<V>, V: ConstructibleJsonValue> IntoJson<V> for Vec<T> {
    fn into_json(self) -> V {
        let mut arr = V::Array::with_capacity(self.len());
        for item in self {
            arr.push(item.into_json());
        }
        V::array(arr)
    }
}

impl<K: fmt::Display, T: IntoJson<V>, V: ConstructibleJsonValue> IntoJson<V> for HashMap<K, T> {
    fn into_json(self) -> V {
        let mut obj = V::Object::with_capacity(self.len());
        for (key, value) in self {
            obj.insert(key.to_string().into(), value.into_json());
        }
        V::object(obj)
    }
}

impl<T: fmt::Display + Eq + Hash, V: ConstructibleJsonValue> IntoJson<V> for HashSet<T> {
    fn into_json(self) -> V {
        let mut obj = V::Object::with_capacity(self.len());
        for item in self {
            obj.insert(item.to_string().into(), V::bool(true));
        }
        V::object(obj)
    }
}

impl<V: DestructibleJsonValue> TryFromJson<V> for Int {
    type Error = TypeErrorOr<IntoIntError>;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        value.try_as_int()
    }
}

/// An unsigned integer in the inclusive range `[0, 2^53 - 1]` (RFC 8984 §1.4.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct UnsignedInt(u64);

impl UnsignedInt {
    /// The minimum representable value (`0`).
    pub const MIN: Self = UnsignedInt(0);
    /// The maximum representable value (`2^53 - 1`).
    pub const MAX: Self = UnsignedInt((1 << 53) - 1);

    /// Creates an `UnsignedInt` from a raw `u64`, returning `None` if out of range.
    #[inline(always)]
    pub const fn new(value: u64) -> Option<Self> {
        match Self::MIN.get() <= value && value <= Self::MAX.get() {
            true => Some(Self(value)),
            false => None,
        }
    }

    /// Returns the numeric value.
    #[inline(always)]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for UnsignedInt {
    fn into_json(self) -> V {
        V::unsigned_int(self)
    }
}

impl<V: DestructibleJsonValue> TryFromJson<V> for UnsignedInt {
    type Error = TypeErrorOr<IntoUnsignedIntError>;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        value.try_as_unsigned_int()
    }
}

/// The type of a JSON value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValueType {
    /// JSON `null`.
    Null,
    /// JSON boolean (`true` or `false`).
    Bool,
    /// JSON number.
    Number,
    /// JSON string.
    String,
    /// JSON array.
    Array,
    /// JSON object.
    Object,
}

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ValueType::Null => "null",
            ValueType::Bool => "bool",
            ValueType::Number => "number",
            ValueType::String => "string",
            ValueType::Array => "array",
            ValueType::Object => "object",
        };

        write!(f, "{s}")
    }
}

/// Either a JSON type mismatch or a domain-specific error `E`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error(transparent)]
pub enum TypeErrorOr<E> {
    TypeError(#[from] TypeError),
    Other(E),
}

/// The JSON value had the wrong type (e.g. expected a string but received an object).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
#[error("expected a value of type {expected} but received type {received} instead")]
pub struct TypeError {
    pub expected: ValueType,
    pub received: ValueType,
}

/// Error returned when a JSON number cannot be converted to [`Int`].
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Error)]
pub enum IntoIntError {
    #[error("expected an integer but received {0}")]
    NotAnInteger(f64),
    #[error("the signed integer {0} falls outside the valid range for Int")]
    OutsideRangeSigned(i64),
    #[error("the unsigned integer {0} falls outside the valid range for Int")]
    OutsideRangeUnsigned(u64),
}

/// Error returned when a JSON number cannot be converted to [`UnsignedInt`].
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Error)]
pub enum IntoUnsignedIntError {
    #[error("expected an integer but received {0}")]
    NotAnInteger(f64),
    #[error("expected an unsigned integer but received {0}")]
    NegativeInteger(i64),
    #[error("the unsigned integer {0} falls outside the valid range for UnsignedInt")]
    OutsideRange(u64),
}

/// A type representing a JSON value.
pub trait JsonValue {
    /// The string representation used by this JSON value.
    type String: AsRef<str> + Into<String>;
    /// The array representation used by this JSON value.
    type Array: JsonArray<Elem = Self>;
    /// The object representation used by this JSON value.
    type Object: JsonObject<Value = Self>;
}

/// A type representing a JSON value that can be converted into Rust values.
pub trait DestructibleJsonValue: Sized + JsonValue {
    // TYPE CHECKS

    /// Returns the [`ValueType`] of this JSON value.
    fn value_type(&self) -> ValueType;

    #[inline(always)]
    fn is_null(&self) -> bool {
        self.value_type() == ValueType::Null
    }

    #[inline(always)]
    fn is_bool(&self) -> bool {
        self.value_type() == ValueType::Bool
    }

    #[inline(always)]
    fn is_number(&self) -> bool {
        self.value_type() == ValueType::Number
    }

    #[inline(always)]
    fn is_string(&self) -> bool {
        self.value_type() == ValueType::String
    }

    #[inline(always)]
    fn is_array(&self) -> bool {
        self.value_type() == ValueType::Array
    }

    #[inline(always)]
    fn is_object(&self) -> bool {
        self.value_type() == ValueType::Object
    }

    // REFERENTIAL DOWNCASTS

    #[inline(always)]
    fn try_as_null(&self) -> Result<(), TypeError> {
        match self.value_type() {
            ValueType::Null => Ok(()),
            received => Err(TypeError {
                expected: ValueType::Null,
                received,
            }),
        }
    }

    /// Tries to extract a boolean value.
    fn try_as_bool(&self) -> Result<bool, TypeError>;
    /// Tries to extract a floating-point number.
    fn try_as_f64(&self) -> Result<f64, TypeError>;
    /// Tries to extract a signed integer.
    fn try_as_int(&self) -> Result<Int, TypeErrorOr<IntoIntError>>;
    /// Tries to extract an unsigned integer.
    fn try_as_unsigned_int(&self) -> Result<UnsignedInt, TypeErrorOr<IntoUnsignedIntError>>;
    /// Tries to borrow the string value.
    fn try_as_string(&self) -> Result<&Self::String, TypeError>;
    /// Tries to borrow the array value.
    fn try_as_array(&self) -> Result<&Self::Array, TypeError>;
    /// Tries to borrow the object value.
    fn try_as_object(&self) -> Result<&Self::Object, TypeError>;

    // OWNED DOWNCASTS

    /// Tries to consume this value as a string.
    fn try_into_string(self) -> Result<Self::String, TypeError>;
    /// Tries to consume this value as an array.
    fn try_into_array(self) -> Result<Self::Array, TypeError>;
    /// Tries to consume this value as an object.
    fn try_into_object(self) -> Result<Self::Object, TypeError>;
}

/// A type representing a JSON value that can be built from Rust values.
pub trait ConstructibleJsonValue: Sized + JsonValue {
    // CONSTRUCTORS

    /// Creates a JSON `null` value.
    fn null() -> Self;
    /// Creates a JSON boolean value.
    fn bool(value: bool) -> Self;

    /// Creates a JSON string from an owned `String`.
    fn string(value: String) -> Self;
    /// Creates a JSON string from a string slice.
    fn str(value: &str) -> Self;
    /// Creates a JSON string from a `Cow<str>`.
    fn cow_str(value: Cow<'_, str>) -> Self;

    /// Creates a JSON number from an `f64`.
    fn f64(value: f64) -> Self;
    /// Creates a JSON number from an [`Int`].
    fn int(value: Int) -> Self;
    /// Creates a JSON number from an [`UnsignedInt`].
    fn unsigned_int(value: UnsignedInt) -> Self;

    /// Creates a JSON array value.
    fn array(value: Self::Array) -> Self;
    /// Creates a JSON object value.
    fn object(value: Self::Object) -> Self;
}

/// A type which represents a JSON object.
pub trait JsonObject: Sized {
    /// The key type for object entries.
    type Key: Borrow<str> + From<String> + for<'a> From<&'a str>;
    /// The value type for object entries.
    type Value;

    /// Creates an empty object with the given capacity hint.
    fn with_capacity(capacity: usize) -> Self;

    /// Returns a reference to the value associated with `key`, if present.
    fn get<Q>(&self, key: &Q) -> Option<&Self::Value>
    where
        Self::Key: Borrow<Q>,
        Q: ?Sized + Hash + Eq + Ord;

    /// Returns `true` if the object contains an entry for `key`.
    fn contains_key<Q>(&self, key: &Q) -> bool
    where
        Self::Key: Borrow<Q>,
        Q: ?Sized + Hash + Eq + Ord;

    /// Converts an owned key into a `String`.
    fn key_into_string(key: Self::Key) -> String;

    /// Inserts a key-value pair into the object.
    fn insert(&mut self, key: Self::Key, value: Self::Value);

    /// Returns the number of entries in the object.
    fn len(&self) -> usize;
    /// Returns an iterator over key-value pairs by reference.
    fn iter(&self) -> impl Iterator<Item = (&Self::Key, &Self::Value)>;
    /// Consumes the object, returning an iterator over owned key-value pairs.
    fn into_iter(self) -> impl Iterator<Item = (Self::Key, Self::Value)>;

    #[inline(always)]
    fn new() -> Self {
        Self::with_capacity(0)
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline(always)]
    fn keys(&self) -> impl Iterator<Item = &Self::Key> {
        self.iter().map(|(key, _)| key)
    }

    #[inline(always)]
    fn values(&self) -> impl Iterator<Item = &Self::Value> {
        self.iter().map(|(_, value)| value)
    }
}

/// A type which represents a JSON array.
pub trait JsonArray: Sized {
    /// The element type of the array.
    type Elem;

    /// Creates an empty array with the given capacity hint.
    fn with_capacity(capacity: usize) -> Self;
    /// Appends an element to the end of the array.
    fn push(&mut self, elem: Self::Elem);
    /// Returns a reference to the element at `index`, if present.
    fn get(&self, index: usize) -> Option<&Self::Elem>;
    /// Returns the number of elements in the array.
    fn len(&self) -> usize;
    /// Returns an iterator over elements by reference.
    fn iter(&self) -> impl Iterator<Item = &Self::Elem>;
    /// Consumes the array, returning an iterator over owned elements.
    fn into_iter(self) -> impl Iterator<Item = Self::Elem>;

    #[inline(always)]
    fn new() -> Self {
        Self::with_capacity(0)
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<K: Eq + Hash + Into<String> + From<String> + for<'a> From<&'a str> + Borrow<str>, V> JsonObject
    for HashMap<K, V>
{
    type Key = K;
    type Value = V;

    #[inline(always)]
    fn with_capacity(capacity: usize) -> Self {
        HashMap::with_capacity(capacity)
    }

    #[inline(always)]
    fn get<Q>(&self, key: &Q) -> Option<&Self::Value>
    where
        Self::Key: Borrow<Q>,
        Q: ?Sized + Eq + Hash + Ord,
    {
        HashMap::get(self, key)
    }

    #[inline(always)]
    fn contains_key<Q>(&self, key: &Q) -> bool
    where
        Self::Key: Borrow<Q>,
        Q: ?Sized + Hash + Eq + Ord,
    {
        HashMap::contains_key(self, key)
    }

    #[inline(always)]
    fn key_into_string(key: Self::Key) -> String {
        key.into()
    }

    #[inline(always)]
    fn insert(&mut self, key: Self::Key, value: Self::Value) {
        HashMap::insert(self, key, value);
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    fn iter(&self) -> impl Iterator<Item = (&Self::Key, &Self::Value)> {
        HashMap::iter(self)
    }

    #[inline(always)]
    fn into_iter(self) -> impl Iterator<Item = (Self::Key, Self::Value)> {
        IntoIterator::into_iter(self)
    }
}

impl<T> JsonArray for Vec<T> {
    type Elem = T;

    #[inline(always)]
    fn with_capacity(capacity: usize) -> Self {
        Vec::with_capacity(capacity)
    }

    #[inline(always)]
    fn push(&mut self, elem: Self::Elem) {
        Vec::push(self, elem);
    }

    #[inline(always)]
    fn get(&self, index: usize) -> Option<&Self::Elem> {
        self.as_slice().get(index)
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    fn iter(&self) -> impl Iterator<Item = &Self::Elem> {
        self.as_slice().iter()
    }

    #[inline(always)]
    fn into_iter(self) -> impl Iterator<Item = Self::Elem> {
        IntoIterator::into_iter(self)
    }
}

#[cfg(feature = "serde_json")]
mod serde_json_impl {
    use std::{borrow::Cow, hash::Hash};

    use serde_json::{Map, Value};

    use super::{
        ConstructibleJsonValue, DestructibleJsonValue, Int, IntoIntError, IntoUnsignedIntError,
        JsonObject, JsonValue, TypeError, TypeErrorOr, UnsignedInt, ValueType,
    };

    impl JsonValue for Value {
        type String = String;
        type Array = Vec<Value>;
        type Object = Map<String, Value>;
    }

    impl DestructibleJsonValue for Value {
        #[inline(always)]
        fn value_type(&self) -> ValueType {
            match self {
                Value::Null => ValueType::Null,
                Value::Bool(_) => ValueType::Bool,
                Value::Number(_) => ValueType::Number,
                Value::String(_) => ValueType::String,
                Value::Array(_) => ValueType::Array,
                Value::Object(_) => ValueType::Object,
            }
        }

        #[inline(always)]
        fn try_as_bool(&self) -> Result<bool, TypeError> {
            self.as_bool().ok_or_else(|| TypeError {
                expected: ValueType::Bool,
                received: self.value_type(),
            })
        }

        #[inline(always)]
        fn try_as_f64(&self) -> Result<f64, TypeError> {
            self.as_number()
                .and_then(|n| n.as_f64())
                .ok_or_else(|| TypeError {
                    expected: ValueType::Number,
                    received: self.value_type(),
                })
        }

        #[inline(always)]
        fn try_as_string(&self) -> Result<&<Self as JsonValue>::String, TypeError> {
            match self {
                Value::String(s) => Ok(s),
                _ => Err(TypeError {
                    expected: ValueType::String,
                    received: self.value_type(),
                }),
            }
        }

        #[inline(always)]
        fn try_as_array(&self) -> Result<&<Self as JsonValue>::Array, TypeError> {
            self.as_array().ok_or_else(|| TypeError {
                expected: ValueType::Array,
                received: self.value_type(),
            })
        }

        #[inline(always)]
        fn try_as_object(&self) -> Result<&<Self as JsonValue>::Object, TypeError> {
            self.as_object().ok_or_else(|| TypeError {
                expected: ValueType::Object,
                received: self.value_type(),
            })
        }

        #[inline(always)]
        fn try_as_int(&self) -> Result<Int, TypeErrorOr<IntoIntError>> {
            let number = match self {
                Value::Number(number) => Ok(number),
                _ => Err(TypeError {
                    expected: ValueType::Number,
                    received: self.value_type(),
                }),
            }?;

            if let Some(n) = number.as_i64() {
                Int::new(n).ok_or(IntoIntError::OutsideRangeSigned(n))
            } else if let Some(n) = number.as_u64() {
                i64::try_from(n)
                    .ok()
                    .and_then(Int::new)
                    .ok_or(IntoIntError::OutsideRangeUnsigned(n))
            } else if let Some(n) = number.as_f64() {
                Err(IntoIntError::NotAnInteger(n))
            } else {
                unreachable!()
            }
            .map_err(TypeErrorOr::Other)
        }

        #[inline(always)]
        fn try_as_unsigned_int(&self) -> Result<UnsignedInt, TypeErrorOr<IntoUnsignedIntError>> {
            let number = match self {
                Value::Number(number) => Ok(number),
                _ => Err(TypeError {
                    expected: ValueType::Number,
                    received: self.value_type(),
                }),
            }?;

            if let Some(n) = number.as_u64() {
                UnsignedInt::new(n).ok_or(IntoUnsignedIntError::OutsideRange(n))
            } else if let Some(n) = number.as_i64() {
                Err(IntoUnsignedIntError::NegativeInteger(n))
            } else if let Some(n) = number.as_f64() {
                Err(IntoUnsignedIntError::NotAnInteger(n))
            } else {
                unreachable!()
            }
            .map_err(TypeErrorOr::Other)
        }

        #[inline(always)]
        fn try_into_string(self) -> Result<<Self as JsonValue>::String, TypeError> {
            match self {
                Value::String(s) => Ok(s),
                _ => Err(TypeError {
                    expected: ValueType::String,
                    received: self.value_type(),
                }),
            }
        }

        #[inline(always)]
        fn try_into_array(self) -> Result<<Self as JsonValue>::Array, TypeError> {
            match self {
                Value::Array(array) => Ok(array),
                _ => Err(TypeError {
                    expected: ValueType::Array,
                    received: self.value_type(),
                }),
            }
        }

        #[inline(always)]
        fn try_into_object(self) -> Result<<Self as JsonValue>::Object, TypeError> {
            match self {
                Value::Object(object) => Ok(object),
                _ => Err(TypeError {
                    expected: ValueType::Object,
                    received: self.value_type(),
                }),
            }
        }
    }

    impl ConstructibleJsonValue for Value {
        #[inline(always)]
        fn null() -> Self {
            Self::Null
        }

        #[inline(always)]
        fn bool(value: bool) -> Self {
            Self::Bool(value)
        }

        #[inline(always)]
        fn string(value: String) -> Self {
            value.into()
        }

        #[inline(always)]
        fn str(value: &str) -> Self {
            value.into()
        }

        #[inline(always)]
        fn cow_str(value: Cow<'_, str>) -> Self {
            value.into()
        }

        #[inline(always)]
        fn f64(value: f64) -> Self {
            value.into()
        }

        #[inline(always)]
        fn int(value: Int) -> Self {
            value.get().into()
        }

        #[inline(always)]
        fn unsigned_int(value: UnsignedInt) -> Self {
            value.get().into()
        }

        #[inline(always)]
        fn array(value: <Self as JsonValue>::Array) -> Self {
            Value::Array(value)
        }

        #[inline(always)]
        fn object(value: <Self as JsonValue>::Object) -> Self {
            Value::Object(value)
        }
    }

    impl JsonObject for Map<String, Value> {
        type Key = String;
        type Value = Value;

        #[inline(always)]
        fn with_capacity(capacity: usize) -> Self {
            Map::with_capacity(capacity)
        }

        #[inline(always)]
        fn get<Q>(&self, key: &Q) -> Option<&Self::Value>
        where
            Self::Key: std::borrow::Borrow<Q>,
            Q: ?Sized + Hash + Eq + Ord,
        {
            Map::get(self, key)
        }

        #[inline(always)]
        fn contains_key<Q>(&self, key: &Q) -> bool
        where
            Self::Key: std::borrow::Borrow<Q>,
            Q: ?Sized + Hash + Eq + Ord,
        {
            Map::contains_key(self, key)
        }

        #[inline(always)]
        fn key_into_string(key: Self::Key) -> String {
            key
        }

        #[inline(always)]
        fn insert(&mut self, key: Self::Key, value: Self::Value) {
            Map::insert(self, key, value);
        }

        #[inline(always)]
        fn len(&self) -> usize {
            self.len()
        }

        #[inline(always)]
        fn iter(&self) -> impl Iterator<Item = (&Self::Key, &Self::Value)> {
            Map::iter(self)
        }

        #[inline(always)]
        fn into_iter(self) -> impl Iterator<Item = (Self::Key, Self::Value)> {
            IntoIterator::into_iter(self)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "serde_json")]
    #[test]
    fn vec_from_serde_json() {
        use serde_json::json;

        let input = json!([true, true, false, true]);
        assert_eq!(Vec::try_from_json(input), Ok(vec![true, true, false, true]));

        let input = json!([[], [0, 1], [2]]);
        assert_eq!(
            Vec::<Vec<Int>>::try_from_json(input),
            Ok(vec![
                vec![],
                vec![Int::new(0).unwrap(), Int::new(1).unwrap()],
                vec![Int::new(2).unwrap()]
            ])
        );

        let input = json!([true, false, "true", false]);
        assert_eq!(
            Vec::<bool>::try_from_json(input),
            Err(DocumentError {
                path: vec![PathSegment::Index(2)].into(),
                error: TypeErrorOr::TypeError(TypeError {
                    expected: ValueType::Bool,
                    received: ValueType::String
                })
            })
        );

        let input = json!([[], [0, 1], [true]]);
        let res = Vec::<Vec<UnsignedInt>>::try_from_json(input);
        assert_eq!(
            res,
            Err(DocumentError {
                path: vec![PathSegment::Index(2), PathSegment::Index(0)].into(),
                error: TypeErrorOr::TypeError(TypeError {
                    expected: ValueType::Number,
                    received: ValueType::Bool
                })
            })
        );

        // heavily nested to demonstrate that the type system automatically flattens the error type
        let input = json!([[[[[{}]]]]]);
        let res: Result<_, DocumentError<TypeErrorOr<Infallible>>> =
            Vec::<Vec<Vec<Vec<Vec<bool>>>>>::try_from_json(input);

        assert_eq!(
            res,
            Err(DocumentError {
                path: vec![PathSegment::Index(0); 5].into(),
                error: TypeErrorOr::TypeError(TypeError {
                    expected: ValueType::Bool,
                    received: ValueType::Object,
                })
            })
        );
    }

    #[cfg(feature = "serde_json")]
    #[test]
    fn hash_map_from_serde_json() {
        use serde_json::json;

        let input = json!({"a": true, "b": false});
        assert_eq!(
            HashMap::<String, bool>::try_from_json(input),
            Ok({
                let mut map = HashMap::new();
                map.insert("a".into(), true);
                map.insert("b".into(), false);
                map
            })
        );

        let input = json!({"a": {"b": -1}});
        assert_eq!(
            HashMap::<String, HashMap<Box<str>, UnsignedInt>>::try_from_json(input),
            Err(DocumentError {
                path: vec![
                    PathSegment::String("a".into()),
                    PathSegment::String("b".into())
                ]
                .into(),
                error: TypeErrorOr::Other(IntoUnsignedIntError::NegativeInteger(-1)),
            })
        );
    }

    #[cfg(feature = "serde_json")]
    #[test]
    fn hash_set_from_serde_json() {
        use serde_json::json;

        let input = json!({
            "a" : true,
            "a" : true,
            "b" : true,
        });

        assert_eq!(
            HashSet::<String>::try_from_json(input),
            Ok(HashSet::<String>::from(["a".into(), "b".into()]))
        );

        let input = json!({
            "a" : true,
            "b" : false,
        });

        assert!(HashSet::<String>::try_from_json(input).is_err());
    }

    #[cfg(feature = "serde_json")]
    #[test]
    fn utc_date_time_from_serde_json() {
        use serde_json::Value;

        let parse =
            |s| DateTime::<Utc>::try_from_json(serde_json::from_str::<'_, Value>(s).unwrap());

        assert!(parse("\"2025-01-01T12:00:00Z\"").is_ok());
        assert!(parse("\"2025-01-01T12:00:00\"").is_err());
    }

    #[cfg(feature = "serde_json")]
    #[test]
    fn local_date_time_from_serde_json() {
        use serde_json::Value;

        let parse =
            |s| DateTime::<Local>::try_from_json(serde_json::from_str::<'_, Value>(s).unwrap());

        assert!(parse("\"2025-01-01T12:00:00\"").is_ok());
        assert!(parse("\"2025-01-01T12:00:00Z\"").is_err());
    }

    #[cfg(feature = "serde_json")]
    #[test]
    fn duration_from_serde_json() {
        use serde_json::Value;

        let parse = |s| Duration::try_from_json(serde_json::from_str::<'_, Value>(s).unwrap());

        assert!(parse("\"P15DT5H0M20S\"").is_ok());
        assert!(parse("\"P7W\"").is_ok());
        assert_eq!(parse("\"P5W\""), parse("\"P5W0D\""))
    }

    #[cfg(feature = "serde_json")]
    #[test]
    fn signed_duration_from_serde_json() {
        use serde_json::Value;

        let parse =
            |s| SignedDuration::try_from_json(serde_json::from_str::<'_, Value>(s).unwrap());

        assert!(parse("\"-P15DT5H0M20S\"").is_ok());
        assert!(parse("\"+P7W\"").is_ok());
        assert!(parse("\"-P7W\"").is_ok());
        assert!(parse("\"P7W\"").is_ok());
        assert_eq!(parse("\"+P5W\""), parse("\"P5W0D\""))
    }

    #[cfg(feature = "serde_json")]
    #[test]
    #[allow(clippy::approx_constant)]
    fn int_from_serde_json() {
        use serde_json::Value;

        use crate::json::{TypeError, ValueType};

        let parse = |s| Int::try_from_json(serde_json::from_str::<'_, Value>(s).unwrap());

        assert_eq!(parse("0"), Ok(Int::new(0).unwrap()));
        assert_eq!(parse("-9007199254740991"), Ok(Int::MIN));
        assert_eq!(parse("9007199254740991"), Ok(Int::MAX));

        assert_eq!(
            parse("2.718281"),
            Err(TypeErrorOr::Other(IntoIntError::NotAnInteger(2.718281)))
        );

        // Int::MIN - 1
        assert_eq!(
            parse("-9007199254740992"),
            Err(TypeErrorOr::Other(IntoIntError::OutsideRangeSigned(
                -9007199254740992
            )))
        );
        // Int::MAX + 1
        assert_eq!(
            parse("9007199254740992"),
            Err(TypeErrorOr::Other(IntoIntError::OutsideRangeSigned(
                9007199254740992
            )))
        );
        // u64::MAX
        assert_eq!(
            parse("18446744073709551615"),
            Err(TypeErrorOr::Other(IntoIntError::OutsideRangeUnsigned(
                u64::MAX
            )))
        );

        assert_eq!(
            parse("true"),
            Err(TypeError {
                expected: ValueType::Number,
                received: ValueType::Bool
            }
            .into())
        );

        assert_eq!(
            parse("{}"),
            Err(TypeError {
                expected: ValueType::Number,
                received: ValueType::Object
            }
            .into())
        );
    }

    #[cfg(feature = "serde_json")]
    #[test]
    #[allow(clippy::approx_constant)]
    fn unsigned_int_from_serde_json() {
        use serde_json::Value;

        use crate::json::{TypeError, ValueType};

        let parse = |s| UnsignedInt::try_from_json(serde_json::from_str::<'_, Value>(s).unwrap());

        assert_eq!(parse("0"), Ok(UnsignedInt::MIN));
        assert_eq!(parse("9007199254740991"), Ok(UnsignedInt::MAX));

        assert_eq!(
            parse("-1"),
            Err(TypeErrorOr::Other(IntoUnsignedIntError::NegativeInteger(
                -1
            )))
        );
        assert_eq!(
            parse("3.141592"),
            Err(TypeErrorOr::Other(IntoUnsignedIntError::NotAnInteger(
                3.141592
            )))
        );
        // UnsignedInt::MAX + 1
        assert_eq!(
            parse("9007199254740992"),
            Err(TypeErrorOr::Other(IntoUnsignedIntError::OutsideRange(
                9007199254740992
            )))
        );

        assert_eq!(
            parse("false"),
            Err(TypeError {
                expected: ValueType::Number,
                received: ValueType::Bool
            }
            .into())
        );

        assert_eq!(
            parse("[]"),
            Err(TypeError {
                expected: ValueType::Number,
                received: ValueType::Array
            }
            .into())
        );
    }
}
