use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::validation::Validator;

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub display_name: String,
    pub password: String,
}

impl RegisterRequest {
    pub fn validate(&self) -> Result<(), crate::shared::errors::ValidationError> {
        Validator::new()
            .email("email", self.email.trim())
            .require("display_name", &self.display_name, 2, 60)
            .require("password", &self.password, 12, 200)
            .into_result()
    }

    /// Addresses are matched case-insensitively.
    pub fn normalized_email(&self) -> String {
        self.email.trim().to_lowercase()
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

impl ChangePasswordRequest {
    pub fn validate(&self) -> Result<(), crate::shared::errors::ValidationError> {
        Validator::new()
            .require("new_password", &self.new_password, 12, 200)
            .check(
                self.current_password != self.new_password,
                "new_password",
                "must differ from the current password",
            )
            .into_result()
    }
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub user: UserProfile,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<&super::models::User> for UserProfile {
    fn from(user: &super::models::User) -> Self {
        Self {
            id: user.id,
            email: user.email.clone(),
            display_name: user.display_name.clone(),
            created_at: user.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}

impl MessageResponse {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_passwords_are_rejected() {
        let request = RegisterRequest {
            email: "rani@example.com".into(),
            display_name: "Rani".into(),
            password: "short".into(),
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn email_is_normalized() {
        let request = RegisterRequest {
            email: "  Rani@Example.COM ".into(),
            display_name: "Rani".into(),
            password: "a long enough password".into(),
        };
        assert!(request.validate().is_ok());
        assert_eq!(request.normalized_email(), "rani@example.com");
    }
}
