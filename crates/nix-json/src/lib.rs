use export_nix::export_nix;
use nix_types::NixValue;

use serde_json::Value as Json;

struct Wrap<T>(T);

impl From<Wrap<Json>> for NixValue {
    fn from(value: Wrap<Json>) -> Self {
        match value.0 {
            Json::Null => Self::Null,
            Json::Bool(b) => b.into(),
            Json::Number(n) => {
                if n.is_i64() {
                    n.as_i64().unwrap().into()
                } else {
                    n.as_f64().unwrap().into()
                }
            }
            Json::String(s) => s.into(),
            Json::Array(arr) => arr.into_iter().map(|v| Wrap(v)).collect(),

            Json::Object(obj) => obj.into_iter().map(|(k, v)| (k, Wrap(v))).collect(),
        }
    }
}

impl From<Wrap<NixValue>> for Json {
    fn from(value: Wrap<NixValue>) -> Self {
        match value.0 {
            NixValue::Null => Json::Null,
            NixValue::Bool(b) => b.into(),
            NixValue::Int(i) => i.into(),
            NixValue::Float(f) => f.into(),
            NixValue::String(s) => s.into(),
            NixValue::Path(_, p) => p
                .to_str()
                .expect("NixValue path should be UTF-8.")
                .to_string()
                .into(),
            NixValue::List(list) => list
                .into_iter()
                .map(|v| Json::from(Wrap(v)))
                .collect::<Vec<_>>()
                .into(),
            NixValue::Attrs(attrs) => attrs
                .into_iter()
                .map(|(k, v)| (k, Json::from(Wrap(v))))
                .collect(),
            NixValue::Function(_) => panic!("Functions cannot be serialized."),
        }
    }
}

// Convert a NixValue value into a JSON string.
#[export_nix]
pub fn toJSON(arg: NixValue) -> NixValue {
    let nix = NixValue::from(arg);
    NixValue::String(serde_json::to_string_pretty(&Json::from(Wrap(nix))).unwrap())
}

// Convert a JSON string into a NixValue.
#[export_nix]
pub fn fromJSON(arg: NixValue) -> NixValue {
    let NixValue::String(s) = arg else {
        panic!("fromJSON can only be called on a string.")
    };
    let json = serde_json::from_str(&s).unwrap();
    NixValue::from(Wrap(json))
}

// Convert a NixValue value into a JSON5 string.
#[export_nix]
pub fn toJSON5(arg: NixValue) -> NixValue {
    let nix = NixValue::from(arg);
    NixValue::String(json5::to_string(&Json::from(Wrap(nix))).unwrap())
}

// Convert a JSON5 string into a NixValue.
#[export_nix]
pub fn fromJSON5(arg: NixValue) -> NixValue {
    let NixValue::String(s) = arg else {
        panic!("fromJSON can only be called on a string.")
    };
    let json = json5::from_str(&s).unwrap();
    NixValue::from(Wrap(json))
}
