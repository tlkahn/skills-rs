/// Errors that can occur when working with skill files.
#[derive(Debug, Clone, thiserror::Error)]
pub enum SkillError {
    /// A parsing error (missing file, bad YAML, etc.)
    #[error("{0}")]
    Parse(String),
    /// One or more validation errors.
    #[error("{message}")]
    Validation {
        message: String,
        errors: Vec<String>,
    },
}

impl SkillError {
    /// Create a parse error.
    pub fn parse(msg: impl Into<String>) -> Self {
        SkillError::Parse(msg.into())
    }

    /// Create a validation error with a single message.
    pub fn validation(msg: impl Into<String>) -> Self {
        let m = msg.into();
        SkillError::Validation {
            message: m.clone(),
            errors: vec![m],
        }
    }

    /// Create a validation error with multiple messages.
    pub fn validation_many(msg: impl Into<String>, errors: Vec<String>) -> Self {
        SkillError::Validation {
            message: msg.into(),
            errors,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_display() {
        let err = SkillError::parse("something broke");
        assert_eq!(err.to_string(), "something broke");
    }

    #[test]
    fn test_validation_error_single() {
        let err = SkillError::validation("field missing");
        assert_eq!(err.to_string(), "field missing");
        if let SkillError::Validation { errors, .. } = &err {
            assert_eq!(errors, &["field missing"]);
        } else {
            panic!("expected Validation variant");
        }
    }

    #[test]
    fn test_validation_error_many() {
        let err = SkillError::validation_many(
            "multiple issues",
            vec!["err1".into(), "err2".into()],
        );
        assert_eq!(err.to_string(), "multiple issues");
        if let SkillError::Validation { errors, .. } = &err {
            assert_eq!(errors.len(), 2);
        } else {
            panic!("expected Validation variant");
        }
    }
}
