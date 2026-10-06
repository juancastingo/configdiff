use regex::Regex;
use std::sync::OnceLock;

static SECRET_KEY_PATTERNS: &[&str] = &[
    "secret",
    "password",
    "passwd",
    "pwd",
    "token",
    "api_key",
    "apikey",
    "auth",
    "private_key",
    "credential",
    "database_url",
    "db_pass",
    "jwt",
    "client_secret",
    "access_key",
    "encryption_key",
];

static JWT_REGEX: OnceLock<Regex> = OnceLock::new();
static AWS_KEY_REGEX: OnceLock<Regex> = OnceLock::new();
static BEARER_REGEX: OnceLock<Regex> = OnceLock::new();
static GITHUB_TOKEN_REGEX: OnceLock<Regex> = OnceLock::new();

pub fn is_secret_key(key_path: &str) -> bool {
    let lower = key_path.to_ascii_lowercase();
    SECRET_KEY_PATTERNS
        .iter()
        .any(|pattern| lower.contains(pattern))
}

pub fn is_secret_value(val: &str) -> bool {
    let trimmed = val.trim();
    if trimmed.is_empty() {
        return false;
    }

    if trimmed.contains("-----BEGIN") {
        return true;
    }

    let jwt_re = JWT_REGEX
        .get_or_init(|| Regex::new(r"^[A-Za-z0-9-_]+\.[A-Za-z0-9-_]+\.[A-Za-z0-9-_]+$").unwrap());
    if jwt_re.is_match(trimmed) {
        return true;
    }

    let aws_re = AWS_KEY_REGEX.get_or_init(|| Regex::new(r"^AKIA[0-9A-Z]{16}$").unwrap());
    if aws_re.is_match(trimmed) {
        return true;
    }

    let gh_re =
        GITHUB_TOKEN_REGEX.get_or_init(|| Regex::new(r"^gh[pousr]_[A-Za-z0-9_]{36,255}$").unwrap());
    if gh_re.is_match(trimmed) {
        return true;
    }

    let bearer_re =
        BEARER_REGEX.get_or_init(|| Regex::new(r"^Bearer\s+[A-Za-z0-9._~+/-]+=*$").unwrap());
    if bearer_re.is_match(trimmed) {
        return true;
    }

    false
}

pub fn mask_value(val: &str) -> String {
    if val.len() <= 4 {
        "***".to_string()
    } else {
        format!("{}***[REDACTED]", &val[..2.min(val.len())])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_key_detection() {
        assert!(is_secret_key("DATABASE_URL"));
        assert!(is_secret_key("app.database.password"));
        assert!(is_secret_key("API_KEY"));
        assert!(is_secret_key("stripe_secret_key"));
        assert!(!is_secret_key("server.port"));
        assert!(!is_secret_key("APP_ENV"));
    }

    #[test]
    fn test_secret_value_detection() {
        assert!(is_secret_value("AKIAIOSFODNN7EXAMPLE"));
        assert!(is_secret_value("Bearer secret_jwt_token_12345"));
        assert!(!is_secret_value("production"));
        assert!(!is_secret_value("8080"));
    }
}
