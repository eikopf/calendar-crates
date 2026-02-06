//! Tools for interacting with types representing JSON values.

use std::{
    borrow::{Borrow, Cow},
    collections::{HashMap, VecDeque},
    convert::Infallible,
    hash::Hash,
};

use thiserror::Error;

use crate::model::primitive::{Int, UnsignedInt};

pub trait TryFromJson: Sized {
    type Error;

    fn try_from_json<V: DestructibleJsonValue>(value: V) -> Result<Self, Self::Error>;
}

impl TryFromJson for bool {
    type Error = TypeError;

    fn try_from_json<V: DestructibleJsonValue>(value: V) -> Result<Self, Self::Error> {
        value.try_as_bool()
    }
}

impl TryFromJson for String {
    type Error = TypeError;

    fn try_from_json<V: DestructibleJsonValue>(value: V) -> Result<Self, Self::Error> {
        value.try_into_string().map(Into::into)
    }
}

impl<T> TryFromJson for Vec<T>
where
    T: TryFromJson,
    T::Error: SplitPath,
    <T::Error as SplitPath>::Residual: SplitTypeError,
{
    type Error =
        DocumentError<TypeErrorOr<<<T::Error as SplitPath>::Residual as SplitTypeError>::Residual>>;

    fn try_from_json<V: DestructibleJsonValue>(value: V) -> Result<Self, Self::Error> {
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
                    let (error, mut path) = error.split_path();
                    path.push_front(PathSegment::Index(i));
                    let error = error.split_type_error();
                    DocumentError { path, error }
                })
            })
            .collect::<Result<Vec<_>, _>>()
    }
}

// TODO: write TryFromJson impls for HashMap<K, V> and a wrapper over HashSet dealing with the
// specific way JSCalendar encodes sets (i.e. as objects with boolean keys)

pub trait TryIntoJson {
    type Error;

    fn try_into_json<V: ConstructibleJsonValue>(self) -> Result<V, Self::Error>;
}

pub trait IntoJson {
    fn into_json<V: ConstructibleJsonValue>(self) -> V;
}

impl<T: IntoJson> TryIntoJson for T {
    type Error = std::convert::Infallible;

    fn try_into_json<V: ConstructibleJsonValue>(self) -> Result<V, Self::Error> {
        Ok(self.into_json())
    }
}

pub trait SplitPath: Sized {
    type Residual;

    fn split_path(self) -> (Self::Residual, VecDeque<PathSegment<Box<str>>>);
}

macro_rules! trivial_split_path {
    ($name:ident) => {
        impl SplitPath for $name {
            type Residual = $name;

            #[inline(always)]
            fn split_path(self) -> (Self::Residual, VecDeque<PathSegment<Box<str>>>) {
                (self, VecDeque::new())
            }
        }
    };
}

trivial_split_path!(IntoIntError);
trivial_split_path!(IntoUnsignedIntError);
trivial_split_path!(TypeError);

impl SplitPath for Infallible {
    type Residual = Infallible;

    #[inline(always)]
    fn split_path(self) -> (Self::Residual, VecDeque<PathSegment<Box<str>>>) {
        match self {}
    }
}

impl<E: SplitPath> SplitPath for TypeErrorOr<E> {
    type Residual = TypeErrorOr<E::Residual>;

    fn split_path(self) -> (Self::Residual, VecDeque<PathSegment<Box<str>>>) {
        match self {
            TypeErrorOr::TypeError(type_error) => (type_error.into(), VecDeque::new()),
            TypeErrorOr::Other(error) => {
                let (error, path) = error.split_path();
                (TypeErrorOr::Other(error), path)
            }
        }
    }
}

impl<E: SplitPath> SplitPath for DocumentError<E> {
    type Residual = E;

    #[inline(always)]
    fn split_path(self) -> (Self::Residual, VecDeque<PathSegment<Box<str>>>) {
        (self.error, self.path)
    }
}

pub trait SplitTypeError {
    type Residual;

    fn split_type_error(self) -> TypeErrorOr<Self::Residual>;
}

macro_rules! trivial_split_type_error {
    ($name:ident) => {
        impl SplitTypeError for $name {
            type Residual = $name;

            #[inline(always)]
            fn split_type_error(self) -> TypeErrorOr<Self::Residual> {
                TypeErrorOr::Other(self)
            }
        }
    };
}

trivial_split_type_error!(IntoIntError);
trivial_split_type_error!(IntoUnsignedIntError);

impl SplitTypeError for TypeError {
    type Residual = Infallible;

    #[inline(always)]
    fn split_type_error(self) -> TypeErrorOr<Self::Residual> {
        self.into()
    }
}

impl<E> SplitTypeError for TypeErrorOr<E> {
    type Residual = E;

    #[inline(always)]
    fn split_type_error(self) -> TypeErrorOr<Self::Residual> {
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentError<E> {
    path: VecDeque<PathSegment<Box<str>>>,
    error: E,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathSegment<S> {
    Index(usize),
    Static(&'static str),
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
    pub fn to_box_str(self) -> PathSegment<Box<str>> {
        self.map(|x| x, |x| x, Into::into)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValueType {
    Null,
    Bool,
    Number,
    String,
    Array,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error(transparent)]
pub enum TypeErrorOr<E> {
    TypeError(#[from] TypeError),
    Other(E),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
#[error("expected a value of type {expected} but received type {received} instead")]
pub struct TypeError {
    pub expected: ValueType,
    pub received: ValueType,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Error)]
pub enum IntoIntError {
    #[error("expected an integer but received {0}")]
    NotAnInteger(f64),
    #[error("the signed integer {0} falls outside the valid range for Int")]
    OutsideRangeSigned(i64),
    #[error("the unsigned integer {0} falls outside the valid range for Int")]
    OutsideRangeUnsigned(u64),
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Error)]
pub enum IntoUnsignedIntError {
    #[error("expected an integer but received {0}")]
    NotAnInteger(f64),
    #[error("expected an unsigned integer but received {0}")]
    NegativeInteger(i64),
    #[error("the unsigned integer {0} falls outside the valid range for UnsignedInt")]
    OutsideRange(u64),
}

/// A type representing a JSON value that can be converted into Rust values.
pub trait DestructibleJsonValue: Sized {
    type String: AsRef<str> + Into<String>;
    type Array: JsonArray<Elem = Self>;
    type Object: JsonObject<Value = Self>;

    // TYPE CHECKS

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

    fn try_as_bool(&self) -> Result<bool, TypeError>;
    fn try_as_f64(&self) -> Result<f64, TypeError>;
    fn try_as_int(&self) -> Result<Int, TypeErrorOr<IntoIntError>>;
    fn try_as_unsigned_int(&self) -> Result<UnsignedInt, TypeErrorOr<IntoUnsignedIntError>>;
    fn try_as_string(&self) -> Result<&Self::String, TypeError>;
    fn try_as_array(&self) -> Result<&Self::Array, TypeError>;
    fn try_as_object(&self) -> Result<&Self::Object, TypeError>;

    // OWNED DOWNCASTS

    fn try_into_string(self) -> Result<Self::String, TypeError>;
    fn try_into_array(self) -> Result<Self::Array, TypeError>;
    fn try_into_object(self) -> Result<Self::Object, TypeError>;
}

/// A type representing a JSON value that can be built from Rust values.
pub trait ConstructibleJsonValue: Sized {
    type Array: JsonArray;
    type Object: JsonObject;

    // CONSTRUCTORS

    fn null() -> Self;
    fn bool(value: bool) -> Self;

    fn string(value: String) -> Self;
    fn str(value: &str) -> Self;
    fn cow_str(value: Cow<'_, str>) -> Self;

    fn f64(value: f64) -> Self;
    fn int(value: Int) -> Self;
    fn unsigned_int(value: UnsignedInt) -> Self;

    fn array(value: Self::Array) -> Self;
    fn object(value: Self::Object) -> Self;
}

/// A type which represents a JSON object.
pub trait JsonObject: Sized {
    type Key;
    type Value;

    fn with_capacity(capacity: usize) -> Self;

    fn get<Q>(&self, key: &Q) -> Option<&Self::Value>
    where
        Self::Key: Borrow<Q>,
        Q: ?Sized + Hash + Eq + Ord;

    fn contains_key<Q>(&self, key: &Q) -> bool
    where
        Self::Key: Borrow<Q>,
        Q: ?Sized + Hash + Eq + Ord;

    fn len(&self) -> usize;
    fn iter(&self) -> impl Iterator<Item = (&Self::Key, &Self::Value)>;
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
    type Elem;

    fn with_capacity(capacity: usize) -> Self;
    fn get(&self, index: usize) -> Option<&Self::Elem>;
    fn len(&self) -> usize;
    fn iter(&self) -> impl Iterator<Item = &Self::Elem>;
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

impl<K: Eq + Hash, V> JsonObject for HashMap<K, V> {
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

    use crate::{
        json::TypeErrorOr,
        model::primitive::{Int, UnsignedInt},
    };

    use super::{
        ConstructibleJsonValue, DestructibleJsonValue, IntoIntError, IntoUnsignedIntError,
        JsonObject, TypeError, ValueType,
    };

    impl DestructibleJsonValue for Value {
        type String = String;
        type Array = Vec<Value>;
        type Object = Map<String, Value>;

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
        fn try_as_string(&self) -> Result<&<Self as DestructibleJsonValue>::String, TypeError> {
            match self {
                Value::String(s) => Ok(s),
                _ => Err(TypeError {
                    expected: ValueType::String,
                    received: self.value_type(),
                }),
            }
        }

        #[inline(always)]
        fn try_as_array(&self) -> Result<&<Self as DestructibleJsonValue>::Array, TypeError> {
            self.as_array().ok_or_else(|| TypeError {
                expected: ValueType::Array,
                received: self.value_type(),
            })
        }

        #[inline(always)]
        fn try_as_object(&self) -> Result<&<Self as DestructibleJsonValue>::Object, TypeError> {
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
        fn try_into_string(self) -> Result<<Self as DestructibleJsonValue>::String, TypeError> {
            match self {
                Value::String(s) => Ok(s),
                _ => Err(TypeError {
                    expected: ValueType::String,
                    received: self.value_type(),
                }),
            }
        }

        #[inline(always)]
        fn try_into_array(self) -> Result<<Self as DestructibleJsonValue>::Array, TypeError> {
            match self {
                Value::Array(array) => Ok(array),
                _ => Err(TypeError {
                    expected: ValueType::Array,
                    received: self.value_type(),
                }),
            }
        }

        #[inline(always)]
        fn try_into_object(self) -> Result<<Self as DestructibleJsonValue>::Object, TypeError> {
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
        type Array = Vec<Self>;
        type Object = Map<String, Self>;

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
        fn array(value: <Self as ConstructibleJsonValue>::Array) -> Self {
            Value::Array(value)
        }

        #[inline(always)]
        fn object(value: <Self as ConstructibleJsonValue>::Object) -> Self {
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
}
