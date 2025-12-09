use lighter_common::prelude::Uuid;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::security::validation::Validator;

#[derive(Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserStoreRequest {
    #[schema(example = "John Doe")]
    pub name: String,
    #[schema(example = "john.doe@example")]
    pub email: String,
    #[schema(example = "john.doe")]
    pub username: String,
    #[schema(example = "password")]
    pub password: String,
    #[schema(example = "password")]
    pub password_confirmation: String,
    #[schema()]
    pub profile_photo_id: Option<String>,
    #[schema()]
    pub permissions: Vec<Uuid>,
    #[schema()]
    pub roles: Vec<Uuid>,
}

impl UserStoreRequest {
    /// Validates the user store request
    ///
    /// Validates:
    /// - name: must not be empty
    /// - email: must be valid email format
    /// - username: must be valid username format (3-32 chars, alphanumeric, underscore, hyphen)
    /// - password: must meet strength requirements
    /// - password_confirmation: must match password
    ///
    /// Returns Ok(()) if all validations pass, Err(Vec<String>) with error messages otherwise
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate name
        if self.name.trim().is_empty() {
            errors.push("Name is required".to_string());
        }

        // Validate email
        let email = self.email.trim();
        if email.is_empty() {
            errors.push("Email is required".to_string());
        } else if !Validator::validate_email(email) {
            errors.push("Email must be a valid email address".to_string());
        }

        // Validate username
        let username = self.username.trim();
        if username.is_empty() {
            errors.push("Username is required".to_string());
        } else if !Validator::validate_username(username) {
            errors.push("Username must be 3-32 characters long and contain only alphanumeric characters, underscores, or hyphens".to_string());
        }

        // Validate password strength
        if let Err(password_errors) = Validator::validate_password(&self.password) {
            errors.extend(password_errors);
        }

        // Validate password confirmation
        if self.password != self.password_confirmation {
            errors.push("Password confirmation does not match".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserUpdateGeneralInformationRequest {
    #[schema(example = "John Doe")]
    pub name: String,
    #[schema(example = "john.doe@example")]
    pub email: String,
    #[schema(example = "john.doe")]
    pub username: String,
    #[schema()]
    pub profile_photo_id: Option<String>,
    #[schema()]
    pub permissions: Vec<Uuid>,
    #[schema()]
    pub roles: Vec<Uuid>,
}

impl UserUpdateGeneralInformationRequest {
    /// Validates the user update general information request
    ///
    /// Validates:
    /// - name: must not be empty
    /// - email: must be valid email format
    /// - username: must be valid username format (3-32 chars, alphanumeric, underscore, hyphen)
    ///
    /// Returns Ok(()) if all validations pass, Err(Vec<String>) with error messages otherwise
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate name
        if self.name.trim().is_empty() {
            errors.push("Name is required".to_string());
        }

        // Validate email
        let email = self.email.trim();
        if email.is_empty() {
            errors.push("Email is required".to_string());
        } else if !Validator::validate_email(email) {
            errors.push("Email must be a valid email address".to_string());
        }

        // Validate username
        let username = self.username.trim();
        if username.is_empty() {
            errors.push("Username is required".to_string());
        } else if !Validator::validate_username(username) {
            errors.push("Username must be 3-32 characters long and contain only alphanumeric characters, underscores, or hyphens".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserUpdatePasswordRequest {
    #[schema(example = "password")]
    pub current_password: String,
    #[schema(example = "password")]
    pub new_password: String,
    #[schema(example = "password")]
    pub password_confirmation: String,
}

impl UserUpdatePasswordRequest {
    /// Validates the user update password request
    ///
    /// Validates:
    /// - current_password: must not be empty
    /// - new_password: must meet strength requirements
    /// - password_confirmation: must match new_password
    ///
    /// Returns Ok(()) if all validations pass, Err(Vec<String>) with error messages otherwise
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate current password
        if self.current_password.is_empty() {
            errors.push("Current password is required".to_string());
        }

        // Validate new password strength
        if let Err(password_errors) = Validator::validate_password(&self.new_password) {
            errors.extend(password_errors);
        }

        // Validate password confirmation
        if self.new_password != self.password_confirmation {
            errors.push("Password confirmation does not match".to_string());
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

    // UserStoreRequest tests
    #[tokio::test]
    async fn test_user_store_request_valid() {
        let request = UserStoreRequest {
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            username: "john_doe".to_string(),
            password: "SecureP@ss123".to_string(),
            password_confirmation: "SecureP@ss123".to_string(),
            profile_photo_id: None,
            permissions: vec![],
            roles: vec![],
        };
        assert!(request.validate().is_ok());
    }

    #[tokio::test]
    async fn test_user_store_request_empty_name() {
        let request = UserStoreRequest {
            name: "".to_string(),
            email: "john@example.com".to_string(),
            username: "john_doe".to_string(),
            password: "SecureP@ss123".to_string(),
            password_confirmation: "SecureP@ss123".to_string(),
            profile_photo_id: None,
            permissions: vec![],
            roles: vec![],
        };
        let result = request.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().iter().any(|e| e.contains("Name is required")));
    }

    #[tokio::test]
    async fn test_user_store_request_invalid_email() {
        let request = UserStoreRequest {
            name: "John Doe".to_string(),
            email: "invalid-email".to_string(),
            username: "john_doe".to_string(),
            password: "SecureP@ss123".to_string(),
            password_confirmation: "SecureP@ss123".to_string(),
            profile_photo_id: None,
            permissions: vec![],
            roles: vec![],
        };
        let result = request.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().iter().any(|e| e.contains("valid email")));
    }

    #[tokio::test]
    async fn test_user_store_request_invalid_username() {
        let request = UserStoreRequest {
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            username: "ab".to_string(), // Too short
            password: "SecureP@ss123".to_string(),
            password_confirmation: "SecureP@ss123".to_string(),
            profile_photo_id: None,
            permissions: vec![],
            roles: vec![],
        };
        let result = request.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().iter().any(|e| e.contains("3-32 characters")));
    }

    #[tokio::test]
    async fn test_user_store_request_weak_password() {
        let request = UserStoreRequest {
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            username: "john_doe".to_string(),
            password: "weak".to_string(),
            password_confirmation: "weak".to_string(),
            profile_photo_id: None,
            permissions: vec![],
            roles: vec![],
        };
        let result = request.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.len() > 1); // Multiple password validation errors
    }

    #[tokio::test]
    async fn test_user_store_request_password_mismatch() {
        let request = UserStoreRequest {
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            username: "john_doe".to_string(),
            password: "SecureP@ss123".to_string(),
            password_confirmation: "DifferentP@ss123".to_string(),
            profile_photo_id: None,
            permissions: vec![],
            roles: vec![],
        };
        let result = request.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().iter().any(|e| e.contains("does not match")));
    }

    // UserUpdateGeneralInformationRequest tests
    #[tokio::test]
    async fn test_user_update_general_info_valid() {
        let request = UserUpdateGeneralInformationRequest {
            name: "John Updated".to_string(),
            email: "john.updated@example.com".to_string(),
            username: "john_updated".to_string(),
            profile_photo_id: None,
            permissions: vec![],
            roles: vec![],
        };
        assert!(request.validate().is_ok());
    }

    #[tokio::test]
    async fn test_user_update_general_info_empty_name() {
        let request = UserUpdateGeneralInformationRequest {
            name: "".to_string(),
            email: "john@example.com".to_string(),
            username: "john_doe".to_string(),
            profile_photo_id: None,
            permissions: vec![],
            roles: vec![],
        };
        let result = request.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().iter().any(|e| e.contains("Name is required")));
    }

    #[tokio::test]
    async fn test_user_update_general_info_invalid_email() {
        let request = UserUpdateGeneralInformationRequest {
            name: "John Doe".to_string(),
            email: "not-an-email".to_string(),
            username: "john_doe".to_string(),
            profile_photo_id: None,
            permissions: vec![],
            roles: vec![],
        };
        let result = request.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().iter().any(|e| e.contains("valid email")));
    }

    // UserUpdatePasswordRequest tests
    #[tokio::test]
    async fn test_user_update_password_valid() {
        let request = UserUpdatePasswordRequest {
            current_password: "OldP@ss123".to_string(),
            new_password: "NewP@ss456".to_string(),
            password_confirmation: "NewP@ss456".to_string(),
        };
        assert!(request.validate().is_ok());
    }

    #[tokio::test]
    async fn test_user_update_password_empty_current() {
        let request = UserUpdatePasswordRequest {
            current_password: "".to_string(),
            new_password: "NewP@ss456".to_string(),
            password_confirmation: "NewP@ss456".to_string(),
        };
        let result = request.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().iter().any(|e| e.contains("Current password is required")));
    }

    #[tokio::test]
    async fn test_user_update_password_weak_new_password() {
        let request = UserUpdatePasswordRequest {
            current_password: "OldP@ss123".to_string(),
            new_password: "weak".to_string(),
            password_confirmation: "weak".to_string(),
        };
        let result = request.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.len() > 1); // Multiple password validation errors
    }

    #[tokio::test]
    async fn test_user_update_password_confirmation_mismatch() {
        let request = UserUpdatePasswordRequest {
            current_password: "OldP@ss123".to_string(),
            new_password: "NewP@ss456".to_string(),
            password_confirmation: "DifferentP@ss456".to_string(),
        };
        let result = request.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().iter().any(|e| e.contains("does not match")));
    }
}
