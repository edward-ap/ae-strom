//! Sanitized configuration locations/reasons. Never retain input values, URLs or names.
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldPath {
    Json {
        line: usize,
        column: usize,
    },
    Root(&'static str),
    Preset {
        index: usize,
        field: &'static str,
    },
    Bank {
        index: usize,
        field: &'static str,
    },
    Slot {
        bank: usize,
        slot: usize,
        field: &'static str,
    },
    CustomEq {
        index: usize,
        field: &'static str,
    },
    EqBand {
        index: usize,
        band: usize,
    },
}
impl fmt::Display for FieldPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json { line, column } => write!(f, "JSON line {line}, column {column}"),
            Self::Root(field) => f.write_str(field),
            Self::Preset { index, field } => write!(f, "presets[{index}].{field}"),
            Self::Bank { index, field } => write!(f, "stationBanks[{index}].{field}"),
            Self::Slot { bank, slot, field } => {
                write!(f, "stationBanks[{bank}].slots[{slot}].{field}")
            }
            Self::CustomEq { index, field } => write!(f, "customEqPresets[{index}].{field}"),
            Self::EqBand { index, band } => write!(f, "customEqPresets[{index}].bands[{band}]"),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigIssue {
    pub path: FieldPath,
    pub reason: &'static str,
    pub structure: bool,
}
impl fmt::Display for ConfigIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Field: {}\nReason: {}", self.path, self.reason)
    }
}
impl std::error::Error for ConfigIssue {}

pub(crate) fn value(path: FieldPath, reason: &'static str) -> crate::config::ConfigError {
    crate::config::ConfigError::Issue(ConfigIssue {
        path,
        reason,
        structure: false,
    })
}
pub(crate) fn structure(path: FieldPath, reason: &'static str) -> crate::config::ConfigError {
    crate::config::ConfigError::Issue(ConfigIssue {
        path,
        reason,
        structure: true,
    })
}
