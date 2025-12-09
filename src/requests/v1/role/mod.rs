use lighter_common::prelude::Uuid;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::security::validation::Validator;

#[derive(Clone, Deserialize, ToSchema)]
pub struct RoleRequest {
    #[schema(example = "Manager")]
    pub name: String,
    #[schema()]
    pub permissions: Vec<Uuid>,
}

impl RoleRequest {
    /// Validates the role request
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
            errors.push("Role name is required".to_string());
        } else if !Validator::validate_length(name, 3, 100) {
            errors.push("Role name must be between 3 and 100 characters".to_string());
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
    async fn test_role_request_valid() {
        let request = RoleRequest {
            name: "Manager".to_string(),
            permissions: vec![],
        };
        assert!(request.validate().is_ok());
    }

    #[tokio::test]
    async fn test_role_request_valid_minimum_length() {
        let request = RoleRequest {
            name: "abc".to_string(), // Exactly 3 characters
            permissions: vec![],
        };
        assert!(request.validate().is_ok());
    }

    #[tokio::test]
    async fn test_role_request_valid_maximum_length() {
        let request = RoleRequest {
            name: "a".repeat(100), // Exactly 100 characters
            permissions: vec![],
        };
        assert!(request.validate().is_ok());
    }

    #[tokio::test]
    async fn test_role_request_empty_name() {
        let request = RoleRequest {
            name: "".to_string(),
            permissions: vec![],
        };
        let result = request.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().iter().any(|e| e.contains("required")));
    }

    #[tokio::test]
    async fn test_role_request_name_too_short() {
        let request = RoleRequest {
            name: "ab".to_string(), // Only 2 characters
            permissions: vec![],
        };
        let result = request.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().iter().any(|e| e.contains("between 3 and 100")));
    }

    #[tokio::test]
    async fn test_role_request_name_too_long() {
        let request = RoleRequest {
            name: "a".repeat(101), // 101 characters
            permissions: vec![],
        };
        let result = request.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().iter().any(|e| e.contains("between 3 and 100")));
    }

    #[tokio::test]
    async fn test_role_request_whitespace_handling() {
        let request = RoleRequest {
            name: "  Administrator  ".to_string(),
            permissions: vec![],
        };
        assert!(request.validate().is_ok());
    }

    #[tokio::test]
    async fn test_role_request_whitespace_only() {
        let request = RoleRequest {
            name: "   ".to_string(),
            permissions: vec![],
        };
        let result = request.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().iter().any(|e| e.contains("required")));
    }
}
