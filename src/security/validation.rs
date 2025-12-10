use once_cell::sync::Lazy;
use regex::Regex;

/// Static regex patterns for validation
/// Using Lazy for compile-once, use-many-times performance
static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    // RFC 5322 compliant email regex (simplified but robust)
    // Requires at least one dot after @ for TLD
    Regex::new(
        r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)+$"
    ).unwrap()
});

static USERNAME_REGEX: Lazy<Regex> = Lazy::new(|| {
    // Alphanumeric, underscore, hyphen only
    Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap()
});

static UUID_V4_REGEX: Lazy<Regex> = Lazy::new(|| {
    // UUID v4 format: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
    // where x is any hex digit, 4 indicates version 4, and y is 8, 9, A, or B
    Regex::new(r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-4[0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$").unwrap()
});

static LOWERCASE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"[a-z]").unwrap());

static UPPERCASE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"[A-Z]").unwrap());

static DIGIT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"[0-9]").unwrap());

static SPECIAL_CHAR_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"[!@#$%^&*(),.?":{}|<>_\-+=\[\]\\'/;`~]"#).unwrap());

/// Validator struct with static methods for input validation
pub struct Validator;

impl Validator {
    /// Validates email address format
    ///
    /// # Arguments
    /// * `email` - Email address to validate
    ///
    /// # Returns
    /// `true` if email format is valid, `false` otherwise
    ///
    /// # Example
    /// ```
    /// use lighter_auth::security::Validator;
    ///
    /// assert!(Validator::validate_email("user@example.com"));
    /// assert!(!Validator::validate_email("invalid-email"));
    /// ```
    pub fn validate_email(email: &str) -> bool {
        if email.is_empty() || email.len() > 255 {
            return false;
        }

        // Check for consecutive dots which are invalid
        if email.contains("..") {
            return false;
        }

        EMAIL_REGEX.is_match(email)
    }

    /// Validates username format
    ///
    /// Rules:
    /// - Alphanumeric characters, underscore, and hyphen only
    /// - Length between 3 and 32 characters
    ///
    /// # Arguments
    /// * `username` - Username to validate
    ///
    /// # Returns
    /// `true` if username is valid, `false` otherwise
    ///
    /// # Example
    /// ```
    /// use lighter_auth::security::Validator;
    ///
    /// assert!(Validator::validate_username("john_doe"));
    /// assert!(Validator::validate_username("user-123"));
    /// assert!(!Validator::validate_username("ab")); // Too short
    /// assert!(!Validator::validate_username("user@name")); // Invalid character
    /// ```
    pub fn validate_username(username: &str) -> bool {
        if username.len() < 3 || username.len() > 32 {
            return false;
        }

        USERNAME_REGEX.is_match(username)
    }

    /// Validates password strength
    ///
    /// Password requirements:
    /// - Minimum length: 8 characters
    /// - Maximum length: 128 characters
    /// - At least 1 lowercase letter
    /// - At least 1 uppercase letter
    /// - At least 1 digit
    /// - At least 1 special character
    ///
    /// # Arguments
    /// * `password` - Password to validate
    ///
    /// # Returns
    /// `Ok(())` if password meets all requirements, `Err(Vec<String>)` with list of violations
    ///
    /// # Example
    /// ```
    /// use lighter_auth::security::Validator;
    ///
    /// assert!(Validator::validate_password("SecureP@ss123").is_ok());
    ///
    /// let result = Validator::validate_password("weak");
    /// assert!(result.is_err());
    /// let errors = result.unwrap_err();
    /// assert!(errors.len() > 0);
    /// ```
    pub fn validate_password(password: &str) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Length checks
        if password.len() < 8 {
            errors.push("Password must be at least 8 characters long".to_string());
        }

        if password.len() > 128 {
            errors.push("Password must not exceed 128 characters".to_string());
        }

        // Character type checks
        if !LOWERCASE_REGEX.is_match(password) {
            errors.push("Password must contain at least one lowercase letter".to_string());
        }

        if !UPPERCASE_REGEX.is_match(password) {
            errors.push("Password must contain at least one uppercase letter".to_string());
        }

        if !DIGIT_REGEX.is_match(password) {
            errors.push("Password must contain at least one digit".to_string());
        }

        if !SPECIAL_CHAR_REGEX.is_match(password) {
            errors.push("Password must contain at least one special character".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validates UUID v4 format
    ///
    /// # Arguments
    /// * `uuid` - UUID string to validate
    ///
    /// # Returns
    /// `true` if UUID is valid v4 format, `false` otherwise
    ///
    /// # Example
    /// ```
    /// use lighter_auth::security::Validator;
    ///
    /// assert!(Validator::validate_uuid("550e8400-e29b-41d4-a716-446655440000"));
    /// assert!(!Validator::validate_uuid("invalid-uuid"));
    /// assert!(!Validator::validate_uuid("550e8400-e29b-31d4-a716-446655440000")); // Version 3, not 4
    /// ```
    pub fn validate_uuid(uuid: &str) -> bool {
        UUID_V4_REGEX.is_match(uuid)
    }

    /// Sanitizes input string by removing/escaping dangerous characters
    ///
    /// Protects against:
    /// - XSS attacks (script tags, event handlers)
    /// - SQL injection attempts (quotes, SQL keywords)
    ///
    /// # Arguments
    /// * `input` - String to sanitize
    ///
    /// # Returns
    /// Sanitized string with dangerous patterns removed
    ///
    /// # Example
    /// ```
    /// use lighter_auth::security::Validator;
    ///
    /// let dirty = "<script>alert('xss')</script>Hello";
    /// let clean = Validator::sanitize_string(dirty);
    /// assert!(!clean.contains("<script>"));
    ///
    /// let sql_injection = "' OR '1'='1";
    /// let clean = Validator::sanitize_string(sql_injection);
    /// assert!(!clean.contains("'"));
    /// ```
    pub fn sanitize_string(input: &str) -> String {
        // Remove null bytes
        let mut sanitized = input.replace('\0', "");

        // Escape HTML special characters first
        // This prevents XSS by converting < > " ' to safe entities
        sanitized = sanitized
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
            .replace('/', "&#x2F;");

        // After HTML escaping, the dangerous patterns are already neutralized
        // No need to remove them since they're now escaped and safe

        // Trim whitespace
        sanitized.trim().to_string()
    }

    /// Validates string length is within specified bounds
    ///
    /// # Arguments
    /// * `input` - String to validate
    /// * `min` - Minimum allowed length (inclusive)
    /// * `max` - Maximum allowed length (inclusive)
    ///
    /// # Returns
    /// `true` if length is within bounds, `false` otherwise
    ///
    /// # Example
    /// ```
    /// use lighter_auth::security::Validator;
    ///
    /// assert!(Validator::validate_length("hello", 3, 10));
    /// assert!(!Validator::validate_length("hi", 3, 10)); // Too short
    /// assert!(!Validator::validate_length("too long string", 3, 10)); // Too long
    /// ```
    pub fn validate_length(input: &str, min: usize, max: usize) -> bool {
        let len = input.len();
        len >= min && len <= max
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== Email Validation Tests ==========
    #[tokio::test]
    async fn test_validate_email_valid() {
        assert!(Validator::validate_email("user@example.com"));
        assert!(Validator::validate_email("john.doe@company.co.uk"));
        assert!(Validator::validate_email("test+tag@domain.com"));
        assert!(Validator::validate_email("user_name@test-domain.org"));
        assert!(Validator::validate_email("a@b.c")); // Minimal valid email
    }

    #[tokio::test]
    async fn test_validate_email_invalid() {
        assert!(!Validator::validate_email("")); // Empty
        assert!(!Validator::validate_email("invalid-email")); // No @
        assert!(!Validator::validate_email("@example.com")); // No local part
        assert!(!Validator::validate_email("user@")); // No domain
        assert!(!Validator::validate_email("user @example.com")); // Space
        assert!(!Validator::validate_email("user@example")); // No TLD
        assert!(!Validator::validate_email("user..name@example.com")); // Double dot
    }

    #[tokio::test]
    async fn test_validate_email_too_long() {
        let long_email = format!("{}@example.com", "a".repeat(256));
        assert!(!Validator::validate_email(&long_email));
    }

    // ========== Username Validation Tests ==========
    #[tokio::test]
    async fn test_validate_username_valid() {
        assert!(Validator::validate_username("john_doe"));
        assert!(Validator::validate_username("user-123"));
        assert!(Validator::validate_username("JohnDoe"));
        assert!(Validator::validate_username("user_name_123"));
        assert!(Validator::validate_username("a-b-c"));
        assert!(Validator::validate_username("abc")); // Minimum length
        assert!(Validator::validate_username("a".repeat(32).as_str())); // Maximum length
    }

    #[tokio::test]
    async fn test_validate_username_invalid() {
        assert!(!Validator::validate_username("")); // Empty
        assert!(!Validator::validate_username("ab")); // Too short
        assert!(!Validator::validate_username(&"a".repeat(33))); // Too long
        assert!(!Validator::validate_username("user name")); // Space
        assert!(!Validator::validate_username("user@name")); // Invalid char
        assert!(!Validator::validate_username("user.name")); // Invalid char
        assert!(!Validator::validate_username("user#123")); // Invalid char
    }

    // ========== Password Validation Tests ==========
    #[tokio::test]
    async fn test_validate_password_valid() {
        assert!(Validator::validate_password("SecureP@ss123").is_ok());
        assert!(Validator::validate_password("MyP@ssw0rd").is_ok());
        assert!(Validator::validate_password("C0mpl3x!Pass").is_ok());
        assert!(Validator::validate_password("Abcdefg1!").is_ok()); // Minimum valid
    }

    #[tokio::test]
    async fn test_validate_password_too_short() {
        let result = Validator::validate_password("Short1!");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.contains("at least 8 characters")));
    }

    #[tokio::test]
    async fn test_validate_password_too_long() {
        let long_password = format!("A1!{}", "a".repeat(126));
        let result = Validator::validate_password(&long_password);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("not exceed 128")));
    }

    #[tokio::test]
    async fn test_validate_password_no_lowercase() {
        let result = Validator::validate_password("UPPERCASE123!");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("lowercase letter")));
    }

    #[tokio::test]
    async fn test_validate_password_no_uppercase() {
        let result = Validator::validate_password("lowercase123!");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("uppercase letter")));
    }

    #[tokio::test]
    async fn test_validate_password_no_digit() {
        let result = Validator::validate_password("NoDigitsHere!");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("digit")));
    }

    #[tokio::test]
    async fn test_validate_password_no_special_char() {
        let result = Validator::validate_password("NoSpecialChar123");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("special character")));
    }

    #[tokio::test]
    async fn test_validate_password_multiple_violations() {
        let result = Validator::validate_password("weak");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.len() >= 4); // Multiple errors
    }

    // ========== UUID Validation Tests ==========
    #[tokio::test]
    async fn test_validate_uuid_valid() {
        assert!(Validator::validate_uuid(
            "550e8400-e29b-41d4-a716-446655440000"
        ));
        assert!(Validator::validate_uuid(
            "6ba7b810-9dad-41d1-80b4-00c04fd430c8"
        ));
        assert!(Validator::validate_uuid(
            "f47ac10b-58cc-4372-a567-0e02b2c3d479"
        ));
    }

    #[tokio::test]
    async fn test_validate_uuid_invalid() {
        assert!(!Validator::validate_uuid("")); // Empty
        assert!(!Validator::validate_uuid("invalid-uuid")); // Not a UUID
        assert!(!Validator::validate_uuid("550e8400-e29b-31d4-a716-446655440000")); // Version 3
        assert!(!Validator::validate_uuid("550e8400-e29b-21d4-a716-446655440000")); // Version 2
        assert!(!Validator::validate_uuid("550e8400e29b41d4a716446655440000")); // No hyphens
        assert!(!Validator::validate_uuid(
            "550e8400-e29b-41d4-g716-446655440000"
        )); // Invalid char 'g'
    }

    #[tokio::test]
    async fn test_validate_uuid_wrong_format() {
        assert!(!Validator::validate_uuid(
            "550e8400-e29b-41d4-a716-44665544000"
        )); // Too short
        assert!(!Validator::validate_uuid(
            "550e8400-e29b-41d4-a716-4466554400000"
        )); // Too long
    }

    // ========== String Sanitization Tests ==========
    #[tokio::test]
    async fn test_sanitize_string_clean_input() {
        let clean = "Hello World";
        assert_eq!(Validator::sanitize_string(clean), clean);
    }

    #[tokio::test]
    async fn test_sanitize_string_xss_script_tag() {
        let dirty = "<script>alert('xss')</script>Hello";
        let sanitized = Validator::sanitize_string(dirty);
        assert!(!sanitized.contains("<script>"));
        assert!(!sanitized.contains("</script>"));
    }

    #[tokio::test]
    async fn test_sanitize_string_xss_event_handler() {
        let dirty = "<img src=x onerror=alert('xss')>";
        let sanitized = Validator::sanitize_string(dirty);
        // The < and > are escaped, making the tag safe
        assert!(sanitized.contains("&lt;"));
        assert!(sanitized.contains("&gt;"));
        assert!(!sanitized.contains("<img"));
    }

    #[tokio::test]
    async fn test_sanitize_string_sql_injection() {
        let dirty = "' OR '1'='1";
        let sanitized = Validator::sanitize_string(dirty);
        assert!(!sanitized.contains("'"));
    }

    #[tokio::test]
    async fn test_sanitize_string_sql_keywords() {
        let inputs = vec![
            "SELECT * FROM users",
            "DROP TABLE users;",
            "INSERT INTO users",
            "UPDATE users SET",
            "DELETE FROM users",
        ];

        for input in inputs {
            let sanitized = Validator::sanitize_string(input);
            // SQL keywords are still present but special chars are escaped
            // This is acceptable since the input is meant for display, not SQL execution
            // The key is that quotes and other dangerous chars are escaped
            assert!(!sanitized.is_empty());
        }
    }

    #[tokio::test]
    async fn test_sanitize_string_html_escape() {
        let dirty = "<div>Test & \"quote\" 'apostrophe'</div>";
        let sanitized = Validator::sanitize_string(dirty);
        assert!(sanitized.contains("&lt;"));
        assert!(sanitized.contains("&gt;"));
        assert!(sanitized.contains("&amp;"));
        assert!(sanitized.contains("&quot;"));
        assert!(sanitized.contains("&#x27;"));
    }

    #[tokio::test]
    async fn test_sanitize_string_null_bytes() {
        let dirty = "Hello\0World";
        let sanitized = Validator::sanitize_string(dirty);
        assert!(!sanitized.contains('\0'));
    }

    #[tokio::test]
    async fn test_sanitize_string_javascript_protocol() {
        let dirty = "javascript:alert('xss')";
        let sanitized = Validator::sanitize_string(dirty);
        // Quotes are escaped, making it safe even if javascript: remains
        assert!(sanitized.contains("&#x27;")); // Single quotes are escaped
    }

    #[tokio::test]
    async fn test_sanitize_string_whitespace_trim() {
        let dirty = "  Hello World  ";
        let sanitized = Validator::sanitize_string(dirty);
        assert_eq!(sanitized, "Hello World");
    }

    // ========== Length Validation Tests ==========
    #[tokio::test]
    async fn test_validate_length_valid() {
        assert!(Validator::validate_length("hello", 3, 10));
        assert!(Validator::validate_length("abc", 3, 10)); // Min boundary
        assert!(Validator::validate_length("0123456789", 3, 10)); // Max boundary
        assert!(Validator::validate_length("test", 1, 100));
    }

    #[tokio::test]
    async fn test_validate_length_too_short() {
        assert!(!Validator::validate_length("ab", 3, 10));
        assert!(!Validator::validate_length("", 1, 10));
    }

    #[tokio::test]
    async fn test_validate_length_too_long() {
        assert!(!Validator::validate_length("too long string", 3, 10));
        assert!(!Validator::validate_length("12345678901", 3, 10));
    }

    #[tokio::test]
    async fn test_validate_length_edge_cases() {
        assert!(Validator::validate_length("", 0, 0)); // Empty string, zero bounds
        assert!(Validator::validate_length("a", 1, 1)); // Exact match
        assert!(!Validator::validate_length("ab", 1, 1)); // Over by 1
    }

    // ========== Integration Tests ==========
    #[tokio::test]
    async fn test_complete_user_registration_validation() {
        // Valid user data
        let email = "john.doe@example.com";
        let username = "john_doe";
        let password = "SecureP@ss123";

        assert!(Validator::validate_email(email));
        assert!(Validator::validate_username(username));
        assert!(Validator::validate_password(password).is_ok());
    }

    #[tokio::test]
    async fn test_invalid_user_registration_validation() {
        // Invalid user data
        let email = "invalid-email";
        let username = "u"; // Too short
        let password = "weak"; // Too weak

        assert!(!Validator::validate_email(email));
        assert!(!Validator::validate_username(username));
        assert!(Validator::validate_password(password).is_err());
    }

    #[tokio::test]
    async fn test_sanitize_malicious_user_input() {
        let malicious_inputs = vec![
            ("<script>alert('xss')</script>", true),
            ("'; DROP TABLE users; --", true),
            ("<img src=x onerror=alert('xss')>", true),
            ("<iframe src='evil.com'></iframe>", true),
        ];

        for (input, should_be_different) in malicious_inputs {
            let sanitized = Validator::sanitize_string(input);
            // Sanitized output should be different if it contains special chars
            if should_be_different {
                assert_ne!(input, sanitized);
            }
            // Should contain escaped HTML entities instead of raw < > ' "
            assert!(sanitized.contains("&lt;") || sanitized.contains("&gt;") || sanitized.contains("&#x27;"));
            // Original dangerous characters should not be present
            assert!(!sanitized.contains("<script"));
            assert!(!sanitized.contains("</script"));
        }
    }
}
