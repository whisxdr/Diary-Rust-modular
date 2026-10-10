use serde::{de::DeserializeOwned, Serialize};

/// A single field-level validation problem.
#[derive(Debug, Clone, Serialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("{0}")]
pub struct ValidationError(pub String);

impl ValidationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }

    pub fn field(field: &str, message: impl Into<String>) -> Self {
        Self(format!("{field}: {}", message.into()))
    }
}

/// Collects field errors so a form can report every problem at once.
#[derive(Debug, Default)]
pub struct Validator {
    errors: Vec<FieldError>,
}

impl Validator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn require(mut self, field: &str, value: &str, min: usize, max: usize) -> Self {
        let length = value.trim().chars().count();
        if length < min || length > max {
            self.errors.push(FieldError {
                field: field.to_owned(),
                message: format!("must be between {min} and {max} characters"),
            });
        }
        self
    }

    pub fn email(mut self, field: &str, value: &str) -> Self {
        let looks_like_email = {
            let mut parts = value.split('@');
            match (parts.next(), parts.next(), parts.next()) {
                (Some(local), Some(domain), None) => {
                    !local.is_empty()
                        && domain.contains('.')
                        && !domain.starts_with('.')
                        && !domain.ends_with('.')
                        && !value.contains(char::is_whitespace)
                }
                _ => false,
            }
        };

        if !looks_like_email {
            self.errors.push(FieldError {
                field: field.to_owned(),
                message: "must be a valid email address".to_owned(),
            });
        }
        self
    }

    pub fn range<T: PartialOrd + std::fmt::Display>(
        mut self,
        field: &str,
        value: T,
        min: T,
        max: T,
    ) -> Self {
        if value < min || value > max {
            self.errors.push(FieldError {
                field: field.to_owned(),
                message: format!("must be between {min} and {max}"),
            });
        }
        self
    }

    pub fn check(mut self, condition: bool, field: &str, message: &str) -> Self {
        if !condition {
            self.errors.push(FieldError {
                field: field.to_owned(),
                message: message.to_owned(),
            });
        }
        self
    }

    pub fn into_result(self) -> Result<(), ValidationError> {
        if self.errors.is_empty() {
            return Ok(());
        }
        let joined = self
            .errors
            .iter()
            .map(|e| format!("{} {}", e.field, e.message))
            .collect::<Vec<_>>()
            .join("; ");
        Err(ValidationError::new(joined))
    }
}

/// Parses an optional identifier coming from a URL segment.
pub fn parse_id<T: DeserializeOwned>(raw: &str) -> Result<T, ValidationError> {
    serde_json::from_value(serde_json::Value::String(raw.to_owned()))
        .map_err(|_| ValidationError::new(format!("`{raw}` is not a valid identifier")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_broken_field() {
        let message = Validator::new()
            .require("display_name", "", 2, 40)
            .email("email", "nope")
            .into_result()
            .unwrap_err()
            .0;
        assert!(message.contains("display_name"));
        assert!(message.contains("email"));
    }

    #[test]
    fn accepts_a_well_formed_registration() {
        let valid = Validator::new()
            .require("display_name", "Rani", 2, 40)
            .require("password", "correct horse", 12, 200)
            .email("email", "rani@example.com");
        assert!(valid.into_result().is_ok());
    }

    #[test]
    fn parses_uuid_ids() {
        let id: uuid::Uuid = parse_id("6b8f1f6c-3f2b-4a0f-8a53-2f2f6d4a1c11").unwrap();
        assert!(parse_id::<uuid::Uuid>("nope").is_err());
        assert!(!id.is_nil());
    }
}
