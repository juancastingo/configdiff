# 🔍 ConfigDiff

[![CI](https://github.com/juancastingo/configdiff/actions/workflows/ci.yml/badge.svg)](https://github.com/juancastingo/configdiff/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/configdiff.svg)](https://crates.io/crates/configdiff)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-linux%20%7C%20macos%20%7C%20windows-lightgrey.svg)](https://github.com/juancastingo/configdiff)

**ConfigDiff** is a fast, semantic configuration file diff tool designed to compare environment configurations across **`.env`**, **`JSON`**, **`YAML`**, and **`TOML`** formats with **automatic secret redaction** and **CI-ready exit codes**.

It solves the classic developer and DevOps headache: *"Why does our service work in staging but fail in production?"*

---

## 📸 Terminal Preview

```text
══════════════════════════════════════════════════════════════════════
 🔍 ConfigDiff: .env.example → .env.production
══════════════════════════════════════════════════════════════════════
 Keys: Source: 14, Target: 13 | Identical: 11
 Summary: Missing: 1 | Extra: 0 | Type Mismatches: 1 | Value Mismatches: 1
──────────────────────────────────────────────────────────────────────
 DIFFERENCES:
  - STRIPE_WEBHOOK_SECRET          (missing in target, was string: whsec_***) [SECRET]
  ~ PORT                           integer(8080) -> string(8080)
  ≠ DATABASE_URL                   po***[REDACTED] -> po***[REDACTED] [SECRET]
══════════════════════════════════════════════════════════════════════
```

---

## ✨ Key Features

- **Cross-Format Comparison**: Compare across different formats seamlessly:
  - Compare `.env` vs `config.json`
  - Compare `values.yaml` vs `config.toml`
  - Compare nested object trees against flattened env variable keys
- **Automatic Secret Redaction**:
  - Automatically identifies sensitive keys (`*SECRET*`, `*KEY*`, `*PASSWORD*`, `*TOKEN*`, `*AUTH*`, `*DATABASE_URL*`)
  - Identifies sensitive values (JWTs, AWS keys, Bearer tokens, private key headers)
  - Safely masks values (`sk-***[REDACTED]`) so terminal screens and CI logs never leak production secrets.
- **Semantic Type Checking**:
  - Detects subtle configuration bugs where a variable expected to be an integer (`PORT=8080`) is parsed as a string (`"8080"`).
  - Optional `--coerce-types` flag to allow compatible representations when desired.
- **CI/CD Integration**:
  - `--fail-on-missing`: Fail the build if any required keys from `.env.example` are missing.
  - `--fail-on-extra`: Fail if unexpected variables are found in target.
  - `--strict`: Fail on any semantic difference.
- **JSON Output**: Full `--json` support for automated auditing, compliance tools, and deployment pipelines.
- **Zero C Dependencies**: Pure, modern Rust compiled to a single static binary.

---

## 🚀 Installation

### Using Cargo
```bash
cargo install configdiff
```

### Pre-compiled Binaries
Download pre-built standalone binaries for Linux, macOS, and Windows from the [Releases](https://github.com/juancastingo/configdiff/releases) page.

### From Source
```bash
git clone https://github.com/juancastingo/configdiff.git
cd configdiff
cargo build --release
./target/release/configdiff --version
```

---

## 📖 Usage Examples

### 1. Verify Production Environment against Template
Ensure all variables defined in `.env.example` are present in `.env.production`:
```bash
configdiff .env.example .env.production --fail-on-missing
```

### 2. Compare Across Formats
Compare a development `.env` with a production `config.yaml`:
```bash
configdiff .env config.yaml
```

### 3. Machine-Readable JSON Output
```bash
configdiff app.staging.json app.prod.json --json | jq .
```

### 4. Ignore Ephemeral or Local Variables
Ignore local debug flags or timestamp variables using regex patterns:
```bash
configdiff .env.example .env.local -i "^LOCAL_" -i "^DEV_ONLY_"
```

### 5. Coerce Compatible Types
Treat numeric strings and integers as equivalent:
```bash
configdiff base.json target.json --coerce-types
```

---

## ⚙️ CLI Options

| Flag | Short | Description |
|---|---|---|
| `<SOURCE>` | | Base configuration file (`.env`, `.json`, `.yaml`, `.toml`, or `-` for stdin) |
| `<TARGET>` | | Target configuration file to compare against |
| `--fail-on-missing` | | Exit with code 1 if any source keys are missing in target |
| `--fail-on-extra` | | Exit with code 1 if target contains unexpected keys |
| `--fail-on-type` | | Exit with code 1 on configuration type mismatches |
| `--strict` | | Exit with code 1 if ANY difference exists |
| `--coerce-types` | | Treat compatible representations (e.g. `"8080"` and `8080`) as equivalent |
| `--no-redact` | | Disable automatic masking of sensitive credentials |
| `-i`, `--ignore` | | Ignore keys matching regex pattern (can be repeated) |
| `--json` | | Output report in structured JSON format |

---

## 📦 Using ConfigDiff as a Rust Library

```toml
[dependencies]
configdiff = "0.1"
```

```rust
use configdiff::{compare_configs, load_config_file, DiffOptions};

fn main() -> anyhow::Result<()> {
    let source = load_config_file(".env.example")?;
    let target = load_config_file(".env.production")?;

    let report = compare_configs(
        ".env.example",
        &source,
        ".env.production",
        &target,
        &DiffOptions::default(),
    );

    if report.missing_count > 0 {
        eprintln!("Warning: {} configuration keys are missing in target!", report.missing_count);
    }

    Ok(())
}
```

---

## 📄 License

Licensed under either the [MIT License](LICENSE) or the Apache License 2.0 at your option.
