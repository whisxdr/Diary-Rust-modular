use uuid::Uuid;

use crate::shared::errors::ValidationError;
use crate::shared::validation::Validator;

pub const MAX_TITLE: usize = 200;
pub const MAX_CONTENT: usize = 400_000;
pub const MAX_TAGS: usize = 20;

/// Rules that belong to entries only. Shared validation is used for the generic
/// field checks; anything diary-specific lives here.
pub fn validate_title(title: &str) -> Result<(), ValidationError> {
    Validator::new()
        .require("title", title, 0, MAX_TITLE)
        .into_result()
}

pub fn validate_content(content: &str) -> Result<(), ValidationError> {
    if content.len() > MAX_CONTENT {
        return Err(ValidationError::field(
            "content",
            format!("must be at most {MAX_CONTENT} bytes"),
        ));
    }
    Ok(())
}

pub fn validate_tags(tags: &[String]) -> Result<(), ValidationError> {
    tags.iter()
        .fold(
            Validator::new().check(tags.len() <= MAX_TAGS, "tags", "too many tags"),
            |validator, tag| validator.require("tags", tag, 1, 40),
        )
        .into_result()
}

pub fn validate_mood(mood_id: Option<Uuid>) -> Result<(), ValidationError> {
    if mood_id.is_some() {
        return Ok(());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oversized_title_is_rejected() {
        let long = "x".repeat(MAX_TITLE + 1);
        assert!(validate_title(&long).is_err());
        assert!(validate_title("A day").is_ok());
    }

    #[test]
    fn tag_limits_are_enforced() {
        let many = (0..MAX_TAGS + 1)
            .map(|i| format!("tag{i}"))
            .collect::<Vec<_>>();
        assert!(validate_tags(&many).is_err());
        assert!(validate_tags(&["work".into()]).is_ok());
    }
}
