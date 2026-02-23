use std::collections::HashMap;
use std::path::Path;

use crate::errors::SkillError;
use crate::parser::{find_skill_md, parse_frontmatter};
use crate::validator::validate_metadata;

/// Parse a `"key=value"` string into `(key, value)`.
pub fn parse_key_value(input: &str) -> Result<(String, String), SkillError> {
    let Some(eq_pos) = input.find('=') else {
        return Err(SkillError::parse(format!(
            "Invalid key=value format: '{}'. Expected 'key=value'.",
            input
        )));
    };
    let key = input[..eq_pos].to_string();
    let value = input[eq_pos + 1..].to_string();
    Ok((key, value))
}

/// Merge key=value properties into a metadata HashMap.
///
/// Simple keys (e.g. `name`) insert a `Value::String` at the top level.
/// Dotted keys (e.g. `metadata.author`) insert into the `metadata` sub-mapping,
/// creating it if needed.
fn merge_properties(
    metadata: &mut HashMap<String, serde_yaml::Value>,
    properties: &[(String, String)],
) -> Result<(), SkillError> {
    for (key, value) in properties {
        if let Some(sub_key) = key.strip_prefix("metadata.") {
            if sub_key.is_empty() {
                return Err(SkillError::parse(
                    "Empty sub-key in 'metadata.' notation",
                ));
            }
            let meta_mapping = metadata
                .entry("metadata".to_string())
                .or_insert_with(|| serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));

            if let serde_yaml::Value::Mapping(ref mut m) = meta_mapping {
                m.insert(
                    serde_yaml::Value::String(sub_key.to_string()),
                    serde_yaml::Value::String(value.clone()),
                );
            }
        } else {
            metadata.insert(key.clone(), serde_yaml::Value::String(value.clone()));
        }
    }
    Ok(())
}

/// Serialize metadata and body back into SKILL.md file content.
///
/// Keys are sorted for deterministic output.
/// Format: `---\n{yaml}---\n\n{body}\n`
fn serialize_skill_md(metadata: &HashMap<String, serde_yaml::Value>, body: &str) -> String {
    // Build a Mapping with keys in sorted order for deterministic output
    let mut mapping = serde_yaml::Mapping::new();
    let mut keys: Vec<&String> = metadata.keys().collect();
    keys.sort();

    for key in keys {
        mapping.insert(
            serde_yaml::Value::String(key.clone()),
            metadata[key].clone(),
        );
    }

    let yaml = serde_yaml::to_string(&mapping).unwrap_or_default();
    format!("---\n{}---\n\n{}\n", yaml, body)
}

/// Set properties in a SKILL.md file.
///
/// Reads the file, merges properties, validates, and writes back.
/// The file is only written if validation passes.
pub fn set_properties(
    skill_dir: &Path,
    properties: &[(String, String)],
) -> Result<(), SkillError> {
    let skill_md = find_skill_md(skill_dir).ok_or_else(|| {
        SkillError::parse(format!(
            "SKILL.md not found in {}",
            skill_dir.display()
        ))
    })?;

    let content = std::fs::read_to_string(&skill_md)
        .map_err(|e| SkillError::parse(format!("Failed to read {}: {}", skill_md.display(), e)))?;

    let (mut metadata, body) = parse_frontmatter(&content)?;

    merge_properties(&mut metadata, properties)?;

    let errors = validate_metadata(&metadata, Some(skill_dir));
    if !errors.is_empty() {
        return Err(SkillError::validation_many(errors.join("; "), errors));
    }

    let output = serialize_skill_md(&metadata, &body);

    std::fs::write(&skill_md, output)
        .map_err(|e| SkillError::parse(format!("Failed to write {}: {}", skill_md.display(), e)))?;

    Ok(())
}
