/// Properties parsed from a skill's SKILL.md frontmatter.
#[derive(Debug, Clone, PartialEq)]
pub struct SkillProperties {
    pub name: String,
    pub description: String,
    pub license: Option<String>,
    pub compatibility: Option<String>,
    pub allowed_tools: Option<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

impl SkillProperties {
    /// Convert to a JSON value, excluding None fields and empty metadata.
    /// Note: `allowed_tools` is serialized as `"allowed-tools"`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        map.insert("name".into(), serde_json::Value::String(self.name.clone()));
        map.insert(
            "description".into(),
            serde_json::Value::String(self.description.clone()),
        );
        if let Some(ref license) = self.license {
            map.insert("license".into(), serde_json::Value::String(license.clone()));
        }
        if let Some(ref compat) = self.compatibility {
            map.insert(
                "compatibility".into(),
                serde_json::Value::String(compat.clone()),
            );
        }
        if let Some(ref tools) = self.allowed_tools {
            map.insert(
                "allowed-tools".into(),
                serde_json::Value::String(tools.clone()),
            );
        }
        if !self.metadata.is_empty() {
            let meta: serde_json::Map<String, serde_json::Value> = self
                .metadata
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect();
            map.insert("metadata".into(), serde_json::Value::Object(meta));
        }
        serde_json::Value::Object(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal() -> SkillProperties {
        SkillProperties {
            name: "test".into(),
            description: "A test".into(),
            license: None,
            compatibility: None,
            allowed_tools: None,
            metadata: Default::default(),
        }
    }

    #[test]
    fn test_minimal_fields() {
        let v = minimal().to_json_value();
        let obj = v.as_object().unwrap();
        assert_eq!(obj.len(), 2);
        assert_eq!(obj["name"], "test");
        assert_eq!(obj["description"], "A test");
    }

    #[test]
    fn test_all_fields() {
        let mut meta = std::collections::HashMap::new();
        meta.insert("author".into(), "Me".into());
        let props = SkillProperties {
            name: "full".into(),
            description: "All fields".into(),
            license: Some("MIT".into()),
            compatibility: Some("Python 3.11+".into()),
            allowed_tools: Some("Bash(git:*)".into()),
            metadata: meta,
        };
        let v = props.to_json_value();
        let obj = v.as_object().unwrap();
        assert_eq!(obj.len(), 6);
        assert!(obj.contains_key("name"));
        assert!(obj.contains_key("description"));
        assert!(obj.contains_key("license"));
        assert!(obj.contains_key("compatibility"));
        assert!(obj.contains_key("allowed-tools"));
        assert!(obj.contains_key("metadata"));
    }

    #[test]
    fn test_allowed_tools_key() {
        let props = SkillProperties {
            allowed_tools: Some("Bash(jq:*)".into()),
            ..minimal()
        };
        let v = props.to_json_value();
        assert!(v.as_object().unwrap().contains_key("allowed-tools"));
        assert!(!v.as_object().unwrap().contains_key("allowed_tools"));
    }

    #[test]
    fn test_empty_metadata_omitted() {
        let v = minimal().to_json_value();
        assert!(!v.as_object().unwrap().contains_key("metadata"));
    }
}
