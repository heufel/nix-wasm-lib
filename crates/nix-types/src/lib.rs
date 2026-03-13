pub use nix_wasm_rust::warn;
use std::{collections::BTreeMap, path::PathBuf};
pub mod compat {
    pub use nix_wasm_rust::{Type, Value};
}

/// These implementations are largely based on that of serde_json.
mod from;

use compat::{Type, Value};

#[derive(Clone, Debug)]
pub enum NixValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Path(Value, PathBuf),
    Attrs(BTreeMap<String, NixValue>),
    List(Vec<NixValue>),
    Function(Value),
}

impl From<Value> for NixValue {
    fn from(value: Value) -> Self {
        let ty = value.get_type();
        match ty {
            Type::Null => NixValue::Null,
            Type::Bool => NixValue::Bool(value.get_bool()),
            Type::Int => NixValue::Int(value.get_int()),
            Type::Float => NixValue::Float(value.get_float()),
            Type::String => NixValue::String(value.get_string()),
            Type::Path => NixValue::Path(value, value.get_path()),
            Type::List => NixValue::List(value.get_list().iter().map(|&v| v.into()).collect()),
            Type::Attrs => NixValue::Attrs(BTreeMap::from_iter(
                value
                    .get_attrset()
                    .iter()
                    .map(|(k, v)| (k.clone(), NixValue::from(v.clone()))),
            )),
            Type::Function => {
                // warn!("Avoid sending functions to WASM. Function ID: {:?}", value);
                NixValue::Function(value)
            }
        }
    }
}

impl From<NixValue> for Value {
    fn from(value: NixValue) -> Self {
        fn from_ref(v: &NixValue) -> Value {
            match v {
                NixValue::Null => Value::make_null(),
                NixValue::Bool(b) => Value::make_bool(*b),
                NixValue::Int(i) => Value::make_int(*i),
                NixValue::Float(f) => Value::make_float(*f),
                NixValue::String(s) => Value::make_string(s),
                NixValue::Path(val, p) => {
                    Value::make_path(&val, p.to_str().expect("Nix path should be UTF-8."))
                }
                NixValue::List(list) => {
                    Value::make_list(&list.into_iter().map(from_ref).collect::<Vec<_>>())
                }
                NixValue::Attrs(attrs) => Value::make_attrset(
                    attrs
                        .into_iter()
                        .map(|(k, v)| (k.as_str(), from_ref(v)))
                        .collect::<Vec<_>>()
                        .as_slice(),
                ),
                NixValue::Function(val) => *val,
            }
        }
        from_ref(&value)
    }
}
