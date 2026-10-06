# Contributing to ConfigDiff

Thank you for helping improve ConfigDiff!

## Guidelines
- Write tests for any new parser feature, edge-case format, or diff rule.
- Ensure strict checks pass before submitting a pull request:
  ```bash
  cargo fmt --check
  cargo clippy --all-targets -- -D warnings
  cargo test
  ```
- Keep the code modular and avoid adding heavy native C dependencies.
