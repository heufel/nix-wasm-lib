use export_nix::export_nix;
use nix_types::NixValue;
use toml::{Table, Value as Toml};

struct Wrap<T>(T);

impl From<Wrap<Toml>> for NixValue {
    fn from(value: Wrap<Toml>) -> Self {
        match value.0 {
            Toml::Boolean(b) => b.into(),
            Toml::Integer(i) => i.into(),
            Toml::Float(f) => f.into(),
            Toml::String(s) => s.into(),
            Toml::Datetime(t) => t.to_string().into(),
            Toml::Array(arr) => arr.into_iter().map(|v| Wrap(v)).collect(),
            Toml::Table(table) => table
                .iter()
                .map(|(k, v)| (k.clone(), Wrap(v.clone())))
                .collect(),
        }
    }
}

impl From<Wrap<NixValue>> for Toml {
    fn from(value: Wrap<NixValue>) -> Self {
        match value.0 {
            NixValue::Null => panic!("Cannot serialize null to TOML"),
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
                .iter()
                .map(|v| Wrap(v.clone()))
                .collect::<Vec<_>>()
                .into(),
            NixValue::Attrs(attrs) => attrs
                .iter()
                .map(|(k, v)| (k.clone(), Toml::from(Wrap(v.clone()))))
                .collect::<Table>()
                .into(),

            NixValue::Function(_) => panic!("Functions cannot be serialized."),
        }
    }
}

// Convert a NixValue::Attrs into a TOML string.
#[export_nix]
pub fn toTOML(arg: NixValue) -> NixValue {
    let nix = NixValue::from(arg);
    let NixValue::Attrs(_) = nix else {
        panic!("Only Attrsets can be serialized with toTOML. To serialize a single value, use toTOMLValue")
    };
    NixValue::String(toml::to_string_pretty(&Toml::from(Wrap(nix))).unwrap())
}

// Convert a NixValue into a TOML string.
#[export_nix]
pub fn toTOMLValue(arg: NixValue) -> NixValue {
    let nix = NixValue::from(arg);
    let mut out = String::new();
    serde::Serialize::serialize(
        &Toml::from(Wrap(nix)),
        toml::ser::ValueSerializer::new(&mut out),
    )
    .unwrap();
    NixValue::String(out)
}

// Convert a TOML string into a NixValue::Attrset.
#[export_nix]
pub fn fromTOML(arg: NixValue) -> NixValue {
    let NixValue::String(s) = arg else {
        panic!("fromTOML can only be called on a string.")
    };
    let toml = toml::from_str(&s).unwrap();
    NixValue::from(Wrap(toml))
}

// Convert a TOML value into a NixValue.
#[export_nix]
pub fn fromTOMLValue(arg: NixValue) -> NixValue {
    use serde::Deserialize;
    let NixValue::String(s) = arg else {
        panic!("fromTOMLValue can only be called on a string.")
    };
    let toml = Toml::deserialize(toml::de::ValueDeserializer::parse(&s).unwrap()).unwrap();
    NixValue::from(Wrap(toml))
}
