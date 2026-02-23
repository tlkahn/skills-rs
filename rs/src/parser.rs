use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::errors::SkillError;
use crate::models::SkillProperties;

/// Find the SKILL.md file in a skill directory.
/// Prefers SKILL.md (uppercase) but accepts skill.md (lowercase).
pub fn find_skill_md(skill_dir: &Path) -> Option<PathBuf> {
    for name in &["SKILL.md", "skill.md"] {
        let path = skill_dir.join(name);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

/// Parse YAML frontmatter from SKILL.md content.
/// Returns (metadata mapping, markdown body).
pub fn parse_frontmatter(content: &str) -> Result<(HashMap<String, serde_yaml::Value>, String), SkillError> {
    if !content.starts_with("---") {
        return Err(SkillError::parse(
            "SKILL.md must start with YAML frontmatter (---)",
        ));
    }

    // Python: content.split("---", 2) splits into at most 3 parts.
    // First part is empty (before the opening ---).
    // We skip the leading "---" and split the rest.
    let after_first = &content[3..];
    let Some(end_pos) = after_first.find("---") else {
        return Err(SkillError::parse(
            "SKILL.md frontmatter not properly closed with ---",
        ));
    };

    let frontmatter_str = &after_first[..end_pos];
    let body = after_first[end_pos + 3..].trim().to_string();

    // Parse YAML
    let yaml_value: serde_yaml::Value = serde_yaml::from_str(frontmatter_str)
        .map_err(|e| SkillError::parse(format!("Invalid YAML in frontmatter: {}", e)))?;

    let mapping = match yaml_value {
        serde_yaml::Value::Mapping(m) => m,
        _ => {
            return Err(SkillError::parse(
                "SKILL.md frontmatter must be a YAML mapping",
            ));
        }
    };

    let mut result: HashMap<String, serde_yaml::Value> = HashMap::new();
    for (k, v) in mapping {
        let key = yaml_value_to_string(&k);
        // If the key is "metadata" and value is a mapping, coerce all values to strings
        if key == "metadata" {
            if let serde_yaml::Value::Mapping(meta_map) = v {
                let mut string_map = serde_yaml::Mapping::new();
                for (mk, mv) in meta_map {
                    let mk_str = yaml_value_to_string(&mk);
                    let mv_str = yaml_value_to_string(&mv);
                    string_map.insert(
                        serde_yaml::Value::String(mk_str),
                        serde_yaml::Value::String(mv_str),
                    );
                }
                result.insert(key, serde_yaml::Value::Mapping(string_map));
            } else {
                result.insert(key, v);
            }
        } else {
            result.insert(key, v);
        }
    }

    Ok((result, body))
}

/// Convert a serde_yaml::Value to a String, matching Python's str() semantics.
fn yaml_value_to_string(v: &serde_yaml::Value) -> String {
    match v {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Number(n) => {
            // For numbers like 1.0, Python's strictyaml keeps it as "1.0" string.
            // serde_yaml parses it as f64. We need to format it nicely.
            if let Some(i) = n.as_u64() {
                i.to_string()
            } else if let Some(i) = n.as_i64() {
                i.to_string()
            } else if let Some(f) = n.as_f64() {
                // Format like Python: 1.0 stays "1.0"
                if f.fract() == 0.0 && f.abs() < i64::MAX as f64 {
                    format!("{:.1}", f)
                } else {
                    f.to_string()
                }
            } else {
                n.to_string()
            }
        }
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Null => "null".to_string(),
        _ => format!("{:?}", v),
    }
}

/// Read skill properties from SKILL.md frontmatter.
/// Does NOT perform full validation — use `validate()` for that.
pub fn read_properties(skill_dir: &Path) -> Result<SkillProperties, SkillError> {
    let skill_md = find_skill_md(skill_dir).ok_or_else(|| {
        SkillError::parse(format!("SKILL.md not found in {}", skill_dir.display()))
    })?;

    let content = std::fs::read_to_string(&skill_md)
        .map_err(|e| SkillError::parse(format!("Failed to read {}: {}", skill_md.display(), e)))?;

    let (metadata, _body) = parse_frontmatter(&content)?;

    // Check required fields
    if !metadata.contains_key("name") {
        return Err(SkillError::validation(
            "Missing required field in frontmatter: name",
        ));
    }
    if !metadata.contains_key("description") {
        return Err(SkillError::validation(
            "Missing required field in frontmatter: description",
        ));
    }

    let name = extract_string(&metadata, "name")
        .ok_or_else(|| SkillError::validation("Field 'name' must be a non-empty string"))?;
    let description = extract_string(&metadata, "description")
        .ok_or_else(|| SkillError::validation("Field 'description' must be a non-empty string"))?;

    let name = name.trim().to_string();
    let description = description.trim().to_string();

    if name.is_empty() {
        return Err(SkillError::validation(
            "Field 'name' must be a non-empty string",
        ));
    }
    if description.is_empty() {
        return Err(SkillError::validation(
            "Field 'description' must be a non-empty string",
        ));
    }

    let license = extract_string(&metadata, "license");
    let compatibility = extract_string(&metadata, "compatibility");
    let allowed_tools = extract_string(&metadata, "allowed-tools");

    let meta = extract_metadata_map(&metadata);

    Ok(SkillProperties {
        name,
        description,
        license,
        compatibility,
        allowed_tools,
        metadata: meta,
    })
}

/// Extract a string value from the metadata map.
fn extract_string(metadata: &HashMap<String, serde_yaml::Value>, key: &str) -> Option<String> {
    metadata.get(key).and_then(|v| match v {
        serde_yaml::Value::String(s) if !s.trim().is_empty() => Some(s.clone()),
        _ => None,
    })
}

/// Extract the metadata sub-map as HashMap<String, String>.
fn extract_metadata_map(metadata: &HashMap<String, serde_yaml::Value>) -> HashMap<String, String> {
    let mut result = HashMap::new();
    if let Some(serde_yaml::Value::Mapping(m)) = metadata.get("metadata") {
        for (k, v) in m {
            if let (serde_yaml::Value::String(ks), serde_yaml::Value::String(vs)) = (k, v) {
                result.insert(ks.clone(), vs.clone());
            }
        }
    }
    result
}
