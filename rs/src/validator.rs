use std::collections::HashSet;
use std::path::Path;

use unicode_normalization::UnicodeNormalization;

use crate::errors::SkillError;
use crate::parser::{find_skill_md, parse_frontmatter};

const MAX_SKILL_NAME_LENGTH: usize = 64;
const MAX_DESCRIPTION_LENGTH: usize = 1024;
const MAX_COMPATIBILITY_LENGTH: usize = 500;

fn allowed_fields() -> HashSet<&'static str> {
    [
        "name",
        "description",
        "license",
        "allowed-tools",
        "metadata",
        "compatibility",
    ]
    .into_iter()
    .collect()
}

fn validate_name(name: &str, skill_dir: Option<&Path>) -> Vec<String> {
    let mut errors = Vec::new();

    if name.is_empty() || name.trim().is_empty() {
        errors.push("Field 'name' must be a non-empty string".into());
        return errors;
    }

    let name: String = name.trim().nfkc().collect();

    if name.chars().count() > MAX_SKILL_NAME_LENGTH {
        errors.push(format!(
            "Skill name '{}' exceeds {} character limit ({} chars)",
            name,
            MAX_SKILL_NAME_LENGTH,
            name.chars().count()
        ));
    }

    let lowered: String = name.chars().flat_map(|c| c.to_lowercase()).collect();
    if name != lowered {
        errors.push(format!("Skill name '{}' must be lowercase", name));
    }

    if name.starts_with('-') || name.ends_with('-') {
        errors.push("Skill name cannot start or end with a hyphen".into());
    }

    if name.contains("--") {
        errors.push("Skill name cannot contain consecutive hyphens".into());
    }

    if !name.chars().all(|c| c.is_alphanumeric() || c == '-') {
        errors.push(format!(
            "Skill name '{}' contains invalid characters. Only letters, digits, and hyphens are allowed.",
            name
        ));
    }

    if let Some(dir) = skill_dir {
        let dir_name: String = dir
            .file_name()
            .map(|n| n.to_string_lossy().nfkc().collect())
            .unwrap_or_default();
        if dir_name != name {
            errors.push(format!(
                "Directory name '{}' must match skill name '{}'",
                dir.file_name().unwrap_or_default().to_string_lossy(),
                name
            ));
        }
    }

    errors
}

fn validate_description(description: &str) -> Vec<String> {
    let mut errors = Vec::new();

    if description.is_empty() || description.trim().is_empty() {
        errors.push("Field 'description' must be a non-empty string".into());
        return errors;
    }

    if description.chars().count() > MAX_DESCRIPTION_LENGTH {
        errors.push(format!(
            "Description exceeds {} character limit ({} chars)",
            MAX_DESCRIPTION_LENGTH,
            description.chars().count()
        ));
    }

    errors
}

fn validate_compatibility(compatibility: &str) -> Vec<String> {
    let mut errors = Vec::new();

    if compatibility.chars().count() > MAX_COMPATIBILITY_LENGTH {
        errors.push(format!(
            "Compatibility exceeds {} character limit ({} chars)",
            MAX_COMPATIBILITY_LENGTH,
            compatibility.chars().count()
        ));
    }

    errors
}

fn validate_metadata_fields(
    metadata: &std::collections::HashMap<String, serde_yaml::Value>,
) -> Vec<String> {
    let mut errors = Vec::new();
    let allowed = allowed_fields();

    let extra: Vec<String> = metadata
        .keys()
        .filter(|k| !allowed.contains(k.as_str()))
        .cloned()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();

    if !extra.is_empty() {
        let mut sorted_allowed: Vec<&str> = allowed.into_iter().collect();
        sorted_allowed.sort();
        errors.push(format!(
            "Unexpected fields in frontmatter: {}. Only {:?} are allowed.",
            extra.join(", "),
            sorted_allowed
        ));
    }

    errors
}

/// Validate parsed skill metadata.
/// Returns a list of validation error messages. Empty list means valid.
pub fn validate_metadata(
    metadata: &std::collections::HashMap<String, serde_yaml::Value>,
    skill_dir: Option<&Path>,
) -> Vec<String> {
    let mut errors = Vec::new();

    errors.extend(validate_metadata_fields(metadata));

    if !metadata.contains_key("name") {
        errors.push("Missing required field in frontmatter: name".into());
    } else if let Some(serde_yaml::Value::String(name)) = metadata.get("name") {
        errors.extend(validate_name(name, skill_dir));
    } else {
        errors.push("Field 'name' must be a non-empty string".into());
    }

    if !metadata.contains_key("description") {
        errors.push("Missing required field in frontmatter: description".into());
    } else if let Some(serde_yaml::Value::String(desc)) = metadata.get("description") {
        errors.extend(validate_description(desc));
    } else {
        errors.push("Field 'description' must be a non-empty string".into());
    }

    if let Some(serde_yaml::Value::String(compat)) = metadata.get("compatibility") {
        errors.extend(validate_compatibility(compat));
    }

    errors
}

/// Validate a skill directory.
/// Returns a list of validation error messages. Empty list means valid.
pub fn validate(skill_dir: &Path) -> Vec<String> {
    if !skill_dir.exists() {
        return vec![format!("Path does not exist: {}", skill_dir.display())];
    }

    if !skill_dir.is_dir() {
        return vec![format!("Not a directory: {}", skill_dir.display())];
    }

    let skill_md = match find_skill_md(skill_dir) {
        Some(p) => p,
        None => return vec!["Missing required file: SKILL.md".into()],
    };

    let content = match std::fs::read_to_string(&skill_md) {
        Ok(c) => c,
        Err(e) => return vec![format!("Failed to read {}: {}", skill_md.display(), e)],
    };

    let metadata = match parse_frontmatter(&content) {
        Ok((m, _)) => m,
        Err(SkillError::Parse(msg)) => return vec![msg],
        Err(e) => return vec![e.to_string()],
    };

    validate_metadata(&metadata, Some(skill_dir))
}
