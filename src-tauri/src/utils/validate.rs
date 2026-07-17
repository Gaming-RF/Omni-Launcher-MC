// Input validation utilities for command handlers.
// Prevents path traversal, SQL injection via IDs, and invalid usernames.

use std::path::Path;

/// Validate an instance/mod ID is a safe alphanumeric + hyphen string.
pub fn validate_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > 64 {
        return Err("ID must be 1-64 characters".to_string());
    }
    if !id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(
            "ID contains invalid characters (only alphanumeric, hyphens, underscores allowed)"
                .to_string(),
        );
    }
    // Prevent path traversal
    if id.contains("..") || id.contains('/') || id.contains('\\') {
        return Err("ID must not contain path separators".to_string());
    }
    Ok(())
}

/// Validate a username (offline mode). 3-16 chars, alphanumeric + underscore.
pub fn validate_username(name: &str) -> Result<(), String> {
    let trimmed = name.trim();
    if trimmed.len() < 3 || trimmed.len() > 16 {
        return Err("Username must be 3-16 characters".to_string());
    }
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err("Username can only contain letters, numbers, and underscores".to_string());
    }
    Ok(())
}

/// Validate an instance name. 1-64 chars, no control characters.
pub fn validate_instance_name(name: &str) -> Result<(), String> {
    let trimmed = name.trim();
    if trimmed.is_empty() || trimmed.len() > 64 {
        return Err("Name must be 1-64 characters".to_string());
    }
    if trimmed.chars().any(|c| c.is_control()) {
        return Err("Name contains invalid characters".to_string());
    }
    Ok(())
}

/// Validate that a path doesn't escape the expected base directory.
pub fn validate_path_within(base: &Path, target: &Path) -> Result<(), String> {
    let canonical_base = base
        .canonicalize()
        .map_err(|_| "Invalid base path".to_string())?;
    let canonical_target = target
        .canonicalize()
        .map_err(|_| format!("Path does not exist: {}", target.display()))?;

    if !canonical_target.starts_with(&canonical_base) {
        return Err(format!(
            "Path traversal detected: {} is outside {}",
            target.display(),
            base.display()
        ));
    }
    Ok(())
}

/// Sanitize a search query — strip control characters and limit length.
pub fn sanitize_query(query: &str) -> String {
    query
        .chars()
        .filter(|c| !c.is_control())
        .take(200)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_id() {
        assert!(validate_id("abc-123").is_ok());
        assert!(validate_id("").is_err());
        assert!(validate_id("../etc/passwd").is_err());
        assert!(validate_id("hello world").is_err());
        assert!(validate_id("a".repeat(100).as_str()).is_err());
    }

    #[test]
    fn test_validate_username() {
        assert!(validate_username("Steve").is_ok());
        assert!(validate_username("ab").is_err());
        assert!(validate_username("a".repeat(20).as_str()).is_err());
        assert!(validate_username("hello world").is_err());
    }

    #[test]
    fn test_sanitize_query() {
        assert_eq!(sanitize_query("sodium\x00mod"), "sodiummod");
        assert_eq!(sanitize_query(&"a".repeat(300)).len(), 200);
    }
}
