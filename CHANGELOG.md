# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-10-06

### Added
- Initial release of **ConfigDiff** CLI and library.
- Semantic comparison engine for `.env`, JSON, YAML, and TOML.
- Automatic secret detection and redaction (API keys, JWTs, AWS credentials, database URLs).
- Type mismatch detection with optional type coercion (`--coerce-types`).
- CI automation flags: `--fail-on-missing`, `--fail-on-extra`, `--fail-on-type`, `--strict`.
- Formatted ANSI terminal output and machine-readable `--json` mode.
- Ignore pattern filtering via regex.
- Cross-platform release workflows and GitHub Actions CI.
