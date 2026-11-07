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
use crate::config_issue::{structure, FieldPath as P};
fn fields<'a>(
    v: &'a Value,
    names: &[&str],
    path: P,
) -> Result<&'a Map<String, Value>, ConfigError> {
    let map = v
        .as_object()
        .ok_or_else(|| structure(path, "Expected a JSON object."))?;
    if map.keys().any(|key| !names.contains(&key.as_str())) {
        return Err(structure(
            path,
            "Object contains an unrecognized field; input field names/values are not displayed.",
        ));
    }
    Ok(map)
}
fn array(v: Option<&Value>, path: P) -> Result<&[Value], ConfigError> {
    v.and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| structure(path, "Expected a JSON array."))
}
fn string(v: Option<&Value>, path: P) -> Result<(), ConfigError> {
    if v.is_some_and(Value::is_string) {
        Ok(())
    } else {
        Err(structure(
            path,
            "Expected a string; the field is missing or has the wrong type.",
        ))
    }
}
fn unsigned(v: Option<&Value>, max: u64, path: P) -> Result<(), ConfigError> {
    if v.and_then(Value::as_u64).is_some_and(|n| n <= max) {
        Ok(())
    } else {
        Err(structure(
            path,
            "Expected a nonnegative integer within the supported storage type.",
        ))
    }
}
fn boolean(v: Option<&Value>, path: P) -> Result<(), ConfigError> {
    if v.is_some_and(Value::is_boolean) {
        Ok(())
    } else {
        Err(structure(
            path,
            "Expected a boolean; the field is missing or has the wrong type.",
        ))
    }
}
const STATION: &[&str] = &["name", "url", "eq", "metadataType", "metadataUrl"];
const ROOT: &[&str] = &[
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
];
pub(crate) fn parse(raw: &str) -> Result<Value, ConfigError> {
    let value = serde_json::from_str::<Unique>(raw)
        .map_err(|error| {
            let reason = if error
                .to_string()
                .starts_with("Duplicate configuration field")
            {
                "Duplicate JSON field; neither occurrence was chosen."
            } else {
                "Malformed JSON or unsupported JSON number; input contents are not displayed."
            };
            structure(
                P::Json {
                    line: error.line(),
                    column: error.column(),
                },
                reason,
            )
        })?
        .0;
    let root = fields(&value, ROOT, P::Root("$"))?;
    for name in ["currentUrl", "apiEndpoint"] {
        string(root.get(name), P::Root(name))?;
    }
    for name in ["muted", "windowPosValid"] {
        boolean(root.get(name), P::Root(name))?;
    }
    unsigned(root.get("volume"), u8::MAX as u64, P::Root("volume"))?;
    unsigned(
        root.get("lastPreset"),
        usize::MAX as u64,
        P::Root("lastPreset"),
    )?;
    if root.contains_key("lastBank") {
        unsigned(root.get("lastBank"), usize::MAX as u64, P::Root("lastBank"))?;
    }
    for name in ["windowW", "windowH"] {
        unsigned(root.get(name), u32::MAX as u64, P::Root(name))?;
    }
    for name in ["windowX", "windowY"] {
        if root
            .get(name)
            .and_then(Value::as_i64)
            .is_none_or(|n| i32::try_from(n).is_err())
        {
            return Err(structure(
                P::Root(name),
                "Expected a signed 32-bit integer.",
            ));
        }
    }
    if let Some(v) = root.get("jsonPollIntervalSec") {
        if !v.is_null() {
            unsigned(Some(v), u64::MAX, P::Root("jsonPollIntervalSec"))?;
        }
    }
    match (root.get("presets"), root.get("stationBanks")) {
        (Some(presets), None) => {
            for (index, p) in array(Some(presets), P::Root("presets"))?.iter().enumerate() {
                let item = fields(
                    p,
                    STATION,
                    P::Preset {
                        index,
                        field: "entry",
                    },
                )?;
                for &field in STATION {
                    string(item.get(field), P::Preset { index, field })?;
                }
            }
        }
        (None, Some(banks)) => {
            for (index, bank) in array(Some(banks), P::Root("stationBanks"))?
                .iter()
                .enumerate()
            {
                let item = fields(
                    bank,
                    &["bank", "slots"],
                    P::Bank {
                        index,
                        field: "entry",
                    },
                )?;
                string(
                    item.get("bank"),
                    P::Bank {
                        index,
                        field: "bank",
                    },
                )?;
                // Slint deserialization defaulted missing slots, but its serializer always
                // wrote them. Missing data is ambiguous; never infer an empty bank.
                if let Some(slots) = item.get("slots") {
                    for (slot, v) in array(
                        Some(slots),
                        P::Bank {
                            index,
                            field: "slots",
                        },
                    )?
                    .iter()
                    .enumerate()
                    {
                        let item = fields(
                            v,
                            &["key", "name", "url", "eq", "metadataType", "metadataUrl"],
                            P::Slot {
                                bank: index,
                                slot,
                                field: "entry",
                            },
                        )?;
                        for field in ["key", "name", "url", "eq", "metadataType", "metadataUrl"] {
                            string(
                                item.get(field),
                                P::Slot {
                                    bank: index,
                                    slot,
                                    field,
                                },
                            )?;
                        }
                    }
                } else {
                    return Err(structure(
                        P::Bank {
                            index,
                            field: "slots",
                        },
                        "Required slots array is missing; the bank was not assumed empty.",
                    ));
                }
            }
        }
        _ => {
            return Err(structure(
                P::Root("presets / stationBanks"),
                "Expected exactly one station-storage field: presets or stationBanks.",
            ))
        }
    }
    for (index, eq) in array(root.get("customEqPresets"), P::Root("customEqPresets"))?
        .iter()
        .enumerate()
    {
        let item = fields(
            eq,
            &["name", "preamp", "bands"],
            P::CustomEq {
                index,
                field: "entry",
            },
        )?;
        string(
            item.get("name"),
            P::CustomEq {
                index,
                field: "name",
            },
        )?;
        if !item.get("preamp").is_some_and(Value::is_number) {
            return Err(structure(
                P::CustomEq {
                    index,
                    field: "preamp",
                },
                "Expected a numeric preamp gain.",
            ));
        }
        let bands = array(
            item.get("bands"),
            P::CustomEq {
                index,
                field: "bands",
            },
        )?;
        if bands.len() != 10 {
            return Err(structure(
                P::CustomEq {
                    index,
                    field: "bands",
                },
                "Expected exactly 10 numeric frequency gains; none were added or dropped.",
            ));
        }
        for (band, gain) in bands.iter().enumerate() {
            if !gain.is_number() {
                return Err(structure(
                    P::EqBand { index, band },
                    "Expected a numeric frequency gain.",
                ));
            }
        }
    }
    Ok(value)
}
