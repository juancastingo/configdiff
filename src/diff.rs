use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};

use crate::model::{ConfigValue, DiffEntry, DiffReport, DiffType};
use crate::redactor::{is_secret_key, is_secret_value, mask_value};

#[derive(Debug, Clone)]
pub struct DiffOptions {
    pub redact_secrets: bool,
    pub coerce_types: bool,
    pub ignore_patterns: Vec<String>,
}

impl Default for DiffOptions {
    fn default() -> Self {
        Self {
            redact_secrets: true,
            coerce_types: false,
            ignore_patterns: Vec::new(),
        }
    }
}

pub fn compare_configs(
    source_path: &str,
    source_map: &BTreeMap<String, ConfigValue>,
    target_path: &str,
    target_map: &BTreeMap<String, ConfigValue>,
    options: &DiffOptions,
) -> DiffReport {
    let ignore_regexes: Vec<Regex> = options
        .ignore_patterns
        .iter()
        .filter_map(|p| Regex::new(p).ok())
        .collect();

    let all_keys: BTreeSet<String> = source_map
        .keys()
        .chain(target_map.keys())
        .cloned()
        .collect();

    let mut diffs = Vec::new();
    let mut missing_count = 0;
    let mut extra_count = 0;
    let mut type_mismatch_count = 0;
    let mut value_mismatch_count = 0;
    let mut identical_count = 0;

    for key in all_keys {
        // Check ignore patterns
        if ignore_regexes.iter().any(|re| re.is_match(&key)) {
            continue;
        }

        let is_secret = is_secret_key(&key);
        let src_val = source_map.get(&key);
        let tgt_val = target_map.get(&key);

        match (src_val, tgt_val) {
            (Some(s), None) => {
                missing_count += 1;
                let display_val = format_value(s, is_secret, options.redact_secrets);
                diffs.push(DiffEntry {
                    path: key.clone(),
                    diff_type: DiffType::Missing,
                    source_val: Some(display_val),
                    target_val: None,
                    source_type: Some(s.type_name().to_string()),
                    target_type: None,
                    is_secret,
                });
            }
            (None, Some(t)) => {
                extra_count += 1;
                let display_val = format_value(t, is_secret, options.redact_secrets);
                diffs.push(DiffEntry {
                    path: key.clone(),
                    diff_type: DiffType::Extra,
                    source_val: None,
                    target_val: Some(display_val),
                    source_type: None,
                    target_type: Some(t.type_name().to_string()),
                    is_secret,
                });
            }
            (Some(s), Some(t)) => {
                if s == t {
                    identical_count += 1;
                    continue;
                }

                // If coerce types is enabled, check if string representations match
                if options.coerce_types && values_are_coercible_equal(s, t) {
                    identical_count += 1;
                    continue;
                }

                let s_type = s.type_name();
                let t_type = t.type_name();

                let s_disp = format_value(s, is_secret, options.redact_secrets);
                let t_disp = format_value(t, is_secret, options.redact_secrets);

                if s_type != t_type {
                    type_mismatch_count += 1;
                    diffs.push(DiffEntry {
                        path: key.clone(),
                        diff_type: DiffType::TypeMismatch,
                        source_val: Some(s_disp),
                        target_val: Some(t_disp),
                        source_type: Some(s_type.to_string()),
                        target_type: Some(t_type.to_string()),
                        is_secret,
                    });
                } else {
                    value_mismatch_count += 1;
                    diffs.push(DiffEntry {
                        path: key.clone(),
                        diff_type: DiffType::ValueMismatch,
                        source_val: Some(s_disp),
                        target_val: Some(t_disp),
                        source_type: Some(s_type.to_string()),
                        target_type: Some(t_type.to_string()),
                        is_secret,
                    });
                }
            }
            (None, None) => {}
        }
    }

    let has_diff = !diffs.is_empty();

    DiffReport {
        source_path: source_path.to_string(),
        target_path: target_path.to_string(),
        total_source_keys: source_map.len(),
        total_target_keys: target_map.len(),
        identical_count,
        missing_count,
        extra_count,
        type_mismatch_count,
        value_mismatch_count,
        has_diff,
        diffs,
    }
}

fn format_value(val: &ConfigValue, is_secret_key: bool, redact: bool) -> String {
    let raw = val.to_display_string();
    if redact && (is_secret_key || is_secret_value(&raw)) {
        mask_value(&raw)
    } else {
        raw
    }
}

fn values_are_coercible_equal(a: &ConfigValue, b: &ConfigValue) -> bool {
    let a_str = a.to_display_string().to_ascii_lowercase();
    let b_str = b.to_display_string().to_ascii_lowercase();
    a_str == b_str
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_missing_and_extra() {
        let mut src = BTreeMap::new();
        src.insert("PORT".to_string(), ConfigValue::Integer(8080));
        src.insert(
            "SECRET_KEY".to_string(),
            ConfigValue::String("supersecret123".to_string()),
        );

        let mut tgt = BTreeMap::new();
        tgt.insert("PORT".to_string(), ConfigValue::Integer(8080));
        tgt.insert("EXTRA_VAR".to_string(), ConfigValue::Bool(true));

        let report = compare_configs("base", &src, "target", &tgt, &DiffOptions::default());
        assert_eq!(report.identical_count, 1);
        assert_eq!(report.missing_count, 1);
        assert_eq!(report.extra_count, 1);

        let missing = report
            .diffs
            .iter()
            .find(|d| d.path == "SECRET_KEY")
            .unwrap();
        assert_eq!(missing.diff_type, DiffType::Missing);
        assert!(
            missing.source_val.as_ref().unwrap().contains("REDACTED")
                || missing.source_val.as_ref().unwrap().contains("***")
        );
    }
}
