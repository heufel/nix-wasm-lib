use super::{NixValue, Value};
use std::{borrow::Cow, collections::BTreeMap, path::PathBuf};

impl From<()> for NixValue {
    fn from(_: ()) -> Self {
        Self::Null
    }
}

// TODO: this behavior might be counterproductive, may remove
impl<T> From<Option<T>> for NixValue
where
    NixValue: From<T>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            None => Self::Null,
            Some(v) => Self::from(v),
        }
    }
}

impl From<bool> for NixValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

macro_rules! impl_int_to_nix {
    ($($t:ty),*) => {
        $(
        impl From<$t> for NixValue {
            fn from(value: $t) -> Self {
                Self::Int(value.try_into().expect(&format!(
                    "Nix integers must be representable as 64-bit signed integers ({}, {}).",
                    i64::MIN,
                    i64::MAX
                )))
            }
        }
        )*
    };
}
impl_int_to_nix! {
    u8, u16, u32, u64, u128, usize,
    i8, i16, i32, i64, i128, isize
}

macro_rules! impl_float_to_nix {
    ($($t:ty),*) => {
        $(
        impl From<$t> for NixValue {
            fn from(value: $t) -> Self {
                Self::Float(value.try_into().expect(&format!(
                    "Nix floats must be representable as 64-bit floats ({}, {}).",
                    f64::MIN,
                    f64::MAX
                )))
            }
        }
        )*
    };
}

// TODO: add f16 and f128 once they are stable (or if i switch to nightly)
impl_float_to_nix! {f32, f64}

impl From<String> for NixValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for NixValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl<'a> From<Cow<'a, str>> for NixValue {
    fn from(value: Cow<'a, str>) -> Self {
        Self::String(value.into_owned())
    }
}

impl<T> From<(T, PathBuf)> for NixValue
where
    u32: From<T>,
{
    fn from(value: (T, PathBuf)) -> Self {
        Self::Path(Value::from_raw(value.0.into()), value.1)
    }
}

impl<V> FromIterator<V> for NixValue
where
    NixValue: From<V>,
{
    fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> Self {
        Self::List(iter.into_iter().map(NixValue::from).collect())
    }
}

impl<V> From<Vec<V>> for NixValue
where
    NixValue: From<V>,
{
    fn from(value: Vec<V>) -> Self {
        Self::List(value.into_iter().map(NixValue::from).collect())
    }
}
impl<V> From<&[V]> for NixValue
where
    NixValue: From<V>,
    V: Clone,
{
    fn from(value: &[V]) -> Self {
        Self::List(
            value
                .into_iter()
                .map(|v| NixValue::from(v.clone()))
                .collect(),
        )
    }
}

impl<K, V> From<BTreeMap<K, V>> for NixValue
where
    String: From<K>,
    NixValue: From<V>,
{
    fn from(value: BTreeMap<K, V>) -> Self {
        Self::Attrs(
            value
                .into_iter()
                .map(|(k, v)| (String::from(k), NixValue::from(v)))
                .collect(),
        )
    }
}

impl<K, V> FromIterator<(K, V)> for NixValue
where
    String: From<K>,
    NixValue: From<V>,
{
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        Self::Attrs(
            iter.into_iter()
                .map(|(k, v)| (String::from(k), NixValue::from(v)))
                .collect(),
        )
    }
}

impl<K, V> From<Vec<(K, V)>> for NixValue
where
    String: From<K>,
    NixValue: From<V>,
{
    fn from(value: Vec<(K, V)>) -> Self {
        Self::Attrs(
            value
                .into_iter()
                .map(|(k, v)| (String::from(k), NixValue::from(v)))
                .collect(),
        )
    }
}

impl<K, V> From<&[(K, V)]> for NixValue
where
    String: From<K>,
    NixValue: From<V>,
    K: Clone,
    V: Clone,
{
    fn from(value: &[(K, V)]) -> Self {
        Self::Attrs(
            value
                .into_iter()
                .map(|(k, v)| (String::from(k.clone()), NixValue::from(v.clone())))
                .collect(),
        )
    }
}
