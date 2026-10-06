use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<ConfigValue>),
    Object(BTreeMap<String, ConfigValue>),
}

impl ConfigValue {
    pub fn type_name(&self) -> &'static str {
        match self {
            ConfigValue::Null => "null",
            ConfigValue::Bool(_) => "bool",
            ConfigValue::Integer(_) => "integer",
            ConfigValue::Float(_) => "float",
            ConfigValue::String(_) => "string",
            ConfigValue::Array(_) => "array",
            ConfigValue::Object(_) => "object",
        }
    }

    pub fn to_display_string(&self) -> String {
        match self {
            ConfigValue::Null => "null".to_string(),
            ConfigValue::Bool(b) => b.to_string(),
            ConfigValue::Integer(i) => i.to_string(),
            ConfigValue::Float(f) => f.to_string(),
            ConfigValue::String(s) => s.clone(),
            ConfigValue::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.to_display_string()).collect();
                format!("[{}]", items.join(", "))
            }
            ConfigValue::Object(_) => "{...}".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffType {
    Missing,
    Extra,
    TypeMismatch,
    ValueMismatch,
}

impl fmt::Display for DiffType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiffType::Missing => write!(f, "MISSING"),
            DiffType::Extra => write!(f, "EXTRA"),
            DiffType::TypeMismatch => write!(f, "TYPE_MISMATCH"),
            DiffType::ValueMismatch => write!(f, "VALUE_MISMATCH"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffEntry {
    pub path: String,
    pub diff_type: DiffType,
    pub source_val: Option<String>,
    pub target_val: Option<String>,
    pub source_type: Option<String>,
    pub target_type: Option<String>,
    pub is_secret: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffReport {
    pub source_path: String,
    pub target_path: String,
    pub total_source_keys: usize,
    pub total_target_keys: usize,
    pub identical_count: usize,
    pub missing_count: usize,
    pub extra_count: usize,
    pub type_mismatch_count: usize,
    pub value_mismatch_count: usize,
    pub has_diff: bool,
    pub diffs: Vec<DiffEntry>,
}
