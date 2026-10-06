use configdiff::diff::{compare_configs, DiffOptions};
use configdiff::model::{ConfigValue, DiffType};
use configdiff::parser::{parse_config_content, FileFormat};
use std::collections::BTreeMap;

#[test]
fn test_cross_format_env_and_json() {
    let env_content = r#"
PORT=8080
DATABASE_URL=postgres://user:pass@localhost:5432/app
DEBUG=false
"#;

    let json_content = r#"{
    "PORT": 8080,
    "DATABASE_URL": "postgres://prod_user:secret@db.prod:5432/app",
    "DEBUG": false,
    "EXTRA_TIMEOUT": 30
}"#;

    let env_map = parse_config_content(env_content, FileFormat::Env).unwrap();
    let json_map = parse_config_content(json_content, FileFormat::Json).unwrap();

    let options = DiffOptions::default();
    let report = compare_configs(".env", &env_map, "config.json", &json_map, &options);

    assert_eq!(report.total_source_keys, 3);
    assert_eq!(report.total_target_keys, 4);
    assert_eq!(report.identical_count, 2); // PORT and DEBUG
    assert_eq!(report.extra_count, 1); // EXTRA_TIMEOUT
    assert_eq!(report.value_mismatch_count, 1); // DATABASE_URL
    assert_eq!(report.missing_count, 0);

    let db_diff = report
        .diffs
        .iter()
        .find(|d| d.path == "DATABASE_URL")
        .unwrap();
    assert!(db_diff.is_secret);
    // Secret must be masked
    assert!(
        db_diff.source_val.as_ref().unwrap().contains("REDACTED")
            || db_diff.source_val.as_ref().unwrap().contains("***")
    );
}

#[test]
fn test_yaml_and_toml_parsing() {
    let yaml_content = r#"
server:
  host: 127.0.0.1
  port: 8080
"#;

    let toml_content = r#"
[server]
host = "127.0.0.1"
port = 9000
"#;

    let yaml_map = parse_config_content(yaml_content, FileFormat::Yaml).unwrap();
    let toml_map = parse_config_content(toml_content, FileFormat::Toml).unwrap();

    let options = DiffOptions::default();
    let report = compare_configs("config.yaml", &yaml_map, "config.toml", &toml_map, &options);

    assert_eq!(report.identical_count, 1); // server.host
    assert_eq!(report.value_mismatch_count, 1); // server.port
}

#[test]
fn test_type_mismatch_and_coercion() {
    let mut src = BTreeMap::new();
    src.insert("PORT".to_string(), ConfigValue::Integer(8080));

    let mut tgt = BTreeMap::new();
    tgt.insert("PORT".to_string(), ConfigValue::String("8080".to_string()));

    // Without coercion: should report TypeMismatch
    let strict_opts = DiffOptions {
        coerce_types: false,
        ..Default::default()
    };
    let report_strict = compare_configs("src", &src, "tgt", &tgt, &strict_opts);
    assert_eq!(report_strict.type_mismatch_count, 1);
    assert!(report_strict
        .diffs
        .iter()
        .any(|d| d.diff_type == DiffType::TypeMismatch));

    // With coercion: should treat as identical
    let coerce_opts = DiffOptions {
        coerce_types: true,
        ..Default::default()
    };
    let report_coerce = compare_configs("src", &src, "tgt", &tgt, &coerce_opts);
    assert_eq!(report_coerce.type_mismatch_count, 0);
    assert_eq!(report_coerce.identical_count, 1);
}

#[test]
fn test_ignore_pattern() {
    let mut src = BTreeMap::new();
    src.insert("INTERNAL_DEBUG_FLAG".to_string(), ConfigValue::Bool(true));
    src.insert("PORT".to_string(), ConfigValue::Integer(8080));

    let mut tgt = BTreeMap::new();
    tgt.insert("PORT".to_string(), ConfigValue::Integer(8080));

    let options = DiffOptions {
        ignore_patterns: vec!["^INTERNAL_".to_string()],
        ..Default::default()
    };

    let report = compare_configs("src", &src, "tgt", &tgt, &options);
    assert_eq!(report.missing_count, 0); // INTERNAL_DEBUG_FLAG ignored
    assert_eq!(report.identical_count, 1);
    assert!(!report.has_diff);
}
