mod common;

use common::create_skill_dir;
use skills_ref::writer::parse_key_value;
use skills_ref::writer::set_properties;

// =============================================================================
// Batch B: Core write path
// =============================================================================

#[test]
fn test_set_single_property() {
    let content = "---\nname: test-skill\ndescription: Original\n---\n\n# Hello";
    let (_tmp, skill_dir) = create_skill_dir("test-skill", content);

    set_properties(
        &skill_dir,
        &[("description".into(), "Updated description".into())],
    )
    .unwrap();

    let updated = std::fs::read_to_string(skill_dir.join("SKILL.md")).unwrap();
    let (metadata, body) = skills_ref::parse_frontmatter(&updated).unwrap();

    assert_eq!(
        metadata["description"],
        serde_yaml::Value::String("Updated description".into())
    );
    assert_eq!(
        metadata["name"],
        serde_yaml::Value::String("test-skill".into())
    );
    assert_eq!(body, "# Hello");
}

#[test]
fn test_set_multiple_properties() {
    let content = "---\nname: test-skill\ndescription: Original\n---\n\nBody";
    let (_tmp, skill_dir) = create_skill_dir("test-skill", content);

    set_properties(
        &skill_dir,
        &[
            ("description".into(), "New desc".into()),
            ("license".into(), "MIT".into()),
        ],
    )
    .unwrap();

    let updated = std::fs::read_to_string(skill_dir.join("SKILL.md")).unwrap();
    let (metadata, _) = skills_ref::parse_frontmatter(&updated).unwrap();

    assert_eq!(
        metadata["description"],
        serde_yaml::Value::String("New desc".into())
    );
    assert_eq!(
        metadata["license"],
        serde_yaml::Value::String("MIT".into())
    );
}

#[test]
fn test_add_new_optional_property() {
    let content = "---\nname: test-skill\ndescription: A skill\n---\n\nBody";
    let (_tmp, skill_dir) = create_skill_dir("test-skill", content);

    set_properties(
        &skill_dir,
        &[("license".into(), "Apache-2.0".into())],
    )
    .unwrap();

    let updated = std::fs::read_to_string(skill_dir.join("SKILL.md")).unwrap();
    let (metadata, _) = skills_ref::parse_frontmatter(&updated).unwrap();

    assert_eq!(
        metadata["license"],
        serde_yaml::Value::String("Apache-2.0".into())
    );
}

#[test]
fn test_body_preserved() {
    let body_text =
        "# Title\n\nParagraph one.\n\n- item 1\n- item 2\n\n```python\nprint('hello')\n```";
    let content = format!(
        "---\nname: test-skill\ndescription: A skill\n---\n\n{}",
        body_text
    );
    let (_tmp, skill_dir) = create_skill_dir("test-skill", &content);

    set_properties(
        &skill_dir,
        &[("description".into(), "Updated".into())],
    )
    .unwrap();

    let updated = std::fs::read_to_string(skill_dir.join("SKILL.md")).unwrap();
    let (_, updated_body) = skills_ref::parse_frontmatter(&updated).unwrap();

    assert_eq!(updated_body, body_text);
}

// =============================================================================
// Batch C: Metadata dot-notation
// =============================================================================

#[test]
fn test_set_metadata_dot_notation() {
    let content = "---\nname: test-skill\ndescription: A skill\n---\n\nBody";
    let (_tmp, skill_dir) = create_skill_dir("test-skill", content);

    set_properties(
        &skill_dir,
        &[("metadata.author".into(), "Jane".into())],
    )
    .unwrap();

    let updated = std::fs::read_to_string(skill_dir.join("SKILL.md")).unwrap();
    let (metadata, _) = skills_ref::parse_frontmatter(&updated).unwrap();

    if let Some(serde_yaml::Value::Mapping(m)) = metadata.get("metadata") {
        assert_eq!(
            m.get(serde_yaml::Value::String("author".into())),
            Some(&serde_yaml::Value::String("Jane".into()))
        );
    } else {
        panic!("Expected metadata to be a Mapping");
    }
}

#[test]
fn test_set_multiple_metadata_keys() {
    let content = "---\nname: test-skill\ndescription: A skill\n---\n\nBody";
    let (_tmp, skill_dir) = create_skill_dir("test-skill", content);

    set_properties(
        &skill_dir,
        &[
            ("metadata.author".into(), "Jane".into()),
            ("metadata.version".into(), "2.0".into()),
        ],
    )
    .unwrap();

    let updated = std::fs::read_to_string(skill_dir.join("SKILL.md")).unwrap();
    let (metadata, _) = skills_ref::parse_frontmatter(&updated).unwrap();

    if let Some(serde_yaml::Value::Mapping(m)) = metadata.get("metadata") {
        assert_eq!(
            m.get(serde_yaml::Value::String("author".into())),
            Some(&serde_yaml::Value::String("Jane".into()))
        );
        assert_eq!(
            m.get(serde_yaml::Value::String("version".into())),
            Some(&serde_yaml::Value::String("2.0".into()))
        );
    } else {
        panic!("Expected metadata to be a Mapping");
    }
}

#[test]
fn test_add_metadata_when_none_exists() {
    let content = "---\nname: test-skill\ndescription: A skill\n---\n\nBody";
    let (_tmp, skill_dir) = create_skill_dir("test-skill", content);

    // Verify no metadata key initially
    let initial = std::fs::read_to_string(skill_dir.join("SKILL.md")).unwrap();
    let (initial_meta, _) = skills_ref::parse_frontmatter(&initial).unwrap();
    assert!(!initial_meta.contains_key("metadata"));

    set_properties(
        &skill_dir,
        &[("metadata.author".into(), "Jane".into())],
    )
    .unwrap();

    let updated = std::fs::read_to_string(skill_dir.join("SKILL.md")).unwrap();
    let (metadata, _) = skills_ref::parse_frontmatter(&updated).unwrap();

    assert!(metadata.contains_key("metadata"));
    if let Some(serde_yaml::Value::Mapping(m)) = metadata.get("metadata") {
        assert_eq!(
            m.get(serde_yaml::Value::String("author".into())),
            Some(&serde_yaml::Value::String("Jane".into()))
        );
    } else {
        panic!("Expected metadata to be a Mapping");
    }
}

#[test]
fn test_overwrite_existing_metadata_key() {
    let content =
        "---\nname: test-skill\ndescription: A skill\nmetadata:\n  author: Old\n---\n\nBody";
    let (_tmp, skill_dir) = create_skill_dir("test-skill", content);

    set_properties(
        &skill_dir,
        &[("metadata.author".into(), "New".into())],
    )
    .unwrap();

    let updated = std::fs::read_to_string(skill_dir.join("SKILL.md")).unwrap();
    let (metadata, _) = skills_ref::parse_frontmatter(&updated).unwrap();

    if let Some(serde_yaml::Value::Mapping(m)) = metadata.get("metadata") {
        assert_eq!(
            m.get(serde_yaml::Value::String("author".into())),
            Some(&serde_yaml::Value::String("New".into()))
        );
    } else {
        panic!("Expected metadata to be a Mapping");
    }
}

// =============================================================================
// Batch D: Validation rejection (file must remain untouched)
// =============================================================================

#[test]
fn test_rejects_invalid_name() {
    let content = "---\nname: test-skill\ndescription: A skill\n---\n\nBody";
    let (_tmp, skill_dir) = create_skill_dir("test-skill", content);
    let original = std::fs::read_to_string(skill_dir.join("SKILL.md")).unwrap();

    let result = set_properties(&skill_dir, &[("name".into(), "INVALID".into())]);

    assert!(result.is_err());
    let after = std::fs::read_to_string(skill_dir.join("SKILL.md")).unwrap();
    assert_eq!(original, after);
}

#[test]
fn test_rejects_name_directory_mismatch() {
    let content = "---\nname: test-skill\ndescription: A skill\n---\n\nBody";
    let (_tmp, skill_dir) = create_skill_dir("test-skill", content);
    let original = std::fs::read_to_string(skill_dir.join("SKILL.md")).unwrap();

    let result = set_properties(&skill_dir, &[("name".into(), "other-name".into())]);

    assert!(result.is_err());
    let after = std::fs::read_to_string(skill_dir.join("SKILL.md")).unwrap();
    assert_eq!(original, after);
}

#[test]
fn test_rejects_description_too_long() {
    let content = "---\nname: test-skill\ndescription: A skill\n---\n\nBody";
    let (_tmp, skill_dir) = create_skill_dir("test-skill", content);
    let original = std::fs::read_to_string(skill_dir.join("SKILL.md")).unwrap();

    let long_desc = "x".repeat(1025);
    let result = set_properties(&skill_dir, &[("description".into(), long_desc)]);

    assert!(result.is_err());
    let after = std::fs::read_to_string(skill_dir.join("SKILL.md")).unwrap();
    assert_eq!(original, after);
}

#[test]
fn test_rejects_unknown_field() {
    let content = "---\nname: test-skill\ndescription: A skill\n---\n\nBody";
    let (_tmp, skill_dir) = create_skill_dir("test-skill", content);
    let original = std::fs::read_to_string(skill_dir.join("SKILL.md")).unwrap();

    let result = set_properties(
        &skill_dir,
        &[("unknown-field".into(), "value".into())],
    );

    assert!(result.is_err());
    let after = std::fs::read_to_string(skill_dir.join("SKILL.md")).unwrap();
    assert_eq!(original, after);
}

// =============================================================================
// Batch E: Error cases
// =============================================================================

#[test]
fn test_missing_skill_md() {
    let tmp = tempfile::TempDir::new().unwrap();
    let skill_dir = tmp.path().join("nonexistent-skill");
    std::fs::create_dir_all(&skill_dir).unwrap();

    let result = set_properties(
        &skill_dir,
        &[("description".into(), "value".into())],
    );

    assert!(result.is_err());
}

#[test]
fn test_invalid_key_value_format() {
    let result = parse_key_value("no-equals-sign");
    assert!(result.is_err());
}

#[test]
fn test_empty_metadata_sub_key() {
    let content = "---\nname: test-skill\ndescription: A skill\n---\n\nBody";
    let (_tmp, skill_dir) = create_skill_dir("test-skill", content);

    let result = set_properties(
        &skill_dir,
        &[("metadata.".into(), "value".into())],
    );

    assert!(result.is_err());
}
