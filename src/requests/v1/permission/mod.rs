use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::security::validation::Validator;

#[derive(Clone, Deserialize, Serialize, ToSchema)]
pub struct PermissionRequest {
    #[schema(example = "Create User")]
    pub name: String,
}

impl PermissionRequest {
    /// Validates the permission request
    ///
    /// Validates:
    /// - name: must not be empty and must be between 3-100 characters
    ///
    /// Returns Ok(()) if all validations pass, Err(Vec<String>) with error messages otherwise
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate name
        let name = self.name.trim();
        if name.is_empty() {
            errors.push("Permission name is required".to_string());
        } else if !Validator::validate_length(name, 3, 100) {
            errors.push("Permission name must be between 3 and 100 characters".to_string());
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
    async fn test_permission_request_valid() {
        let request = PermissionRequest {
            name: "Create User".to_string(),
        };
        assert!(request.validate().is_ok());
    }

    #[tokio::test]
    async fn test_permission_request_valid_minimum_length() {
        let request = PermissionRequest {
            name: "abc".to_string(), // Exactly 3 characters
        };
        assert!(request.validate().is_ok());
    }

    #[tokio::test]
    async fn test_permission_request_valid_maximum_length() {
        let request = PermissionRequest {
            name: "a".repeat(100), // Exactly 100 characters
        };
        assert!(request.validate().is_ok());
    }

    #[tokio::test]
    async fn test_permission_request_empty_name() {
        let request = PermissionRequest {
            name: "".to_string(),
        };
        let result = request.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().iter().any(|e| e.contains("required")));
    }

    #[tokio::test]
    async fn test_permission_request_name_too_short() {
        let request = PermissionRequest {
            name: "ab".to_string(), // Only 2 characters
        };
        let result = request.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().iter().any(|e| e.contains("between 3 and 100")));
    }

    #[tokio::test]
    async fn test_permission_request_name_too_long() {
        let request = PermissionRequest {
            name: "a".repeat(101), // 101 characters
        };
        let result = request.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().iter().any(|e| e.contains("between 3 and 100")));
    }

    #[tokio::test]
    async fn test_permission_request_whitespace_handling() {
        let request = PermissionRequest {
            name: "  Create User  ".to_string(),
        };
        assert!(request.validate().is_ok());
    }

    #[tokio::test]
    async fn test_permission_request_whitespace_only() {
        let request = PermissionRequest {
            name: "   ".to_string(),
        };
        let result = request.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().iter().any(|e| e.contains("required")));
    }
}
