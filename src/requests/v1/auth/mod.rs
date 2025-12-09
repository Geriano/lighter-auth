use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::security::validation::Validator;

#[derive(Clone, Debug, Deserialize, Serialize, ToSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    #[schema(example = "john.doe")]
    pub email_or_username: String,
    #[schema(example = "password")]
    pub password: String,
}

impl LoginRequest {
    /// Validates the login request
    ///
    /// Validates:
    /// - email_or_username: must be valid email OR valid username
    /// - password: must not be empty
    ///
    /// Returns Ok(()) if all validations pass, Err(Vec<String>) with error messages otherwise
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate email_or_username (can be either email OR username)
        let trimmed = self.email_or_username.trim();
        if trimmed.is_empty() {
            errors.push("Email or username is required".to_string());
        } else {
            // Check if it's a valid email OR a valid username
            let is_valid_email = Validator::validate_email(trimmed);
            let is_valid_username = Validator::validate_username(trimmed);

            if !is_valid_email && !is_valid_username {
                errors.push("Email or username must be a valid email address or username (3-32 alphanumeric characters, underscores, or hyphens)".to_string());
            }
        }

        // Validate password
        if self.password.is_empty() {
            errors.push("Password is required".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_login_request_valid_with_email() {
        let request = LoginRequest {
            email_or_username: "test@example.com".to_string(),
            password: "password123".to_string(),
        };
        assert!(request.validate().is_ok());
    }

    #[tokio::test]
    async fn test_login_request_valid_with_username() {
        let request = LoginRequest {
            email_or_username: "john_doe".to_string(),
            password: "password123".to_string(),
        };
        assert!(request.validate().is_ok());
    }

    #[tokio::test]
    async fn test_login_request_empty_email_or_username() {
        let request = LoginRequest {
            email_or_username: "".to_string(),
            password: "password123".to_string(),
        };
        let result = request.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("required"));
    }

    #[tokio::test]
    async fn test_login_request_invalid_email_or_username() {
        let request = LoginRequest {
            email_or_username: "ab".to_string(), // Too short for username, invalid email
            password: "password123".to_string(),
        };
        let result = request.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("valid email address or username"));
    }

    #[tokio::test]
    async fn test_login_request_empty_password() {
        let request = LoginRequest {
            email_or_username: "test@example.com".to_string(),
            password: "".to_string(),
        };
        let result = request.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Password is required"));
    }

    #[tokio::test]
    async fn test_login_request_multiple_errors() {
        let request = LoginRequest {
            email_or_username: "".to_string(),
            password: "".to_string(),
        };
        let result = request.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 2);
    }

    #[tokio::test]
    async fn test_login_request_whitespace_handling() {
        let request = LoginRequest {
            email_or_username: "  test@example.com  ".to_string(),
            password: "password123".to_string(),
        };
        assert!(request.validate().is_ok());
    }
}
