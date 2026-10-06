use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Read};
use std::path::Path;

use crate::model::ConfigValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    Env,
    Json,
    Yaml,
    Toml,
}

pub fn detect_format(path: &str, content: &str) -> FileFormat {
    let lower_path = path.to_lowercase();
    if lower_path == "-" {
        // Inspect content
        let trimmed = content.trim();
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            return FileFormat::Json;
        }
        if trimmed
            .lines()
            .any(|l| l.contains('=') && !l.starts_with('['))
        {
            return FileFormat::Env;
        }
        return FileFormat::Yaml;
    }

    if lower_path.ends_with(".env")
        || lower_path.contains(".env.")
        || lower_path.starts_with(".env")
    {
        FileFormat::Env
    } else if lower_path.ends_with(".json") {
        FileFormat::Json
    } else if lower_path.ends_with(".yaml") || lower_path.ends_with(".yml") {
        FileFormat::Yaml
    } else if lower_path.ends_with(".toml") {
        FileFormat::Toml
    } else {
        // Fallback heuristic based on content
        let trimmed = content.trim();
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            FileFormat::Json
        } else if trimmed.lines().any(|l| l.contains('=')) {
            FileFormat::Env
        } else {
            FileFormat::Yaml
        }
    }
}

pub fn load_config_file(path: &str) -> Result<BTreeMap<String, ConfigValue>> {
    let content = if path == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        fs::read_to_string(Path::new(path))
            .map_err(|e| anyhow!("Failed to read file '{}': {}", path, e))?
    };

    let format = detect_format(path, &content);
    parse_config_content(&content, format)
}

pub fn parse_config_content(
    content: &str,
    format: FileFormat,
) -> Result<BTreeMap<String, ConfigValue>> {
    match format {
        FileFormat::Env => parse_env(content),
        FileFormat::Json => parse_json(content),
        FileFormat::Yaml => parse_yaml(content),
        FileFormat::Toml => parse_toml(content),
    }
}

fn parse_env(content: &str) -> Result<BTreeMap<String, ConfigValue>> {
    let mut map = BTreeMap::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let clean_line = if let Some(stripped) = trimmed.strip_prefix("export ") {
            stripped.trim()
        } else {
            trimmed
        };

        if let Some((raw_key, raw_val)) = clean_line.split_once('=') {
            let key = raw_key.trim().to_string();
            let val_str = raw_val.trim();

            let unquoted_val = if (val_str.starts_with('"') && val_str.ends_with('"'))
                || (val_str.starts_with('\'') && val_str.ends_with('\''))
            {
                if val_str.len() >= 2 {
                    &val_str[1..val_str.len() - 1]
                } else {
                    val_str
                }
            } else {
                // If unquoted, strip trailing comments like `KEY=val # comment`
                if let Some((v, _)) = val_str.split_once('#') {
                    v.trim()
                } else {
                    val_str
                }
            };

            let config_val = infer_env_value(unquoted_val);
            map.insert(key, config_val);
        }
    }

    Ok(map)
}

fn infer_env_value(val: &str) -> ConfigValue {
    let lower = val.to_ascii_lowercase();
    if lower == "true" {
        ConfigValue::Bool(true)
    } else if lower == "false" {
        ConfigValue::Bool(false)
    } else if lower == "null" || lower == "none" || val.is_empty() {
        ConfigValue::Null
    } else if let Ok(i) = val.parse::<i64>() {
        ConfigValue::Integer(i)
    } else if let Ok(f) = val.parse::<f64>() {
        ConfigValue::Float(f)
    } else {
        ConfigValue::String(val.to_string())
    }
}

fn parse_json(content: &str) -> Result<BTreeMap<String, ConfigValue>> {
    let json_val: serde_json::Value =
        serde_json::from_str(content).map_err(|e| anyhow!("Failed to parse JSON: {}", e))?;

    let mut map = BTreeMap::new();
    flatten_json_value("", &json_val, &mut map);
    Ok(map)
}

fn flatten_json_value(
    prefix: &str,
    val: &serde_json::Value,
    map: &mut BTreeMap<String, ConfigValue>,
) {
    match val {
        serde_json::Value::Object(obj) => {
            for (k, v) in obj {
                let full_key = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{}.{}", prefix, k)
                };
                flatten_json_value(&full_key, v, map);
            }
        }
        serde_json::Value::Array(arr) => {
            let converted: Vec<ConfigValue> = arr.iter().map(json_to_config_value).collect();
            map.insert(prefix.to_string(), ConfigValue::Array(converted));
        }
        _ => {
            map.insert(prefix.to_string(), json_to_config_value(val));
        }
    }
}

fn json_to_config_value(v: &serde_json::Value) -> ConfigValue {
    match v {
        serde_json::Value::Null => ConfigValue::Null,
        serde_json::Value::Bool(b) => ConfigValue::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                ConfigValue::Integer(i)
            } else if let Some(f) = n.as_f64() {
                ConfigValue::Float(f)
            } else {
                ConfigValue::String(n.to_string())
            }
        }
        serde_json::Value::String(s) => ConfigValue::String(s.clone()),
        serde_json::Value::Array(arr) => {
            ConfigValue::Array(arr.iter().map(json_to_config_value).collect())
        }
        serde_json::Value::Object(obj) => {
            let mut tree = BTreeMap::new();
            for (k, val) in obj {
                tree.insert(k.clone(), json_to_config_value(val));
            }
            ConfigValue::Object(tree)
        }
    }
}

fn parse_yaml(content: &str) -> Result<BTreeMap<String, ConfigValue>> {
    let yaml_val: serde_yaml::Value =
        serde_yaml::from_str(content).map_err(|e| anyhow!("Failed to parse YAML: {}", e))?;

    let mut map = BTreeMap::new();
    flatten_yaml_value("", &yaml_val, &mut map);
    Ok(map)
}

fn flatten_yaml_value(
    prefix: &str,
    val: &serde_yaml::Value,
    map: &mut BTreeMap<String, ConfigValue>,
) {
    match val {
        serde_yaml::Value::Mapping(m) => {
            for (k, v) in m {
                let key_str = match k {
                    serde_yaml::Value::String(s) => s.clone(),
                    _ => format!("{:?}", k),
                };
                let full_key = if prefix.is_empty() {
                    key_str
                } else {
                    format!("{}.{}", prefix, key_str)
                };
                flatten_yaml_value(&full_key, v, map);
            }
        }
        serde_yaml::Value::Sequence(seq) => {
            let converted: Vec<ConfigValue> = seq.iter().map(yaml_to_config_value).collect();
            map.insert(prefix.to_string(), ConfigValue::Array(converted));
        }
        _ => {
            map.insert(prefix.to_string(), yaml_to_config_value(val));
        }
    }
}

fn yaml_to_config_value(v: &serde_yaml::Value) -> ConfigValue {
    match v {
        serde_yaml::Value::Null => ConfigValue::Null,
        serde_yaml::Value::Bool(b) => ConfigValue::Bool(*b),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                ConfigValue::Integer(i)
            } else if let Some(f) = n.as_f64() {
                ConfigValue::Float(f)
            } else {
                ConfigValue::String(n.to_string())
            }
        }
        serde_yaml::Value::String(s) => ConfigValue::String(s.clone()),
        serde_yaml::Value::Sequence(seq) => {
            ConfigValue::Array(seq.iter().map(yaml_to_config_value).collect())
        }
        serde_yaml::Value::Mapping(m) => {
            let mut tree = BTreeMap::new();
            for (k, val) in m {
                let key_str = match k {
                    serde_yaml::Value::String(s) => s.clone(),
                    _ => format!("{:?}", k),
                };
                tree.insert(key_str, yaml_to_config_value(val));
            }
            ConfigValue::Object(tree)
        }
        serde_yaml::Value::Tagged(tagged) => yaml_to_config_value(&tagged.value),
    }
}

fn parse_toml(content: &str) -> Result<BTreeMap<String, ConfigValue>> {
    let toml_val: toml::Value =
        toml::from_str(content).map_err(|e| anyhow!("Failed to parse TOML: {}", e))?;

    let mut map = BTreeMap::new();
    flatten_toml_value("", &toml_val, &mut map);
    Ok(map)
}

fn flatten_toml_value(prefix: &str, val: &toml::Value, map: &mut BTreeMap<String, ConfigValue>) {
    match val {
        toml::Value::Table(tbl) => {
            for (k, v) in tbl {
                let full_key = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{}.{}", prefix, k)
                };
                flatten_toml_value(&full_key, v, map);
            }
        }
        toml::Value::Array(arr) => {
            let converted: Vec<ConfigValue> = arr.iter().map(toml_to_config_value).collect();
            map.insert(prefix.to_string(), ConfigValue::Array(converted));
        }
        _ => {
            map.insert(prefix.to_string(), toml_to_config_value(val));
        }
    }
}

fn toml_to_config_value(v: &toml::Value) -> ConfigValue {
    match v {
        toml::Value::String(s) => ConfigValue::String(s.clone()),
        toml::Value::Integer(i) => ConfigValue::Integer(*i),
        toml::Value::Float(f) => ConfigValue::Float(*f),
        toml::Value::Boolean(b) => ConfigValue::Bool(*b),
        toml::Value::Datetime(dt) => ConfigValue::String(dt.to_string()),
        toml::Value::Array(arr) => {
            ConfigValue::Array(arr.iter().map(toml_to_config_value).collect())
        }
        toml::Value::Table(tbl) => {
            let mut tree = BTreeMap::new();
            for (k, val) in tbl {
                tree.insert(k.clone(), toml_to_config_value(val));
            }
            ConfigValue::Object(tree)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_env_basic() {
        let env_text = "PORT=8080\nDEBUG=true\nAPI_KEY=\"secret123\"\n# comment\nEMPTY=";
        let map = parse_env(env_text).unwrap();
        assert_eq!(map.get("PORT"), Some(&ConfigValue::Integer(8080)));
        assert_eq!(map.get("DEBUG"), Some(&ConfigValue::Bool(true)));
        assert_eq!(
            map.get("API_KEY"),
            Some(&ConfigValue::String("secret123".to_string()))
        );
        assert_eq!(map.get("EMPTY"), Some(&ConfigValue::Null));
    }

    #[test]
    fn test_parse_json_nested() {
        let json_text = r#"{"server":{"port":3000,"host":"127.0.0.1"}}"#;
        let map = parse_json(json_text).unwrap();
        assert_eq!(map.get("server.port"), Some(&ConfigValue::Integer(3000)));
        assert_eq!(
            map.get("server.host"),
            Some(&ConfigValue::String("127.0.0.1".to_string()))
        );
    }
}
