use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "configdiff",
    author = "Juan Castiñeira <juancastingo@gmail.com>",
    version,
    about = "Semantic configuration and environment diff tool (.env, JSON, YAML, TOML) with automatic secret redaction and CI exit codes.",
    long_about = "ConfigDiff compares configuration files across different formats (.env, JSON, YAML, TOML) or environments, detects missing or extra keys, identifies type discrepancies, automatically masks sensitive credentials, and supports strict exit codes for CI/CD pipelines."
)]
pub struct CliArgs {
    /// Base configuration file (e.g. .env.example, config.default.json, or '-' for stdin)
    #[arg(value_name = "SOURCE")]
    pub source: String,

    /// Target configuration file to compare against (e.g. .env.production, config.yaml)
    #[arg(value_name = "TARGET")]
    pub target: String,

    /// Fail with non-zero exit code if any keys from source are missing in target
    #[arg(long = "fail-on-missing")]
    pub fail_on_missing: bool,

    /// Fail with non-zero exit code if target contains unexpected extra keys
    #[arg(long = "fail-on-extra")]
    pub fail_on_extra: bool,

    /// Fail with non-zero exit code on configuration type mismatches
    #[arg(long = "fail-on-type")]
    pub fail_on_type: bool,

    /// Strict CI mode: fail with exit code 1 if ANY difference exists (missing, extra, type, or value)
    #[arg(long = "strict")]
    pub strict: bool,

    /// Disable automatic secret masking (show plaintext values)
    #[arg(long = "no-redact")]
    pub no_redact: bool,

    /// Coerce compatible types (e.g. treat numeric string "8080" and integer 8080 as equivalent)
    #[arg(long = "coerce-types")]
    pub coerce_types: bool,

    /// Ignore keys matching regex pattern (can be specified multiple times)
    #[arg(short = 'i', long = "ignore")]
    pub ignore_patterns: Vec<String>,

    /// Output report in structured JSON format
    #[arg(long = "json")]
    pub json: bool,
}
