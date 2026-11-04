//! Strict structural gate before serde defaults or any migration can erase data.
use crate::config::ConfigError;
use serde::de::{Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Value};
use std::fmt;

struct Unique(Value);
impl<'de> Deserialize<'de> for Unique {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct JsonVisitor;
        impl<'de> Visitor<'de> for JsonVisitor {
            type Value = Unique;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("JSON without duplicate fields")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Unique, A::Error> {
                let mut values = Map::new();
                while let Some((key, value)) = map.next_entry::<String, Unique>()? {
                    if values.insert(key, value.0).is_some() {
                        return Err(serde::de::Error::custom(
                            "Duplicate configuration field; file was not changed",
                        ));
                    }
                }
                Ok(Unique(Value::Object(values)))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Unique, A::Error> {
                let mut values = Vec::new();
                while let Some(value) = seq.next_element::<Unique>()? {
                    values.push(value.0);
                }
                Ok(Unique(Value::Array(values)))
            }
            fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<Unique, E> {
                Ok(Unique(Value::Bool(v)))
            }
            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Unique, E> {
                Ok(Unique(v.into()))
            }
            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Unique, E> {
                Ok(Unique(v.into()))
            }
            fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Unique, E> {
                serde_json::Number::from_f64(v)
                    .map(|v| Unique(Value::Number(v)))
                    .ok_or_else(|| E::custom("Invalid configuration number"))
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Unique, E> {
                Ok(Unique(v.into()))
            }
            fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Unique, E> {
                Ok(Unique(v.into()))
            }
            fn visit_unit<E: serde::de::Error>(self) -> Result<Unique, E> {
                Ok(Unique(Value::Null))
            }
        }
        d.deserialize_any(JsonVisitor)
    }
}
fn invalid() -> ConfigError {
    std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "Unrecognized or incomplete configuration structure; file was not changed",
    )
    .into()
}
fn fields<'a>(v: &'a Value, names: &[&str]) -> Result<&'a Map<String, Value>, ConfigError> {
    let map = v.as_object().ok_or_else(invalid)?;
    if map.keys().any(|key| !names.contains(&key.as_str())) {
        return Err(invalid());
    }
    Ok(map)
}
fn array(v: Option<&Value>) -> Result<&[Value], ConfigError> {
    v.and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(invalid)
}
const STATION: &[&str] = &["name", "url", "eq", "metadataType", "metadataUrl"];
pub(crate) fn parse(raw: &str) -> Result<Value, ConfigError> {
    let value = serde_json::from_str::<Unique>(raw)?.0;
    let root = fields(
        &value,
        &[
            "currentUrl",
            "volume",
            "muted",
            "lastPreset",
            "lastBank",
            "presets",
            "stationBanks",
            "windowW",
            "windowH",
            "windowX",
            "windowY",
            "windowPosValid",
            "apiEndpoint",
            "customEqPresets",
            "jsonPollIntervalSec",
        ],
    )?;
    match (root.get("presets"), root.get("stationBanks")) {
        (Some(presets), None) => {
            for p in array(Some(presets))? {
                fields(p, STATION)?;
            }
        }
        (None, Some(banks)) => {
            for bank in array(Some(banks))? {
                let b = fields(bank, &["bank", "slots"])?;
                for slot in array(b.get("slots"))? {
                    fields(
                        slot,
                        &["key", "name", "url", "eq", "metadataType", "metadataUrl"],
                    )?;
                }
            }
        }
        _ => return Err(invalid()),
    }
    for eq in array(root.get("customEqPresets"))? {
        fields(eq, &["name", "preamp", "bands"])?;
    }
    Ok(value)
}
